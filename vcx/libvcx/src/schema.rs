use serde_json;

use std::string::ToString;

use api::PublicEntityStateType;
use utils::libindy::anoncreds;
use utils::libindy::ledger;
use utils::libindy::payments::PaymentTxn;
use utils::constants::DEFAULT_SERIALIZE_VERSION;
use object_cache::ObjectCache;
use messages::ObjectWithVersion;
use error::prelude::*;

lazy_static! {
    static ref SCHEMA_MAP: ObjectCache<CreateSchema> = Default::default();
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaData {
    name: String,
    version: String,
    #[serde(rename = "attrNames")]
    attr_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateSchema {
    data: Vec<String>,
    version: String,
    schema_id: String,
    name: String,
    source_id: String,
    payment_txn: Option<PaymentTxn>,
    #[serde(default)]
    state: PublicEntityStateType,
}

impl CreateSchema {
    pub fn get_source_id(&self) -> &String { &self.source_id }

    pub fn get_schema_id(&self) -> &String { &self.schema_id }

    fn get_payment_txn(&self) -> VcxResult<PaymentTxn> {
        trace!("CreateSchema::get_payment_txn >>>");
        self.payment_txn.clone()
            .ok_or(VcxError::from(VcxErrorKind::NoPaymentInformation))
    }

    fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.extend("Cannot serialize Schema"))
    }

    fn from_str(data: &str) -> VcxResult<CreateSchema> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<CreateSchema>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize Schema"))
    }

    fn update_state(&mut self) -> VcxResult<u32> {
        if anoncreds::get_schema_json(&self.schema_id).is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    fn get_state(&self) -> u32 { self.state as u32 }
}

pub fn create_and_publish_schema(source_id: &str,
                                 issuer_did: String,
                                 name: String,
                                 version: String,
                                 data: String) -> VcxResult<u32> {
    trace!("create_new_schema >>> source_id: {}, issuer_did: {}, name: {}, version: {}, data: {}", source_id, issuer_did, name, version, data);
    debug!("creating schema with source_id: {}, name: {}, issuer_did: {}", source_id, name, issuer_did);

    let (schema_id, schema) = anoncreds::create_schema(&name, &version, &data)?;
    let payment_txn = anoncreds::publish_schema(&schema)?;

    debug!("created schema on ledger with id: {}", schema_id);

    let schema_handle = _store_schema(source_id, name, version, schema_id, data, payment_txn, PublicEntityStateType::Published)?;

    Ok(schema_handle)
}

pub fn prepare_schema_for_endorser(source_id: &str,
                                   issuer_did: String,
                                   name: String,
                                   version: String,
                                   data: String,
                                   endorser: String) -> VcxResult<(u32, String)> {
    trace!("create_schema_for_endorser >>> source_id: {}, issuer_did: {}, name: {}, version: {}, data: {}, endorser: {}", source_id, issuer_did, name, version, data, endorser);
    debug!("preparing schema for endorser with source_id: {}, name: {}, issuer_did: {}", source_id, name, issuer_did);

    let (schema_id, schema) = anoncreds::create_schema(&name, &version, &data)?;
    let schema_request = anoncreds::build_schema_request(&schema)?;
    let schema_request = ledger::set_endorser(&schema_request, &endorser)?;

    debug!("prepared schema for endorser with id: {}", schema_id);

    let schema_handle = _store_schema(source_id, name, version, schema_id, data, None, PublicEntityStateType::Built)?;

    Ok((schema_handle, schema_request))
}

fn _store_schema(source_id: &str,
                 name: String,
                 version: String,
                 schema_id: String,
                 data: String,
                 payment_txn: Option<PaymentTxn>,
                 state: PublicEntityStateType) -> VcxResult<u32> {
    let schema = CreateSchema {
        source_id: source_id.to_string(),
        name,
        data: serde_json::from_str(&data).unwrap_or_default(),
        version,
        schema_id,
        payment_txn,
        state,
    };

    SCHEMA_MAP.add(schema)
        .or(Err(VcxError::from(VcxErrorKind::CreateSchema)))
}

pub fn get_schema_attrs(source_id: String, schema_id: String) -> VcxResult<(u32, String)> {
    trace!("get_schema_attrs >>> source_id: {}, schema_id: {}", source_id, schema_id);

    let (schema_id, schema_data_json) = anoncreds::get_schema_json(&schema_id)
        .map_err(|err| err.map(VcxErrorKind::InvalidSchemaSeqNo, "Schema not found"))?;

    let schema_data: SchemaData = serde_json::from_str(&schema_data_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize schema: {}", err)))?;

    let schema = CreateSchema {
        source_id,
        schema_id,
        name: schema_data.name,
        version: schema_data.version,
        data: schema_data.attr_names,
        payment_txn: None,
        state: PublicEntityStateType::Published,
    };

    let schema_json = schema.to_string()?;

    let handle = SCHEMA_MAP.add(schema)
        .or(Err(VcxError::from(VcxErrorKind::CreateSchema)))?;

    Ok((handle, schema_json))
}

pub fn is_valid_handle(handle: u32) -> bool {
    SCHEMA_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| {
        s.to_string()
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| {
        Ok(s.get_source_id().to_string())
    })
}

pub fn get_schema_id(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| {
        Ok(s.get_schema_id().to_string())
    })
}

pub fn get_payment_txn(handle: u32) -> VcxResult<PaymentTxn> {
    SCHEMA_MAP.get(handle, |s| {
        s.get_payment_txn()
    })
}

pub fn from_string(schema_data: &str) -> VcxResult<u32> {
    let schema: CreateSchema = CreateSchema::from_str(schema_data)?;
    SCHEMA_MAP.add(schema)
}

pub fn release(handle: u32) -> VcxResult<()> {
    SCHEMA_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidSchemaHandle)))
}

pub fn release_all() {
    SCHEMA_MAP.drain().ok();
}

pub fn update_state(handle: u32) -> VcxResult<u32> {
    SCHEMA_MAP.get_mut(handle, |s| {
        s.update_state()
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    SCHEMA_MAP.get_mut(handle, |s| {
        Ok(s.get_state())
    })
}

#[cfg(test)]
pub mod tests {
    extern crate rand;

    use settings;

    use super::*;
    use rand::Rng;
    use utils::constants::SCHEMA_ID;
    use utils::devsetup::*;
    #[cfg(feature = "pool_tests")]
    use utils::libindy::payments::add_new_did;
    #[cfg(feature = "pool_tests")]
    use utils::libindy::anoncreds::tests::create_and_write_test_schema;
    #[cfg(feature = "pool_tests")]
    use utils::constants;

    fn data() -> Vec<String> {
        vec!["address1".to_string(), "address2".to_string(), "zip".to_string(), "city".to_string(), "state".to_string()]
    }

    pub fn prepare_schema_data() -> (String, String, String, String) {
        let data = json!(data()).to_string();
        let schema_name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
        let schema_version: String = format!("{}.{}", rand::thread_rng().gen::<u32>().to_string(),
                                             rand::thread_rng().gen::<u32>().to_string());
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        (did, schema_name, schema_version, data)
    }

    pub fn create_schema_real() -> u32 {
        let (did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema("id", did, schema_name, schema_version, data).unwrap()
    }

    fn check_schema(schema_handle: u32, schema_json: &str, schema_id: &str, data: &str) {
        let schema: CreateSchema = CreateSchema::from_str(schema_json).unwrap();
        assert_eq!(schema.schema_id, schema_id.to_string());
        assert_eq!(schema.data.clone().sort(), vec!(data).sort());
        assert!(schema_handle > 0);
    }

    #[test]
    fn test_create_schema_to_string() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        let handle = create_and_publish_schema("test_create_schema_success",
                                               did,
                                               schema_name,
                                               schema_version,
                                               data.clone()).unwrap();

        let schema_id = get_schema_id(handle).unwrap();
        let create_schema_json = to_string(handle).unwrap();

        let value: serde_json::Value = serde_json::from_str(&create_schema_json).unwrap();
        assert_eq!(value["version"], "1.0");
        assert!(value["data"].is_object());

        let handle = from_string(&create_schema_json).unwrap();

        assert_eq!(get_source_id(handle).unwrap(), String::from("test_create_schema_success"));
        check_schema(handle, &create_schema_json, &schema_id, &data);
    }

    #[test]
    fn test_create_schema_success() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema("test_create_schema_success",
                                  did,
                                  schema_name,
                                  schema_version,
                                  data).unwrap();
    }

    #[test]
    fn test_prepare_schema_success() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        prepare_schema_for_endorser("test_create_schema_success",
                                    did,
                                    schema_name,
                                    schema_version,
                                    data,
                                    "V4SGRU86Z58d6TV7PBUe6f".to_string()).unwrap();
    }

    #[test]
    fn test_get_schema_attrs_success() {
        let _setup = SetupMocks::init();

        let (handle, schema_json) = get_schema_attrs("Check For Success".to_string(), SCHEMA_ID.to_string()).unwrap();

        check_schema(handle, &schema_json, SCHEMA_ID, r#"["name","age","height","sex"]"#);
    }

    #[test]
    fn test_create_schema_fails() {
        let _setup = SetupDefaults::init();

        let err = create_and_publish_schema("1", "VsKV7grR1BUE29mG2Fm2kX".to_string(),
                                            "name".to_string(),
                                            "1.0".to_string(),
                                            "".to_string()).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidLibindyParam)
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_schema_attrs_from_ledger() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (schema_id, _) = create_and_write_test_schema(constants::DEFAULT_SCHEMA_ATTRS);

        let (schema_handle, schema_attrs) = get_schema_attrs("id".to_string(), schema_id.clone()).unwrap();

        check_schema(schema_handle, &schema_attrs, &schema_id, constants::DEFAULT_SCHEMA_ATTRS);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_schema_with_pool() {
        let _setup = SetupLibraryWalletPool::init();

        let handle = create_schema_real();

        let _source_id = get_source_id(handle).unwrap();
        let _schema_id = get_schema_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();

        let payment = &get_payment_txn(handle).unwrap();
        assert!(payment.amount > 0);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_schema_no_fees_with_pool() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let handle = create_schema_real();

        let _source_id = get_source_id(handle).unwrap();
        let _schema_id = get_schema_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();

        // No Payment performed
        let _payment = get_payment_txn(handle).unwrap_err();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_duplicate_fails_no_fees() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();

        create_and_publish_schema("id", did.clone(), schema_name.clone(), schema_version.clone(), data.clone()).unwrap();

        let err = create_and_publish_schema("id_2", did, schema_name, schema_version, data).unwrap_err();

        assert_eq!(err.kind(), VcxErrorKind::DuplicationSchema)
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let (did, schema_name, version, data) = prepare_schema_data();

        let h1 = create_and_publish_schema("1", did.clone(), schema_name.clone(), version.clone(), data.clone()).unwrap();
        let h2 = create_and_publish_schema("2", did.clone(), schema_name.clone(), version.clone(), data.clone()).unwrap();
        let h3 = create_and_publish_schema("3", did.clone(), schema_name.clone(), version.clone(), data.clone()).unwrap();
        let h4 = create_and_publish_schema("4", did.clone(), schema_name.clone(), version.clone(), data.clone()).unwrap();
        let h5 = create_and_publish_schema("5", did.clone(), schema_name.clone(), version.clone(), data.clone()).unwrap();

        release_all();

        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
    }

    #[test]
    fn test_handle_errors() {
        let _setup = SetupEmpty::init();

        assert_eq!(to_string(13435178).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_endorse_schema() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();

        let (endorser_did, _) = add_new_did(Some("ENDORSER"));

        let (handle, schema_request) = prepare_schema_for_endorser("test_vcx_schema_update_state_with_ledger", did, schema_name, schema_version, data, endorser_did.clone()).unwrap();
        assert_eq!(0, get_state(handle).unwrap());
        assert_eq!(0, update_state(handle).unwrap());

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &endorser_did);
        ledger::endorse_transaction(&schema_request).unwrap();

        ::std::thread::sleep(::std::time::Duration::from_millis(1000));

        assert_eq!(1, update_state(handle).unwrap());
        assert_eq!(1, get_state(handle).unwrap());
        ::utils::libindy::wallet::close_wallet();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_schema_get_state_with_ledger() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let handle = create_schema_real();
        assert_eq!(1, get_state(handle).unwrap());
        ::utils::libindy::wallet::close_wallet();
    }
}
