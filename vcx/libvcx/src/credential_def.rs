use serde_json;

use api::PublicEntityStateType;
use object_cache::ObjectCache;
use messages::ObjectWithVersion;
use error::prelude::*;
use utils::constants::DEFAULT_SERIALIZE_VERSION;
use utils::libindy::payments::PaymentTxn;
use utils::libindy::anoncreds;
use utils::libindy::ledger;

lazy_static! {
    static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> = Default::default();
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct CredentialDef {
    id: String,
    tag: String,
    name: String,
    source_id: String,
    issuer_did: Option<String>,
    cred_def_payment_txn: Option<PaymentTxn>,
    rev_reg_def_payment_txn: Option<PaymentTxn>,
    rev_reg_delta_payment_txn: Option<PaymentTxn>,
    rev_reg_id: Option<String>,
    rev_reg_def: Option<String>,
    rev_reg_entry: Option<String>,
    tails_file: Option<String>,
    #[serde(default)]
    state: PublicEntityStateType
}

#[derive(Deserialize, Debug, Serialize)]
pub struct RevocationDetails {
    pub support_revocation: Option<bool>,
    pub tails_file: Option<String>,
    pub max_creds: Option<u32>,
}

impl CredentialDef {
    pub fn from_str(data: &str) -> VcxResult<CredentialDef> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<CredentialDef>| obj.data)
            .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot deserialize CredentialDefinition"))
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.extend("Cannot serialize CredentialDefinition"))
    }

    pub fn get_source_id(&self) -> &String { &self.source_id }

    pub fn get_rev_reg_id(&self) -> Option<&String> { self.rev_reg_id.as_ref() }

    pub fn get_tails_file(&self) -> Option<&String> { self.tails_file.as_ref() }

    pub fn get_rev_reg_def(&self) -> Option<&String> { self.rev_reg_def.as_ref() }

    pub fn get_cred_def_id(&self) -> &String { &self.id }

    pub fn set_name(&mut self, name: String) { self.name = name.clone(); }

    pub fn set_source_id(&mut self, source_id: String) { self.source_id = source_id.clone(); }

    fn get_cred_def_payment_txn(&self) -> VcxResult<PaymentTxn> {
        self.cred_def_payment_txn.clone()
            .ok_or(VcxError::from(VcxErrorKind::NoPaymentInformation))
    }

    fn get_rev_reg_def_payment_txn(&self) -> Option<PaymentTxn> { self.rev_reg_def_payment_txn.clone() }

    fn get_rev_reg_delta_payment_txn(&self) -> Option<PaymentTxn> { self.rev_reg_delta_payment_txn.clone() }

    fn update_state(&mut self) -> VcxResult<u32> {
        if let Some(ref rev_reg_id) = self.rev_reg_id.as_ref() {
            if let (Ok(_), Ok(_), Ok(_)) = (anoncreds::get_cred_def_json(&self.id),
                                            anoncreds::get_rev_reg_def_json(rev_reg_id),
                                            anoncreds::get_rev_reg(rev_reg_id, ::time::get_time().sec as u64)) {
                self.state = PublicEntityStateType::Published
            }
        } else {
            if let Ok(_) = anoncreds::get_cred_def_json(&self.id) {
                self.state = PublicEntityStateType::Published
            }
        }

        Ok(self.state as u32)
    }

    fn get_state(&self) -> u32 { self.state as u32 }
}

fn _parse_revocation_details(revocation_details: &str) -> VcxResult<RevocationDetails> {
    serde_json::from_str::<RevocationDetails>(&revocation_details)
        .to_vcx(VcxErrorKind::InvalidRevocationDetails, "Cannot deserialize RevocationDeltas")
}

fn _create_credentialdef(issuer_did: &str,
                         schema_id: &str,
                         tag: &str,
                         revocation_details: &RevocationDetails) -> VcxResult<(String, String, Option<String>, Option<String>, Option<String>)> {
    let (_, schema_json) = anoncreds::get_schema_json(&schema_id)?;

    let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(issuer_did,
                                                                    &schema_json,
                                                                    tag,
                                                                    None,
                                                                    revocation_details.support_revocation)?;

    let (rev_reg_id, rev_reg_def, rev_reg_entry) = match revocation_details.support_revocation {
        Some(true) => {
            let tails_file = revocation_details
                .tails_file
                .as_ref()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `tails_file` field not found"))?;

            let max_creds = revocation_details
                .max_creds
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `max_creds` field not found"))?;

            let (rev_reg_id, rev_reg_def, rev_reg_entry) =
                anoncreds::generate_rev_reg(&issuer_did, &cred_def_id, &tails_file, max_creds)
                    .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

            (Some(rev_reg_id), Some(rev_reg_def), Some(rev_reg_entry))
        }
        _ => (None, None, None),
    };

    Ok((cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry))
}

pub fn prepare_credentialdef_for_endorser(source_id: String,
                                          name: String,
                                          issuer_did: String,
                                          schema_id: String,
                                          tag: String,
                                          revocation_details: String,
                                          endorser: String) -> VcxResult<(u32, String, Option<String>, Option<String>)> {
    trace!("prepare_credentialdef_for_endorser >>> source_id: {}, name: {}, issuer_did: {}, schema_id: {}, revocation_details: {}, endorser: {}",
           source_id, name, issuer_did, schema_id, revocation_details, endorser);

    let revocation_details: RevocationDetails = _parse_revocation_details(&revocation_details)?;

    // Creates Credential Definition and Revocation Definition in wallet
    let (cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry) = _create_credentialdef(&issuer_did, &schema_id, &tag, &revocation_details)?;

    // Creates Credential Definition request
    let cred_def_req = anoncreds::build_cred_def_request(&issuer_did, &cred_def_json)?;
    let cred_def_req = ledger::set_endorser(&cred_def_req, &endorser)?;

    // Creates Revocation related requests
    let (rev_reg_def_req, rev_reg_delta_req) = match (&rev_reg_id, &rev_reg_def, &rev_reg_entry) {
        (Some(ref rev_reg_id), Some(ref rev_reg_def), Some(ref rev_reg_entry)) => {
            let rev_reg_def_req =
                anoncreds::build_rev_reg_request(&issuer_did, &rev_reg_def)
                    .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

            let rev_reg_delta_req = anoncreds::build_rev_reg_delta_request(&issuer_did, &rev_reg_id, &rev_reg_entry)
                .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

            let rev_reg_delta_req = ledger::set_endorser(&rev_reg_delta_req, &endorser)?;
            let rev_reg_def_req = ledger::set_endorser(&rev_reg_def_req, &endorser)?;

            (Some(rev_reg_def_req), Some(rev_reg_delta_req))
        }
        _ => (None, None)
    };

    let cred_def = CredentialDef {
        source_id,
        name,
        tag,
        id: cred_def_id,
        issuer_did: Some(issuer_did),
        cred_def_payment_txn: None,
        rev_reg_def_payment_txn: None,
        rev_reg_delta_payment_txn: None,
        rev_reg_id,
        rev_reg_def,
        rev_reg_entry,
        tails_file: revocation_details.tails_file,
        state: PublicEntityStateType::Built,
    };

    let handle = CREDENTIALDEF_MAP.add(cred_def).or(Err(VcxError::from(VcxErrorKind::CreateCredDef)))?;

    Ok((handle, cred_def_req, rev_reg_def_req, rev_reg_delta_req))
}

pub fn create_and_publish_credentialdef(source_id: String,
                                        name: String,
                                        issuer_did: String,
                                        schema_id: String,
                                        tag: String,
                                        revocation_details: String) -> VcxResult<u32> {
    trace!("create_new_credentialdef >>> source_id: {}, name: {}, issuer_did: {}, schema_id: {}, revocation_details: {}",
           source_id, name, issuer_did, schema_id, revocation_details);

    let revocation_details: RevocationDetails = _parse_revocation_details(&revocation_details)?;

    // Creates Credential Definition and Revocation Definition in wallet
    let (cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry) = _create_credentialdef(&issuer_did, &schema_id, &tag, &revocation_details)?;

    // Publish Credential Definition on the ledger
    let cred_def_payment_txn = anoncreds::publish_cred_def(&issuer_did, &cred_def_json)?;

    // Publish Revocation related requests on the ledger
    let (rev_def_payment, rev_delta_payment) = match (&rev_reg_id, &rev_reg_def, &rev_reg_entry) {
        (Some(ref rev_reg_id), Some(ref rev_reg_def), Some(ref rev_reg_entry)) => {
            let rev_def_payment = anoncreds::publish_rev_reg_def(&issuer_did, &rev_reg_def)
                .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

            let (rev_delta_payment, _) = anoncreds::publish_rev_reg_delta(&issuer_did, &rev_reg_id, &rev_reg_entry)
                .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

            (rev_def_payment, rev_delta_payment)
        }
        _ => (None, None)
    };

    let cred_def = CredentialDef {
        source_id,
        name,
        tag,
        id: cred_def_id,
        issuer_did: Some(issuer_did),
        cred_def_payment_txn,
        rev_reg_def_payment_txn: rev_def_payment,
        rev_reg_delta_payment_txn: rev_delta_payment,
        rev_reg_id,
        rev_reg_def,
        rev_reg_entry,
        tails_file: revocation_details.tails_file,
        state: PublicEntityStateType::Published,
    };

    let handle = CREDENTIALDEF_MAP.add(cred_def).or(Err(VcxError::from(VcxErrorKind::CreateCredDef)))?;

    Ok(handle)
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| {
        cd.to_string()
    })
}

pub fn from_string(data: &str) -> VcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_str(data)?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_source_id().clone())
    })
}

pub fn get_cred_def_payment_txn(handle: u32) -> VcxResult<PaymentTxn> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        c.get_cred_def_payment_txn()
    })
}

pub fn get_cred_def_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_cred_def_id().clone())
    })
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_id().cloned())
    })
}

pub fn get_tails_file(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_tails_file().cloned())
    })
}

pub fn get_rev_reg_def(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_def().cloned())
    })
}

pub fn get_rev_reg_def_payment_txn(handle: u32) -> VcxResult<Option<PaymentTxn>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_def_payment_txn())
    })
}


pub fn get_rev_reg_delta_payment_txn(handle: u32) -> VcxResult<Option<PaymentTxn>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_delta_payment_txn())
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle)))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub fn update_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        s.update_state()
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(s.get_state())
    })
}

pub fn check_is_published(handle: u32) -> VcxResult<bool> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(PublicEntityStateType::Published == s.state)
    })
}

#[cfg(test)]
pub mod tests {
    use utils::{
        constants::SCHEMA_ID,
        get_temp_dir_path,
    };
    use super::*;
    use settings;
    use std::{
        thread::sleep,
        time::Duration,
    };
    use utils::devsetup::*;
    #[cfg(feature = "pool_tests")]
    use utils::libindy::payments::add_new_did;

    static CREDENTIAL_DEF_NAME: &str = "Test Credential Definition";
    static ISSUER_DID: &str = "4fUDR9R7fjwELRvH9JT6HH";

    pub fn revocation_details(revoc: bool) -> serde_json::Value {
        let mut revocation_details = json!({"support_revocation":revoc});
        if revoc {
            revocation_details["tails_file"] = json!(get_temp_dir_path("tails_file.txt").to_str().unwrap());
            revocation_details["max_creds"] = json!(10);
        }
        revocation_details
    }

    pub fn prepare_create_cred_def_data(revoc: bool) -> (u32, String, String, serde_json::Value) {
        let schema_handle = ::schema::tests::create_schema_real();
        sleep(Duration::from_secs(2));
        let schema_id = ::schema::get_schema_id(schema_handle).unwrap();
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let revocation_details = revocation_details(revoc);
        (schema_handle, schema_id, did, revocation_details)
    }

    pub fn create_cred_def_real(revoc: bool) -> (u32, u32) {
        let (schema_handle, schema_id, did, revocation_details) = prepare_create_cred_def_data(revoc);
        sleep(Duration::from_secs(2));
        let cred_def_handle = create_and_publish_credentialdef("1".to_string(),
                                                               CREDENTIAL_DEF_NAME.to_string(),
                                                               did,
                                                               schema_id,
                                                               "tag_1".to_string(),
                                                               revocation_details.to_string()).unwrap();

        (schema_handle, cred_def_handle)
    }

    pub fn create_cred_def_fake() -> u32 {
        create_and_publish_credentialdef("SourceId".to_string(),
                                         CREDENTIAL_DEF_NAME.to_string(),
                                         ISSUER_DID.to_string(),
                                         SCHEMA_ID.to_string(),
                                         "tag".to_string(),
                                         "{}".to_string()).unwrap()
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_cred_def_without_rev_will_have_no_rev_id() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, handle) = create_cred_def_real(false);
        let rev_reg_id = get_rev_reg_id(handle).unwrap();
        assert!(rev_reg_id.is_none());

        let (_, handle) = create_cred_def_real(true);
        let rev_reg_id = get_rev_reg_id(handle).unwrap();
        assert!(rev_reg_id.is_some());
    }

    #[test]
    fn test_create_cred_def() {
        let _setup = SetupMocks::init();

        let (_, handle) = create_cred_def_real(false);

        let payment = &get_cred_def_payment_txn(handle).unwrap();
        assert!(payment.amount > 0);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_credential_def() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        let (_, _, cred_def_id, cred_def_json, _, _) = ::utils::libindy::anoncreds::tests::create_and_store_credential_def(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let (id, r_cred_def_json) = ::utils::libindy::anoncreds::get_cred_def_json(&cred_def_id).unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_fails_with_no_tails_file() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (schema_id, _) = ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let rc = create_and_publish_credentialdef("1".to_string(),
                                                  "test_create_revocable_fails_with_no_tails_file".to_string(),
                                                  did,
                                                  schema_id,
                                                  "tag_1".to_string(),
                                                  r#"{"support_revocation":true}"#.to_string());
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_cred_def_with_payments() {
        let _setup = SetupLibraryWalletPool::init();

        let (schema_id, _) = ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_create_revocable_cred_def".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag_1".to_string(),
                                                      revocation_details).unwrap();

        assert!(get_rev_reg_def(handle).unwrap().is_some());
        assert!(get_rev_reg_id(handle).unwrap().is_some());
        assert!(get_rev_reg_def_payment_txn(handle).unwrap().is_some());
        assert!(get_rev_reg_delta_payment_txn(handle).unwrap().is_some());
        let cred_id = get_cred_def_id(handle).unwrap();
        ::utils::libindy::anoncreds::get_cred_def_json(&cred_id).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_def_real() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, handle) = create_cred_def_real(false);

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();

        let payment = &get_cred_def_payment_txn(handle).unwrap();
        assert!(payment.amount > 0);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_def_no_fees_real() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, handle) = create_cred_def_real(false);

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();

        // No Payment performed
        let _payment = get_cred_def_payment_txn(handle).unwrap_err();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_duplicate_credential() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, schema_id, did, revocation_details) = prepare_create_cred_def_data(false);

        let handle_1 = create_and_publish_credentialdef("1".to_string(),
                                                        "name".to_string(),
                                                        did.clone(),
                                                        schema_id.clone(),
                                                        "tag_1".to_string(),
                                                        revocation_details.to_string()).unwrap();

        let handle_2 = create_and_publish_credentialdef("1".to_string(),
                                                        "name".to_string(),
                                                        did.clone(),
                                                        schema_id.clone(),
                                                        "tag_1".to_string(),
                                                        revocation_details.to_string()).unwrap();

        assert_ne!(handle_1, handle_2);
    }

    #[test]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();

        let credential_string = to_string(handle).unwrap();
        let credential_values: serde_json::Value = serde_json::from_str(&credential_string).unwrap();
        assert_eq!(credential_values["version"].clone(), "1.0");
    }

    #[test]
    fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();
        let credentialdef_data = to_string(handle).unwrap();
        assert!(!credentialdef_data.is_empty());
        release(handle).unwrap();

        let new_handle = from_string(&credentialdef_data).unwrap();
        let new_credentialdef_data = to_string(new_handle).unwrap();

        let credentialdef1: CredentialDef = CredentialDef::from_str(&credentialdef_data).unwrap();
        let credentialdef2: CredentialDef = CredentialDef::from_str(&new_credentialdef_data).unwrap();

        assert_eq!(credentialdef1, credentialdef2);
        assert_eq!(CredentialDef::from_str("{}").unwrap_err().kind(), VcxErrorKind::CreateCredDef);
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h2 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h3 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h4 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h5 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_endorse_cred_def() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, schema_id, did, revocation_details) = prepare_create_cred_def_data(false);

        let (endorser_did, _) = add_new_did(Some("ENDORSER"));

        let (handle, cred_def_request, rev_reg_def_req, rev_reg_entry_req) = prepare_credentialdef_for_endorser("test_vcx_endorse_cred_def".to_string(), "Test Credential Def".to_string(), did, schema_id, "tag".to_string(), revocation_details.to_string(), endorser_did.clone()).unwrap();
        assert_eq!(0, get_state(handle).unwrap());
        assert_eq!(0, update_state(handle).unwrap());
        assert!(rev_reg_def_req.is_none());
        assert!(rev_reg_entry_req.is_none());

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &endorser_did);
        ledger::endorse_transaction(&cred_def_request).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_millis(1000));

        assert_eq!(1, update_state(handle).unwrap());
        assert_eq!(1, get_state(handle).unwrap());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_endorse_cred_def_with_revocation() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, schema_id, did, revocation_details) = prepare_create_cred_def_data(true);

        let (endorser_did, _) = add_new_did(Some("ENDORSER"));

        let (handle, cred_def_request, rev_reg_def_req, rev_reg_entry_req) = prepare_credentialdef_for_endorser("test_vcx_endorse_cred_def".to_string(), "Test Credential Def".to_string(), did, schema_id, "tag".to_string(), revocation_details.to_string(), endorser_did.clone()).unwrap();
        assert_eq!(0, get_state(handle).unwrap());
        assert_eq!(0, update_state(handle).unwrap());

        let rev_reg_def_req = rev_reg_def_req.unwrap();
        let rev_reg_entry_req = rev_reg_entry_req.unwrap();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &endorser_did);
        ledger::endorse_transaction(&cred_def_request).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_millis(500));
        assert_eq!(0, update_state(handle).unwrap());

        ledger::endorse_transaction(&rev_reg_def_req).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_millis(500));
        assert_eq!(0, update_state(handle).unwrap());

        ledger::endorse_transaction(&rev_reg_entry_req).unwrap();
        ::std::thread::sleep(::std::time::Duration::from_millis(500));

        assert_eq!(1, update_state(handle).unwrap());
        assert_eq!(1, get_state(handle).unwrap());
    }
}
