use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use ursa::cl::{
    new_nonce,
    RevocationRegistryDelta as CryptoRevocationRegistryDelta,
    Witness,
};
use ursa::cl::{CredentialKeyCorrectnessProof, CredentialPrivateKey};

use crate::commands::BoxedCallbackStringStringSend;
use crate::domain::anoncreds::credential::{CredentialValues, Credential};
use crate::domain::anoncreds::credential_definition::{
    CredentialDefinition,
    CredentialDefinitionConfig,
    CredentialDefinitionCorrectnessProof,
    CredentialDefinitionData,
    CredentialDefinitionPrivateKey,
    CredentialDefinitionV1,
    SignatureType,
    TemporaryCredentialDefinition,
    CredentialDefinitionId,
};
use crate::domain::anoncreds::credential_offer::CredentialOffer;
use crate::domain::anoncreds::credential_request::CredentialRequest;
use crate::domain::anoncreds::revocation_registry::{
    RevocationRegistry,
    RevocationRegistryV1,
};
use crate::domain::anoncreds::revocation_registry_definition::{
    IssuanceType,
    RegistryType,
    RevocationRegistryConfig,
    RevocationRegistryDefinition,
    RevocationRegistryDefinitionPrivate,
    RevocationRegistryDefinitionV1,
    RevocationRegistryDefinitionValue,
    RevocationRegistryInfo,
    RevocationRegistryId,
};
use crate::domain::anoncreds::revocation_registry_delta::{
    RevocationRegistryDelta,
    RevocationRegistryDeltaV1,
};
use crate::domain::anoncreds::schema::{AttributeNames, Schema, SchemaV1, SchemaId};
use crate::domain::crypto::did::DidValue;
use indy_api_types::domain::wallet::Tags;
use indy_api_types::errors::prelude::*;
use crate::services::anoncreds::AnoncredsService;
use crate::services::anoncreds::helpers::parse_cred_rev_id;
use crate::services::blob_storage::BlobStorageService;
use crate::services::crypto::CryptoService;
use crate::services::pool::PoolService;
use indy_wallet::{RecordOptions, WalletService};

use super::tails::{SDKTailsAccessor, store_tails_from_generator};
use indy_api_types::WalletHandle;
use indy_vdr::utils::qualifier::Qualifiable;

pub enum IssuerCommand {
    CreateSchema(
        DidValue, // issuer did
        String, // name
        String, // version
        AttributeNames, // attribute names
        BoxedCallbackStringStringSend),
    CreateAndStoreCredentialDefinition(
        WalletHandle,
        DidValue, // issuer did
        Schema, // schema
        String, // tag
        Option<String>, // type
        Option<CredentialDefinitionConfig>, // config
        BoxedCallbackStringStringSend),
    RotateCredentialDefinitionStart(
        WalletHandle,
        CredentialDefinitionId, // cred def id
        Option<CredentialDefinitionConfig>, // config
        Box<dyn Fn(IndyResult<String>) + Send>),
    RotateCredentialDefinitionApply(
        WalletHandle,
        CredentialDefinitionId, // cred def id
        Box<dyn Fn(IndyResult<()>) + Send>),
    CreateAndStoreRevocationRegistry(
        WalletHandle,
        DidValue, // issuer did
        Option<String>, // type
        String, // tag
        CredentialDefinitionId, // credential definition id
        RevocationRegistryConfig, // config
        i32, // tails writer handle
        Box<dyn Fn(IndyResult<(String, String, String)>) + Send>),
    CreateCredentialOffer(
        WalletHandle,
        CredentialDefinitionId, // credential definition id
        Box<dyn Fn(IndyResult<String>) + Send>),
    CreateCredential(
        WalletHandle,
        CredentialOffer, // credential offer
        CredentialRequest, // credential request
        CredentialValues, // credential values
        Option<RevocationRegistryId>, // revocation registry id
        Option<i32>, // blob storage reader config handle
        Box<dyn Fn(IndyResult<(String, Option<String>, Option<String>)>) + Send>),
    RevokeCredential(
        WalletHandle,
        i32, // blob storage reader config handle
        RevocationRegistryId, //revocation registry id
        String, //credential revoc id
        Box<dyn Fn(IndyResult<String>) + Send>),
    /*    RecoverCredential(
            WalletHandle,
            i32, // blob storage reader config handle
            String, //revocation revoc id
            String, //credential revoc id
            Box<dyn Fn(Result<String, IndyError>) + Send>),*/
    MergeRevocationRegistryDeltas(
        RevocationRegistryDelta, //revocation registry delta
        RevocationRegistryDelta, //other revocation registry delta
        Box<dyn Fn(IndyResult<String>) + Send>),
}

pub struct IssuerCommandExecutor {
    pub anoncreds_service: Rc<AnoncredsService>,
    pub blob_storage_service: Rc<BlobStorageService>,
    pub pool_service: Rc<PoolService>,
    pub wallet_service: Rc<WalletService>,
    pub crypto_service: Rc<CryptoService>,
}

impl IssuerCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               pool_service: Rc<PoolService>,
               blob_storage_service: Rc<BlobStorageService>,
               wallet_service: Rc<WalletService>,
               crypto_service: Rc<CryptoService>) -> IssuerCommandExecutor {
        IssuerCommandExecutor {
            anoncreds_service,
            pool_service,
            blob_storage_service,
            wallet_service,
            crypto_service,
        }
    }

    pub async fn execute(&self, command: IssuerCommand) {
        match command {
            IssuerCommand::CreateSchema(issuer_did, name, version, attrs, cb) => {
                debug!(target: "issuer_command_executor", "CreateSchema command received");
                cb(self.create_schema(&issuer_did, &name, &version, attrs));
            }
            IssuerCommand::CreateAndStoreCredentialDefinition(wallet_handle, issuer_did, schema, tag, type_, config, cb) => {
                debug!(target: "issuer_command_executor", "CreateAndStoreCredentialDefinition command received");
                match schema {
                    Schema::SchemaV1(schema) =>
                        cb(self.create_and_store_credential_definition(wallet_handle, &issuer_did, schema, &tag,
                                                                       type_.as_ref().map(String::as_str), config).await)
                }
            }
            IssuerCommand::RotateCredentialDefinitionStart(wallet_handle, cred_def_id, cred_def_config, cb) => {
                debug!(target: "wallet_command_executor", "RotateCredentialDefinitionStart command received");
                cb(self.rotate_credential_definition_start(wallet_handle, &cred_def_id, cred_def_config.as_ref()).await);
            }
            IssuerCommand::RotateCredentialDefinitionApply(wallet_handle, cred_def_id, cb) => {
                debug!(target: "wallet_command_executor", "RotateCredentialDefinitionApply command received");
                cb(self.rotate_credential_definition_apply(wallet_handle, &cred_def_id));
            }
            IssuerCommand::CreateAndStoreRevocationRegistry(wallet_handle, issuer_did, type_, tag, cred_def_id, config,
                                                            tails_writer_handle, cb) => {
                debug!(target: "issuer_command_executor", "CreateAndStoreRevocationRegistryRegistry command received");
                cb(self.create_and_store_revocation_registry(wallet_handle,
                                                             &issuer_did,
                                                             type_.as_ref().map(String::as_str),
                                                             &tag,
                                                             &cred_def_id,
                                                             &config,
                                                             tails_writer_handle));
            }
            IssuerCommand::CreateCredentialOffer(wallet_handle, cred_def_id, cb) => {
                debug!(target: "issuer_command_executor", "CreateCredentialOffer command received");
                cb(self.create_credential_offer(wallet_handle, &cred_def_id));
            }
            IssuerCommand::CreateCredential(wallet_handle, cred_offer, cred_req, cred_values, rev_reg_id, blob_storage_reader_handle, cb) => {
                debug!(target: "issuer_command_executor", "CreateCredential command received");
                cb(self.new_credential(wallet_handle, &cred_offer, &cred_req, &cred_values, rev_reg_id.as_ref(), blob_storage_reader_handle));
            }
            IssuerCommand::RevokeCredential(wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id, cb) => {
                debug!(target: "issuer_command_executor", "RevokeCredential command received");
                cb(self.revoke_credential(wallet_handle, blob_storage_reader_handle, &rev_reg_id, &cred_revoc_id));
            }
            /*            IssuerCommand::RecoverCredential(wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id, cb) => {
                            debug!(target: "issuer_command_executor", "RecoverCredential command received");
                            cb(self.recovery_credential(wallet_handle, blob_storage_reader_handle, &rev_reg_id, &cred_revoc_id));
                        }*/
            IssuerCommand::MergeRevocationRegistryDeltas(rev_reg_delta, other_rev_reg_delta, cb) => {
                debug!(target: "issuer_command_executor", "MergeRevocationRegistryDeltas command received");
                match (rev_reg_delta, other_rev_reg_delta) {
                    (RevocationRegistryDelta::RevocationRegistryDeltaV1(mut rev_reg_delta),
                        RevocationRegistryDelta::RevocationRegistryDeltaV1(other_rev_reg_delta)) => {
                        cb(self.merge_revocation_registry_deltas(&mut rev_reg_delta, &other_rev_reg_delta));
                    }
                }
            }
        };
    }

    fn create_schema(&self,
                     issuer_did: &DidValue,
                     name: &str,
                     version: &str,
                     attrs: AttributeNames) -> IndyResult<(String, String)> {
        debug!("create_schema >>> issuer_did: {:?}, name: {:?}, version: {:?}, attrs: {:?}", issuer_did, name, version, attrs);

        self.crypto_service.validate_did(issuer_did)?;

        let schema_id = SchemaId::new(&issuer_did, name, version);

        let schema = Schema::SchemaV1(SchemaV1 {
            id: schema_id.clone(),
            name: name.to_string(),
            version: version.to_string(),
            attr_names: attrs,
            seq_no: None,
        });

        let schema_json = serde_json::to_string(&schema)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Schema")?;

        debug!("create_schema <<< schema_id: {:?}, schema_json: {:?}", schema_id, schema_json);

        Ok((schema_id.0, schema_json))
    }

    async fn create_and_store_credential_definition(&self,
                                                    wallet_handle: WalletHandle,
                                                    issuer_did: &DidValue,
                                                    mut schema: SchemaV1,
                                                    tag: &str,
                                                    type_: Option<&str>,
                                                    config: Option<CredentialDefinitionConfig>) -> IndyResult<(String, String)> {
        debug!("create_and_store_credential_definition >>> wallet_handle: {:?}, issuer_did: {:?}, schema: {:?}, tag: {:?}, \
              type_: {:?}, config: {:?}", wallet_handle, issuer_did, schema, tag, type_, config);

        let cred_def_config = config.unwrap_or(CredentialDefinitionConfig::default());

        let signature_type = if let Some(type_) = type_ {
            serde_json::from_str::<SignatureType>(&format!("\"{}\"", type_))
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid Signature Type format")?
        } else {
            SignatureType::CL
        };

        match (issuer_did.get_method(), schema.id.get_method()) {
            (None, Some(_)) => {
                return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "You can't use unqualified Did with fully qualified Schema"));
            }
            (Some(prefix_), None) => {
                schema.id = schema.id.to_qualified(&prefix_)?;
            }
            _ => {}
        };

        let schema_id_for_cred_def = schema.seq_no.map(|n| SchemaId(n.to_string())).unwrap_or_else(|| schema.id.clone());

        let cred_def_id = CredentialDefinitionId::new(issuer_did, &schema_id_for_cred_def, &signature_type.to_str(), tag);

        if let Ok(cred_def) = self.wallet_service.get_indy_record_value::<CredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value()) {
            return Ok((cred_def_id.0, cred_def));
        }

        let (credential_definition_value, cred_priv_key, cred_key_correctness_proof) =
            self._issuer_generate_credential_definition_keys(&schema.attr_names, cred_def_config.support_revocation).await?;

        let cred_def =
            CredentialDefinition::CredentialDefinitionV1(
                CredentialDefinitionV1 {
                    id: cred_def_id.clone(),
                    schema_id: schema_id_for_cred_def.clone(),
                    signature_type,
                    tag: tag.to_string(),
                    value: credential_definition_value,
                });

        let cred_def_priv_key = CredentialDefinitionPrivateKey {
            value: cred_priv_key
        };

        let cred_def_correctness_proof = CredentialDefinitionCorrectnessProof {
            value: cred_key_correctness_proof
        };

        let schema_ = Schema::SchemaV1(schema.clone());

        let cred_def_json = self.wallet_service.add_indy_object(wallet_handle, &cred_def_id.0, &cred_def, &HashMap::new())?;
        self.wallet_service.add_indy_object(wallet_handle, &cred_def_id.0, &cred_def_priv_key, &HashMap::new())?;
        self.wallet_service.add_indy_object(wallet_handle, &cred_def_id.0, &cred_def_correctness_proof, &HashMap::new())?;
        let _ = self.wallet_service.add_indy_object(wallet_handle, &schema_id_for_cred_def.0, &schema_, &HashMap::new()).ok();

        self._wallet_set_schema_id(wallet_handle, &cred_def_id.0, &schema.id)?; // TODO: FIXME delete temporary storing of full schema id

        debug!("create_and_store_credential_definition <<< cred_def_id: {:?}, cred_def_json: {:?}", cred_def_id, cred_def_json);
        Ok((cred_def_id.0, cred_def_json))
    }

    async fn _issuer_generate_credential_definition_keys(&self,
                                                         attr_names: &AttributeNames,
                                                         support_revocation: bool) -> IndyResult<(CredentialDefinitionData,
                                                                                    CredentialPrivateKey,
                                                                                    CredentialKeyCorrectnessProof)> {
        let attr_names = attr_names.clone();

        let (s, r) = futures::channel::oneshot::channel();
        crate::commands::THREADPOOL.lock().unwrap().execute(move || {
            let res = crate::services::anoncreds::issuer::Issuer::new_credential_definition(&attr_names, support_revocation);
            s.send(res).unwrap();
        });
        r.await?
    }

    async fn rotate_credential_definition_start(&self,
                                                wallet_handle: WalletHandle,
                                                cred_def_id: &CredentialDefinitionId,
                                                cred_def_config: Option<&CredentialDefinitionConfig>) -> IndyResult<String> {
        debug!("rotate_credential_definition_start >>> wallet_handle: {:?}, cred_def_id: {:?}, cred_def_config: {:?}",
               wallet_handle, cred_def_id, cred_def_config);

        let mut cred_def: CredentialDefinitionV1 =
            match self.wallet_service.get_indy_object::<CredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value()) {
                Ok(CredentialDefinition::CredentialDefinitionV1(cred_def)) => cred_def,
                Err(err) => return Err(err)
            };

        if let Ok(temp_cred_def) = self.wallet_service.get_indy_object::<TemporaryCredentialDefinition>(wallet_handle, &cred_def_id.0, &RecordOptions::id_value()) {
            debug!("Temporary Credential Definition already exists. Return it: {:?}", temp_cred_def.cred_def);

            let cred_def_json = ::serde_json::to_string(&temp_cred_def.cred_def)
                .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot serialize CredentialDefinition: {}", err)))?;

            return Ok(cred_def_json);
        }

        let schema: SchemaV1 =
            match self.wallet_service.get_indy_object::<Schema>(wallet_handle, &cred_def.schema_id.0, &RecordOptions::id_value()) {
                Ok(Schema::SchemaV1(schema)) => schema,
                Err(err) => return Err(err)
            };

        let support_revocation = cred_def_config.map(|config| config.support_revocation).unwrap_or_default();

        let (credential_definition_value, cred_priv_key, cred_key_correctness_proof) =
            self._issuer_generate_credential_definition_keys(&&schema.attr_names, support_revocation).await?;

        cred_def.value = credential_definition_value;
        
        let cred_def = CredentialDefinition::CredentialDefinitionV1(cred_def);

        let cred_def_priv_key = CredentialDefinitionPrivateKey {
            value: cred_priv_key
        };

        let cred_def_correctness_proof = CredentialDefinitionCorrectnessProof {
            value: cred_key_correctness_proof
        };

        let cred_def_json = ::serde_json::to_string(&cred_def)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot serialize CredentialDefinition: {}", err)))?;

        let temp_cred_def = TemporaryCredentialDefinition {
            cred_def,
            cred_def_priv_key,
            cred_def_correctness_proof,
        };

        self.wallet_service.add_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def, &HashMap::new())?;

        debug!("_rotate_credential_definition_start_complete <<< cred_def_id: {:?}, cred_def_json: {:?}", cred_def_id, cred_def_json);
        Ok(cred_def_json)
    }

    fn rotate_credential_definition_apply(&self,
                                          wallet_handle: WalletHandle,
                                          cred_def_id: &CredentialDefinitionId) -> IndyResult<()> {
        debug!("rotate_credential_definition_apply >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

        let _cred_def: CredentialDefinition = self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())?;
        let temp_cred_def: TemporaryCredentialDefinition = self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())?;

        self.wallet_service.update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def)?;
        self.wallet_service.update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def_priv_key)?;
        self.wallet_service.update_indy_object(wallet_handle, &cred_def_id.0, &temp_cred_def.cred_def_correctness_proof)?;

        self.wallet_service.delete_indy_record::<TemporaryCredentialDefinition>(wallet_handle, &cred_def_id.0)?;

        debug!("rotate_credential_definition_apply <<<");

        Ok(())
    }

    fn create_and_store_revocation_registry(&self,
                                            wallet_handle: WalletHandle,
                                            issuer_did: &DidValue,
                                            type_: Option<&str>,
                                            tag: &str,
                                            cred_def_id: &CredentialDefinitionId,
                                            config: &RevocationRegistryConfig,
                                            tails_writer_handle: i32) -> IndyResult<(String, String, String)> {
        debug!("create_and_store_revocation_registry >>> wallet_handle: {:?}, issuer_did: {:?}, type_: {:?}, tag: {:?}, cred_def_id: {:?}, config: {:?}, \
               tails_handle: {:?}", wallet_handle, issuer_did, type_, tag, cred_def_id, config, tails_writer_handle);

        match (issuer_did.get_method(), cred_def_id.get_method()) {
            (None, Some(_)) => {
                return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "You can't use unqualified Did with fully qualified Credential Definition"));
            }
            (Some(_), None) => {
                return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "You can't use fully qualified Did with unqualified Credential Definition"));
            }
            _ => {}
        };

        let rev_reg_type = if let Some(type_) = type_ {
            serde_json::from_str::<RegistryType>(&format!("\"{}\"", type_))
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid Registry Type format")?
        } else {
            RegistryType::CL_ACCUM
        };

        let issuance_type = config.issuance_type.clone().unwrap_or(IssuanceType::ISSUANCE_ON_DEMAND);
        let max_cred_num = config.max_cred_num.unwrap_or(100000);

        let rev_reg_id = RevocationRegistryId::new(&issuer_did, &cred_def_id, &rev_reg_type.to_str(), tag);

        if let (Ok(rev_reg_def), Ok(rev_reg)) = (self.wallet_service.get_indy_record_value::<RevocationRegistryDefinition>(wallet_handle, &rev_reg_id.0, &RecordOptions::id_value()),
                                                 self.wallet_service.get_indy_record_value::<RevocationRegistry>(wallet_handle, &rev_reg_id.0, &RecordOptions::id_value())) {
            return Ok((cred_def_id.0.to_string(), rev_reg_def, rev_reg));
        }

        let cred_def = match self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value()) {
            Ok(CredentialDefinition::CredentialDefinitionV1(cred_def)) => cred_def,
            Err(err) => return Err(err)
        };

        let (revoc_public_keys, revoc_key_private, revoc_registry, mut revoc_tails_generator) =
            self.anoncreds_service.issuer.new_revocation_registry(&cred_def,
                                                                  max_cred_num,
                                                                  issuance_type.to_bool(),
                                                                  &issuer_did)?;

        let (tails_location, tails_hash) =
            store_tails_from_generator(self.blob_storage_service.clone(), tails_writer_handle, &mut revoc_tails_generator)?;

        let revoc_reg_def_value = RevocationRegistryDefinitionValue {
            max_cred_num,
            issuance_type,
            public_keys: revoc_public_keys,
            tails_location,
            tails_hash,
        };

        let revoc_reg_def =
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(
                RevocationRegistryDefinitionV1 {
                    id: rev_reg_id.clone(),
                    revoc_def_type: rev_reg_type,
                    tag: tag.to_string(),
                    cred_def_id: cred_def_id.clone(),
                    value: revoc_reg_def_value,
                });

        let revoc_reg =
            RevocationRegistry::RevocationRegistryV1(
                RevocationRegistryV1 {
                    value: revoc_registry
                }
            );

        let revoc_reg_def_priv = RevocationRegistryDefinitionPrivate {
            value: revoc_key_private
        };

        let revoc_reg_def_json = self.wallet_service.add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg_def, &HashMap::new())?;

        let revoc_reg_json = self.wallet_service.add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg, &HashMap::new())?;

        self.wallet_service.add_indy_object(wallet_handle, &rev_reg_id.0, &revoc_reg_def_priv, &HashMap::new())?;

        let rev_reg_info = RevocationRegistryInfo {
            id: rev_reg_id.clone(),
            curr_id: 0,
            used_ids: HashSet::new(),
        };

        self.wallet_service.add_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info, &HashMap::new())?;

        debug!("create_and_store_revocation_registry <<< rev_reg_id: {:?}, revoc_reg_def_json: {:?}, revoc_reg_json: {:?}",
               rev_reg_id, revoc_reg_def_json, revoc_reg_json);

        Ok((rev_reg_id.0, revoc_reg_def_json, revoc_reg_json))
    }

    fn create_credential_offer(&self,
                               wallet_handle: WalletHandle,
                               cred_def_id: &CredentialDefinitionId) -> IndyResult<String> {
        debug!("create_credential_offer >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

        let cred_def_correctness_proof: CredentialDefinitionCorrectnessProof =
            self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())?;

        let nonce = new_nonce()?;

        let schema_id = self._wallet_get_schema_id(wallet_handle, &cred_def_id.0)?; // TODO: FIXME get CredDef from wallet and use CredDef.schema_id

        let credential_offer = CredentialOffer {
            schema_id,
            cred_def_id: cred_def_id.clone(),
            key_correctness_proof: cred_def_correctness_proof.value,
            nonce,
            method_name: None,
        };

        let credential_offer_json = serde_json::to_string(&credential_offer)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize CredentialOffer")?;

        debug!("create_credential_offer <<< credential_offer_json: {:?}", credential_offer_json);

        Ok(credential_offer_json)
    }

    fn new_credential(&self,
                      wallet_handle: WalletHandle,
                      cred_offer: &CredentialOffer,
                      cred_request: &CredentialRequest,
                      cred_values: &CredentialValues,
                      rev_reg_id: Option<&RevocationRegistryId>,
                      blob_storage_reader_handle: Option<i32>) -> IndyResult<(String, Option<String>, Option<String>)> {
        debug!("new_credential >>> wallet_handle: {:?}, cred_offer: {:?}, cred_req: {:?}, cred_values_json: {:?}, rev_reg_id: {:?}, blob_storage_reader_handle: {:?}",
               wallet_handle, secret!(&cred_offer), secret!(&cred_request), secret!(&cred_values), rev_reg_id, blob_storage_reader_handle);

        let cred_def_id = match cred_offer.method_name {
            Some(ref method_name) => cred_offer.cred_def_id.to_qualified(method_name)?,
            None => cred_offer.cred_def_id.clone()
        };

        let cred_def = match self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value()) {
            Ok(CredentialDefinition::CredentialDefinitionV1(cred_def)) => cred_def,
            Err(err) => return Err(err)
        };

        let cred_def_priv_key: CredentialDefinitionPrivateKey =
            self.wallet_service.get_indy_object(wallet_handle, &cred_def_id.0, &RecordOptions::id_value())?;

        let (rev_reg_def, mut rev_reg,
            rev_reg_def_priv, sdk_tails_accessor, rev_reg_info) = match rev_reg_id {
            Some(ref r_reg_id) => {
                let rev_reg_def: RevocationRegistryDefinitionV1 = match self._wallet_get_rev_reg_def(wallet_handle, &r_reg_id)? {
                    RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
                };

                let rev_reg: RevocationRegistryV1 = match self._wallet_get_rev_reg(wallet_handle, &r_reg_id)? {
                    RevocationRegistry::RevocationRegistryV1(rev_reg) => rev_reg
                };

                let rev_key_priv: RevocationRegistryDefinitionPrivate =
                    self.wallet_service.get_indy_object(wallet_handle, &r_reg_id.0, &RecordOptions::id_value())?;

                let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &r_reg_id)?;

                rev_reg_info.curr_id += 1;

                if rev_reg_info.curr_id > rev_reg_def.value.max_cred_num {
                    return Err(err_msg(IndyErrorKind::RevocationRegistryFull, "RevocationRegistryAccumulator is full"));
                }

                if rev_reg_def.value.issuance_type == IssuanceType::ISSUANCE_ON_DEMAND {
                    rev_reg_info.used_ids.insert(rev_reg_info.curr_id);
                }

                // TODO: FIXME: Review error kind!
                let blob_storage_reader_handle = blob_storage_reader_handle
                    .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "TailsReaderHandle not found"))?;

                let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                               blob_storage_reader_handle,
                                                               &rev_reg_def)?;

                (Some(rev_reg_def), Some(rev_reg), Some(rev_key_priv), Some(sdk_tails_accessor), Some(rev_reg_info))
            }
            None => (None, None, None, None, None)
        };

        let (credential_signature, signature_correctness_proof, rev_reg_delta) =
            self.anoncreds_service.issuer.new_credential(&cred_def,
                                                         &cred_def_priv_key.value,
                                                         &cred_offer.nonce,
                                                         &cred_request,
                                                         &cred_values,
                                                         rev_reg_info.as_ref().map(|r_reg_info| r_reg_info.curr_id),
                                                         rev_reg_def.as_ref(),
                                                         rev_reg.as_mut().map(|r_reg| &mut r_reg.value),
                                                         rev_reg_def_priv.as_ref().map(|r_reg_def_priv| &r_reg_def_priv.value),
                                                         sdk_tails_accessor.as_ref())?;

        let witness =
            if let (&Some(ref r_reg_def), &Some(ref r_reg), &Some(ref rev_tails_accessor), &Some(ref rev_reg_info)) =
            (&rev_reg_def, &rev_reg, &sdk_tails_accessor, &rev_reg_info) {
                let (issued, revoked) = match r_reg_def.value.issuance_type {
                    IssuanceType::ISSUANCE_ON_DEMAND => (rev_reg_info.used_ids.clone(), HashSet::new()),
                    IssuanceType::ISSUANCE_BY_DEFAULT => (HashSet::new(), rev_reg_info.used_ids.clone())
                };

                let rev_reg_delta = CryptoRevocationRegistryDelta::from_parts(None, &r_reg.value, &issued, &revoked);

                Some(Witness::new(rev_reg_info.curr_id, r_reg_def.value.max_cred_num,
                                  r_reg_def.value.issuance_type.to_bool(), &rev_reg_delta, rev_tails_accessor)?)
            } else {
                None
            };

        let cred_rev_reg_id = match (rev_reg_id, cred_offer.method_name.as_ref()) {
            (Some(rev_reg_id), Some(ref _method_name)) => Some(rev_reg_id.to_unqualified()),
            (rev_reg_id, _) => rev_reg_id.cloned()
        };

        let credential = Credential {
            schema_id: cred_offer.schema_id.clone(),
            cred_def_id: cred_offer.cred_def_id.clone(),
            rev_reg_id: cred_rev_reg_id,
            values: cred_values.clone(),
            signature: credential_signature,
            signature_correctness_proof,
            rev_reg: rev_reg.map(|r_reg| r_reg.value),
            witness,
        };

        let cred_json = serde_json::to_string(&credential)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Credential")?;

        let rev_reg_delta_json = rev_reg_delta
            .map(|r_reg_delta| RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: r_reg_delta }))
            .as_ref()
            .map(serde_json::to_string)
            .map_or(Ok(None), |v| v.map(Some))
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        if let (Some(r_reg), Some(r_reg_id), Some(r_reg_info)) = (credential.rev_reg, rev_reg_id, rev_reg_info.clone()) {
            let revoc_reg = RevocationRegistry::RevocationRegistryV1(RevocationRegistryV1 { value: r_reg });

            self.wallet_service.update_indy_object(wallet_handle, &r_reg_id.0, &revoc_reg)?;
            self.wallet_service.update_indy_object(wallet_handle, &r_reg_id.0, &r_reg_info)?;
        };

        let cred_rev_id = rev_reg_info.map(|r_reg_info| r_reg_info.curr_id.to_string());

        debug!("new_credential <<< cred_json: {:?}, cred_rev_id: {:?}, rev_reg_delta_json: {:?}", secret!(&cred_json), secret!(&cred_rev_id), rev_reg_delta_json);

        Ok((cred_json, cred_rev_id, rev_reg_delta_json))
    }

    fn revoke_credential(&self,
                         wallet_handle: WalletHandle,
                         blob_storage_reader_handle: i32,
                         rev_reg_id: &RevocationRegistryId,
                         cred_revoc_id: &str) -> IndyResult<String> {
        debug!("revoke_credential >>> wallet_handle: {:?}, blob_storage_reader_handle:  {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
               wallet_handle, blob_storage_reader_handle, rev_reg_id, secret!(cred_revoc_id));

        let cred_revoc_id = parse_cred_rev_id(cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 = match self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id)? {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
        };

        let mut rev_reg: RevocationRegistryV1 = match self._wallet_get_rev_reg(wallet_handle, &rev_reg_id)? {
            RevocationRegistry::RevocationRegistryV1(rev_reg) => rev_reg
        };

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                       blob_storage_reader_handle,
                                                       &revocation_registry_definition)?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id)?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
                };
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
                }
            }
        };

        let rev_reg_delta =
            self.anoncreds_service.issuer.revoke(&mut rev_reg.value, revocation_registry_definition.value.max_cred_num, cred_revoc_id, &sdk_tails_accessor)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: rev_reg_delta });

        let rev_reg_delta_json = serde_json::to_string(&rev_reg_delta)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self.wallet_service.update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg)?;
        self.wallet_service.update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info)?;

        debug!("revoke_credential <<< rev_reg_delta_json: {:?}", rev_reg_delta_json);

        Ok(rev_reg_delta_json)
    }

    fn _recovery_credential(&self,
                            wallet_handle: WalletHandle,
                            blob_storage_reader_handle: i32,
                            rev_reg_id: &RevocationRegistryId,
                            cred_revoc_id: &str) -> IndyResult<String> {
        debug!("recovery_credential >>> wallet_handle: {:?}, blob_storage_reader_handle: {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
               wallet_handle, blob_storage_reader_handle, rev_reg_id, secret!(cred_revoc_id));

        let cred_revoc_id = parse_cred_rev_id(cred_revoc_id)?;

        let revocation_registry_definition: RevocationRegistryDefinitionV1 = match self._wallet_get_rev_reg_def(wallet_handle, &rev_reg_id)? {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
        };

        let mut rev_reg: RevocationRegistryV1 = match self._wallet_get_rev_reg(wallet_handle, &rev_reg_id)? {
            RevocationRegistry::RevocationRegistryV1(rev_reg) => rev_reg
        };

        let sdk_tails_accessor = SDKTailsAccessor::new(self.blob_storage_service.clone(),
                                                       blob_storage_reader_handle,
                                                       &revocation_registry_definition)?;

        if cred_revoc_id > revocation_registry_definition.value.max_cred_num + 1 {
            return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
        }

        let mut rev_reg_info = self._wallet_get_rev_reg_info(wallet_handle, &rev_reg_id)?;

        match revocation_registry_definition.value.issuance_type {
            IssuanceType::ISSUANCE_ON_DEMAND => {
                if !rev_reg_info.used_ids.insert(cred_revoc_id) {
                    return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
                }
            }
            IssuanceType::ISSUANCE_BY_DEFAULT => {
                if !rev_reg_info.used_ids.remove(&cred_revoc_id) {
                    return Err(err_msg(IndyErrorKind::InvalidUserRevocId, format!("Revocation id: {:?} not found in RevocationRegistry", cred_revoc_id)));
                }
            }
        };

        let revocation_registry_delta =
            self.anoncreds_service.issuer.recovery(&mut rev_reg.value, revocation_registry_definition.value.max_cred_num, cred_revoc_id, &sdk_tails_accessor)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(RevocationRegistryDeltaV1 { value: revocation_registry_delta });

        let rev_reg_delta_json = serde_json::to_string(&rev_reg_delta)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta: {:?}")?;

        let rev_reg = RevocationRegistry::RevocationRegistryV1(rev_reg);

        self.wallet_service.update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg)?;
        self.wallet_service.update_indy_object(wallet_handle, &rev_reg_id.0, &rev_reg_info)?;

        debug!("recovery_credential <<< rev_reg_delta_json: {:?}", rev_reg_delta_json);

        Ok(rev_reg_delta_json)
    }

    fn merge_revocation_registry_deltas(&self,
                                        rev_reg_delta: &mut RevocationRegistryDeltaV1,
                                        other_rev_reg_delta: &RevocationRegistryDeltaV1) -> IndyResult<String> {
        debug!("merge_revocation_registry_deltas >>> rev_reg_delta: {:?}, other_rev_reg_delta: {:?}", rev_reg_delta, other_rev_reg_delta);

        rev_reg_delta.value.merge(&other_rev_reg_delta.value)?;

        let rev_reg_delta = RevocationRegistryDelta::RevocationRegistryDeltaV1(rev_reg_delta.clone());

        let merged_rev_reg_delta_json = serde_json::to_string(&rev_reg_delta)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?;

        debug!("merge_revocation_registry_deltas <<< merged_rev_reg_delta: {:?}", merged_rev_reg_delta_json);

        Ok(merged_rev_reg_delta_json)
    }

    // TODO: DELETE IT
    fn _wallet_set_schema_id(&self, wallet_handle: WalletHandle, id: &str, schema_id: &SchemaId) -> IndyResult<()> {
        self.wallet_service.add_record(wallet_handle, &self.wallet_service.add_prefix("SchemaId"), id, &schema_id.0, &Tags::new())
    }

    // TODO: DELETE IT
    fn _wallet_get_schema_id(&self, wallet_handle: WalletHandle, key: &str) -> IndyResult<SchemaId> {
        let schema_id_record = self.wallet_service.get_record(wallet_handle, &self.wallet_service.add_prefix("SchemaId"), &key, &RecordOptions::id_value())?;
        schema_id_record
            .get_value()
            .map(|id| SchemaId(id.to_string()))
            .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, format!("SchemaId not found for id: {}", key)))
    }

    fn _wallet_get_rev_reg_def(&self, wallet_handle: WalletHandle, key: &RevocationRegistryId) -> IndyResult<RevocationRegistryDefinition> {
        self.wallet_service.get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
    }

    fn _wallet_get_rev_reg(&self, wallet_handle: WalletHandle, key: &RevocationRegistryId) -> IndyResult<RevocationRegistry> {
        self.wallet_service.get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
    }

    fn _wallet_get_rev_reg_info(&self, wallet_handle: WalletHandle, key: &RevocationRegistryId) -> IndyResult<RevocationRegistryInfo> {
        self.wallet_service.get_indy_object(wallet_handle, &key.0, &RecordOptions::id_value())
    }
}
