extern crate futures;

use indy::IndyError;
use indy::anoncreds;
use crate::indy::future::Future;
use serde_json;

use crate::utils::{environment, wallet, blob_storage, test};
use crate::utils::types::CredentialOfferInfo;

use std::sync::Once;
use std::mem;
use crate::utils::constants::*;

use std::collections::{HashSet, HashMap};

use crate::utils::domain::anoncreds::schema::{Schema, SchemaV1, SchemaId};
use crate::utils::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionConfig, CredentialDefinitionId};
use crate::utils::domain::anoncreds::revocation_registry_definition::{RevocationRegistryConfig, IssuanceType, RevocationRegistryId};
use crate::utils::domain::anoncreds::credential::{AttributeValues, CredentialInfo};
use crate::utils::domain::anoncreds::credential_for_proof_request::CredentialsForProofRequest;
use crate::utils::domain::crypto::did::DidValue;

use indy::WalletHandle;

pub static mut CREDENTIAL_DEF_JSON: &'static str = "";
pub static mut CREDENTIAL_OFFER_JSON: &'static str = "";
pub static mut CREDENTIAL_REQUEST_JSON: &'static str = "";
pub static mut CREDENTIAL_JSON: &'static str = "";
pub const ANONCREDS_WALLET_CONFIG: &'static str = r#"{"id": "anoncreds_wallet"}"#;
pub const COMMON_MASTER_SECRET: &'static str = "common_master_secret_name";
pub const CREDENTIAL1_ID: &'static str = "credential1_id";
pub const CREDENTIAL1_SUB_ID: &'static str = "credential1_sub_id";
pub const CREDENTIAL2_ID: &'static str = "credential2_id";
pub const CREDENTIAL3_ID: &'static str = "credential3_id";
pub const DELIMITER: &'static str = ":";
pub const CRED_DEF_MARKER: &'static str = "3";

macro_rules! map (
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

pub fn issuer_create_schema(issuer_did: &str, name: &str, version: &str, attr_names: &str) -> Result<(String, String), IndyError> {
    anoncreds::issuer_create_schema(issuer_did, name, version, attr_names).wait()
}

pub fn issuer_create_credential_definition(wallet_handle: WalletHandle, issuer_did: &str, schema: &str, tag: &str,
                                           signature_type: Option<&str>, config: Option<&str>) -> Result<(String, String), IndyError> {
    anoncreds::issuer_create_and_store_credential_def(wallet_handle, issuer_did, schema, tag, signature_type, config.unwrap_or("{}")).wait() // TODO: FIXME OPTIONAL CONFIG
}

pub fn issuer_rotate_credential_def_start(wallet_handle: WalletHandle, cred_def_id: &str, config_json: Option<&str>) -> Result<String, IndyError> {
    anoncreds::issuer_rotate_credential_def_start(wallet_handle, cred_def_id, config_json).wait()
}

pub fn issuer_rotate_credential_def_apply(wallet_handle: WalletHandle, cred_def_id: &str) -> Result<(), IndyError> {
    anoncreds::issuer_rotate_credential_def_apply(wallet_handle, cred_def_id).wait()
}

pub fn issuer_create_and_store_revoc_reg(wallet_handle: WalletHandle, issuer_did: &str, type_: Option<&str>, tag: &str,
                                         cred_def_id: &str, config_json: &str, tails_writer_handle: i32)
                                         -> Result<(String, String, String), IndyError> {
    anoncreds::issuer_create_and_store_revoc_reg(wallet_handle, issuer_did, type_, tag, cred_def_id, config_json, tails_writer_handle).wait()
}

pub fn issuer_create_credential_offer(wallet_handle: WalletHandle, cred_def_id: &str) -> Result<String, IndyError> {
    anoncreds::issuer_create_credential_offer(wallet_handle, cred_def_id).wait()
}

pub fn issuer_create_credential(wallet_handle: WalletHandle, cred_offer_json: &str, cred_req_json: &str, cred_values_json: &str,
                                rev_reg_id: Option<&str>, blob_storage_reader_handle: Option<i32>) -> Result<(String, Option<String>, Option<String>), IndyError> {
    anoncreds::issuer_create_credential(wallet_handle, cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, blob_storage_reader_handle.unwrap_or(-1)).wait() // TODO OPTIONAL blob_storage_reader_handle
}

pub fn issuer_revoke_credential(wallet_handle: WalletHandle, blob_storage_reader_handle: i32, rev_reg_id: &str, cred_revoc_id: &str) -> Result<String, IndyError> {
    anoncreds::issuer_revoke_credential(wallet_handle, blob_storage_reader_handle, rev_reg_id, cred_revoc_id).wait()
}

pub fn issuer_merge_revocation_registry_deltas(rev_reg_delta: &str, other_rev_reg_delta: &str) -> Result<String, IndyError> {
    anoncreds::issuer_merge_revocation_registry_deltas(rev_reg_delta, other_rev_reg_delta).wait()
}

pub fn prover_create_master_secret(wallet_handle: WalletHandle, master_secret_id: &str) -> Result<String, IndyError> {
    anoncreds::prover_create_master_secret(wallet_handle, Some(master_secret_id)).wait()
}

pub fn prover_create_credential_req(wallet_handle: WalletHandle, prover_did: &str, cred_offer_json: &str,
                                    cred_def_json: &str, master_secret_id: &str) -> Result<(String, String), IndyError> {
    anoncreds::prover_create_credential_req(wallet_handle, prover_did, cred_offer_json, cred_def_json, master_secret_id).wait()
}

pub fn prover_set_credential_attr_tag_policy(wallet_handle: WalletHandle, cred_def_id: &str, tag_attrs_json: Option<&str>,
                                             retroactive: bool) -> Result<(), IndyError> {
    anoncreds::prover_set_credential_attr_tag_policy(wallet_handle, cred_def_id, tag_attrs_json, retroactive).wait()
}

pub fn prover_get_credential_attr_tag_policy(wallet_handle: WalletHandle, cred_def_id: &str) -> Result<String, IndyError> {
    anoncreds::prover_get_credential_attr_tag_policy(wallet_handle, cred_def_id).wait()
}

pub fn prover_store_credential(wallet_handle: WalletHandle, cred_id: &str, cred_req_metadata_json: &str, cred_json: &str,
                               cred_def_json: &str, rev_reg_def_json: Option<&str>) -> Result<String, IndyError> {
    anoncreds::prover_store_credential(wallet_handle, Some(cred_id), cred_req_metadata_json, cred_json, cred_def_json, rev_reg_def_json).wait()
}

pub fn prover_delete_credential(wallet_handle: WalletHandle, cred_id: &str) -> Result<(), IndyError> {
    anoncreds::prover_delete_credential(wallet_handle, cred_id).wait()
}

//TODO mark as deprecated and use only in target tests
pub fn prover_get_credentials(wallet_handle: WalletHandle, filter_json: &str) -> Result<String, IndyError> {
    anoncreds::prover_get_credentials(wallet_handle, Some(filter_json)).wait()
}

pub fn prover_get_credential(wallet_handle: WalletHandle, cred_id: &str) -> Result<String, IndyError> {
    anoncreds::prover_get_credential(wallet_handle, cred_id).wait()
}

pub fn prover_search_credentials(wallet_handle: WalletHandle, filter_json: &str) -> Result<(i32, usize), IndyError> {
    anoncreds::prover_search_credentials(wallet_handle, Some(filter_json)).wait()
}

pub fn prover_fetch_credentials(search_handle: i32, count: usize) -> Result<String, IndyError> {
    anoncreds::prover_fetch_credentials(search_handle, count).wait()
}

pub fn prover_close_credentials_search(search_handle: i32) -> Result<(), IndyError> {
    anoncreds::prover_close_credentials_search(search_handle).wait()
}

//TODO mark as deprecated and use only in target tests
pub fn prover_get_credentials_for_proof_req(wallet_handle: WalletHandle, proof_request_json: &str) -> Result<String, IndyError> {
    anoncreds::prover_get_credentials_for_proof_req(wallet_handle, proof_request_json).wait()
}

pub fn prover_search_credentials_for_proof_req(wallet_handle: WalletHandle, proof_request_json: &str, extra_query_json: Option<&str>) -> Result<i32, IndyError> {
    anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_request_json, extra_query_json).wait()
}

pub fn prover_fetch_next_credentials_for_proof_req(search_handle: i32, item_ref: &str, count: usize) -> Result<String, IndyError> {
    anoncreds::prover_fetch_credentials_for_proof_req(search_handle, item_ref, count).wait()
}

pub fn prover_close_credentials_search_for_proof_req(search_handle: i32) -> Result<(), IndyError> {
    anoncreds::prover_close_credentials_search_for_proof_req(search_handle).wait()
}

pub fn prover_create_proof(wallet_handle: WalletHandle, proof_req_json: &str, requested_credentials_json: &str,
                           master_secret_name: &str, schemas_json: &str, cred_defs_json: &str,
                           rev_states_json: &str) -> Result<String, IndyError> {
    anoncreds::prover_create_proof(wallet_handle, proof_req_json, requested_credentials_json,
                                   master_secret_name, schemas_json, cred_defs_json, rev_states_json).wait()
}

pub fn verifier_verify_proof(proof_request_json: &str, proof_json: &str, schemas_json: &str,
                             cred_defs_json: &str, rev_reg_defs_json: &str, rev_regs_json: &str) -> Result<bool, IndyError> {
    anoncreds::verifier_verify_proof(proof_request_json, proof_json, schemas_json, cred_defs_json, rev_reg_defs_json, rev_regs_json).wait()
}

pub fn create_revocation_state(blob_storage_reader_handle: i32, rev_reg_def_json: &str,
                               rev_reg_delta_json: &str, timestamp: u64, cred_rev_id: &str) -> Result<String, IndyError> {
    anoncreds::create_revocation_state(blob_storage_reader_handle, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id).wait()
}

pub fn update_revocation_state(tails_reader_handle: i32, rev_state_json: &str, rev_reg_def_json: &str,
                               rev_reg_delta_json: &str, timestamp: u64, cred_rev_id: &str) -> Result<String, IndyError> {
    anoncreds::update_revocation_state(tails_reader_handle, rev_state_json, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id).wait()
}

pub fn generate_nonce() -> Result<String, IndyError> {
    anoncreds::generate_nonce().wait()
}

pub fn to_unqualified(entity: &str) -> Result<String, IndyError> {
    anoncreds::to_unqualified(entity).wait()
}

pub fn default_cred_def_config() -> String {
    serde_json::to_string(&CredentialDefinitionConfig { support_revocation: false }).unwrap()
}

pub fn revocation_cred_def_config() -> String {
    serde_json::to_string(&CredentialDefinitionConfig { support_revocation: true }).unwrap()
}

pub fn issuance_on_demand_rev_reg_config() -> String {
    serde_json::to_string(&RevocationRegistryConfig { max_cred_num: Some(5), issuance_type: None }).unwrap()
}

pub fn issuance_by_default_rev_reg_config() -> String {
    serde_json::to_string(&RevocationRegistryConfig { max_cred_num: Some(5), issuance_type: Some(IssuanceType::ISSUANCE_BY_DEFAULT) }).unwrap()
}

pub fn gvt_schema_id() -> String {
    SchemaId::new(&DidValue(ISSUER_DID.to_string()), GVT_SCHEMA_NAME, SCHEMA_VERSION).0
}

pub fn gvt_sub_schema_id() -> String {
    SchemaId::new(&DidValue(ISSUER_DID_2.to_string()), GVT_SUB_SCHEMA_NAME, SCHEMA_SUB_VERSION).0
}

pub fn gvt_schema_id_fully_qualified() -> String {
    SchemaId::new(&DidValue(ISSUER_DID_V1.to_string()), GVT_SCHEMA_NAME, SCHEMA_VERSION).0
}

pub fn gvt_cred_def_id() -> String {
    CredentialDefinitionId::new(&DidValue(ISSUER_DID.to_string()), &SchemaId(SEQ_NO.to_string()), SIGNATURE_TYPE, TAG_1).0
}

pub fn local_gvt_cred_def_id() -> String {
    CredentialDefinitionId::new(&DidValue(ISSUER_DID.to_string()), &SchemaId(gvt_schema_id()), SIGNATURE_TYPE, TAG_1).0
}

pub fn gvt_cred_def_id_fully_qualified() -> String {
    CredentialDefinitionId::new(&DidValue(ISSUER_DID_V1.to_string()), &SchemaId(SEQ_NO.to_string()), SIGNATURE_TYPE, TAG_1).0
}

pub fn local_gvt_cred_def_id_fully_qualified() -> String {
    CredentialDefinitionId::new(&DidValue(ISSUER_DID_V1.to_string()), &SchemaId(gvt_schema_id_fully_qualified()), SIGNATURE_TYPE, TAG_1).0
}

pub fn gvt_rev_reg_id() -> String {
    RevocationRegistryId::new(&DidValue(ISSUER_DID.to_string()), &CredentialDefinitionId(gvt_cred_def_id()), REVOC_REG_TYPE, TAG_1).0
}

pub fn gvt_rev_reg_id_fully_qualified() -> String {
    RevocationRegistryId::new(&DidValue(ISSUER_DID_V1.to_string()), &CredentialDefinitionId(gvt_cred_def_id()), REVOC_REG_TYPE, TAG_1).0
}

pub fn gvt_schema() -> SchemaV1 {
    SchemaV1 {
        id: SchemaId(gvt_schema_id()),
        version: SCHEMA_VERSION.to_string(),
        name: GVT_SCHEMA_NAME.to_string(),
        attr_names: serde_json::from_str::<HashSet<String>>(GVT_SCHEMA_ATTRIBUTES).unwrap().into(),
        seq_no: None,
    }
}

pub fn gvt_sub_schema() -> SchemaV1 {
    SchemaV1 {
        id: SchemaId(gvt_sub_schema_id()),
        version: SCHEMA_SUB_VERSION.to_string(),
        name: GVT_SUB_SCHEMA_NAME.to_string(),
        attr_names: serde_json::from_str::<HashSet<String>>(GVT_SUB_SCHEMA_ATTRIBUTES).unwrap().into(),
        seq_no: None,
    }
}

pub fn gvt_schema_json() -> String {
    serde_json::to_string(&Schema::SchemaV1(gvt_schema())).unwrap()
}

pub fn gvt_sub_schema_json() -> String {
    serde_json::to_string(&Schema::SchemaV1(gvt_sub_schema())).unwrap()
}

pub fn gvt_schema_id_issuer2() -> String {
    SchemaId::new(&DidValue(ISSUER_DID_2.to_string()), GVT_SCHEMA_NAME, SCHEMA_VERSION).0
}

pub fn gvt_schema_issuer2() -> SchemaV1 {
    SchemaV1 {
        id: SchemaId(gvt_schema_id_issuer2()),
        version: SCHEMA_VERSION.to_string(),
        name: GVT_SCHEMA_NAME.to_string(),
        attr_names: serde_json::from_str::<HashSet<String>>(GVT_SCHEMA_ATTRIBUTES).unwrap().into(),
        seq_no: None,
    }
}

pub fn gvt_schema_issuer2_json() -> String {
    serde_json::to_string(&Schema::SchemaV1(gvt_schema_issuer2())).unwrap()
}


pub fn xyz_schema_id() -> String {
    SchemaId::new(&DidValue(ISSUER_DID.to_string()), XYZ_SCHEMA_NAME, SCHEMA_VERSION).0
}

pub fn xyz_schema() -> SchemaV1 {
    SchemaV1 {
        id: SchemaId(xyz_schema_id()),
        version: SCHEMA_VERSION.to_string(),
        name: XYZ_SCHEMA_NAME.to_string(),
        attr_names: serde_json::from_str::<HashSet<String>>(XYZ_SCHEMA_ATTRIBUTES).unwrap().into(),
        seq_no: None,
    }
}

pub fn xyz_schema_json() -> String {
    serde_json::to_string(&Schema::SchemaV1(xyz_schema())).unwrap()
}

pub fn xyz_schema_id_tag2() -> String {
    SchemaId::new(&DidValue(ISSUER_DID.to_string()), &format!("{}{}", XYZ_SCHEMA_NAME, TAG_2), SCHEMA_VERSION).0
}

pub fn xyz_schema_tag2() -> SchemaV1 {
    SchemaV1 {
        id: SchemaId(xyz_schema_id_tag2()),
        version: SCHEMA_VERSION.to_string(),
        name: format!("{}{}", XYZ_SCHEMA_NAME, TAG_2),
        attr_names: serde_json::from_str::<HashSet<String>>(XYZ_SCHEMA_ATTRIBUTES).unwrap().into(),
        seq_no: None,
    }
}

pub fn xyz_schema_tag2_json() -> String {
    serde_json::to_string(&Schema::SchemaV1(xyz_schema_tag2())).unwrap()
}

pub fn cred_def_id(did: &str, schema_id: &str, signature_type: &str, tag: &str) -> String {
    format!("{}{}{}{}{}{}{}{}{}", did, DELIMITER, CRED_DEF_MARKER, DELIMITER, signature_type, DELIMITER, schema_id, DELIMITER, tag)
}

pub fn issuer_1_gvt_cred_def_id() -> String {
    cred_def_id(ISSUER_DID, &gvt_schema_id(), SIGNATURE_TYPE, TAG_1)
}

pub fn issuer_2_gvt_cred_def_id() -> String {
    cred_def_id(ISSUER_DID_2, &gvt_schema_id(), SIGNATURE_TYPE, TAG_1)
}

pub fn issuer_1_xyz_cred_def_id() -> String {
    cred_def_id(ISSUER_DID, &xyz_schema_id(), SIGNATURE_TYPE, TAG_1)
}

pub fn issuer_1_xyz_tag2_cred_def_id() -> String {
    cred_def_id(ISSUER_DID, &xyz_schema_id_tag2(), SIGNATURE_TYPE, TAG_2)
}

pub fn issuer_1_gvt_cred_offer_info() -> CredentialOfferInfo {
    CredentialOfferInfo { cred_def_id: issuer_1_gvt_cred_def_id() }
}

pub fn issuer_1_xyz_cred_offer_info() -> CredentialOfferInfo {
    CredentialOfferInfo { cred_def_id: issuer_1_xyz_cred_def_id() }
}

pub fn issuer_2_gvt_cred_offer_info() -> CredentialOfferInfo {
    CredentialOfferInfo { cred_def_id: issuer_2_gvt_cred_def_id() }
}

// note that encoding is not standardized by Indy except that 32-bit integers are encoded as themselves. IS-786
pub fn gvt_credential_values() -> HashMap<String, AttributeValues> {
    map! {
            "sex".to_string() => AttributeValues {raw: "male".to_string(), encoded: "5944657099558967239210949258394887428692050081607692519917050011144233115103".to_string()},
            "name".to_string() => AttributeValues {raw: "Alex".to_string(), encoded: "1139481716457488690172217916278103335".to_string()},
            "height".to_string() => AttributeValues {raw: "175".to_string(), encoded: "175".to_string()},
            "age".to_string() => AttributeValues {raw: "28".to_string(), encoded: "28".to_string()}
          }
}

pub fn gvt_credential_values_json() -> String {
    serde_json::to_string(&gvt_credential_values()).unwrap()
}

pub fn gvt_credential_values_2() -> HashMap<String, AttributeValues> {
    map! {
            "sex".to_string() => AttributeValues {raw: "female".to_string(), encoded: "5944657099558967239210949258394887428692050081607692519917050011144233115103".to_string()},
            "name".to_string() => AttributeValues {raw: "Alec".to_string(), encoded: "1139481716457488690172217916278103335".to_string()},
            "height".to_string() => AttributeValues {raw: "155".to_string(), encoded: "155".to_string()},
            "age".to_string() => AttributeValues {raw: "28".to_string(), encoded: "28".to_string()}
          }
}

pub fn gvt_credential_values_2_json() -> String {
    serde_json::to_string(&gvt_credential_values_2()).unwrap()
}

pub fn gvt_sub_credential_values() -> HashMap<String, AttributeValues> {
    map! {
            "sex".to_string() => AttributeValues {raw: "male".to_string(), encoded: "5944657099558967239210949258394887428692050081607692519917050011144233115103".to_string()},
            "height_sub".to_string() => AttributeValues {raw: "175".to_string(), encoded: "175".to_string()}
          }
}

pub fn gvt_sub_credential_values_json() -> String {
    serde_json::to_string(&gvt_sub_credential_values()).unwrap()
}

pub fn xyz_credential_values() -> HashMap<String, AttributeValues> {
    map! {
            "status".to_string() => AttributeValues {raw: "partial".to_string(), encoded: "51792877103171595686471452153480627530895".to_string()},
            "period".to_string() => AttributeValues {raw: "8".to_string(), encoded: "8".to_string()}
          }
}

pub fn xyz_credential_values_json() -> String {
    serde_json::to_string(&xyz_credential_values()).unwrap()
}

pub fn gvt2_credential_values() -> HashMap<String, AttributeValues> {
    map! {
            "sex".to_string() => AttributeValues {raw: "male".to_string(), encoded: "2142657394558967239210949258394838228692050081607692519917028371144233115103".to_string()},
            "name".to_string() => AttributeValues {raw: "Alexander".to_string(), encoded: "21332817548165488690172217217278169335".to_string()},
            "height".to_string() => AttributeValues {raw: "170".to_string(), encoded: "170".to_string()},
            "Age".to_string() => AttributeValues {raw: "28".to_string(), encoded: "28".to_string()}
          }
}

pub fn gvt2_credential_values_json() -> String {
    serde_json::to_string(&gvt2_credential_values()).unwrap()
}

pub fn gvt3_credential_values() -> HashMap<String, AttributeValues> {
    map! {
            "sex".to_string() => AttributeValues {raw: "male".to_string(), encoded: "1234567890442222223345678958394838228692050081607692519917028371144233115103".to_string()},
            "name".to_string() => AttributeValues {raw: "Artem".to_string(), encoded: "12356325715837025980172217217278169335".to_string()},
            "height".to_string() => AttributeValues {raw: "180".to_string(), encoded: "180".to_string()},
            "age".to_string() => AttributeValues {raw: "25".to_string(), encoded: "25".to_string()}
          }
}

pub fn gvt3_credential_values_json() -> String {
    serde_json::to_string(&gvt3_credential_values()).unwrap()
}

pub fn issuer_1_gvt_credential() -> CredentialInfo {
    CredentialInfo {
        schema_id: SchemaId(gvt_schema_id()),
        cred_def_id: CredentialDefinitionId(issuer_1_gvt_cred_def_id()),
        referent: CREDENTIAL1_ID.to_string(),
        rev_reg_id: None,
        cred_rev_id: None,
        attrs: map! {
                       "sex".to_string() => "male".to_string(),
                       "name".to_string() => "Alex".to_string(),
                       "height".to_string() => "175".to_string(),
                       "age".to_string() => "28".to_string()
                   },
    }
}

pub fn issuer_1_xyz_credential() -> CredentialInfo {
    CredentialInfo {
        schema_id: SchemaId(xyz_schema_id()),
        cred_def_id: CredentialDefinitionId(issuer_1_xyz_cred_def_id()),
        referent: CREDENTIAL2_ID.to_string(),
        rev_reg_id: None,
        cred_rev_id: None,
        attrs: map! {
                       "status".to_string() => "partial".to_string(),
                       "period".to_string() => "8".to_string()
                   },
    }
}

pub fn issuer_2_gvt_credential() -> CredentialInfo {
    CredentialInfo {
        schema_id: SchemaId(gvt_schema_id()),
        cred_def_id: CredentialDefinitionId(issuer_2_gvt_cred_def_id()),
        referent: CREDENTIAL3_ID.to_string(),
        rev_reg_id: None,
        cred_rev_id: None,
        attrs: map! {
                       "sex".to_string() => "male".to_string(),
                       "name".to_string() => "Alexander".to_string(),
                       "height".to_string() => "170".to_string(),
                       "Age".to_string() => "28".to_string()
                   },
    }
}

pub fn credential_def_json() -> String {
    r#"{
           "ver":"1.0",
           "id":"NcYxiDXkpYi6ov5FcYDi1e:3:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:CL:TAG_1",
           "schemaId":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
           "type":"CL",
           "tag":"TAG_1",
           "value":{
              "primary":{
                 "n":"94752773003676215520340390286428145970577435379747248974837494389412082076547661891067434652276048522392442077335235388384984508621151996372559370276527598415204914831299768834758349425880859567795461321350412568232531440683627330032285846734752711268206613305069973750567165548816744023441650243801226580089078611213688037852063937259593837571943085718154394160122127891902723469618952030300431400181642597638732611518885616750614674142486169255034160093153314427704384760404032620300207070597238445621198019686315730573836193179483581719638565112589368474184957790046080767607443902003396643479910885086397579016949",
                 "s":"69412039600361800795429063472749802282903100455399422661844374992112119187258494682747330126416608111152308407310993289705267392969490079422545377823004584691698371089275086755756916575365439635768831063415050875440259347714303092581127338698890829662982679857654396534761554232914231213603075653629534596880597317047082696083166437821687405393805812336036647064899914817619861844092002636340952247588092904075021313598848481976631171767602864723880294787434756140969093416957086578979859382777377267118038126527549503876861370823520292585383483415337137062969402135540724590433024573312636828352734474276871187481042",
                 "r":{
                    "master_secret": "51663676247842478814965591806476166314018329779100758392678204435864101706276421100107118776199283981546682625125866769910726045178868995629346547166162207336629797340989495021248125384357605197654315399409367101440127312902706857104045262430326903112478154165057770802221835566137181123204394005042244715693211063132775814710986488082414421678086296488865286754803461178476006057306298883090062534704773627985221339716152111236985859907502262026150818487846053415153813804554830872575193396851274528558072704096323791923604931528594861707067370303707070124331485728734993074005001622035563911923643592706985074084035",
                    "age":"90213462228557102785520674066817329607065098280886260103565465379328385444439123494955469500769864345819799623656302322427095342533906338563811194606234218499052997878891037890681314502037670093285650999142741875494918117023196753133733183769000368858655309319559871473827485381905587653145346258174022279515774231018893119774525087260785417971477049379955435611260162822960318458092151247522911151421981946748062572207451174079699745404644326303405628719711440096340436702151418321760375229323874027809433387030362543124015034968644213166988773750220839778654632868402703075643503247560457217265822566406481434257658",
                    "height":"5391629214047043372090966654120333203094518833743674393685635640778311836867622750170495792524304436281896432811455146477306501487333852472234525296058562723428516533641819658096275918819548576029252844651857904411902677509566190811985500618327955392620642519618001469964706236997279744030829811760566269297728600224591162795849338756438466021999870256717098048301453122263380103723520670896747657149140787953289875480355961166269553534983692005983375091110745903845958291035125718192228291126861666488320123420563113398593180368102996188897121307947248313167444374640621348136184583596487812048321382789134349482978",
                    "name":"77620276231641170120118188540269028385259155493880444038204934044861538875241492581309232702380290690573764595644801264135299029620031922004969464948925209245961139274806949465303313280327009910224580146266877846633558282936147503639084871235301887617650455108586169172459479774206351621894071684884758716731250212971549835402948093455393537573942251389197338609379019568250835525301455105289583537704528678164781839386485243301381405947043141406604458853106372019953011725448481499511842635580639867624862131749700424467221215201558826025502015289693451254344465767556321748122037274143231500322140291667454975911415",
                    "sex":"9589127953934298285127566793382980040568251918610023890115614786922171891298122457059996745443282235104668609426602496632245081143706804923757991602521162900045665258654877250328921570207935035808607238170708932487500434929591458680514420504595293934408583558084774019418964434729989362874165849497341625769388145344718883550286508846516335790153998186614300493752317413537864956171451048868305380731285315760405126912629495204641829764230906698870575251861738847175174907714361155400020318026100833368698707674675548636610079631382774152211885405135045997623813094890524761824654025566099289284433567918244183562578"
                 },
                 "rctxt":"60293229766149238310917923493206871325969738638348535857162249827595080348039120693847207728852550647187915587987334466582959087190830489258423645708276339586344792464665557038628519694583193692804909304334143467285824750999826903922956158114736424517794036832742439893595716442609416914557200249087236453529632524328334442017327755310827841619727229956823928475210644630763245343116656886668444813463622336899670813312626960927341115875144198394937398391514458462051400588820774593570752884252721428948286332429715774158007033348855655388287735570407811513582431434394169600082273657382209764160600063473877124656503",
                 "z":"70486542646006986754234343446999146345523665952265004264483059055307042644604796098478326629348068818272043688144751523020343994424262034067120716287162029288580118176972850899641747743901392814182335879624697285262287085187745166728443417803755667806532945136078671895589773743252882095592683767377435647759252676700424432160196120135306640079450582642553870190550840243254909737360996391470076977433525925799327058405911708739601511578904084479784054523375804238021939950198346585735956776232824298799161587408330541161160988641895300133750453032202142977745163418534140360029475702333980267724847703258887949227842"
              },
              "revocation":null
           }
        }"#.to_string()
}

pub fn issuer_1_gvt_cred_def_json() -> String {
    json!({
          "id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
          "schemaId":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
          "tag":"TAG_1",
          "type":"CL",
          "value":{
             "primary":{
                "n":"92350374552839711400010998051856168880584623107816629542564859679141348100181549143955937805002883786736648893833454403771476469383232073357892638870522820029784272548590228895084155575606514130118020198787705474009349819876316593979253296603642666882905514401563072757620597159426006843796025556364548723315647036196641703511022140487745501090278947088279592905347698356768668351916634485185842460391518040277900236416250492234752113280064552920729758884891886502956276819820126337763434447198740566425391248198739104356061857562286871767142382737736444469253191071167379320037202943385470678785290657975805052023941",
                "r":{
                   "age":"56744856453948477170533578260879967221884864286831348344344408204742986611702847911357864034173168603640796491921479068793244397873278658833713438597855106810406760853551357767941229648944280184284278272963271582050145898079652302868934980507300722507295303685266522645266384766632223322090849818322615290260417931178643051352003166572173402164394028350485391130566809125877870361898805306762759473790067325155247385062144666641639686004567563873394978319123455691967905556704825241167363302608510900942945867105695360796479987872521208627679663384733105423983249650701651252415025032599703671624904348318438414383767",
                   "height":"26847920967328338262507189125449220294281092907193606305769992642866869721505887189808078570896263006705032785010296507768074987423698680834493611746579188808622353418309656984722074844075820540162412729252330258327237875787210747300163823713702027817816500551636170593905747948736295353522746955597949371343251947489147485721093912804552771453629787501613256623943053201905973732220291224899748186197299795812325065510600027292429956937331590780644140627422046207471511330931069029509141480426186480414921222917478644072751862249471296940647373102331663489086555762682093326139884206212252251378581788546483220353079",
                   "master_secret":"70988433464940766268407551576973657153516298939485026181977433223057892371092033086180755947967070294538716059449508499397830039018210336004391199838188382656652385788678102251175426622481962084821317066357001086563010133540992958980642547233540116320776300278487098749382786902013030168984540628882684060755610387590081267968213212289910768102473813739269409358089569190891858482484563792114827737922811444113773971708809192907392341972165701257794500866056985994756879576261044526041343596637944026899556988967754465110776925604868780626335608831509094221289701497028509009384505381834129664641480044544102348830695",
                   "name":"41224884122050869562389782028967261357596711257993580928756838679204666462756730587839580429858365677027067949847820701274103755876710990189783581347404054941336896355902407052784066075594123268429274960800940249127820871136800961591780090828737842221547099770387740175241499315443685590041374424981311517782821606534919034726155781822058075264208926534036346165270616244647943016932812464243989681814521474373295879073368508345574942371011896357337811554191048750105737227920993112775217086021668796606191060810731902926747642895328410250642616922782889572147022788135721976391010702281183849336625313308144668138013",
                   "sex":"6674802470582167326843092743823994587067551251262422924106220739678061620522048841669855856205426549670071698080378167293469762159490214625046377504229023747176316026897530155217274654342151672399574439903463965632027706685280606587045413050071028106687859072966249995391333482009288577339160990316853910291950441638587874595572563536209747629803332150050320247778950397884631416759155331235246178949697859031254624609981857389092553586845455281170446911342243056372676189854493117170283378926713501269516004351944305429641556975863969278160227907714866384312498687666925505371078195830881180635119626925387445952370"
                },
                "rctxt":"80256483299943095884634993874248283229784738793094932755433363612469268897290650801602238331452123234100440011230231177641656003762750536295768683834876044155394709468330964144711998099847416804964531920000803353438848152219617280192604127097111649248830719913695728782026157708367770322299084538779976145570086949853075155632658215534626004659852016669386690435795549394052014734109683952066232988535088856774822306472127883182615281443702827941187463248746821609501020803033307124688146854012774096161864917763450904434828571398210823626796296873822343803468504003607756118138800619690574731362402156516594086826019",
                "s":"57531889362021260733051849370093219544090717562710935492916725894803707633346369458172610917056310876885189868767106864218754286501245499248513385371780502925568275701219480865926301222693275123860607832838649162719104555469887137045686990892857687146478532170753165132849950649409812078179496283328726247407210324508283340284500410284051719574586043236716276500805943747148735488838576721178987379048385111817459078606659827323290670697894154395235975083192995104037359168869289176490145923471514594307829928467274364547128765042798633271568491565637905178435789138732582204809817842620153323585953575587256862182317",
                "z":"81217500381312052618543400981090510768794290034027411760608534227994901040989828076565896133209107327121264881341339577114883968481760309966418055530259108826574952805013101284242332819994946907556079171137168220985785116107656188553635207754756879960113569769664212882242526141538844378478517203608306167109685658425878729108709729658212405864805416277755852243635231804897485863097808438751169861235714919375773413183617794273737328874238378299805427306076230159889301296456440192928932607169941148542211969442689521577002066004562153599913676231172806680723796660532166770177600916529585380339902441978997559177464"
             }
          },
          "ver":"1.0"
       }).to_string()
}

// This uses gvt schema issued by ISSUER_2
pub fn issuer_2_gvt_cred_def_json() -> String {
    json!({
        "id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:3:CL:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0:TAG_1",
        "schemaId":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0",
        "tag":"TAG_1",
        "type":"CL",
        "value":{
        "primary":{
            "n":"105652611204686043309922821126508962155697601118559227056651896675004245085511617901139816028132828658211841318878945231982222798201153720957080769197636939737674936064728954030108242180926535040235724701912155675053756699726502981311695235589984722082396337758689584638894165647537400312129807627723809348017788136886133248621497677261444920139969688824377233320568582359948230972704592062104778976992821144202735798956389939732670907210927032557956581807374475763705511016743121637603057085947094745597848695746234008270581917998759326422454711501005953352706174562013046924258942608822133093279093585098890810161173",
            "r":{
                "age":"36030572121980916637268508219046971020099942718763613419395831969809076673639205025941593283718875018159238891157533367054487124851216972873524572900461622012785749684126364325471393434394456233915014679770933909464886422420245161275385974478130269660730861645429076418692313956192802527393016981153052930399732055611428466372636122076853863517749314988995773103402423537104198707337275827396876671199861449444720167861638143531969062475619057801121698308258907809872053157624522826470448183786089749600364717003539094852213884558931232005479546558196663960440101409361329023680731148325590570733229857355571254652138",
                "height":"67617743980960110485749237184457069144060902812167539544596312128151021120590474118069588872834320293401670678798425816372618268567357165459503768606850407797022555152644291874170447706915297916204629862436338587074334624150884326293446873369208868982907668442312576951074701196034996080379451616012839030239191210050952217795266367399351105935915252760596774326136072937642459958267665608914685098241268983874807695350367722244223278436943879759641743576749904402428968901788273109735519935248594648499219540426990916688716680327760235337679620522418026689288728927998546322338675064317755438034661836342590062246602",
                "master_secret":"40970967747266756141163774882680209218168806343297701738766495424704898210673680431359619459816267989090620455448383743975291970075364711557973355188062157074377990819108995637823309505559344448249088757017930790634673396513757324918594694433017924365765087193500371384233711392853601652226824761634502902602814057963348564299474244041847254947360938657112836872034975803064021094219683230445241466305709676023877777127650672346327975305913446725639898757021352526534782017871524397839824543317870198579887149357543096692215716737717743897005139669853250787727124724052132584139067458824646092832582161579182356474056",
                "name":"55175277250510441831292588500489821470105462029931633974233164909524020392835474942753154942261078780102042234709642677979594472121740329039096939407862290413981125046579979726509648525040384800966407599496577711912387920555640328195969903745811104905409265295561718059627535565801374587732354719288578700325381238665701068981833510183663974665166259529777453143268585290110128804698405445769122274226023043351402088932222301723742845333979137688513392456273128476117280434751069415582240148078615054377522664430185913899386278250795198772327739729663096682619971255632388880796300436755200458208506829602291713661087",
                "sex":"57557054923926469633758875051561482805848930392687585736888822106276580951736101270935360058382582918091342472575550763937405762309172562832446486185818468538712493963394600155890502368586127145690779461811819794775887702862923402452020778174220219881150747700182796123721567086471448286456165221395819688815911698363826949525339216506513252954177347785773483440799788415973069161787801498842190828102162699527451826935406078138449215853674032455413127960612352397550535518720166463309067121703840634019506722536673224120194553674571369507355519485964216921278831380708189731705879988320442988842054230420334279394107"
            },
            "rctxt":"100682413111906436416125292068082714378111547382938670534710287749203139099597173663957386114866312729066631628240371951858843152777460352064503767472673212324543485075317856264737499548196984818547548302474469502696999887923485876142838071881853381074800979534219818623769950460633617946232313352067708978267598188795020379239750110936590656136438478665726562250545242943311693011754741758572765527003279483943163225731738200840690230060498426484631080335091101158721776342560865543749929426885740682897418936162736762052707489952456967460763179488784177554145858315713805573156246359757974550485660910228120083164886",
            "s":"82686813070973867880463249278612661908299885262559572768325742889587225542919367385336545883188564494450939301240378081495388240699447128615543117912249729074357977521156630405542772517940546969692439440763661800692953426644776587251031510369553607630317520172973156560609189786743881957719851937529704955896892542755887165442549295432707339118200368279079021069252615592048223113143983562212583018280522367428727455995512262244001394519560162122909298995760435043661593664470190629552389406046468329599957181992020916977892615750617467380097471920085490714024110460123803937570236143596241021154836009085307747397235",
            "z":"9309103271259375039309811791534346172720451053399916178016337299549755610552344285712169036748292226172454114771219222675125444956762832557557925426082991257991283791717489747384474293099686087734988315304421434168332951275789834269908331590296411321158854456508421784028845528141008822450258608678686234819845085712198603216700574443379630726501310426177214814498352192105399241045109757206927795802259902426934662489633700245703575301259185088086658729572249347604889897999180203975984742193102502546602494778287709734703847977037098767460940425943011986333117682797142948357690689644414369918227896990617989659714"
        }
    },
        "ver":"1.0"
    }).to_string()
}

pub fn issuer_1_xyz_cred_def_json() -> String {
    json!({
          "id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0:TAG_1",
          "schemaId":"NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0",
          "tag":"TAG_1",
          "type":"CL",
          "value":{
             "primary":{
                "n":"91617707403619020134651408800914754167654528460061285715324956334903124453892337441547327992940044554608687024187961399784986046790696743303507978121055505358167097041060098782853141067594117017800061876816906916469755639377175050709690374936822731765026607509857316340295070485643028603337203333290445908392649689877364286889224553735253143860177338013980397935375043128413095160654845234903949180296245655699099880037714025221030900310874135489281577732142042565644102188436212595292732482239172040396198350646141224484806367801243981744644292148322302863259040322210747093561878598508434221062339815233221785789481",
                "r":{
                   "master_secret":"52923944663739917893792334919588902770406993427596578539743002240679772855824294305860240392425520595443315638926908565702536521525770071663502298815476605292321763225377391334947076329091051207470169351643496841978811470129302699694004470234551276427478412463122457447554403805185943165760447298592703207474308853161285861198890848661575228803665490958341561727974192056813620859062517859875550831917625112068107096435717856940808794389522404006354512225444429626716662400560321663372925421429216034496322973502630430429937197923598918783427300014429550479201657662060911188507020505585458900233506050267723921131618",
                   "period":"5920327329319057307540450710504205946122291178819625311570313937420180032353977806444631853200403992234605455823170108692740818232895486181776143558509707161474170260916726862380358363048290429278429650623242423271463269618772562610018913367911785411580717711186959376363478821605031385935768717725450503048853773990328122472879021561521450388724815889375264707996371485062599926042216549815911055240446164141220397853598067057841890374997501556187531491236913319855705184128969530643556371442986185687924380898922292859388511404901001134383322455989971662211937623063751483481618400453001001326347093702554244268605",
                   "status":"84575896146200123954066550851435203747102446030237469332156612396383018187331080833858699860185591280958923995997486745855302857118588388891929056886613818038813466840707862074312630706723026669921256971571585683300956278833436175050977943321346830403206658111081056850374790368456687433721949740845953048663637793339732612231768794620144827477438144864391377855859717797118490416057070462818186692501794081323884476107929025262236912420224414916960034680102204869824380946423299784514470067447344625942082512645722356178837154388941859249309163049728972370837107524160172168535816899719413520038350865767317207590535"
                },
                "rctxt":"56341114891256742278745072587999866338038797066938363763791554929879350208406904743336736039537867154963215589150978715557629530705954956911277956771555906656711163303160378676640595564385748355759873919462618572830405212603487577986319553536258485730546828010884154972163667764347787187403413881745686374023374810493040147752168931191071938291260772789021427849474044673091960380509362996887529558756549650539710288698074231928897121264243896325144449311398460945050635223302936978059934892765893751891511545957521460341668617680518447468301265053295752715198290661369672279599801060953421567524308649049757931787298",
                "s":"7251904439617677076647515881166821779629336303025328998700032137309829910914446100301277823059567539569297203382304605827126146628427301946455076203883887682144068810892604604362321869114213591138764003653149504577769511214141960483111446526360207078115562316182278138713801655015920269991650706134359902853173745434407671513862637863013737240514109523088628624353145954694613230885575934569653470265594480820594525659288374582533772573190660085805730573835952839185032974717216075745253548091292198312225979878399039182354112447841558122799215751833282228336456725251859669649658327001463777097313017973020442918332",
                "z":"6793974137963149027577387144720139828206952439818209947518945202672331001984461998132008967016358110674247797159372265610261007702805617595984247783467157602866498151104537440813558469256714549840945566879985383562625181423488167742496381515216716891735775560717995953941254288036080059360396192495660623532594402127812770352394669973786919089037752125713047436442798468725686972118664101641336992402640147552826193082612663629092137305157490776510291264018022188899289080469913987687720684607393448169998596908050530860551030641487731831213448078360301080222901594263698128390930759882384005181904986892750788814296"
             }
          },
          "ver":"1.0"
       }).to_string()
}

pub fn issuer_1_xyz_tag2_cred_def_json() -> String {
    json!({
              "id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0:TAG_2",
              "schemaId":"NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0",
              "tag":"TAG_2",
              "type":"CL",
              "value":{
            "primary":{
                "n":"100537797268403858842880783249332898828937923023696903240457428939689436148190984143233783897910017936004063318839240481455164105779558225506818680456870891186331882415775780003509593712094246241831579955442203700069502345629747369204525483138125066079852535028915269930350427587604823831358322339327783190988253972684315369595769576580527686220398628646241935319589507141922676759800195835673294554853822858041283573132383826300282282474540727205095533881241554325814614993466984409783543906414184214314371082730728005364050092411382496821060710836092473277597476993549363987166493397036212672329689021458588507158621",
                "r":{
                    "master_secret":"92577810920785476340013423333608651136983239898824634247577918967243557422745031456668315208551788633332633652296256143008819135396048693733096476634428479329811425961567158144716002342396703511710190006432471711845267222690338738133774227280077080952726218388306875689244907279218878640210310004983043330093472067042469212256857069177887717572326291171452714228791648796945878789096838269416360520820048331245180183301491971985285681530921034289398144630818458527775395615338326775214493166992855255676845345828019294139648066577670819247258320977562797947828523400203929654818198607181596357660549815471464138546228",
                    "period":"45351574846656885227598703806273085900159015894913355471829600140469216966128216972020523537007823625996482282040669615620437579105948862484844114240938935018588710518373989035680900017082017373111443697179828985541253031760931995595529345462630919673234323228286876050271695586386340843911092876370494938597438602512907124500742853594104042077000500412201921804813679655067984690643937241759001395851155987659508824704841739405655867203879996309553571870609893218825378869572028940304231465198204442962287286416059984358303860158374369796749525599035256797817614651250679635894378828926227588455026332140025513013317",
                    "status":"33194165055601195675485688692810132099692797840720284317540426310430129895247281080272446985784885789279438710945334395517486951925758371300740495487395115932940937688488141686746535998422810882010336304989146436469329969275892558318347180502773012351228569166012330509829770847767252109254090083280267096328317011155292698009740920972617325206464789482002761408133876956441108905003132058618204468494217175531147106031438429785492355521928279660223586422842062166735241032188414390882758894773778195176723016679262921724355304537537362274192901523599849047464073777998188640535657156754269070259719404352600974902994"
                },
                "rctxt":"17118074554672216032232662733731483105301491921219669140150267291355078097039903487681737834992231199967056869659442987847324626945448882078352504832112167861880703319097098855694358874655206931132828829011222706903281417083240699938968941336772213828989741204006522183146740662389176761344568296332312108270761273220776771554916911505752232843044145234191805016607216725476921918113718220634812645447536940553804369557649807312805016935652880444659657724332481026512809956963075882998367167034996517079749206948243121323867317340066882653170349974788734441105014639587521976694688181991585067454662323587163797843701",
                "s":"66983692142129914942660777086065084634595901838454662069338187050977123454572417042245163524804195757969191453073776686966821591109835282634173914964987287866627607219529843156552914017171475697987242522631017874879136098990508794550210986085532759228157060514914221148112967259623291251799670436546084428773586910032703926502431761256829286984781011389842636707926956383684794363123686082620659360994121609259262189521518982593207636344669889177195833885123725852438100119140819788105015073220261409628326075453224419713051708956315370536601820544946228210356066164895769793660360626090745318197759153655086221192362",
                "z":"1479761782332625976024218950115659100395454322465597322785467724311465496342471714251192906120023868774853800745882105578559938220614419050555025473161742597043979698228700625416255411591359573935590764724925552189644795628203841173109068914250251332683054382365622905014472794923835925352663342503124045373006479413760845331350025898423645230306001439477844873176978573467987731262583981393407905595678862210189891990250909317952282033150648306260038760841063162231125957140135265819084497946844476726453373435233542415253030754834911200258983912300445412393831712175639729043777524411544145067000459784724436496048"
            }
        },
              "ver":"1.0"
          }).to_string()
}

pub fn proof_request_attr_and_predicate() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "attr1_referent": json!({
                   "name":"name"
               })
           }),
           "requested_predicates": json!({
                "predicate1_referent": json!({ "name":"age", "p_type":">=", "p_value":18 })

           }),
        }).to_string()
}

pub fn proof_request_attr_names() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
               "attr1_referent": {
                   "names":["name", "age"],
                   "revealed": "true"
               }
           },
           "requested_predicates": {},
        }).to_string()
}

pub fn proof_request_attr_no_name_or_names() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
               "attr1_referent": {
                    "revealed": "true"
               }
           },
           "requested_predicates": {},
        }).to_string()
}

pub fn proof_request_attr_both_name_and_names() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
               "attr1_referent": {
                    "name": "test",
                    "names": ["test_1", "test_2"],
                    "revealed": "true"
               }
           },
           "requested_predicates": {},
        }).to_string()
}

pub fn proof_request_attr_empty_names() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": {
               "attr1_referent": {
                    "names": [],
                    "revealed": "true"
               }
           },
           "requested_predicates": {},
        }).to_string()
}

pub fn proof_request_attr() -> String {
    json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "attr1_referent": json!({
                   "name":"name"
               })
           }),
           "requested_predicates": json!({}),
        }).to_string()
}

pub fn proof_json_names() -> String {
    r#"{"proof":{"proofs":[{"primary_proof":{"eq_proof":{"revealed_attrs":{"age":"28","name":"1139481716457488690172217916278103335"},"a_prime":"37059438242994259998790346319434066563259262851520355480091172150889826130883393237926093321799440681909856961532640980048449223384805671899607862659376095023842581006802537983950207932496792785144734656825477404865618508668977376090591355838593886141620410371749415095554373277355509688260699210360869358785006563706591833856987245193114424070438013063355809733139650073893157583349774448671800293597451613258120928500855420774040035399988790635466156695875407657672993004830316697816281308896527384225080552033515729968331567171219679379473102757252104487685693002070188976642694202528257589046582080709109752935022","e":"109922005779405233943121743010554164412839436706339658425836679038556189672243290902929142208097769762943034219777885671776752614244091909","v":"1284125915860499669248852735479317096776719124959570347995473846767193414681232645848163339499108677754895570637169906769917439890666153543756837147366902490935443325961637122673889400243827346197368006028001780374947587482033411185560936517011006508036126537084103997207939297366657758940447914263690958368525082493075666522936208229179177506160262689731500367805118210523383098538513341379010456780856240340164863012154755712101591845153767452478810954083326872665765593321265442487243886298797961711223784823025655610368018613908959702591553249764645833524021917311744642968807457723747488191724643933369323339439131438966193247834546274609039204958605178132803990774813639824288602105473621379074346108397569767201430866324929811858040334046133024151120890828681541301553801379580949720247081708351217629622993044672922686194020828891720587862076224530881559036468002049156322198695898248214640042505280000582614623246","m":{"height":"6959077476651161779684574416838553328889210956914819956415532052678753320921972039328902793176095008644286859578890788916500546569517337307957606535111643517870736617007894947290","sex":"893288084832905070533301048840996400689551521791830443783389909394169951782753885801543504768735861345250554844473703958225223057610041365470394500145847061634125263984993695118","master_secret":"28964969107326245445186522007509714666417333989344308192807238824533729139528375723094650589872809641576900097517905181465585338145114953472640727001478696440390332622000037914"},"m2":"13335055121995665245887259625985450346942763879809728105637603608948578710861386965292046088372415755254360438523216452082325286343438346537397088988052200472812564806383761270685"},"ge_proofs":[]},"non_revoc_proof":null}],"aggregated_proof":{"c_hash":"2779655636103467443483025522910754087684294079111604105703620998863293181442","c_list":[[1,37,145,58,133,4,199,32,127,85,224,91,135,252,1,247,67,6,63,22,34,5,55,78,75,84,54,197,117,43,172,54,219,107,79,237,139,15,215,69,5,242,97,100,62,149,165,29,92,48,24,53,219,159,64,249,101,15,37,229,76,53,121,85,239,147,0,58,114,117,21,194,171,66,218,252,154,125,62,228,230,169,71,45,37,131,247,32,212,204,231,168,243,246,39,152,121,130,187,213,109,99,186,203,7,198,67,162,25,109,38,188,234,208,204,15,93,215,47,110,122,179,202,85,185,90,23,114,35,44,253,236,144,220,41,228,44,21,55,3,107,55,37,229,2,13,0,217,116,33,220,242,68,157,149,90,55,188,20,23,253,222,13,114,4,109,230,123,100,218,210,201,139,251,58,90,204,70,132,153,194,192,25,168,98,161,193,236,221,143,219,136,242,217,10,235,250,43,179,153,224,110,35,83,168,201,162,4,182,51,38,9,62,74,249,153,217,94,174,89,224,99,20,18,108,29,130,38,239,41,188,44,154,170,194,1,185,157,35,185,71,40,240,63,94,62,133,30,166,53,50,249,198,220,14,50,110]]}},"requested_proof":{"revealed_attrs":{},"revealed_attr_groups":{"attr1_referent":{"sub_proof_index":0,"values":{"age":{"sub_proof_index":0,"raw":"28","encoded":"28"},"name":{"sub_proof_index":0,"raw":"Alex","encoded":"1139481716457488690172217916278103335"}}}},"self_attested_attrs":{},"unrevealed_attrs":{},"predicates":{}},"identifiers":[{"schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0","cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1","rev_reg_id":null,"timestamp":null}]}"#.to_string()
}

pub fn proof_json_names_diff_creds() -> String {
    r#"{"proof":{"proofs":[{"primary_proof":{"eq_proof":{"revealed_attrs":{"name":"1139481716457488690172217916278103335", "age": "29"},"a_prime":"18017896431923240505908365485075789148452168415116206328291489071230607620619028995263932900056921507032528080183268960807711894122144333310053202926605966192328263536945647116166810074344574038293730607257155382357985975688088422223520166731897840822699744762240546655662386494770620420488496975385522964510724614716862290911965443814238358554653895723431607230003718708950593308633603143490127732842456632850013987212326223160169815330300582908768800090611574776175074188808918847861865749073430719711540128976239401026267393103013986040292911362679944546062820846133778574434979875055932311508478389914735000207003","e":"83559519121287395878717669622136044815646318620514874311709459571206234463327454292630917628062700115436217341863867347698462127710406070","v":"297806490214829420686766230078875761656411019398322192340660159408525437640356447298628486670199561773289627996231003178671763674854234378582742045593152090786356168548722050923919022437294919948020197450167438513621012738971001614937494783631173417424906834450196046209089992511255664827703967570135975184640411362392658102117144696565696778260226160184205166662563339868760502203244658568114681738362874425865611483890117120551575951385574973597994708955280610971400101828731369032088887281109139124193124939540584287884165616875914678163284636131307587450221606961891189538460301724046217944798864977650080735467274438530373131735445981882185267246263755499371761873417456882351522370247911160083566690732440450139886893927311033972081322669303047930705926280689044826420220280688545215188219142706778436864160051113903892157722488901121137377923901441440141923017426694473886970495058317115339857648593543510755814225","m":{"height":"12006903624253341849897414349528320249916574584890409251234494044009772650118943979177359842712363134185042426360823508114521187446806481412797317456510257618078868574564525071674","master_secret":"6919686208640537777905634061289167918950072425929046468445054398351611354176109670910397217504584288829564076584234033108276953010251873235101744003423827510526144551331667620558","sex":"13165539014140341601812739414059384997370961293541830348392691981504671830007166315875893835129825775324685721111876571565247345930495388423719630010802561730225421962397547387507","age":"1464765639220538429462362069428301097018775282316000466628871517541386680138790057277673679234302501393691766391852725957692111731926315506123255980720728976368733501055706839955"},"m2":"13578671683654549858852643988215668906078095112870901210808381907605213325376098264588427251159007228787606553068475182937945754440107583457297550054908391037290899567766980153744"},"ge_proofs":[]},"non_revoc_proof":null}],"aggregated_proof":{"c_hash":"92451972292295535930734088937643227098723372505615593632453853864067889786715","c_list":[[1,143,68,138,222,128,151,4,206,134,41,21,121,239,118,74,230,249,16,232,65,80,218,64,216,231,211,85,196,120,71,36,165,150,25,158,3,248,76,1,127,104,97,110,107,164,97,185,127,42,250,73,52,137,139,170,152,200,177,163,50,0,128,112,156,124,215,45,69,40,200,65,65,129,107,134,218,129,232,8,222,219,218,178,196,40,89,181,94,123,198,220,5,28,193,85,7,17,26,116,159,145,239,160,41,158,27,12,144,112,27,129,34,150,100,234,235,144,173,188,60,108,75,168,141,190,59,142,4,72,35,228,121,176,195,68,52,169,92,66,108,7,214,176,200,8,147,25,192,146,253,3,159,116,102,228,136,64,182,89,164,29,154,20,47,173,76,47,255,177,109,203,173,212,210,112,254,205,131,110,170,36,214,59,175,220,116,60,24,150,88,63,12,29,22,122,58,55,94,33,31,201,8,2,98,142,50,52,164,9,215,168,55,135,15,37,42,124,42,73,184,191,96,105,242,172,243,131,14,130,2,31,59,152,154,14,2,213,112,10,191,53,209,239,145,251,163,175,192,184,160,29,191,79,214,40,100],[142,186,175,2,20,110,52,23,79,202,111,137,29,71,73,90,209,23,34,203,73,30,188,128,68,129,77,72,76,249,91,77,148,242,147,74,60,49,156,202,153,188,180,191,181,222,44,227,144,164,247,79,150,172,154,162,172,164,204,2,215,214,97,86,254,3,44,236,183,84,9,219,168,125,237,3,121,132,163,74,104,146,99,216,95,206,227,89,232,183,191,156,206,133,4,14,143,177,17,147,177,0,224,218,75,186,205,60,79,214,79,30,43,28,228,93,252,216,164,10,43,224,40,235,38,179,38,246,213,219,151,140,95,24,108,61,23,160,133,110,143,196,118,116,112,14,194,174,207,133,209,130,158,201,124,34,17,125,165,225,80,136,20,153,215,42,113,89,81,18,192,172,174,122,234,36,169,176,120,37,195,252,19,247,85,12,30,165,250,240,153,241,36,134,90,224,157,158,215,177,24,185,121,155,52,151,208,141,181,196,159,172,134,182,51,228,247,243,193,156,138,222,106,104,234,6,89,211,99,222,155,214,123,174,185,188,36,10,61,94,40,146,193,77,27,140,185,0,6,188,187,152,1,152,190,155]]}},
        "requested_proof":{
	        "revealed_attrs":{},
            "revealed_attr_groups":{
                "attr1_referent":{
                    "sub_proof_index":0,
                    "values":{
                        "name":{"raw":"Alex","encoded":"1139481716457488690172217916278103335"},
                        "age":{"raw":"29","encoded":"29"}
                    }
                }
            },
            "self_attested_attrs":{},
            "unrevealed_attrs":{},
            "predicates":{}
        },
        "identifiers":[{"schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0","cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1","rev_reg_id":null,"timestamp":null}]}
    "#.to_string()
}

pub fn cred_defs_names() -> String {
    r#"{
        "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1": {"ver":"1.0","id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1","schemaId":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0","type":"CL","tag":"TAG1","value":{"primary":{"n":"103643346393845275674640947963517640959341364800001498302360615696811800436853570598226635556206782056017108616401751128764891668126024496689984398292790001833962277974492640424375663143977133523408000338914980071623310865759379704138347909743626704834849589208010894771669047489759670624223420738983274636985733732817381003552388020158483750044902942029694408803816687233383567950712661075932249633288902332456154719732901523792530993107020670255716148528908061724351279359013028125494116528692296179832938992525116968924864645033367866112782093052304842841695662188904575461773278691171441164640468498936898548822961","s":"54822618443866209168352390646516970171692268351255705964324890820655555536844269368083112564663040886691690357518137913913851456980804740769095081281331568188360750646498021934662032327693250468598454967722301464234459636495850865224391999449766311288958890434234613688144418472914263652993730498111573243983592129437106960567928938213693844755547617988707695273081422650864025005777423456919937278227759790112692946315871730838728313573505822719779055770101438556244978220637745583964035779120946484981484244869612719059958094106508205010216749504887420219701355521415946533068900478171105140027940119076255040696360","r":{"master_secret":"21425866197572384062628227194464891714156374610141012384461581348365120908605278410323220952474017412306135450580118550540437728188359888590948197635134588545589091052407046545942207235016161583149373682351466989066269120996279864435557569196540520391810838084585341114461860501161109395768522889847155923115130975545305402980362921716271119462622840400234342486330844075138870120860006843715759294992210691837620004539067400600516841749888381337846509599423365443728150963320181855847744030640726283764098892020276321707557615206879891875680106353497981731503326804905717237138201176626474351449246082684820132978570","age":"45896901353995160445978387844699592703133036025099352750898266372964830207166941136996790217676372532920553925138777466267003700135162350479522623300298584973661267820445439715765094754232744337013710464559658285188600621507020083609102806722359565815605695315341934730703660211844694918436823660402045649465334189417440798816135712983731377808041977980814757299823667534167999849188593391970063412813082841269439697272607574256268026035546914701198698550305688087387956114332278588283870629094318506670933049355915020126916771611552278002263251015892361545719715049068470300032003538304598091054469910389331384577974","height":"4221002407005984423212190198398284728440929641232368725218578512393813615156013892338789867281385924668638715574080419943329990870453359527853553186274717009654214731069115515044687088555910844826756000998130090765935804499830649613782409467283723293807074901087954196385641523552555530081140452877980829429262438118263341391791477073355602772244978401151361021396142300113344103180780419266132906324807999802516470516337247456030064493854194598878878359527963789497903050121642959744956972333750467805569828376906169006006677887190946169426595114354749816600318887852407081680689591565211969531983309394999310215690","name":"55821888754379574444274347969239309214669096344182795896442772767165802253625684782808092946663161154477245454437228295689924108373383345714837117496635275551463599418309612940839462839591434978877169781158874575725325352168719700821644499270490964481646961843688156989732559770914667683505609881195966214647319014277753992611817499000283181845433053783962788019962661502416268571849389197531749145685201938251556822532563899886899730589195281288488435420894032614744402247844326503178107540034949843472286314368667720380983335175064713086835421532934021299267345727166356932677108561402137557792441291994395308417091","sex":"42643025673446729762816674570769849409692245764106121818625902368695453319759554688888639016334648500645929017602360733256156654476564346054352017290215025747835557451392247630617314944559117043358985248641744757651131432238247602477096654010555300286716932508602113826095821317420814670523504108998519549091300130622886149492085258276862787678317172035682824071514406288636316907042524503317048761222456138230146059037916323087426042597632575233092855412252116770053173039780130616320676368552589658421120777318093236117148547256237045434870125481935340815386532833182205775276361508091040101087580291158395096248559"},"rctxt":"31992296843801676419002251244571234188908617875737111965307805411711440137833955675753481353370798422552299763575457897093806000022648849035961238035756079922314435615410225785025116609642145363614683598253278559703740325116325475904143371799075727822754390855047833763476357818627562176083677917632158653889771251787647538508513720930934901860753297059032955828284586924945763206106801946523945886996921411204653206076328679835066062941303627608933435349077918619011152320089446868864551416857096600755315328457840592276640681321837903065178924598004689000075403324248676571699195373649922706248102368135426433140480","z":"48080767937237636685843182901757011094735969085033070985311264407046854205230155767794004046881564849977637323672450699564097991649050552022493835142041689529039228856996103514300988580136934919156886001156783367772685023905188703162742930724665619126199211612466903711166572493356027661192032647618322906856866625035472531927030350990340200726083246566521827993906484376817111857571652986215187880532663473563453916686603372697643738169264640142823640194129012710689193402020962720271825844652556914903310346673139441328343095408798925136070900501618726857155844753922445347010608331282659277380357914960811457571299"}}}
    }"#.to_string()
}

pub fn schema_names() -> String {
    r#"{
        "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0": {"ver":"1.0","id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0","name":"gvt","version":"1.0","attrNames":["sex","height","name","age"],"seqNo":null}
    }"#.to_string()
}

pub fn proof_json() -> String {
    r#"{
            "proof":{
                "proofs":[
                    {
                        "primary_proof":{
                            "eq_proof":{"revealed_attrs":{"name":"1139481716457488690172217916278103335"},"a_prime":"73051896986344783783621559954466052240337632808477729510525777007534198657123370460809453476237905269777928500034476888078179811369103091702326392092669222868996323974762333077146800752404116534730748685092400106417894776122280960547391515814302192999142386455183675790870578615457141270148590712693325301185445330992767208427208215818892089082206123243055148017865514286222759353929656015594529211154843197464055996993778878163967106658629893439206203941596066380562586058713924055616953462170537040600604826428201808405436865130230174790116739542071871153581967170346076628186863101926791732126528122264782281465094","e":"26894279258848531841414955598838798345606055130059418263879278878511424413654641307014787224496208858379991228288791608261549931755104416","v":"769593829417540943566687651216000708099616242062220026508500847265211856977241087739974159673381844796906987056271685312217722655254322996792650873775611656861273544234724432321045515309211146266498852589181986850053751764534235454974453901933962390148609111520973909072559803423360526975061164422239685006387576029266210201929872373313392190241424322333321394922891207577033519614434276723347140746548441162607411616008633618021962845423830579218345578253882839612570986096830936195064001459565147361336597305783767484298283647710212770870573787603073109857430854719681849489345098539472090186844042540487233617799636327572785715912348265648433678177765454231546725849288046905854444755145184654162149010359429569273734847400697627028832950969890252877892391103230391674009825009176344665382964776819962789472959504523580584494299815960094679820651071251157496967617834816772303813309035759721203718921501821175528106375","m":{"age":"1143281854280323408461665818853228702279803847691030529301464848501919856277927436364331044530711281448694432838145799412204154542183613877104383361274202256495017144684827419222","sex":"13123681697669364600723785784083768668401173003182555407713667959884184961072036088391942098105496874381346284841774772987179772727928471347011107103459387881602408580853389973314","height":"5824877563809831190436025794795529331411852203759926644567286594845018041324472260994302109635777382645241758582661313361940262319244084725507113643699421966391425299602530147274","master_secret":"8583218861046444624186479147396651631579156942204850397797096661516116684243552483174250620744158944865553535495733571632663325011575249979223204777745326895517953843420687756433"},"m2":"5731555078708393357614629066851705238802823277918949054467378429261691189252606979808518037016695141384783224302687321866277811431449642994233365265728281815807346591371594096297"},
                            "ge_proofs":[]
                        },
                        "non_revoc_proof":null
                    }
                ],
                "aggregated_proof":{"c_hash":"83823592657816121785961198553253620031199104930943156818597639614860312075063","c_list":[[2,66,174,183,214,178,122,180,186,63,14,80,155,85,150,14,217,66,149,176,133,171,1,26,238,182,223,250,20,5,23,250,187,84,179,207,13,147,67,92,135,47,152,151,93,9,90,133,13,250,155,255,236,150,10,32,56,173,28,213,29,208,126,57,225,129,173,51,233,189,32,201,139,82,153,42,8,222,131,35,246,39,85,114,168,183,150,197,192,212,171,99,158,9,192,212,61,24,7,95,188,144,164,79,43,149,163,156,241,105,34,114,197,160,90,232,244,72,122,177,186,233,82,107,1,66,231,153,178,57,101,174,240,63,7,50,168,21,134,165,133,105,244,106,115,4,93,227,249,77,58,24,219,122,95,128,87,249,247,119,163,1,197,94,230,66,56,58,203,213,201,219,52,134,122,200,20,210,10,225,231,124,232,0,34,112,168,133,157,202,13,47,132,162,140,159,133,104,24,133,150,66,116,106,250,18,9,84,4,249,4,184,75,216,144,55,119,233,139,217,138,27,215,38,114,20,34,209,179,90,237,184,124,207,14,59,104,25,219,37,162,82,5,24,12,20,94,208,227,162,61,76,247,121,109,93,6]]}
            },
            "requested_proof":{
                "revealed_attrs":{
                    "attr1_referent":{"sub_proof_index":0,"raw":"Alex","encoded":"1139481716457488690172217916278103335"}
                },
                "revealed_attr_groups": {},
                "self_attested_attrs":{},
                "unrevealed_attrs":{},
                "predicates":{}
            },
            "identifiers":[
                {
                    "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
                    "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
                    "rev_reg_id":null,
                    "timestamp":null
                }
            ]
        }"#.to_string()
}

pub fn proof_json_restrictions() -> String {
    r#"{
        "proof":{
            "proofs":[
                {
                    "primary_proof":{
                        "eq_proof":{"revealed_attrs":{"status":"51792877103171595686471452153480627530895"},"a_prime":"46170527191009162423195992242911202103745247654447961500834202212345068009374227624831811551880681759277039900687689896452219186403815804172798044598038478723202756585969464253290456116320469023496728487569265451931469014947181138279866772248099396337282765049622216043600966530384168236615679826417920589053368355843168947460937197862544724107783748252366182290305430613521649545827648490599746085790650073222267185231427960159454697216413474182940613302723290493046088477110311348355173193004345133051531339039792454111152366833075661125855536539678836086879186283850040248748367109735949797285126446788943682315783","e":"181783181698181287495598785909854459213941720342231191315494769362252467200171328876365896944450719161246346597789982456988074483787827687","v":"339831053156953156083378725722185909585588868019377723413874674083043571601326927248381619119118511689596408976908511742779597745102527235095564587461217765867730783915059382821863995153110497389262273514277935606201423027255344476130326234605522297800186521576370166163815072706648957630262525459324683337483089121033663405587642444879856296279144650981580585409074569982014305371909237583750412629192046031051373681196201244206248430717409572038175014303799476261327706680933927028697252613069628119388532625791557512950586933934332368827529578558762065962417807377849615797559391422141438284278795244378558652893608769455338854487615262205341095666637460639861921485525664318299210263989610342461887219960555436282419366361829349520118953248028050723787812556344016717035063691770786048525812405406221100196750659073230684781582428748449041250850968789483265772295760336285360190196911058737477358927646367690587832544","m":{"period":"855016494737677138075197580322245198108187783880084387490259701916325134243045199960400257682749962414091279800356947542021730593233059355297033594612342025877034119912682655117","master_secret":"575429170288335490678550169774117882467935371984760437052292584088637021048907783807481719535313962818042461474628694769016736380919400662693355726852800569629684181700446938190"},"m2":"1257988172835494196097443801291607274683495028565264477830434571898046175420209619421094460572576016795527005256661597760484794649276248873373922627836389203730566962898811813602"},
                        "ge_proofs":[]
                    },
                    "non_revoc_proof":null
                },
                {
                    "primary_proof":{
                        "eq_proof":{"revealed_attrs":{"age":"28","name":"1139481716457488690172217916278103335"},"a_prime":"54941599904541333278784939655493850533022350609101004025929689872296119270882882370167259395612875365447492210716890625542369466056448977653429190820660946372227998299025173151178000114747137114988511218633880562105997027807429934367967737021905684672412020656470877774205884488383689963133452387679901859388949468165986328558565253731071589582506641938833916342772814636596354016849609040464390499915411292376210138968056320593318442880724979538402904210545495482767582012604462718675602770978567078659407750734341967885912701767556318598721785933276528858042680667743835901176990041608403972777611603660840367370975","e":"60555930672597292814237343120897830788616646085170453184286866521490191315083076844094594131297239656146545948711221960578237257491009630","v":"550010825945837588071467544423189736564601014884458946048080337803074413904051634899003807222673931083661509257271039594561533888378991329183919013218735160530158344131509176012969685514263731732318836840839584525214457842590464394796709202280665439944484262531013369943083255231730911060713568614601359706770970418137107938499016600617192874897146710728627404586928931109015224667300161420311352727726168320510196551990616153934908821336524279218976159961552837554097483343907517871199302301001881302567004502499073620427736990545700590402715497013346262662032692470261354132461143484736398481980722024213712045266267420161254685802856422504704218043057621323011644563178695694019900775017954813675388525565419945652255816248831897681735659725305470669312625019552813351676079150583246203079380525178743342942584562582460505057452311750392065423003911528657470588045172728072762403902669216961931377964248959848356362896","m":{"height":"6910731274496482918185267459967628982661789976007389294855147609004039554288101361418561212147140523600852434744430738216190360942038001797167665107606230809288436430941324619199","master_secret":"575429170288335490678550169774117882467935371984760437052292584088637021048907783807481719535313962818042461474628694769016736380919400662693355726852800569629684181700446938190","sex":"12359827179467790658712633963029479850393167180155910404695624370390632524213070241977413288325045552017010251044013705940876971706087785366691373986284237694871213815216413793908"},"m2":"15144472886457961517884598941507147747566898056822045941553212434427380031911793621646872762666619042147613443300437584836183164117834286370267613601096961619738685982660425732875"},
                        "ge_proofs":[]
                    },
                    "non_revoc_proof":null
                },
                {
                    "primary_proof":{
                        "eq_proof":{"revealed_attrs":{"sex":"5944657099558967239210949258394887428692050081607692519917050011144233115103"},"a_prime":"10925006343680319341430149004007920683817999600361556591133061748150721153274183269957914414851596945471160985621018629598771928054653876572193869225903125621702250694709341316330107194397189828614025908577627877172410804371376187493741370655191407051546507156077921271387955231822220496167335262747937178035432300897149809810673925661802774047477989143114882623020694696619105295666691244811255166945185767410850940975364823971326892901882598331838907963294554880712658595844978171601862781162951165280015828012192316173119094093432763384657093309265752338442414272694361359041947575645785536554229049545783924816161","e":"131969741286916870245126919019115539176145020814618299090964185753787229047127379961637244211840133239104809006940704653341778018823711238","v":"110864361218655268823463163848638481265504230345260344806081388934289640006078039829608293849032174652111035208661101619128760288436682536257038049359913360115151379162992511679195284748259850329623251895528660905656907214906405375839871277038042113222861851188442798407154093113597720336697625223492652410536618171480895898917492545630435869062355424896745112118524492824252596990253894323023935130155459799362776125332860328225285739327804885555018079775860343397051714513883849984720984911997864063355539739258197938755447063429695041564044795471124488986093653679620045413679566511673309321324695892435812061286695176314492765420167541595545754782054674177267539623289911220902953548415542059020805141673957457144471607494296485113355362531908588501335640864596861273263545247743864471240635066874012814735445683356551409704903161680710246997602589612815182395269077201707457440074694932481830491330048327014888726192","m":{"name":"3211173352939144906195511215934829950731837630182039806469760057320297097063520676679840045189229813410290306060642700446315711169866797203599328926231346949385016948457569813450","height":"982257476170806196494587854834657155271926472116897423192773247119152025447384607122861245760020434570012757963361750459871834323972669435806807080881705316250668520779668979979","age":"6921436203847117453150133531893388278514382816859132050706385348922655969218118202584063645110635908686006197294793662935125696260468111642817631628863923881309033651581048123073","master_secret":"575429170288335490678550169774117882467935371984760437052292584088637021048907783807481719535313962818042461474628694769016736380919400662693355726852800569629684181700446938190"},"m2":"768386074943020312165003874928078516931155623272316603368837594930941574694427296291378904249042892997543956974606292521731071420600785101343074701186121865133259918961077819789"},
                        "ge_proofs":[]
                    },
                    "non_revoc_proof":null
                },
                {
                    "primary_proof":{
                        "eq_proof":{"revealed_attrs":{"period":"8"},"a_prime":"13540087254977906875349622691711873268507633443943746272330983235340594788782068979818768657805956008948692322303923543797292321676941958941482807786896428581859927815668557376747197742851623634186354528145648112113537499832699666853768325179174040653690497862254139131373828791083194656184828620278199031987382107127393587868045445877706604378324783229682480385498595141007982117151164861318006514853538053123377672315474612085335985904512190230775062136863352771132454311103525244250463116944223565932390677986842292741239828837902908427594672562021981877323172883291011216736485418947760833489827241621769343997876","e":"24579611097916574757063937066734559506419431890641518926498459784581440886059289556837894619477046778288752733390635035831281581765509977","v":"204279326136187809122912846047267836643012121377122221566077234338765682161479000942248353562737424977422293498790421933543998863691598782365756276425394505096926719812022207372386118912086085376636443637818318992852912650626363221687560239983114716279836110684089773673138214633312142562977379715910957396825955836907560802648718488744823865286766935520281678313396350732419052243994063188932188573966419038139784409359049582487868485482185686733143058777243192044559688516059158575701837086993678525785872281371157541767431059474215278778482096578608975651909805433774973672559309741936978812898434975724365309288070915006695662425101038394602465311327534996679030154830743326371929962644240074574521398783278186444360682066295434593001953321400683941273668570884609022087544947818832749973546391139184603069816329186240549076655735732149196823564105595420056815795648139393760371109325826911512016693405763766814920407","m":{"master_secret":"575429170288335490678550169774117882467935371984760437052292584088637021048907783807481719535313962818042461474628694769016736380919400662693355726852800569629684181700446938190","status":"7895737114192117995770879725398316512778400467751932009044864621898814916860369117094577925851273951479131958996646425600315621261062077592095547439522229502434836512303137380619"},"m2":"10590242435802307860373618117072399794732798028956760886770058446341544980668503787874412039475473938119881682361831184477982020264928650067153244658267214468672955849027361150252"},
                        "ge_proofs":[]
                    },
                    "non_revoc_proof":null
                }
            ],
            "aggregated_proof":{"c_hash":"71024995123257198522276062560276855573334945969673603388820662572761310428719","c_list":[[1,109,189,179,226,227,79,132,171,189,16,237,240,75,195,112,138,37,109,51,137,205,251,249,37,79,38,132,88,232,122,46,91,237,103,68,137,171,237,149,17,194,240,219,147,238,248,18,121,108,185,68,36,59,202,132,127,117,41,238,253,24,165,249,170,206,231,51,230,144,37,212,144,218,26,170,79,205,91,48,41,23,236,242,232,23,185,129,140,39,62,33,55,142,21,13,146,63,242,72,4,217,228,225,122,139,124,253,10,86,48,24,69,220,116,69,118,156,167,252,81,217,232,98,160,213,5,73,84,97,12,57,74,249,106,233,80,132,99,174,95,75,195,158,26,6,13,186,1,181,65,206,45,10,113,167,37,189,189,200,56,66,233,189,158,98,235,162,147,37,208,76,98,187,94,68,195,124,28,249,217,83,108,242,35,64,193,147,40,171,144,99,129,233,82,197,215,139,242,1,14,89,100,216,205,201,88,155,158,128,126,94,76,208,66,228,5,164,232,246,182,131,43,214,61,46,58,103,237,159,154,99,245,21,64,169,246,209,7,65,112,134,250,166,133,144,255,203,118,230,231,144,56,150,25,166,7],[1,179,56,167,123,234,212,255,206,240,106,86,65,246,125,155,119,68,85,130,50,194,97,164,147,225,18,36,78,105,27,57,122,199,178,145,158,175,6,117,239,149,213,157,80,155,198,140,4,241,72,71,233,206,89,43,203,46,236,20,25,147,28,255,8,221,142,225,80,44,122,157,144,71,142,41,1,40,120,69,143,112,147,216,109,242,201,172,184,133,29,203,45,15,17,144,245,113,113,78,62,187,31,103,235,55,123,53,161,158,226,213,231,66,188,244,199,226,225,23,8,159,113,107,196,177,204,103,167,195,209,178,139,101,99,185,204,146,52,97,83,114,106,119,146,69,152,223,35,203,0,112,151,109,255,94,107,138,162,67,34,214,203,53,6,160,132,199,183,188,130,157,30,2,147,109,238,131,123,97,163,134,201,131,23,229,160,154,6,76,69,195,172,99,84,124,55,131,68,42,251,135,157,121,99,213,184,251,95,226,200,183,192,123,27,24,32,116,236,228,239,158,185,179,204,132,140,153,220,178,200,75,7,126,227,183,48,25,150,27,233,74,98,93,229,160,22,212,101,154,248,161,161,190,53,50,223],[86,138,239,187,35,22,53,97,222,177,78,217,94,1,147,150,78,196,249,231,201,138,178,21,116,16,81,237,40,114,168,71,11,83,245,76,142,220,13,225,43,15,94,3,243,251,116,74,25,193,99,255,158,137,149,123,190,60,197,201,28,131,181,238,144,206,87,189,22,211,240,140,248,71,126,186,229,239,249,244,191,251,94,90,2,198,209,184,129,133,101,112,78,254,196,214,243,70,202,246,73,234,72,163,209,93,12,214,146,24,253,105,217,17,159,202,99,65,146,217,179,211,192,227,170,221,4,120,150,252,43,133,180,122,201,2,63,180,76,163,252,143,83,97,170,97,86,36,249,0,22,129,105,233,222,132,105,234,32,109,241,150,245,33,112,188,182,3,211,164,245,173,250,240,154,243,118,108,124,17,173,184,188,160,76,107,143,183,120,197,165,45,4,97,175,94,233,250,10,231,226,141,210,77,27,211,87,205,187,167,199,91,0,120,243,38,29,54,82,74,225,156,3,95,155,89,52,11,205,136,188,51,169,151,238,154,131,251,184,6,253,120,131,174,164,215,7,27,193,243,88,47,5,168,249,33],[107,66,22,57,148,84,51,110,157,73,247,47,247,52,113,158,178,138,231,72,224,50,119,198,30,101,80,206,99,126,168,208,56,20,123,32,95,240,48,62,8,204,1,220,68,241,177,114,17,30,98,238,77,186,133,64,37,162,85,30,33,151,163,204,255,66,82,159,141,23,95,17,196,245,141,97,202,48,144,66,158,54,241,149,79,172,84,102,244,180,133,142,108,73,247,141,108,91,87,71,148,231,249,120,145,86,199,62,65,96,247,85,101,223,91,95,213,198,131,208,84,119,47,234,14,246,148,88,212,131,229,31,2,5,118,88,1,150,65,226,220,81,129,93,223,146,126,227,122,149,220,242,159,86,56,183,70,148,5,177,169,220,126,167,77,239,253,38,231,158,101,128,121,25,103,20,193,74,32,82,102,93,32,169,98,98,200,166,160,243,126,127,106,84,98,195,190,20,183,53,191,92,79,3,161,198,53,31,229,189,112,39,174,196,201,190,34,147,113,88,218,28,142,185,35,170,246,171,29,162,5,218,26,150,69,0,61,31,53,181,173,245,135,103,194,42,128,230,166,187,30,68,116,108,23,180]]}
        },
        "requested_proof":{
            "revealed_attrs":{
                "attr5_referent":{"sub_proof_index":3,"raw":"8","encoded":"8"},
                "attr2_referent":{"sub_proof_index":1,"raw":"28","encoded":"28"},
                "attr1_referent":{"sub_proof_index":1,"raw":"Alex","encoded":"1139481716457488690172217916278103335"},
                "attr3_referent":{"sub_proof_index":0,"raw":"partial","encoded":"51792877103171595686471452153480627530895"},
                "attr4_referent":{"sub_proof_index":2,"raw":"male","encoded":"5944657099558967239210949258394887428692050081607692519917050011144233115103"}
            },
            "revealed_attr_groups": {},
            "self_attested_attrs":{},
            "unrevealed_attrs":{},
            "predicates":{}
        },
        "identifiers":[
            {
                "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0",
                "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0:TAG_1",
                "rev_reg_id":null,
                "timestamp":null
            },
            {
                "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
                "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
                "rev_reg_id":null,
                "timestamp":null
            },
            {
                "schema_id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0",
                "cred_def_id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:3:CL:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0:TAG_1",
                "rev_reg_id":null,
                "timestamp":null
            },
            {
                "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0",
                "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0:TAG_2",
                "rev_reg_id":null,
                "timestamp":null
            }
        ]
    }"#.to_string()
}

pub fn proof_request_restrictions() -> String {
    json!({
        "name":"proof_req_1",
        "nonce":"123432421212",
        "requested_attributes":{
            "attr1_referent":{
                "name":"name",
                "restrictions":{
                    "cred_def_id":{
                        "$in":[
                            "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
                            "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0:TAG_1"
                        ]
                    }
                }
            },
            "attr2_referent":{
                "name":"age",
                "restrictions":[
                    {
                        "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
                        "issuer_did":"NcYxiDXkpYi6ov5FcYDi1e"
                    },
                    {
                        "cred_def_id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:3:CL:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0:TAG_1",
                        "issuer_did":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW"
                    }
                ]
            },
            "attr3_referent":{
                "name":"status",
                "restrictions":{ "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0" }
            },
            "attr4_referent":{
                "name":"sex",
                "restrictions":{
                    "$or":[
                        { "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:xyz:1.0" },
                        { "cred_def_id":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:3:CL:CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW:2:gvt:1.0:TAG_1" }
                    ]
                }
            },
            "attr5_referent":{
                "name":"period",
                "restrictions":{
                    "$or":[
                        {
                            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_1",
                            "issuer_did":"NcYxiDXkpYi6ov5FcYDi1e",
                            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0"
                        },
                        {
                            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0:TAG_2",
                            "issuer_did":"NcYxiDXkpYi6ov5FcYDi1e",
                            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:xyzTAG_2:1.0"
                        },
                        {
                            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG_13",
                            "issuer_did":"CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW3",
                            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.03"
                        }
                    ]
                }
            }
        },
        "requested_predicates":{

        },
        "version":"0.1"
    }).to_string()
}

pub fn schemas_for_proof_restrictions() -> String {
    json!({
       gvt_schema_id_issuer2(): serde_json::from_str::<Schema>(&gvt_schema_issuer2_json()).unwrap(),
       gvt_schema_id(): serde_json::from_str::<Schema>(&gvt_schema_json()).unwrap(),
       xyz_schema_id(): serde_json::from_str::<Schema>(&xyz_schema_json()).unwrap(),
       xyz_schema_id_tag2(): serde_json::from_str::<Schema>(&xyz_schema_tag2_json()).unwrap(),
   }).to_string()
}

pub fn cred_defs_for_proof_restrictions() -> String {
    json!({
       cred_def_id(ISSUER_DID_2, &gvt_schema_id_issuer2(), SIGNATURE_TYPE, TAG_1): serde_json::from_str::<CredentialDefinition>(&issuer_2_gvt_cred_def_json()).unwrap(),
       issuer_1_gvt_cred_def_id(): serde_json::from_str::<CredentialDefinition>(&issuer_1_gvt_cred_def_json()).unwrap(),
       issuer_1_xyz_cred_def_id(): serde_json::from_str::<CredentialDefinition>(&issuer_1_xyz_cred_def_json()).unwrap(),
       issuer_1_xyz_tag2_cred_def_id(): serde_json::from_str::<CredentialDefinition>(&issuer_1_xyz_tag2_cred_def_json()).unwrap(),
    }).to_string()
}

pub fn schemas_for_proof() -> String {
    json!({
            gvt_schema_id(): serde_json::from_str::<Schema>(&gvt_schema_json()).unwrap(),
        }).to_string()
}

pub fn cred_defs_for_proof() -> String {
    json!({
            issuer_1_gvt_cred_def_id(): serde_json::from_str::<CredentialDefinition>(&credential_def_json()).unwrap()
        }).to_string()
}

pub fn get_credential_for_attr_referent(credentials_json: &str, referent: &str) -> CredentialInfo {
    let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
    let credentials_for_referent = credentials.attrs.get(referent).unwrap();
    credentials_for_referent[0].cred_info.clone()
}

pub fn get_credential_for_predicate_referent(credentials_json: &str, referent: &str) -> CredentialInfo {
    let credentials: CredentialsForProofRequest = serde_json::from_str(&credentials_json).unwrap();
    let credentials_for_referent = credentials.predicates.get(referent).unwrap();
    credentials_for_referent[0].cred_info.clone()
}

pub fn tails_writer_config() -> String {
    let mut base_dir = environment::tmp_path();
    base_dir.push("tails");

    let json = json!({
                "base_dir": base_dir.to_str().unwrap(),
                "uri_pattern":"",
            });
    json.to_string()
}

pub fn init_common_wallet() -> (&'static str, &'static str, &'static str, &'static str) {
    lazy_static! {
                    static ref COMMON_WALLET_INIT: Once = Once::new();
                 }

    unsafe {
        COMMON_WALLET_INIT.call_once(|| {
            // this name must match the one in ANONCREDS_WALLET_CONFIG
            test::cleanup_storage("anoncreds_wallet");

            //1. Create and Open wallet
            wallet::create_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();
            let wallet_handle = wallet::open_wallet(ANONCREDS_WALLET_CONFIG, WALLET_CREDENTIALS).unwrap();

            //2. Issuer1 Creates GVT CredentialDefinition
            let (issuer1_gvt_cred_deg_id, issuer1_gvt_credential_def_json) =
                issuer_create_credential_definition(wallet_handle,
                                                    ISSUER_DID,
                                                    &gvt_schema_json(),
                                                    TAG_1,
                                                    None,
                                                    Some(&default_cred_def_config())).unwrap();

            //2.1 Issuer1 Creates GVT Subscheme (for "names" tests, IS-1381)
            let (issuer1_gvt_sub_cred_def_id, issuer1_gvt_sub_credential_def_json) =
                issuer_create_credential_definition(wallet_handle,
                                                    ISSUER_DID_SUB,
                                                    &gvt_sub_schema_json(),
                                                    TAG_1,
                                                    None,
                                                    Some(&default_cred_def_config())).unwrap();

            //3. Issuer1 Creates XYZ CredentialDefinition
            let (issuer1_xyz_cred_deg_id, issuer1_xyz_credential_def_json) =
                issuer_create_credential_definition(wallet_handle,
                                                    ISSUER_DID,
                                                    &xyz_schema_json(),
                                                    TAG_1,
                                                    None,
                                                    Some(&default_cred_def_config())).unwrap();

            //4. Issuer2 Creates GVT CredentialDefinition
            let (issuer2_gvt_cred_def_id, issuer2_gvt_credential_def_json) =
                issuer_create_credential_definition(wallet_handle,
                                                    ISSUER_DID_2,
                                                    &gvt_schema_json(),
                                                    TAG_1,
                                                    None,
                                                    Some(&default_cred_def_config())).unwrap();

            //5. Issuer1 Creates GVT CredentialOffer
            let issuer1_gvt_credential_offer = issuer_create_credential_offer(wallet_handle, &issuer1_gvt_cred_deg_id).unwrap();

            //5.1 Issuer1 Creates GVT sub CredentialOffer
            let issuer1_gvt_sub_credential_offer = issuer_create_credential_offer(wallet_handle, &issuer1_gvt_sub_cred_def_id).unwrap();

            //6. Issuer1 Creates XYZ CredentialOffer
            let issuer1_xyz_credential_offer = issuer_create_credential_offer(wallet_handle, &issuer1_xyz_cred_deg_id).unwrap();

            //7. Issuer2 Creates GVT CredentialOffer
            let issuer2_gvt_credential_offer = issuer_create_credential_offer(wallet_handle, &issuer2_gvt_cred_def_id).unwrap();

            //8. Prover creates MasterSecret
            prover_create_master_secret(wallet_handle, COMMON_MASTER_SECRET).unwrap();

            // Issuer1 issues GVT Credential
            //9. Prover creates  Credential Request
            let (issuer1_gvt_credential_req, issuer1_gvt_credential_req_metadata) = prover_create_credential_req(wallet_handle,
                                                                                                                 DID_MY1,
                                                                                                                 &issuer1_gvt_credential_offer,
                                                                                                                 &issuer1_gvt_credential_def_json,
                                                                                                                 COMMON_MASTER_SECRET).unwrap();
            //10. Issuer1 creates GVT Credential
            let (issuer1_gvt_cred, _, _) = issuer_create_credential(wallet_handle,
                                                                    &issuer1_gvt_credential_offer,
                                                                    &issuer1_gvt_credential_req,
                                                                    &gvt_credential_values_json(),
                                                                    None,
                                                                    None).unwrap();

            //11. Prover stores Credential
            prover_store_credential(wallet_handle,
                                    CREDENTIAL1_ID,
                                    &issuer1_gvt_credential_req_metadata,
                                    &issuer1_gvt_cred,
                                    &issuer1_gvt_credential_def_json,
                                    None).unwrap();

            // Issuer1 issues GVT SUB Credential
            //9.1 Prover creates Credential Request
            let (issuer1_gvt_sub_credential_req, issuer1_gvt_sub_credential_req_metadata) = prover_create_credential_req(wallet_handle,
                                                                                                                         DID_MY1,
                                                                                                                         &issuer1_gvt_sub_credential_offer,
                                                                                                                         &issuer1_gvt_sub_credential_def_json,
                                                                                                                         COMMON_MASTER_SECRET).unwrap();
            //10.1 Issuer1 creates GVT Credential
            let (issuer1_gvt_sub_cred, _, _) = issuer_create_credential(wallet_handle,
                                                                        &issuer1_gvt_sub_credential_offer,
                                                                        &issuer1_gvt_sub_credential_req,
                                                                        &gvt_sub_credential_values_json(),
                                                                        None,
                                                                        None).unwrap();

            //11. Prover stores Credential
            prover_store_credential(wallet_handle,
                                    CREDENTIAL1_SUB_ID,
                                    &issuer1_gvt_sub_credential_req_metadata,
                                    &issuer1_gvt_sub_cred,
                                    &issuer1_gvt_sub_credential_def_json,
                                    None).unwrap();

            // Issuer1 issue XYZ Credential
            //12. Prover Creates Credential Request
            let (issuer1_xyz_credential_req, issuer1_xyz_credential_req_metadata) = prover_create_credential_req(wallet_handle,
                                                                                                                 DID_MY1,
                                                                                                                 &issuer1_xyz_credential_offer,
                                                                                                                 &issuer1_xyz_credential_def_json,
                                                                                                                 COMMON_MASTER_SECRET).unwrap();
            //13. Issuer1 Creates XYZ Credential
            let (issuer1_xyz_cred, _, _) = issuer_create_credential(wallet_handle,
                                                                    &issuer1_xyz_credential_offer,
                                                                    &issuer1_xyz_credential_req,
                                                                    &xyz_credential_values_json(),
                                                                    None,
                                                                    None).unwrap();

            //14. Prover stores Credential
            prover_store_credential(wallet_handle,
                                    CREDENTIAL2_ID,
                                    &issuer1_xyz_credential_req_metadata,
                                    &issuer1_xyz_cred,
                                    &issuer1_xyz_credential_def_json,
                                    None).unwrap();

            // Issuer2 issues GVT Credential
            //15. Prover Creates Credential Request
            let (issuer2_gvt_credential_req, issuer2_gvt_credential_req_metadata) = prover_create_credential_req(wallet_handle,
                                                                                                                 DID_MY1,
                                                                                                                 &issuer2_gvt_credential_offer,
                                                                                                                 &issuer2_gvt_credential_def_json,
                                                                                                                 COMMON_MASTER_SECRET).unwrap();

            //16. Issuer2 Creates XYZ Credential
            let (issuer2_gvt_cred, _, _) = issuer_create_credential(wallet_handle,
                                                                    &issuer2_gvt_credential_offer,
                                                                    &issuer2_gvt_credential_req,
                                                                    &gvt2_credential_values_json(),
                                                                    None,
                                                                    None).unwrap();

            //17. Prover Stores Credential
            prover_store_credential(wallet_handle,
                                    CREDENTIAL3_ID,
                                    &issuer2_gvt_credential_req_metadata,
                                    &issuer2_gvt_cred,
                                    &issuer2_gvt_credential_def_json,
                                    None).unwrap();

            let res = mem::transmute(&issuer1_gvt_credential_def_json as &str);
            mem::forget(issuer1_gvt_credential_def_json);
            CREDENTIAL_DEF_JSON = res;

            let res = mem::transmute(&issuer1_gvt_credential_offer as &str);
            mem::forget(issuer1_gvt_credential_offer);
            CREDENTIAL_OFFER_JSON = res;

            let res = mem::transmute(&issuer1_gvt_credential_req as &str);
            mem::forget(issuer1_gvt_credential_req);
            CREDENTIAL_REQUEST_JSON = res;

            let res = mem::transmute(&issuer1_gvt_cred as &str);
            mem::forget(issuer1_gvt_cred);
            CREDENTIAL_JSON = res;

            wallet::close_wallet(wallet_handle).unwrap();
        });

        (CREDENTIAL_DEF_JSON, CREDENTIAL_OFFER_JSON, CREDENTIAL_REQUEST_JSON, CREDENTIAL_JSON)
    }
}

pub fn multi_steps_issuer_preparation(wallet_handle: WalletHandle,
                                      did: &str,
                                      schema_name: &str,
                                      schema_attrs: &str) -> (String, String, String, String) {
    let (schema_id, schema_json) = issuer_create_schema(did,
                                                        schema_name,
                                                        SCHEMA_VERSION,
                                                        schema_attrs).unwrap();

    let (cred_def_id, cred_def_json) = issuer_create_credential_definition(wallet_handle,
                                                                           did,
                                                                           &schema_json,
                                                                           TAG_1,
                                                                           None,
                                                                           Some(&default_cred_def_config())).unwrap();

    (schema_id, schema_json, cred_def_id, cred_def_json)
}

pub fn multi_steps_issuer_revocation_preparation(wallet_handle: WalletHandle,
                                                 did: &str,
                                                 schema_name: &str,
                                                 schema_attrs: &str,
                                                 revoc_reg_def_config: &str) -> (String, String, String, String, String, String, String, i32) {
    // Issuer creates schema
    let (schema_id, schema_json) = issuer_create_schema(did,
                                                        schema_name,
                                                        SCHEMA_VERSION,
                                                        schema_attrs).unwrap();

    // Issuer creates credential definition
    let (cred_def_id, cred_def_json) = issuer_create_credential_definition(wallet_handle,
                                                                           did,
                                                                           &schema_json,
                                                                           TAG_1,
                                                                           None,
                                                                           Some(&revocation_cred_def_config())).unwrap();

    // Issuer creates revocation registry
    let tails_writer_config = tails_writer_config();
    let tails_writer_handle = blob_storage::open_writer("default", &tails_writer_config).unwrap();

    let (rev_reg_id, revoc_reg_def_json, revoc_reg_entry_json) =
        issuer_create_and_store_revoc_reg(wallet_handle,
                                          did,
                                          None,
                                          TAG_1,
                                          &cred_def_id,
                                          revoc_reg_def_config,
                                          tails_writer_handle).unwrap();

    let blob_storage_reader_handle = blob_storage::open_reader(TYPE, &tails_writer_config).unwrap();

    (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, revoc_reg_def_json, revoc_reg_entry_json, blob_storage_reader_handle)
}

pub fn multi_steps_create_credential(prover_master_secret_id: &str,
                                     prover_wallet_handle: WalletHandle,
                                     issuer_wallet_handle: WalletHandle,
                                     cred_id: &str,
                                     cred_values: &str,
                                     cred_def_id: &str,
                                     cred_def_json: &str) {
    // Issuer creates Credential Offer
    let cred_offer_json = issuer_create_credential_offer(issuer_wallet_handle, &cred_def_id).unwrap();

    // Prover creates Credential Request
    let (cred_req, cred_req_metadata) = prover_create_credential_req(prover_wallet_handle,
                                                                     DID_MY1,
                                                                     &cred_offer_json,
                                                                     &cred_def_json,
                                                                     prover_master_secret_id).unwrap();

    // Issuer creates Credential
    let (cred_json, _, _) = issuer_create_credential(issuer_wallet_handle,
                                                     &cred_offer_json,
                                                     &cred_req,
                                                     &cred_values,
                                                     None,
                                                     None).unwrap();

    // Prover stores received Credential
    prover_store_credential(prover_wallet_handle,
                            cred_id,
                            &cred_req_metadata,
                            &cred_json,
                            &cred_def_json,
                            None).unwrap();
}

pub fn multi_steps_create_revocation_credential(prover_master_secret_id: &str,
                                                prover_wallet_handle: WalletHandle,
                                                issuer_wallet_handle: WalletHandle,
                                                credential_id: &str,
                                                cred_values: &str,
                                                cred_def_id: &str,
                                                cred_def_json: &str,
                                                rev_reg_id: &str,
                                                revoc_reg_def_json: &str,
                                                blob_storage_reader_handle: i32)
                                                -> (String, Option<String>) {
    // Issuer creates Credential Offer for Prover
    let cred_offer_for_prover1_json = issuer_create_credential_offer(issuer_wallet_handle, cred_def_id).unwrap();

    // Prover creates Credential Request
    let (prover1_cred_req_json, prover1_cred_req_metadata_json) = prover_create_credential_req(prover_wallet_handle,
                                                                                               DID_MY1,
                                                                                               &cred_offer_for_prover1_json,
                                                                                               cred_def_json,
                                                                                               prover_master_secret_id).unwrap();

    // Issuer creates Credential for Prover1
    let (prover1_cred_json, prover1_cred_rev_id, revoc_reg_delta1_json) = issuer_create_credential(issuer_wallet_handle,
                                                                                                   &cred_offer_for_prover1_json,
                                                                                                   &prover1_cred_req_json,
                                                                                                   &cred_values,
                                                                                                   Some(rev_reg_id),
                                                                                                   Some(blob_storage_reader_handle)).unwrap();
    let revoc_reg_delta1_json = revoc_reg_delta1_json;
    let prover1_cred_rev_id = prover1_cred_rev_id.unwrap();

    // Prover1 stores Credential
    prover_store_credential(prover_wallet_handle,
                            credential_id,
                            &prover1_cred_req_metadata_json,
                            &prover1_cred_json,
                            &cred_def_json,
                            Some(&revoc_reg_def_json)).unwrap();

    (prover1_cred_rev_id, revoc_reg_delta1_json)
}
