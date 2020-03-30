extern crate futures;

use indy::IndyError;
use indy::pairwise;
use crate::indy::future::Future;

use indy::WalletHandle;

pub fn pairwise_exists(wallet_handle: WalletHandle, their_did: &str) -> Result<bool, IndyError> {
    pairwise::is_pairwise_exists(wallet_handle, their_did).wait()
}

pub fn create_pairwise(wallet_handle: WalletHandle, their_did: &str, my_did: &str, metadata: Option<&str>) -> Result<(), IndyError> {
    pairwise::create_pairwise(wallet_handle, their_did, my_did, metadata).wait()
}

pub fn list_pairwise(wallet_handle: WalletHandle) -> Result<String, IndyError> {
    pairwise::list_pairwise(wallet_handle).wait()
}

pub fn get_pairwise(wallet_handle: WalletHandle, their_did: &str) -> Result<String, IndyError> {
    pairwise::get_pairwise(wallet_handle, their_did).wait()
}

pub fn set_pairwise_metadata(wallet_handle: WalletHandle, their_did: &str, metadata: Option<&str>) -> Result<(), IndyError> {
    pairwise::set_pairwise_metadata(wallet_handle, their_did, metadata).wait()
}