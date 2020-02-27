use indy_api_types::errors::prelude::*;

use crate::crypto::base64::{encode_urlsafe, decode_urlsafe};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct JWE {
    pub protected: String,
    pub iv: String,
    pub ciphertext: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Recipient {
    pub encrypted_key: String,
    pub header: Header,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Header {
    pub kid: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iv: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Protected {
    pub enc: String,
    pub typ: String,
    pub alg: String,
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UnpackMessage {
    pub message: String,
    pub recipient_verkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_verkey: Option<String>,
}

impl JWE {
    pub fn new(base64_protected: &str,
               ciphertext: &str,
               iv: &str,
               tag: &str,
    ) -> JWE {
        JWE {
            protected: base64_protected.to_string(),
            iv: iv.to_string(),
            ciphertext: ciphertext.to_string(),
            tag: tag.to_string(),
        }
    }

    pub fn as_bytes(&self) -> IndyResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to serialize JWE {}",
                err
            ))
        })
    }

    pub fn get_recipients(&self) -> IndyResult<Vec<Recipient>> {
        let protected = Protected::from_base64_encoded_string(&self.protected)?;
        Ok(protected.recipients)
    }
}

impl Recipient {
    pub fn new(kid: &str) -> Recipient {
        Recipient {
            encrypted_key: String::new(),
            header: Header {
                kid: kid.to_string(),
                sender: None,
                iv: None,
            },
        }
    }

    pub fn set_encrypted_key(mut self, encrypted_key: &[u8]) -> Self {
        self.encrypted_key = encode_urlsafe(encrypted_key);
        self
    }

    pub fn set_sender(mut self, sender: &[u8]) -> Self {
        self.header.sender = Some(encode_urlsafe(sender));
        self
    }

    pub fn set_iv(mut self, iv: &[u8]) -> Self {
        self.header.iv = Some(encode_urlsafe(iv));
        self
    }

    pub fn get_encrypted_key(&self) -> IndyResult<Vec<u8>> {
        decode_urlsafe(&self.encrypted_key)
    }

    pub fn get_sender(&self) -> IndyResult<Vec<u8>> {
        match self.header.sender.as_ref() {
            Some(sender) => decode_urlsafe(sender),
            None => Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "Recipient Header doesn't contain sender field"))
        }
    }

    pub fn get_iv(&self) -> IndyResult<Vec<u8>> {
        match self.header.iv.as_ref() {
            Some(iv) => decode_urlsafe(&iv),
            None => Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, "Recipient Header doesn't contain iv field"))
        }
    }
}

impl Protected {
    const PROTECTED_HEADER_ENC: &'static str = "xchacha20poly1305_ietf";
    const PROTECTED_HEADER_TYP: &'static str = "JWM/1.0";
    const PROTECTED_HEADER_ALG_AUTH: &'static str = "Authcrypt";
    const PROTECTED_HEADER_ALG_ANON: &'static str = "Anoncrypt";

    pub fn new(
        encrypted_recipients_struct: Vec<Recipient>,
        alg_is_authcrypt: bool) -> Protected {
        let alg_val = if alg_is_authcrypt { String::from(Self::PROTECTED_HEADER_ALG_AUTH) } else { String::from(Self::PROTECTED_HEADER_ALG_ANON) };

        //structure protected and base64URL encode it
        Protected {
            enc: Self::PROTECTED_HEADER_ENC.to_string(),
            typ: Self::PROTECTED_HEADER_TYP.to_string(),
            alg: alg_val,
            recipients: encrypted_recipients_struct,
        }
    }

    pub fn to_base64_encoded_string(&self) -> IndyResult<String> {
        let protected_encoded = serde_json::to_string(self).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to serialize protected field {}",
                err
            ))
        })?;

        Ok(encode_urlsafe(protected_encoded.as_bytes()))
    }

    pub fn from_base64_encoded_string(protected: &str) -> IndyResult<Protected> {
        //decode protected data
        let protected_decoded_vec = decode_urlsafe(protected)?;
        let protected_decoded_str = String::from_utf8(protected_decoded_vec).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to utf8 encode data {}",
                err
            ))
        })?;
        //convert protected_data_str to struct
        let protected_struct: Protected = serde_json::from_str(&protected_decoded_str).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to deserialize protected data {}",
                err
            ))
        })?;

        Ok(protected_struct)
    }
}