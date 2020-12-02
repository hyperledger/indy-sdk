extern crate futures;

use indy::did;
use indy::IndyError;
use indy::future::Future;

use crate::utils::{ledger, pool};
use crate::utils::types::ResponseType;
use crate::utils::constants::DEFAULT_METHOD_NAME;
use indy::{WalletHandle, PoolHandle};

pub fn create_store_and_publish_did(wallet_handle: WalletHandle, pool_handle: PoolHandle, role: &str, method_name: Option<&str>) -> Result<(String, String), IndyError> {
    let my_did_json = json!({"method_name": method_name, "seed": crate::utils::constants::TRUSTEE_SEED}).to_string();
    let (trustee_did, _) = create_my_did(wallet_handle, &my_did_json)?;
    let my_did_json = json!({"method_name": method_name}).to_string();
    let (did, vk) = create_my_did(wallet_handle, &my_did_json)?;
    let nym = ledger::build_nym_request(&trustee_did, &did, Some(&vk), None, Some(role))?;
    let response = ledger::sign_and_submit_request(pool_handle, wallet_handle, &trustee_did, &nym)?;
    pool::check_response_type(&response, ResponseType::REPLY);
    Ok((did, vk))
}

pub fn create_store_and_publish_my_did_from_trustee(wallet_handle: WalletHandle, pool_handle: PoolHandle) -> Result<(String, String), IndyError> {
    create_store_and_publish_did(wallet_handle, pool_handle, "TRUSTEE", None)
}

pub fn create_store_and_publish_my_did_from_trustee_v1(wallet_handle: WalletHandle, pool_handle: PoolHandle) -> Result<(String, String), IndyError> {
    create_store_and_publish_did(wallet_handle, pool_handle, "TRUSTEE", Some("sov"))
}

pub fn create_store_and_publish_my_did_from_steward(wallet_handle: WalletHandle, pool_handle: PoolHandle) -> Result<(String, String), IndyError> {
    create_store_and_publish_did(wallet_handle, pool_handle, "STEWARD", None)
}

pub fn create_and_store_my_did(wallet_handle: WalletHandle, seed: Option<&str>) -> Result<(String, String), IndyError> {
    let my_did_json = json!({"seed": seed}).to_string();
    did::create_and_store_my_did(wallet_handle, &my_did_json).wait()
}

pub fn create_and_store_my_did_v1(wallet_handle: WalletHandle, seed: Option<&str>) -> Result<(String, String), IndyError> {
    let my_did_json = json!({"seed": seed, "method_name": DEFAULT_METHOD_NAME}).to_string();
    did::create_and_store_my_did(wallet_handle, &my_did_json).wait()
}

pub fn create_my_did(wallet_handle: WalletHandle, my_did_json: &str) -> Result<(String, String), IndyError> {
    did::create_and_store_my_did(wallet_handle, my_did_json).wait()
}

pub fn store_their_did(wallet_handle: WalletHandle, identity_json: &str) -> Result<(), IndyError> {
    did::store_their_did(wallet_handle, identity_json).wait()
}

pub fn store_their_did_from_parts(wallet_handle: WalletHandle, their_did: &str, their_verkey: &str) -> Result<(), IndyError> {
    let their_identity_json = json!({"did": their_did, "verkey": their_verkey}).to_string();
    did::store_their_did(wallet_handle, &their_identity_json).wait()
}

pub fn replace_keys_start(wallet_handle: WalletHandle, did: &str, identity_json: &str) -> Result<String, IndyError> {
    did::replace_keys_start(wallet_handle, did, identity_json).wait()
}

pub fn replace_keys_apply(wallet_handle: WalletHandle, did: &str) -> Result<(), IndyError> {
    did::replace_keys_apply(wallet_handle, did).wait()
}

pub fn replace_keys(pool_handle: PoolHandle, wallet_handle: WalletHandle, did: &str) -> Result<String, IndyError> {
    let verkey = did::replace_keys_start(wallet_handle, did, "{}").wait().unwrap();

    let nym_request = ledger::build_nym_request(did, did, Some(&verkey), None, None).unwrap();
    ledger::sign_and_submit_request(pool_handle, wallet_handle, did, &nym_request).unwrap();

    replace_keys_apply(wallet_handle, did).unwrap();

    Ok(verkey)
}

pub fn key_for_did(pool_handle: PoolHandle, wallet_handle: WalletHandle, did: &str) -> Result<String, IndyError> {
    did::key_for_did(pool_handle, wallet_handle, did).wait()
}

pub fn key_for_local_did(wallet_handle: WalletHandle, did: &str) -> Result<String, IndyError> {
    did::key_for_local_did(wallet_handle, did).wait()
}

pub fn set_endpoint_for_did(wallet_handle: WalletHandle, did: &str, address: &str, transport_key: &str) -> Result<(), IndyError> {
    did::set_endpoint_for_did(wallet_handle, did, address, transport_key).wait()
}

pub fn get_endpoint_for_did(wallet_handle: WalletHandle, pool_handle: PoolHandle, did: &str) -> Result<(String, Option<String>), IndyError> {
    did::get_endpoint_for_did(wallet_handle, pool_handle, did).wait()
}

pub fn set_did_metadata(wallet_handle: WalletHandle, did: &str, metadata: &str) -> Result<(), IndyError> {
    did::set_did_metadata(wallet_handle, did, metadata).wait()
}

pub fn get_did_metadata(wallet_handle: WalletHandle, did: &str) -> Result<String, IndyError> {
    did::get_did_metadata(wallet_handle, did).wait()
}

pub fn get_my_did_with_metadata(wallet_handle: WalletHandle, did: &str) -> Result<String, IndyError> {
    did::get_my_did_with_metadata(wallet_handle, did).wait()
}

pub fn list_my_dids_with_meta(wallet_handle: WalletHandle) -> Result<String, IndyError> {
    did::list_my_dids_with_metadata(wallet_handle).wait()
}

pub fn abbreviate_verkey(did: &str, verkey: &str) -> Result<String, IndyError> {
    did::abbreviate_verkey(did, verkey).wait()
}

pub fn qualify_did(wallet_handle: WalletHandle, did: &str, prefix: &str) -> Result<String, IndyError> {
    did::qualify_did(wallet_handle, did, prefix).wait()
}
