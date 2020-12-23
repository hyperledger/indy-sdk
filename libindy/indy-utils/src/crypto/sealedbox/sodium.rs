extern crate sodiumoxide;

use indy_api_types::errors::prelude::*;
use self::sodiumoxide::crypto::sealedbox;
use super::ed25519_box;

pub fn encrypt(pk: &ed25519_box::PublicKey, doc: &[u8]) -> Result<Vec<u8>, IndyError> {
    Ok(sealedbox::seal(doc, &pk.0))
}

pub fn decrypt(pk: &ed25519_box::PublicKey, sk: &ed25519_box::SecretKey, doc: &[u8]) -> Result<Vec<u8>, IndyError> {
    sealedbox::open(&doc,
                    &pk.0,
                    &sk.0)
        .map_err(|_| IndyError::from_msg(IndyErrorKind::InvalidStructure, "Unable to open sodium sealedbox"))
}

#[cfg(test)]
mod tests {
    use self::sodiumoxide::crypto::box_;
    use super::*;
    use crate::crypto::ed25519_box::{PublicKey, SecretKey};
    use rand::Rng;

    #[test]
    fn encrypt_decrypt_works() {
        let (pk, sk) = box_::gen_keypair();
        let (pk, sk) = (PublicKey(pk), SecretKey(sk));
        let doc = rand::thread_rng().gen::<[u8; 16]>();

        let encrypted_data = encrypt(&pk, &doc).unwrap();
        let decrypt_result = decrypt(&pk, &sk, &encrypted_data).unwrap();

        assert_eq!(doc, decrypt_result);
    }
}