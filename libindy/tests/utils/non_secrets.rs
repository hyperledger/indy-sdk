extern crate futures;

use indy::IndyError;
use crate::indy::future::Future;
use serde_json;

use indy::wallet;
use crate::utils::{test};
use crate::utils::constants::WALLET_CREDENTIALS;
use crate::utils::types::WalletRecord;

use std::sync::Once;
use std::collections::HashMap;

use indy::WalletHandle;

pub const SEARCH_COMMON_WALLET_CONFIG: &'static str = r#"{"id":"search_common"}"#;
pub const TYPE: &'static str = "TestType";
pub const TYPE_2: &'static str = "TestType2";
pub const ID: &'static str = "RecordId";
pub const ID_2: &'static str = "RecordId2";
pub const ID_3: &'static str = "RecordId3";
pub const ID_4: &'static str = "RecordId4";
pub const ID_5: &'static str = "RecordId5";
pub const VALUE: &'static str = "RecordValue";
pub const VALUE_2: &'static str = "RecordValue2";
pub const VALUE_3: &'static str = "RecordValue3";
pub const VALUE_4: &'static str = "RecordValue4";
pub const VALUE_5: &'static str = "RecordValue5";
pub const QUERY_EMPTY: &'static str = r#"{}"#;
pub const OPTIONS_EMPTY: &'static str = r#"{}"#;
pub const OPTIONS_ID_TYPE_VALUE: &'static str = r#"{"retrieveType":true, "retrieveValue":true, "retrieveTags":false}"#;
pub const OPTIONS_FULL: &'static str = r#"{"retrieveType":true, "retrieveValue":true, "retrieveTags":true, "retrieveTotalCount":true}"#;
pub const TAGS_EMPTY: &'static str = r#"{}"#;
pub const TAGS: &'static str = r#"{"tagName1":"str1","~tagName2":"5","~tagName3":"8"}"#;
pub const TAGS_2: &'static str = r#"{"tagName1":"str2","~tagName2":"pre_str3","~tagName3":"2"}"#;
pub const TAGS_3: &'static str = r#"{"tagName1":"str1","tagName2":"str2","tagName3":"str3"}"#;
pub const TAGS_4: &'static str = r#"{"tagName1":"somestr","~tagName2":"4","~tagName3":"5"}"#;
pub const TAGS_5: &'static str = r#"{"tagName1":"prefix_str2","~tagName2":"str3","~tagName3":"6"}"#;

pub fn add_wallet_record(wallet_handle: WalletHandle, type_: &str, id: &str, value: &str, tags_json: Option<&str>) -> Result<(), IndyError> {
    wallet::add_wallet_record(wallet_handle, type_, id, value, tags_json).wait()
}

pub fn update_wallet_record_value(wallet_handle: WalletHandle, type_: &str, id: &str, value: &str) -> Result<(), IndyError> {
    wallet::update_wallet_record_value(wallet_handle, type_, id, value).wait()
}

pub fn update_wallet_record_tags(wallet_handle: WalletHandle, type_: &str, id: &str, tags_json: &str) -> Result<(), IndyError> {
    wallet::update_wallet_record_tags(wallet_handle, type_, id, tags_json).wait()
}

pub fn add_wallet_record_tags(wallet_handle: WalletHandle, type_: &str, id: &str, tags_json: &str) -> Result<(), IndyError> {
    wallet::add_wallet_record_tags(wallet_handle, type_, id, tags_json).wait()
}

pub fn delete_wallet_record_tags(wallet_handle: WalletHandle, type_: &str, id: &str, tag_names_json: &str) -> Result<(), IndyError> {
    wallet::delete_wallet_record_tags(wallet_handle, type_, id, tag_names_json).wait()
}

pub fn delete_wallet_record(wallet_handle: WalletHandle, type_: &str, id: &str) -> Result<(), IndyError> {
    wallet::delete_wallet_record(wallet_handle, type_, id).wait()
}

pub fn get_wallet_record(wallet_handle: WalletHandle, type_: &str, id: &str, options_json: &str) -> Result<String, IndyError> {
    wallet::get_wallet_record(wallet_handle, type_, id, options_json).wait()
}

pub fn open_wallet_search(wallet_handle: WalletHandle, type_: &str, query_json: &str, options_json: &str) -> Result<i32, IndyError> {
    wallet::open_wallet_search(wallet_handle, type_, query_json, options_json).wait()
}

pub fn fetch_wallet_search_next_records(wallet_handle: WalletHandle, wallet_search_handle: i32, count: usize) -> Result<String, IndyError> {
    wallet::fetch_wallet_search_next_records(wallet_handle, wallet_search_handle, count).wait()
}

pub fn close_wallet_search(wallet_search_handle: i32) -> Result<(), IndyError> {
    wallet::close_wallet_search(wallet_search_handle).wait()
}

pub fn tags_1() -> HashMap<String, String> {
    serde_json::from_str(TAGS).unwrap()
}

pub fn tags_2() -> HashMap<String, String> {
    serde_json::from_str(TAGS_2).unwrap()
}

pub fn tags_3() -> HashMap<String, String> {
    serde_json::from_str(TAGS_3).unwrap()
}

pub fn tags_4() -> HashMap<String, String> {
    serde_json::from_str(TAGS_4).unwrap()
}

pub fn tags_5() -> HashMap<String, String> {
    serde_json::from_str(TAGS_5).unwrap()
}

pub fn record_1() -> WalletRecord {
    WalletRecord { id: ID.to_string(), type_: Some(TYPE.to_string()), value: Some(VALUE.to_string()), tags: Some(tags_1()) }
}

pub fn record_2() -> WalletRecord {
    WalletRecord { id: ID_2.to_string(), type_: Some(TYPE.to_string()), value: Some(VALUE_2.to_string()), tags: Some(tags_2()) }
}

pub fn record_3() -> WalletRecord {
    WalletRecord { id: ID_3.to_string(), type_: Some(TYPE.to_string()), value: Some(VALUE_3.to_string()), tags: Some(tags_3()) }
}

pub fn record_4() -> WalletRecord {
    WalletRecord { id: ID_4.to_string(), type_: Some(TYPE.to_string()), value: Some(VALUE_4.to_string()), tags: Some(tags_4()) }
}

pub fn record_5() -> WalletRecord {
    WalletRecord { id: ID_5.to_string(), type_: Some(TYPE.to_string()), value: Some(VALUE_5.to_string()), tags: Some(tags_5()) }
}

pub fn init_non_secret_test_wallet(name: &str, wallet_config: &str) {

    test::cleanup_storage(name);

    //1. Create and Open wallet
    wallet::create_wallet(wallet_config, WALLET_CREDENTIALS).wait().unwrap();
    let wallet_handle = wallet::open_wallet(wallet_config, WALLET_CREDENTIALS).wait().unwrap();

    let record_1 = record_1();
    add_wallet_record(wallet_handle,
                      TYPE,
                      &record_1.id,
                      &record_1.value.clone().unwrap(),
                      Some(TAGS)).unwrap();

    let record_2 = record_2();
    add_wallet_record(wallet_handle,
                      TYPE,
                      &record_2.id,
                      &record_2.value.clone().unwrap(),
                      Some(TAGS_2)).unwrap();

    let record_3 = record_3();
    add_wallet_record(wallet_handle,
                      TYPE,
                      &record_3.id,
                      &record_3.value.clone().unwrap(),
                      Some(TAGS_3)).unwrap();

    let record_4 = record_4();
    add_wallet_record(wallet_handle,
                      TYPE,
                      &record_4.id,
                      &record_4.value.clone().unwrap(),
                      Some(TAGS_4)).unwrap();

    let record_5 = record_5();
    add_wallet_record(wallet_handle,
                      TYPE,
                      &record_5.id,
                      &record_5.value.clone().unwrap(),
                      Some(TAGS_5)).unwrap();

    wallet::close_wallet(wallet_handle).wait().unwrap();
}

pub fn populate_common_wallet_for_search() {
    lazy_static! {
                    static ref COMMON_WALLET_INIT: Once = Once::new();

                }

    COMMON_WALLET_INIT.call_once(|| {
        const SEARCH_WALLET_CONFIG: &str = r#"{"id":"common_non_secret_wallet"}"#;
        init_non_secret_test_wallet("common_non_secret_wallet", SEARCH_WALLET_CONFIG)
    });
}
