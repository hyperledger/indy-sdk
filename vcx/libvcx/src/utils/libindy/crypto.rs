/* test isn't ready until > libindy 1.0.1 */
use futures::Future;
use indy::crypto;

use utils::libindy::LibindyMock;
use settings;
use error::prelude::*;

pub fn prep_msg(sender_vk: &str, recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        let rc = LibindyMock::get_result();
        if rc != 0 { return Err(VcxError::from(VcxErrorKind::Common(rc))); };
        return Ok(Vec::from(msg).to_owned());
    }

    crypto::auth_crypt(::utils::libindy::wallet::get_wallet_handle(), sender_vk, recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn prep_anonymous_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::anon_crypt(recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn parse_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<(String, Vec<u8>)> {
    if settings::indy_mocks_enabled() { return Ok((::utils::constants::VERKEY.to_string(), Vec::from(msg).to_owned())); }

    crypto::auth_decrypt(::utils::libindy::wallet::get_wallet_handle(), recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn parse_anonymous_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::anon_decrypt(::utils::libindy::wallet::get_wallet_handle(), recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn sign(my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::sign(::utils::libindy::wallet::get_wallet_handle(), my_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() { return Ok(true); }

    crypto::verify(vk, msg, signature)
        .wait()
        .map_err(VcxError::from)
}

pub fn pack_message(sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(msg.to_vec()); }

    crypto::pack_message(::utils::libindy::wallet::get_wallet_handle(), msg, receiver_keys, sender_vk)
        .wait()
        .map_err(VcxError::from)
}

pub fn unpack_message(msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::unpack_message(::utils::libindy::wallet::get_wallet_handle(), msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn create_key(seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({"seed": seed}).to_string();

    crypto::create_key(::utils::libindy::wallet::get_wallet_handle(), Some(&key_json))
        .wait()
        .map_err(VcxError::from)
}
