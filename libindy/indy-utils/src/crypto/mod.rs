use std::str;

use indy_api_types::domain::crypto::combo_box::ComboBox;
use indy_api_types::domain::crypto::did::DidValue;
use indy_api_types::domain::crypto::key::{Key, KeyInfo};

use hex::FromHex;
use rust_base58::{FromBase58, ToBase58};

#[macro_use]
pub mod sodium_type;

#[cfg(feature = "base64_rust_base64")]
#[path = "base64/rust_base64.rs"]
pub mod base64;

#[cfg(feature = "chacha20poly1305_ietf_sodium")]
#[path = "chacha20poly1305_ietf/sodium.rs"]
pub mod chacha20poly1305_ietf;

#[cfg(feature = "hash_openssl")]
#[path = "hash/openssl.rs"]
pub mod hash;

#[cfg(feature = "hmacsha256_sodium")]
#[path = "hmacsha256/sodium.rs"]
pub mod hmacsha256;

#[cfg(feature = "pwhash_argon2i13_sodium")]
#[path = "pwhash_argon2i13/sodium.rs"]
pub mod pwhash_argon2i13;

#[cfg(feature = "randombytes_sodium")]
#[path = "randombytes/sodium.rs"]
pub mod randombytes;

#[cfg(feature = "sealedbox_sodium")]
#[path = "sealedbox/sodium.rs"]
pub mod sealedbox;

#[allow(dead_code)] /* FIXME Do we really need this module? */
#[cfg(feature = "xsalsa20_sodium")]
#[path = "xsalsa20/sodium.rs"]
pub mod xsalsa20;

#[cfg(feature = "ed25519_sign_sodium")]
#[path = "ed25519_sign/sodium.rs"]
pub mod ed25519_sign;

pub mod pack;

#[cfg(feature = "ed25519_box_sodium")]
#[path = "ed25519_box/sodium.rs"]
// TODO: The name is misleading as the operations do not happen over ed25519 curve
pub mod ed25519_box;

pub mod verkey_builder;

use indy_api_types::errors::prelude::*;
use verkey_builder::{DEFAULT_CRYPTO_TYPE, clear_verkey};

pub fn create_key(key_info: &KeyInfo) -> IndyResult<Key> {
    trace!("create_key >>> key_info: {:?}", secret!(key_info));

    match key_info.crypto_type.as_ref().map(String::as_str) {
        Some(DEFAULT_CRYPTO_TYPE) | None => {}
        crypto_type => {
            return Err(IndyError::from_msg(IndyErrorKind::UnknownCrypto, format!("Unknown crypto type: {:?}", crypto_type)));
        }
    }

    let seed = convert_seed(key_info.seed.as_ref().map(String::as_ref))?;
    let (vk, sk) = ed25519_sign::create_key_pair_for_signature(seed.as_ref())?;
    let vk = vk[..].to_base58();
    let sk = sk[..].to_base58();

    let key = Key::new(vk, sk);

    trace!("create_key <<< key: {:?}", key);

    Ok(key)
}

pub fn sign(my_key: &Key, doc: &[u8]) -> IndyResult<Vec<u8>> {
    trace!("sign >>> my_key: {:?}, doc: {:?}", my_key, doc);

    let my_sk = ed25519_sign::SecretKey::from_slice(&my_key.signkey.as_str().from_base58()?.as_slice())?;
    let signature = ed25519_sign::sign(&my_sk, doc)?[..].to_vec();

    trace!("sign <<< signature: {:?}", signature);

    Ok(signature)
}

pub fn verify(their_vk: &str, msg: &[u8], signature: &[u8]) -> IndyResult<bool> {
    trace!("verify >>> their_vk: {:?}, msg: {:?}, signature: {:?}", their_vk, msg, signature);

    let their_vk = clear_verkey(their_vk)?;

    let their_vk = ed25519_sign::PublicKey::from_slice(&their_vk.from_base58()?)?;
    let signature = ed25519_sign::Signature::from_slice(&signature)?;

    let valid = ed25519_sign::verify(&their_vk, msg, &signature)?;

    trace!("verify <<< valid: {:?}", valid);

    Ok(valid)
}

pub fn create_combo_box(my_key: &Key, their_vk: &str, doc: &[u8]) -> IndyResult<ComboBox> {
    trace!("create_combo_box >>> my_key: {:?}, their_vk: {:?}, doc: {:?}", my_key, their_vk, doc);

    let (msg, nonce) = crypto_box(my_key, their_vk, doc)?;

    let res = ComboBox {
        msg: base64::encode(msg.as_slice()),
        sender: my_key.verkey.to_string(),
        nonce: base64::encode(nonce.as_slice()),
    };

    trace!("create_combo_box <<< res: {:?}", res);

    Ok(res)
}

pub fn crypto_box(my_key: &Key, their_vk: &str, doc: &[u8]) -> IndyResult<(Vec<u8>, Vec<u8>)> {
    trace!("crypto_box >>> my_key: {:?}, their_vk: {:?}, doc: {:?}", my_key, their_vk, doc);

    let their_vk = clear_verkey(their_vk)?;

    let my_sk = ed25519_sign::SecretKey::from_slice(my_key.signkey.as_str().from_base58()?.as_slice())?;
    let their_vk = ed25519_sign::PublicKey::from_slice(their_vk.from_base58()?.as_slice())?;
    let nonce = ed25519_box::gen_nonce();

    let encrypted_doc = ed25519_box::encrypt(&my_sk.to_curve25519()?, &their_vk.to_curve25519()?, doc, &nonce)?;
    let nonce = nonce[..].to_vec();

    trace!("crypto_box <<< encrypted_doc: {:?}, nonce: {:?}", encrypted_doc, nonce);

    Ok((encrypted_doc, nonce))
}

pub fn crypto_box_open(my_key: &Key, their_vk: &str, doc: &[u8], nonce: &[u8]) -> IndyResult<Vec<u8>> {
    trace!("crypto_box_open >>> my_key: {:?}, their_vk: {:?}, doc: {:?}, nonce: {:?}", my_key, their_vk, doc, nonce);

    let their_vk = clear_verkey(their_vk)?;

    let my_sk = ed25519_sign::SecretKey::from_slice(&my_key.signkey.from_base58()?.as_slice())?;
    let their_vk = ed25519_sign::PublicKey::from_slice(their_vk.from_base58()?.as_slice())?;
    let nonce = ed25519_box::Nonce::from_slice(&nonce)?;

    let decrypted_doc = ed25519_box::decrypt(&my_sk.to_curve25519()?, &their_vk.to_curve25519()?, &doc, &nonce)?;

    trace!("crypto_box_open <<< decrypted_doc: {:?}", decrypted_doc);

    Ok(decrypted_doc)
}

pub fn crypto_box_seal(their_vk: &str, doc: &[u8]) -> IndyResult<Vec<u8>> {
    trace!("crypto_box_seal >>> their_vk: {:?}, doc: {:?}", their_vk, doc);

    let their_vk = clear_verkey(their_vk)?;

    let their_vk = ed25519_sign::PublicKey::from_slice(their_vk.from_base58()?.as_slice())?;

    let encrypted_doc = sealedbox::encrypt(&their_vk.to_curve25519()?, doc)?;

    trace!("crypto_box_seal <<< encrypted_doc: {:?}", encrypted_doc);

    Ok(encrypted_doc)
}

pub fn crypto_box_seal_open(my_key: &Key, doc: &[u8]) -> IndyResult<Vec<u8>> {
    trace!("crypto_box_seal_open >>> my_key: {:?}, doc: {:?}", my_key, doc);

    let my_vk = clear_verkey(&my_key.verkey)?;

    let my_vk = ed25519_sign::PublicKey::from_slice(my_vk.from_base58()?.as_slice())?;
    let my_sk = ed25519_sign::SecretKey::from_slice(my_key.signkey.as_str().from_base58()?.as_slice())?;

    let decrypted_doc = sealedbox::decrypt(&my_vk.to_curve25519()?, &my_sk.to_curve25519()?, doc)?;

    trace!("crypto_box_seal_open <<< decrypted_doc: {:?}", decrypted_doc);

    Ok(decrypted_doc)
}

pub fn convert_seed(seed: Option<&str>) -> IndyResult<Option<ed25519_sign::Seed>> {
    trace!("convert_seed >>> seed: {:?}", secret!(seed));

    if seed.is_none() {
        trace!("convert_seed <<< res: None");
        return Ok(None);
    }

    let seed = seed.unwrap();

    let bytes = if seed.as_bytes().len() == ed25519_sign::SEEDBYTES {
        // is acceptable seed length
        seed.as_bytes().to_vec()
    } else if seed.ends_with('=') {
        // is base64 string
        let decoded = base64::decode(&seed)
            .to_indy(IndyErrorKind::InvalidStructure, "Can't deserialize Seed from Base64 string")?;
        if decoded.len() == ed25519_sign::SEEDBYTES {
            decoded
        } else {
            return Err(err_msg(IndyErrorKind::InvalidStructure,
                               format!("Trying to use invalid base64 encoded `seed`. \
                                   The number of bytes must be {} ", ed25519_sign::SEEDBYTES)));
        }
    } else if seed.as_bytes().len() == ed25519_sign::SEEDBYTES * 2 {
        // is hex string
        Vec::from_hex(seed)
            .to_indy(IndyErrorKind::InvalidStructure, "Seed is invalid hex")?
    } else {
        return Err(err_msg(IndyErrorKind::InvalidStructure,
                           format!("Trying to use invalid `seed`. It can be either \
                               {} bytes string or base64 string or {} bytes HEX string", ed25519_sign::SEEDBYTES, ed25519_sign::SEEDBYTES * 2)));
    };

    let res = ed25519_sign::Seed::from_slice(bytes.as_slice())?;

    trace!("convert_seed <<< res: {:?}", secret!(&res));

    Ok(Some(res))
}

pub fn validate_key(vk: &str) -> IndyResult<()> {
    trace!("validate_key >>> vk: {:?}", vk);

    let vk = clear_verkey(vk)?;

    if vk.starts_with('~') {
        let _ = vk[1..].from_base58()?; // TODO: proper validate abbreviated verkey
    } else {
        let _vk = ed25519_sign::PublicKey::from_slice(vk.from_base58()?.as_slice())?;
    };

    trace!("validate_key <<<");

    Ok(())
}

pub fn validate_did(did: &DidValue) -> IndyResult<()> {
    trace!("validate_did >>> did: {:?}", did);
    // Useful method, huh?
    // Soon some state did validation will be put here
    trace!("validate_did <<< res: ()");

    Ok(())
}

pub fn random_key(seed: Option<&str>) -> IndyResult<String> {
    let key = match convert_seed(seed)? {
        Some(seed) => randombytes::randombytes_deterministic(chacha20poly1305_ietf::KEYBYTES, &randombytes::Seed::from_slice(&seed[..])?),
        None => randombytes::randombytes(chacha20poly1305_ietf::KEYBYTES)
    };

    let res = key[..].to_base58();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _key_info() -> KeyInfo {
        KeyInfo { seed: None, crypto_type: None }
    }

    fn _key() -> Key {
        create_key(&_key_info()).unwrap()
    }

    fn _message() -> &'static [u8] {
        r#"message"#.as_bytes()
    }

    #[test]
    fn create_my_key_works_for_empty_info() {
        let _my_key = create_key(&_key_info()).unwrap();
    }

    #[test]
    fn create_my_key_works_for_seed() {
        let seed = Some("00000000000000000000000000000My1".to_string());
        let key_info_with_seed = KeyInfo { seed, crypto_type: None };

        let key_1 = create_key(&key_info_with_seed).unwrap();
        let key_2 = create_key(&key_info_with_seed).unwrap();

        assert_eq!(key_1, key_2);

        let key_info_without_seed = _key_info();
        let key_3 = create_key(&key_info_without_seed).unwrap();

        assert_ne!(key_1, key_3);
    }

    #[test]
    fn sign_works() {
        let _sig = sign(&_key(), _message()).unwrap();
    }

    #[test]
    fn sign_works_for_invalid_signkey() {
        let my_key = Key::new("8wZcEriaNLNKtteJvx7f8i".to_string(), "5L2HBnzbu6Auh2pkDRbFt5f4prvgE2LzknkuYLsKkacp".to_string());
        let err = sign(&my_key, _message()).unwrap_err();
        assert_eq!(IndyErrorKind::InvalidStructure, err.kind());
    }

    #[test]
    fn sign_verify_works() {
        let key = _key();
        let signature = sign(&key, _message()).unwrap();
        let valid = verify(&key.verkey, _message(), &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn verify_not_works_for_invalid_verkey() {
        let key = _key();
        let signature = sign(&key, _message()).unwrap();
        let verkey = "AnnxV4t3LUHKZaxVQDWoVaG44NrGmeDYMA4Gz6C2tCZd";
        let valid = verify(verkey, _message(), &signature).unwrap();
        assert!(!valid);
    }

    #[test]
    fn crypto_box_works() {
        let my_key = _key();
        let their_key = _key();
        let _encrypted_message = crypto_box(&my_key, &their_key.verkey, _message()).unwrap();
    }

    #[test]
    fn crypto_box_and_crypto_box_open_works() {
        let my_key = _key();
        let their_key = _key();
        let (encrypted_message, nonce) = crypto_box(&my_key, &their_key.verkey, _message()).unwrap();

        let decrypted_message = crypto_box_open(&their_key, &my_key.verkey, &encrypted_message, &nonce).unwrap();

        assert_eq!(_message().to_vec(), decrypted_message);
    }

    #[test]
    fn crypto_box_seal_works() {
        let my_key = _key();
        let _encrypted_message = crypto_box_seal(&my_key.verkey, _message()).unwrap();
    }

    #[test]
    fn crypto_box_seal_and_crypto_box_seal_open_works() {
        let my_key = _key();
        let encrypted_message = crypto_box_seal(&my_key.verkey, _message()).unwrap();
        let decrypted_message = crypto_box_seal_open(&my_key, &encrypted_message).unwrap();
        assert_eq!(_message().to_vec(), decrypted_message);
    }
}