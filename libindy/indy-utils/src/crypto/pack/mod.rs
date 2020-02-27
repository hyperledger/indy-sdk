use std::str;

use indy_api_types::domain::crypto::key::Key;

use crate::crypto::{chacha20poly1305_ietf, base64, crypto_box_seal, crypto_box, crypto_box_seal_open, crypto_box_open};

pub mod jwe;

use indy_api_types::errors::prelude::*;
use jwe::{JWE, Recipient, Protected, UnpackMessage};

pub fn pack_msg(
    message: Vec<u8>,
    sender_key: Option<Key>,
    receiver_list: Vec<String>,
) -> IndyResult<Vec<u8>> {

    //break early and error out if no receivers keys are provided
    if receiver_list.is_empty() {
        return Err(err_msg(IndyErrorKind::InvalidStructure, "No receiver keys found".to_string()));
    }

    //generate content encryption key that will encrypt `message`
    let cek = chacha20poly1305_ietf::gen_key();

    let base64_protected = match sender_key {
        Some(sender_key_) => {
            //returns authcrypted pack_message format. See Wire message format HIPE for details
            _prepare_protected_authcrypt(&cek, receiver_list, sender_key_)?
        }
        None => {
            //returns anoncrypted pack_message format. See Wire message format HIPE for details
            _prepare_protected_anoncrypt(&cek, receiver_list)?
        }
    };

    // Use AEAD to encrypt `message` with "protected" data as "associated data"
    let (ciphertext, iv, tag) =
        encrypt_plaintext(message, &base64_protected, &cek);

    JWE::new(&base64_protected, &ciphertext, &iv, &tag)
        .as_bytes()
}

fn encrypt_plaintext(plaintext: Vec<u8>,
                     aad: &str,
                     cek: &chacha20poly1305_ietf::Key)
                     -> (String, String, String) {
    //encrypt message with aad
    let (ciphertext, iv, tag) = chacha20poly1305_ietf::gen_nonce_and_encrypt_detached(
        plaintext.as_slice(), aad.as_bytes(), &cek);

    //base64 url encode data
    let iv_encoded = base64::encode_urlsafe(&iv[..]);
    let ciphertext_encoded = base64::encode_urlsafe(ciphertext.as_slice());
    let tag_encoded = base64::encode_urlsafe(&tag[..]);

    (ciphertext_encoded, iv_encoded, tag_encoded)
}

fn _prepare_protected_anoncrypt(cek: &chacha20poly1305_ietf::Key,
                                receiver_list: Vec<String>,
) -> IndyResult<String> {
    let mut encrypted_recipients_struct: Vec<Recipient> = Vec::with_capacity(receiver_list.len());

    for their_vk in receiver_list {
        //encrypt sender verkey
        let enc_cek = crypto_box_seal(&their_vk, &cek[..])?;

        //create recipient struct and push to encrypted list
        encrypted_recipients_struct.push(
            Recipient::new(&their_vk)
                .set_encrypted_key(enc_cek.as_slice())
        );
    } // end for-loop

    //structure protected and base64URL encode it
    Protected::new(encrypted_recipients_struct, false)
        .to_base64_encoded_string()
}

fn _prepare_protected_authcrypt(cek: &chacha20poly1305_ietf::Key,
                                receiver_list: Vec<String>,
                                sender_vk: Key,
) -> IndyResult<String> {
    let mut encrypted_recipients_struct: Vec<Recipient> = vec![];

    //encrypt cek for recipient
    for their_vk in receiver_list {
        let (enc_cek, iv) = crypto_box(&sender_vk, &their_vk, &cek[..])?;

        let enc_sender = crypto_box_seal(&their_vk, sender_vk.verkey.as_bytes())?;

        //create recipient struct and push to encrypted list
        encrypted_recipients_struct.push(
            Recipient::new(&their_vk)
                .set_encrypted_key(enc_cek.as_slice())
                .set_sender(enc_sender.as_slice())
                .set_iv(iv.as_slice())
        );
    } // end for-loop

    Protected::new(encrypted_recipients_struct, true)
        .to_base64_encoded_string()
}

pub fn msg_recipients(jwe_struct: &JWE) -> IndyResult<Vec<Recipient>> {
    jwe_struct.get_recipients()
}

pub fn unpack_msg(jwe_struct: JWE, recipient: Recipient, recipient_key: Key) -> IndyResult<Vec<u8>> {
    //get cek and sender data
    let (sender_verkey, cek) = match recipient.header.sender.as_ref() {
        Some(_) => _unpack_cek_authcrypt(&recipient, &recipient_key)?,
        None => _unpack_cek_anoncrypt(&recipient, &recipient_key)?
    };

    //decrypt message
    let message = decrypt_ciphertext(
        &jwe_struct.ciphertext,
        &jwe_struct.protected,
        &jwe_struct.iv,
        &jwe_struct.tag,
        &cek,
    )?;

    //serialize and return decrypted message
    let res = UnpackMessage {
        message,
        sender_verkey,
        recipient_verkey: recipient.header.kid.to_string(),
    };

    serde_json::to_vec(&res).map_err(|err| {
        err_msg(IndyErrorKind::InvalidStructure, format!(
            "Failed to serialize message {}",
            err
        ))
    })
}

/* ciphertext helper functions*/
fn decrypt_ciphertext(
    ciphertext: &str,
    aad: &str,
    iv: &str,
    tag: &str,
    cek: &chacha20poly1305_ietf::Key,
) -> Result<String, IndyError> {
    //convert ciphertext to bytes
    let ciphertext_as_vec = base64::decode_urlsafe(ciphertext).map_err(|err| {
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decode ciphertext {}", err))
    })?;
    let ciphertext_as_bytes = ciphertext_as_vec.as_ref();

    //convert IV from &str to &Nonce
    let nonce_as_vec = base64::decode_urlsafe(iv).map_err(|err|
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decode IV {}", err))
    )?;
    let nonce_as_slice = nonce_as_vec.as_slice();
    let nonce = chacha20poly1305_ietf::Nonce::from_slice(nonce_as_slice).map_err(|err| {
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to convert IV to Nonce type {}", err))
    })?;

    //convert tag from &str to &Tag
    let tag_as_vec = base64::decode_urlsafe(tag).map_err(|err|
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decode tag {}", err))
    )?;
    let tag_as_slice = tag_as_vec.as_slice();
    let tag = chacha20poly1305_ietf::Tag::from_slice(tag_as_slice).map_err(|err| {
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to convert tag to Tag type {}", err))
    })?;

    //decrypt message
    let plaintext_bytes =
        chacha20poly1305_ietf::decrypt_detached(ciphertext_as_bytes,
                                                cek,
                                                &nonce,
                                                &tag,
                                                Some(aad.as_bytes()))
            .map_err(|err| {
                err_msg(IndyErrorKind::UnknownCrypto, format!("Failed to decrypt ciphertext {}", err))
            })?;

    //convert message to readable (UTF-8) string
    String::from_utf8(plaintext_bytes).map_err(|err| {
        err_msg(IndyErrorKind::InvalidStructure, format!("Failed to convert message to UTF-8 {}", err))
    })
}

fn _unpack_cek_authcrypt(recipient: &Recipient, recipient_key: &Key) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
    let encrypted_key_vec = recipient.get_encrypted_key()?;
    let iv = recipient.get_iv()?;
    let enc_sender_vk = recipient.get_sender()?;

    //decrypt sender_vk
    let sender_vk_vec = crypto_box_seal_open(recipient_key, enc_sender_vk.as_slice())?;
    let sender_vk = String::from_utf8(sender_vk_vec)
        .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Failed to utf-8 encode sender_vk {}", err)))?;

    //decrypt cek
    let cek_as_vec = crypto_box_open(
        recipient_key,
        &sender_vk,
        encrypted_key_vec.as_slice(),
        iv.as_slice())?;

    //convert cek to chacha Key struct
    let cek: chacha20poly1305_ietf::Key =
        chacha20poly1305_ietf::Key::from_slice(&cek_as_vec[..]).map_err(
            |err| {
                err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decrypt cek {}", err))
            },
        )?;

    Ok((Some(sender_vk), cek))
}

fn _unpack_cek_anoncrypt(recipient: &Recipient, recipient_key: &Key) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
    let encrypted_key_vec = recipient.get_encrypted_key()?;

    //decrypt cek
    let cek_as_vec = crypto_box_seal_open(&recipient_key, encrypted_key_vec.as_slice())?;

    //convert cek to chacha Key struct
    let cek: chacha20poly1305_ietf::Key =
        chacha20poly1305_ietf::Key::from_slice(&cek_as_vec[..]).map_err(
            |err| {
                err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decrypt cek {}", err))
            },
        )?;

    Ok((None, cek))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::chacha20poly1305_ietf::gen_key;

    #[test]
    pub fn test_encrypt_plaintext_and_decrypt_ciphertext_works() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (expected_ciphertext, iv_encoded, tag) = encrypt_plaintext(plaintext.clone(), aad, &cek);


        let expected_plaintext = decrypt_ciphertext(&expected_ciphertext, aad, &iv_encoded, &tag, &cek).unwrap();

        assert_eq!(expected_plaintext.as_bytes().to_vec(), plaintext);
    }


    #[test]
    pub fn test_encrypt_plaintext_decrypt_ciphertext_empty_string_works() {
        let plaintext = "".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (expected_ciphertext, iv_encoded, tag) = encrypt_plaintext(plaintext.clone(), aad, &cek);


        let expected_plaintext = decrypt_ciphertext(&expected_ciphertext, aad, &iv_encoded, &tag, &cek).unwrap();

        assert_eq!(expected_plaintext.as_bytes().to_vec(), plaintext);
    }

    #[test]
    pub fn test_encrypt_plaintext_decrypt_ciphertext_bad_iv_fails() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (expected_ciphertext, _, tag) = encrypt_plaintext(plaintext, aad, &cek);

        //convert values to base64 encoded strings
        let bad_iv_input = "invalid_iv";

        let expected_error = decrypt_ciphertext(&expected_ciphertext, bad_iv_input, &tag, aad, &cek);
        assert!(expected_error.is_err());
    }

    #[test]
    pub fn test_encrypt_plaintext_decrypt_ciphertext_bad_ciphertext_fails() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (_, iv_encoded, tag) = encrypt_plaintext(plaintext, aad, &cek);

        let bad_ciphertext = base64::encode_urlsafe("bad_ciphertext".as_bytes());

        let expected_error = decrypt_ciphertext(&bad_ciphertext, &iv_encoded, &tag, aad, &cek);
        assert!(expected_error.is_err());
    }

    #[test]
    pub fn test_encrypt_plaintext_and_decrypt_ciphertext_wrong_cek_fails() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = chacha20poly1305_ietf::gen_key();

        let (expected_ciphertext, iv_encoded, tag) = encrypt_plaintext(plaintext, aad, &cek);

        let bad_cek = gen_key();

        let expected_error = decrypt_ciphertext(&expected_ciphertext, &iv_encoded, &tag, aad, &bad_cek);
        assert!(expected_error.is_err());
    }

    #[test]
    pub fn test_encrypt_plaintext_and_decrypt_ciphertext_bad_tag_fails() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (expected_ciphertext, iv_encoded, _) = encrypt_plaintext(plaintext, aad, &cek);

        let bad_tag = "bad_tag".to_string();

        let expected_error = decrypt_ciphertext(&expected_ciphertext, &iv_encoded, &bad_tag, aad, &cek);
        assert!(expected_error.is_err());
    }

    #[test]
    pub fn test_encrypt_plaintext_and_decrypt_ciphertext_bad_aad_fails() {
        let plaintext = "Hello World".as_bytes().to_vec();
        // AAD allows the sender to tie extra (protocol) data to the encryption. Example JWE enc and alg
        // Which the receiver MUST then check before decryption
        let aad = "some protocol data input to the encryption";
        let cek = gen_key();

        let (expected_ciphertext, iv_encoded, tag) = encrypt_plaintext(plaintext, aad, &cek);

        let bad_aad = "bad aad";

        let expected_error = decrypt_ciphertext(&expected_ciphertext, &iv_encoded, &tag, bad_aad, &cek);
        assert!(expected_error.is_err());
    }
}
