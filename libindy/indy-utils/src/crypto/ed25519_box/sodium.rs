extern crate sodiumoxide;

use indy_api_types::errors::prelude::*;
use self::sodiumoxide::crypto::box_;


pub const NONCEBYTES: usize = box_::curve25519xsalsa20poly1305::NONCEBYTES;
pub const PUBLICKEYBYTES: usize = box_::curve25519xsalsa20poly1305::PUBLICKEYBYTES;
pub const SECRETKEYBYTES: usize = box_::curve25519xsalsa20poly1305::SECRETKEYBYTES;

sodium_type!(Nonce, box_::Nonce, NONCEBYTES);
sodium_type!(PublicKey, box_::PublicKey, PUBLICKEYBYTES);
sodium_type!(SecretKey, box_::SecretKey, SECRETKEYBYTES);

pub fn encrypt(secret_key: &SecretKey, public_key: &PublicKey, doc: &[u8], nonce: &Nonce) -> Result<Vec<u8>, IndyError> {
    Ok(box_::seal(
        doc,
        &nonce.0,
        &public_key.0,
        &secret_key.0,
    ))
}

pub fn decrypt(secret_key: &SecretKey, public_key: &PublicKey, doc: &[u8], nonce: &Nonce) -> Result<Vec<u8>, IndyError> {
    box_::open(
        doc,
        &nonce.0,
        &public_key.0,
        &secret_key.0,
    )
        .map_err(|_| IndyError::from_msg(IndyErrorKind::InvalidStructure, "Unable to open sodium _box"))
}

pub fn gen_nonce() -> Nonce {
    Nonce(box_::gen_nonce())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519_sign;
    use rand::Rng;

    #[test]
    fn encrypt_decrypt_works() {
        let text = rand::thread_rng().gen::<[u8; 16]>();
        let nonce = gen_nonce();
        let seed_bytes = rand::thread_rng().gen::<[u8; 32]>();
        let seed = ed25519_sign::Seed::from_slice(&seed_bytes).unwrap();

        let (alice_ver_key, alice_sign_key) = ed25519_sign::create_key_pair_for_signature(Some(&seed)).unwrap();
        let alice_pk = ed25519_sign::vk_to_curve25519(&alice_ver_key).unwrap();
        let alice_sk = ed25519_sign::sk_to_curve25519(&alice_sign_key).unwrap();

        let (bob_ver_key, bob_sign_key) = ed25519_sign::create_key_pair_for_signature(Some(&seed)).unwrap();
        let bob_pk = ed25519_sign::vk_to_curve25519(&bob_ver_key).unwrap();
        let bob_sk = ed25519_sign::sk_to_curve25519(&bob_sign_key).unwrap();

        let bob_encrypted_text = encrypt(&bob_sk, &alice_pk, &text, &nonce).unwrap();
        let bob_decrypt_result = decrypt(&alice_sk, &bob_pk, &bob_encrypted_text, &nonce);
        assert!(bob_decrypt_result.is_ok());
        assert_eq!(text, bob_decrypt_result.unwrap());

        let alice_encrypted_text = encrypt(&alice_sk, &bob_pk, &text, &nonce).unwrap();
        let alice_decrypted_text = decrypt(&bob_sk, &alice_pk, &alice_encrypted_text, &nonce);
        assert!(alice_decrypted_text.is_ok());
        assert_eq!(text, alice_decrypted_text.unwrap());
    }
}
