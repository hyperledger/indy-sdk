extern crate futures;

use indy::cache;
use indy::future::Future;
use indy::IndyError;
use indy::{WalletHandle, PoolHandle};


pub fn get_schema_cache(pool_handle: PoolHandle, wallet_handle: WalletHandle, submitter_did: &str, id: &str, options_json: &str) -> Result<String, IndyError> {
    cache::get_schema(pool_handle, wallet_handle, submitter_did, id, options_json).wait()
}

pub fn get_cred_def_cache(pool_handle: PoolHandle, wallet_handle: WalletHandle, submitter_did: &str, id: &str, options_json: &str) -> Result<String, IndyError> {
    cache::get_cred_def(pool_handle, wallet_handle, submitter_did, id, options_json).wait()
}

pub fn purge_schema_cache(wallet_handle: WalletHandle, options_json: &str) -> Result<(), IndyError> {
    cache::purge_schema_cache(wallet_handle, options_json).wait()
}

pub fn purge_cred_def_cache(wallet_handle: WalletHandle, options_json: &str) -> Result<(), IndyError> {
    cache::purge_cred_def_cache(wallet_handle, options_json).wait()
}