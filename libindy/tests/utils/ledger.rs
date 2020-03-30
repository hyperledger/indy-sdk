extern crate futures;
extern crate indy_sys;

use indy::{IndyError, ErrorCode};
use indy::ledger;
use indy_utils::crypto::hash::hash;
use crate::indy::future::Future;
use self::indy_sys::ledger::{CustomTransactionParser, CustomFree, indy_register_transaction_parser_for_sp};

use crate::utils::{timeout, anoncreds, blob_storage, did, wallet, pool, callback};
use crate::utils::constants::*;

use std::sync::{Once};
use std::mem;
use std::ffi::CString;

use indy::{WalletHandle, PoolHandle};

pub static mut SCHEMA_ID: &'static str = "";
pub static mut SCHEMA_ID_V2: &'static str = "";
pub static mut CRED_DEF_ID: &'static str = "";
pub static mut CRED_DEF_ID_V2: &'static str = "";
pub static mut REV_REG_DEF_ID: &'static str = "";
pub const SCHEMA_DATA: &'static str = r#"{"id":"id","name":"gvt","version":"1.0","attr_names":["name", "age", "sex", "height"]}"#;

const SUBMIT_RETRY_CNT: usize = 3;

pub fn sign_and_submit_request(pool_handle: PoolHandle, wallet_handle: WalletHandle, submitter_did: &str, request_json: &str) -> Result<String, IndyError> {
    ledger::sign_and_submit_request(pool_handle, wallet_handle, submitter_did, request_json).wait()
}

pub fn submit_request_with_retries(pool_handle: PoolHandle, request_json: &str, previous_response: &str) -> Result<String, IndyError> {
    _submit_retry(extract_seq_no_from_reply(previous_response).unwrap(), || {
        submit_request(pool_handle, request_json)
    })
}

pub fn submit_request(pool_handle: PoolHandle, request_json: &str) -> Result<String, IndyError> {
    ledger::submit_request(pool_handle, request_json).wait()
}

pub fn submit_action(pool_handle: PoolHandle, request_json: &str, nodes: Option<&str>, timeout: Option<i32>) -> Result<String, IndyError> {
    ledger::submit_action(pool_handle, request_json, nodes, timeout).wait()
}

pub fn sign_request(wallet_handle: WalletHandle, submitter_did: &str, request_json: &str) -> Result<String, IndyError> {
    ledger::sign_request(wallet_handle, submitter_did, request_json).wait()
}

pub fn multi_sign_request(wallet_handle: WalletHandle, submitter_did: &str, request_json: &str) -> Result<String, IndyError> {
    ledger::multi_sign_request(wallet_handle, submitter_did, request_json).wait()
}

pub fn extract_seq_no_from_reply(reply: &str) -> Result<u64, &'static str> {
    let metadata = get_response_metadata(reply).map_err(|_| "Can not get Metadata from Reply")?;

    ::serde_json::from_str::<::serde_json::Value>(&metadata).map_err(|_| "Metadata isn't valid JSON")?
        ["seqNo"]
        .as_u64().ok_or("Missed seqNo in reply")
}

fn _submit_retry<F>(minimal_timestamp: u64, submit_action: F) -> Result<String, IndyError>
    where F: Fn() -> Result<String, IndyError> {
    let mut i = 0;
    let action_result = loop {
        let action_result = submit_action()?;

        let retry = extract_seq_no_from_reply(&action_result)
            .map(|received_timestamp| received_timestamp < minimal_timestamp)
            .unwrap_or(true);

        if retry && i < SUBMIT_RETRY_CNT {
            ::std::thread::sleep(timeout::short_timeout());
            i += 1;
        } else {
            break action_result;
        }
    };
    Ok(action_result)
}

pub fn calculate_hash(text: &str, version: &str) -> String {
    let content: String = version.to_string() + text;
    let digest = hash(content.as_bytes()).unwrap();
    hex::encode(digest)
}

pub fn build_get_ddo_request(submitter_did: Option<&str>, target_did: &str) -> Result<String, IndyError> {
    ledger::build_get_ddo_request(submitter_did, target_did).wait()
}

pub fn build_nym_request(submitter_did: &str, target_did: &str, verkey: Option<&str>,
                         alias: Option<&str>, role: Option<&str>) -> Result<String, IndyError> {
    ledger::build_nym_request(submitter_did, target_did, verkey, alias, role).wait()
}

pub fn parse_get_nym_response(get_nym_response: &str) -> Result<String, IndyError> {
    ledger::parse_get_nym_response(get_nym_response).wait()
}

pub fn build_attrib_request(submitter_did: &str, target_did: &str, hash: Option<&str>, raw: Option<&str>, enc: Option<&str>) -> Result<String, IndyError> {
    ledger::build_attrib_request(submitter_did, target_did, hash, raw, enc).wait()
}

pub fn build_get_attrib_request(submitter_did: Option<&str>, target_did: &str, raw: Option<&str>, hash: Option<&str>, enc: Option<&str>) -> Result<String, IndyError> {
    ledger::build_get_attrib_request(submitter_did, target_did, raw, hash, enc).wait()
}

pub fn build_get_nym_request(submitter_did: Option<&str>, target_did: &str) -> Result<String, IndyError> {
    ledger::build_get_nym_request(submitter_did, target_did).wait()
}

pub fn build_schema_request(submitter_did: &str, data: &str) -> Result<String, IndyError> {
    ledger::build_schema_request(submitter_did, data).wait()
}

pub fn build_get_schema_request(submitter_did: Option<&str>, id: &str) -> Result<String, IndyError> {
    ledger::build_get_schema_request(submitter_did, id).wait()
}

pub fn build_cred_def_txn(submitter_did: &str, cred_def_json: &str) -> Result<String, IndyError> {
    ledger::build_cred_def_request(submitter_did, cred_def_json).wait()
}

pub fn build_get_cred_def_request(submitter_did: Option<&str>, id: &str) -> Result<String, IndyError> {
    ledger::build_get_cred_def_request(submitter_did, id).wait()
}

pub fn build_node_request(submitter_did: &str, target_did: &str, data: &str) -> Result<String, IndyError> {
    ledger::build_node_request(submitter_did, target_did, data).wait()
}

pub fn build_get_validator_info_request(submitter_did: &str) -> Result<String, IndyError> {
    ledger::build_get_validator_info_request(submitter_did).wait()
}

pub fn build_get_txn_request(submitter_did: Option<&str>, data: i32, ledger_type: Option<&str>) -> Result<String, IndyError> {
    ledger::build_get_txn_request(submitter_did, ledger_type, data).wait()
}

pub fn build_pool_config_request(submitter_did: &str, writes: bool, force: bool) -> Result<String, IndyError> {
    ledger::build_pool_config_request(submitter_did, writes, force).wait()
}

pub fn build_pool_restart_request(submitter_did: &str,
                                  action: &str,
                                  datetime: Option<&str>) -> Result<String, IndyError> {
    ledger::build_pool_restart_request(submitter_did, action, datetime).wait()
}

pub fn build_pool_upgrade_request(submitter_did: &str, name: &str, version: &str, action: &str, sha256: &str, timeout: Option<u32>, schedule: Option<&str>,
                                  justification: Option<&str>, reinstall: bool, force: bool, package: Option<&str>) -> Result<String, IndyError> {
    ledger::build_pool_upgrade_request(submitter_did, name, version, action, sha256,
                                       timeout, schedule, justification, reinstall, force, package).wait()
}

pub fn build_revoc_reg_def_request(submitter_did: &str, data: &str) -> Result<String, IndyError> {
    ledger::build_revoc_reg_def_request(submitter_did, data).wait()
}

pub fn build_revoc_reg_entry_request(submitter_did: &str, rev_reg_def_id: &str, rev_reg_type: &str, value: &str) -> Result<String, IndyError> {
    ledger::build_revoc_reg_entry_request(submitter_did, rev_reg_def_id, rev_reg_type, value).wait()
}

pub fn build_get_revoc_reg_def_request(submitter_did: Option<&str>, id: &str) -> Result<String, IndyError> {
    ledger::build_get_revoc_reg_def_request(submitter_did, id).wait()
}

pub fn build_get_revoc_reg_request(submitter_did: Option<&str>, rev_reg_def_id: &str, timestamp: u64) -> Result<String, IndyError> {
    ledger::build_get_revoc_reg_request(submitter_did, rev_reg_def_id, timestamp as i64).wait()
}

pub fn build_get_revoc_reg_delta_request(submitter_did: Option<&str>, rev_reg_def_id: &str, from: Option<u64>, to: u64) -> Result<String, IndyError> {
    ledger::build_get_revoc_reg_delta_request(submitter_did, rev_reg_def_id, from.map(|f| f as i64).unwrap_or(-1), to as i64).wait()
}

pub fn parse_get_schema_response(get_schema_response: &str) -> Result<(String, String), IndyError> {
    ledger::parse_get_schema_response(get_schema_response).wait()
}

pub fn parse_get_cred_def_response(get_cred_def_response: &str) -> Result<(String, String), IndyError> {
    ledger::parse_get_cred_def_response(get_cred_def_response).wait()
}

pub fn parse_get_revoc_reg_def_response(get_revoc_reg_def_response: &str) -> Result<(String, String), IndyError> {
    ledger::parse_get_revoc_reg_def_response(get_revoc_reg_def_response).wait()
}

pub fn parse_get_revoc_reg_response(get_revoc_reg_response: &str) -> Result<(String, String, u64), IndyError> {
    ledger::parse_get_revoc_reg_response(get_revoc_reg_response).wait()
}

pub fn parse_get_revoc_reg_delta_response(get_revoc_reg_delta_response: &str) -> Result<(String, String, u64), IndyError> {
    ledger::parse_get_revoc_reg_delta_response(get_revoc_reg_delta_response).wait()
}

pub fn register_transaction_parser_for_sp(txn_type: &str, parse: CustomTransactionParser, free: CustomFree) -> Result<(), ErrorCode> {
    let (receiver, command_handle, cb) = callback::_closure_to_cb_ec();

    let txn_type = CString::new(txn_type).unwrap();

    let err =
        unsafe {
            indy_register_transaction_parser_for_sp(command_handle,
                                                    txn_type.as_ptr(),
                                                    Some(parse),
                                                    Some(free),
                                                    cb)
        };

    super::results::result_to_empty(err, receiver)
}

pub fn get_response_metadata(response: &str) -> Result<String, IndyError> {
    ledger::get_response_metadata(response).wait()
}

pub fn build_auth_rule_request(submitter_did: &str,
                               txn_type: &str,
                               action: &str,
                               field: &str,
                               old_value: Option<&str>,
                               new_value: Option<&str>,
                               constraint: &str, ) -> Result<String, IndyError> {
    ledger::build_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value, constraint).wait()
}

pub fn build_auth_rules_request(submitter_did: &str,
                                data: &str, ) -> Result<String, IndyError> {
    ledger::build_auth_rules_request(submitter_did, data).wait()
}

pub fn build_get_auth_rule_request(submitter_did: Option<&str>,
                                   auth_type: Option<&str>,
                                   auth_action: Option<&str>,
                                   field: Option<&str>,
                                   old_value: Option<&str>,
                                   new_value: Option<&str>, ) -> Result<String, IndyError> {
    ledger::build_get_auth_rule_request(submitter_did, auth_type, auth_action, field, old_value, new_value).wait()
}

pub fn build_txn_author_agreement_request(submitter_did: &str,
                                          text: Option<&str>,
                                          version: &str,
                                          ratification_ts: Option<u64>,
                                          retirement_ts: Option<u64>) -> Result<String, IndyError> {
    ledger::build_txn_author_agreement_request(submitter_did, text, version, ratification_ts, retirement_ts).wait()
}

pub fn build_disable_all_txn_author_agreements_request(submitter_did: &str) -> Result<String, IndyError> {
    ledger::build_disable_all_txn_author_agreements_request(submitter_did).wait()
}

pub fn build_get_txn_author_agreement_request(submitter_did: Option<&str>,
                                              data: Option<&str>, ) -> Result<String, IndyError> {
    ledger::build_get_txn_author_agreement_request(submitter_did, data).wait()
}

pub fn build_acceptance_mechanisms_request(submitter_did: &str,
                                           aml: &str,
                                           version: &str,
                                           aml_context: Option<&str>) -> Result<String, IndyError> {
    ledger::build_acceptance_mechanisms_request(submitter_did, aml, version, aml_context).wait()
}

pub fn build_get_acceptance_mechanisms_request(submitter_did: Option<&str>,
                                               timestamp: Option<i64>,
                                               version: Option<&str>) -> Result<String, IndyError> {
    ledger::build_get_acceptance_mechanisms_request(submitter_did, timestamp, version).wait()
}

pub fn append_txn_author_agreement_acceptance_to_request(request_json: &str,
                                                         text: Option<&str>,
                                                         version: Option<&str>,
                                                         taa_digest: Option<&str>,
                                                         acc_mech_type: &str,
                                                         time_of_acceptance: u64) -> Result<String, IndyError> {
    ledger::append_txn_author_agreement_acceptance_to_request(request_json, text, version, taa_digest, acc_mech_type, time_of_acceptance).wait()
}

pub fn append_request_endorser(request_json: &str,
                               endorser_did: &str) -> Result<String, IndyError> {
    ledger::append_request_endorser(request_json, endorser_did).wait()
}

pub fn post_entities() -> (&'static str, &'static str, &'static str) {
    lazy_static! {
                    static ref COMMON_ENTITIES_INIT: Once = Once::new();

                }

    unsafe {
        COMMON_ENTITIES_INIT.call_once(|| {
            let pool_and_wallet_name = "COMMON_ENTITIES_POOL";
            super::test::cleanup_storage(pool_and_wallet_name);

            let pool_handle = pool::create_and_open_pool_ledger(pool_and_wallet_name).unwrap();

            let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(pool_and_wallet_name).unwrap();

            let (issuer_did, _) = did::create_store_and_publish_my_did_from_trustee(wallet_handle, pool_handle).unwrap();

            let (schema_id, schema_json) = anoncreds::issuer_create_schema(&issuer_did,
                                                                           GVT_SCHEMA_NAME,
                                                                           SCHEMA_VERSION,
                                                                           GVT_SCHEMA_ATTRIBUTES).unwrap();

            let schema_request = build_schema_request(&issuer_did, &schema_json).unwrap();
            let schema_response = sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &schema_request).unwrap();
            pool::check_response_type(&schema_response, crate::utils::types::ResponseType::REPLY);

            let get_schema_request = build_get_schema_request(Some(&issuer_did), &schema_id).unwrap();
            let get_schema_response = submit_request_with_retries(pool_handle, &get_schema_request, &schema_response).unwrap();
            let (schema_id, schema_json) = parse_get_schema_response(&get_schema_response).unwrap();

            let (cred_def_id, cred_def_json) = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                                              &issuer_did,
                                                                                              &schema_json,
                                                                                              TAG_1,
                                                                                              None,
                                                                                              Some(&anoncreds::revocation_cred_def_config())).unwrap();
            let cred_def_request = build_cred_def_txn(&issuer_did, &cred_def_json).unwrap();
            let cred_def_response = sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &cred_def_request).unwrap();
            pool::check_response_type(&cred_def_response, crate::utils::types::ResponseType::REPLY);

            let tails_writer_config = anoncreds::tails_writer_config();
            let tails_writer_handle = blob_storage::open_writer("default", &tails_writer_config).unwrap();

            let (rev_reg_id, revoc_reg_def_json, rev_reg_entry_json) =
                anoncreds::issuer_create_and_store_revoc_reg(wallet_handle,
                                                             &issuer_did,
                                                             None,
                                                             TAG_1,
                                                             &cred_def_id,
                                                             &anoncreds::issuance_on_demand_rev_reg_config(),
                                                             tails_writer_handle).unwrap();

            let rev_reg_def_request = build_revoc_reg_def_request(&issuer_did, &revoc_reg_def_json).unwrap();
            let rev_reg_def_response = sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &rev_reg_def_request).unwrap();
            pool::check_response_type(&rev_reg_def_response, crate::utils::types::ResponseType::REPLY);

            let rev_reg_entry_request = build_revoc_reg_entry_request(&issuer_did, &rev_reg_id, REVOC_REG_TYPE, &rev_reg_entry_json).unwrap();
            sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &rev_reg_entry_request).unwrap();

            let res = mem::transmute(&schema_id as &str);
            mem::forget(schema_id);
            SCHEMA_ID = res;

            let res = mem::transmute(&cred_def_id as &str);
            mem::forget(cred_def_id);
            CRED_DEF_ID = res;

            let res = mem::transmute(&rev_reg_id as &str);
            mem::forget(rev_reg_id);
            REV_REG_DEF_ID = res;

            pool::close(pool_handle).unwrap();
            pool::delete(pool_and_wallet_name).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
            wallet::delete_wallet(&wallet_config, WALLET_CREDENTIALS).unwrap();
        });

        (SCHEMA_ID, CRED_DEF_ID, REV_REG_DEF_ID)
    }
}

pub fn post_qualified_entities() -> (&'static str, &'static str) {
    lazy_static! {
                    static ref COMMON_ENTITIES_INIT: Once = Once::new();

                }

    unsafe {
        COMMON_ENTITIES_INIT.call_once(|| {
            let pool_and_wallet_name = "COMMON_ENTITIES_POOL";
            super::test::cleanup_storage(pool_and_wallet_name);

            let pool_handle = pool::create_and_open_pool_ledger(pool_and_wallet_name).unwrap();

            let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(pool_and_wallet_name).unwrap();

            let (issuer_did, _) = did::create_store_and_publish_my_did_from_trustee_v1(wallet_handle, pool_handle).unwrap();

            let (schema_id, schema_json) = anoncreds::issuer_create_schema(&issuer_did,
                                                                           GVT_SCHEMA_NAME,
                                                                           SCHEMA_VERSION,
                                                                           GVT_SCHEMA_ATTRIBUTES).unwrap();

            let schema_request = build_schema_request(&issuer_did, &schema_json).unwrap();
            let schema_response = sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &schema_request).unwrap();
            pool::check_response_type(&schema_response, crate::utils::types::ResponseType::REPLY);

            let get_schema_request = build_get_schema_request(Some(&issuer_did), &schema_id).unwrap();
            let get_schema_response = submit_request_with_retries(pool_handle, &get_schema_request, &schema_response).unwrap();
            let (schema_id, schema_json) = parse_get_schema_response(&get_schema_response).unwrap();

            let (cred_def_id, cred_def_json) = anoncreds::issuer_create_credential_definition(wallet_handle,
                                                                                              &issuer_did,
                                                                                              &schema_json,
                                                                                              TAG_1,
                                                                                              None,
                                                                                              Some(&anoncreds::revocation_cred_def_config())).unwrap();
            let cred_def_request = build_cred_def_txn(&issuer_did, &cred_def_json).unwrap();
            let cred_def_response = sign_and_submit_request(pool_handle, wallet_handle, &issuer_did, &cred_def_request).unwrap();
            pool::check_response_type(&cred_def_response, crate::utils::types::ResponseType::REPLY);

            let res = mem::transmute(&schema_id as &str);
            mem::forget(schema_id);
            SCHEMA_ID_V2 = res;

            let res = mem::transmute(&cred_def_id as &str);
            mem::forget(cred_def_id);
            CRED_DEF_ID_V2 = res;

            pool::close(pool_handle).unwrap();
            pool::delete(pool_and_wallet_name).unwrap();
            wallet::close_wallet(wallet_handle).unwrap();
            wallet::delete_wallet(&wallet_config, WALLET_CREDENTIALS).unwrap();
        });

        (SCHEMA_ID_V2, CRED_DEF_ID_V2)
    }
}