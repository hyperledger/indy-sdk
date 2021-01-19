use std::collections::HashMap;

use crate::domain::crypto::key::{Key, KeyInfo, KeyMetadata};
use crate::domain::crypto::pack::*;
use indy_api_types::errors::prelude::*;
use crate::services::crypto::CryptoService;
use indy_wallet::{RecordOptions, WalletService};

use std::rc::Rc;
use std::str;
use indy_utils::crypto::base64;
use indy_utils::crypto::chacha20poly1305_ietf;
use crate::domain::crypto::combo_box::ComboBox;
use indy_api_types::WalletHandle;
use crate::services::metrics::MetricsService;

pub const PROTECTED_HEADER_ENC: &str = "xchacha20poly1305_ietf";
pub const PROTECTED_HEADER_TYP: &str = "JWM/1.0";
pub const PROTECTED_HEADER_ALG_AUTH: &str = "Authcrypt";
pub const PROTECTED_HEADER_ALG_ANON: &str = "Anoncrypt";

pub enum CryptoCommand {
    CreateKey(
        WalletHandle,
        KeyInfo, // key info
        Box<dyn Fn(IndyResult<String /*verkey*/>, Rc<MetricsService>) + Send>,
    ),
    SetKeyMetadata(
        WalletHandle,
        String, // verkey
        String, // metadata
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>,
    ),
    GetKeyMetadata(
        WalletHandle,
        String, // verkey
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>,
    ),
    CryptoSign(
        WalletHandle,
        String,  // my vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
    CryptoVerify(
        String,  // their vk
        Vec<u8>, // msg
        Vec<u8>, // signature
        Box<dyn Fn(IndyResult<bool>, Rc<MetricsService>) + Send>,
    ),
    AuthenticatedEncrypt(
        WalletHandle,
        String,  // my vk
        String,  // their vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
    AuthenticatedDecrypt(
        WalletHandle,
        String,  // my vk
        Vec<u8>, // encrypted msg
        Box<dyn Fn(IndyResult<(String, Vec<u8>)>, Rc<MetricsService>) + Send>,
    ),
    AnonymousEncrypt(
        String,  // their vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
    AnonymousDecrypt(
        WalletHandle,
        String,  // my vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
    PackMessage(
        Vec<u8>, // plaintext message
        Vec<String>,  // list of receiver's keys
        Option<String>,  // senders verkey
        WalletHandle,
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
    UnpackMessage(
        JWE,
        WalletHandle,
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>,
    ),
}

pub struct CryptoCommandExecutor {
    wallet_service: Rc<WalletService>,
    crypto_service: Rc<CryptoService>,
    metrics_service: Rc<MetricsService>,
}

impl CryptoCommandExecutor {
    pub fn new(
        wallet_service: Rc<WalletService>,
        crypto_service: Rc<CryptoService>,
        metrics_service: Rc<MetricsService>,
    ) -> CryptoCommandExecutor {
        CryptoCommandExecutor {
            wallet_service,
            crypto_service,
            metrics_service,
        }
    }

    pub fn execute(&self, command: CryptoCommand) {
        match command {
            CryptoCommand::CreateKey(wallet_handle, key_info, cb) => {
                debug!("CreateKey command received");
                cb(self.create_key(wallet_handle, &key_info), self.metrics_service.clone());
            }
            CryptoCommand::SetKeyMetadata(wallet_handle, verkey, metadata, cb) => {
                debug!("SetKeyMetadata command received");
                cb(self.set_key_metadata(wallet_handle, &verkey, &metadata), self.metrics_service.clone());
            }
            CryptoCommand::GetKeyMetadata(wallet_handle, verkey, cb) => {
                debug!("GetKeyMetadata command received");
                cb(self.get_key_metadata(wallet_handle, &verkey), self.metrics_service.clone());
            }
            CryptoCommand::CryptoSign(wallet_handle, my_vk, msg, cb) => {
                debug!("CryptoSign command received");
                cb(self.crypto_sign(wallet_handle, &my_vk, &msg), self.metrics_service.clone());
            }
            CryptoCommand::CryptoVerify(their_vk, msg, signature, cb) => {
                debug!("CryptoVerify command received");
                cb(self.crypto_verify(&their_vk, &msg, &signature), self.metrics_service.clone());
            }
            CryptoCommand::AuthenticatedEncrypt(wallet_handle, my_vk, their_vk, msg, cb) => {
                debug!("AuthenticatedEncrypt command received");
                cb(self.authenticated_encrypt(wallet_handle, &my_vk, &their_vk, &msg), self.metrics_service.clone());
            }
            CryptoCommand::AuthenticatedDecrypt(wallet_handle, my_vk, encrypted_msg, cb) => {
                debug!("AuthenticatedDecrypt command received");
                cb(self.authenticated_decrypt(wallet_handle, &my_vk, &encrypted_msg), self.metrics_service.clone());
            }
            CryptoCommand::AnonymousEncrypt(their_vk, msg, cb) => {
                debug!("AnonymousEncrypt command received");
                cb(self.anonymous_encrypt(&their_vk, &msg), self.metrics_service.clone());
            }
            CryptoCommand::AnonymousDecrypt(wallet_handle, my_vk, encrypted_msg, cb) => {
                debug!("AnonymousDecrypt command received");
                cb(self.anonymous_decrypt(wallet_handle, &my_vk, &encrypted_msg), self.metrics_service.clone());
            }
            CryptoCommand::PackMessage(message, receivers, sender_vk, wallet_handle, cb) => {
                debug!("PackMessage command received");
                cb(self.pack_msg(message, receivers, sender_vk, wallet_handle), self.metrics_service.clone());
            }
            CryptoCommand::UnpackMessage(jwe_json, wallet_handle, cb) => {
                debug!("UnpackMessage command received");
                cb(self.unpack_msg(jwe_json, wallet_handle), self.metrics_service.clone());
            }
        };
    }

    fn create_key(&self, wallet_handle: WalletHandle, key_info: &KeyInfo) -> IndyResult<String> {
        debug!(
            "create_key >>> wallet_handle: {:?}, key_info: {:?}",
            wallet_handle,
            secret!(key_info)
        );

        let key = self.crypto_service.create_key(key_info)?;
        self.wallet_service
            .add_indy_object(wallet_handle, &key.verkey, &key, &HashMap::new())?;

        let res = key.verkey.to_string();
        debug!("create_key <<< res: {:?}", res);
        Ok(res)
    }

    fn crypto_sign(&self, wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> IndyResult<Vec<u8>> {
        trace!(
            "crypto_sign >>> wallet_handle: {:?}, sender_vk: {:?}, msg: {:?}",
            wallet_handle, my_vk, msg
        );

        self.crypto_service.validate_key(my_vk)?;

        let key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            &my_vk,
            &RecordOptions::id_value(),
        )?;

        let res = self.crypto_service.sign(&key, msg)?;

        trace!("crypto_sign <<< res: {:?}", res);

        Ok(res)
    }

    fn crypto_verify(&self,
                     their_vk: &str,
                     msg: &[u8],
                     signature: &[u8]) -> IndyResult<bool> {
        trace!(
            "crypto_verify >>> their_vk: {:?}, msg: {:?}, signature: {:?}",
            their_vk, msg, signature
        );

        self.crypto_service.validate_key(their_vk)?;

        let res = self.crypto_service.verify(their_vk, msg, signature)?;

        trace!("crypto_verify <<< res: {:?}", res);

        Ok(res)
    }

    //TODO begin deprecation process this function. It will be replaced by pack
    fn authenticated_encrypt(
        &self,
        wallet_handle: WalletHandle,
        my_vk: &str,
        their_vk: &str,
        msg: &[u8],
    ) -> IndyResult<Vec<u8>> {
        trace!("authenticated_encrypt >>> wallet_handle: {:?}, my_vk: {:?}, their_vk: {:?}, msg: {:?}", wallet_handle, my_vk, their_vk, msg);

        self.crypto_service.validate_key(my_vk)?;
        self.crypto_service.validate_key(their_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            my_vk,
            &RecordOptions::id_value(),
        )?;

        let msg = self.crypto_service.create_combo_box(&my_key, &their_vk, msg)?;

        let msg = msg.to_msg_pack()
            .map_err(|e| err_msg(IndyErrorKind::InvalidState, format!("Can't serialize ComboBox: {:?}", e)))?;

        let res = self.crypto_service.crypto_box_seal(&their_vk, &msg)?;

        trace!("authenticated_encrypt <<< res: {:?}", res);

        Ok(res)
    }

    //TODO begin deprecation process this function. It will be replaced by unpack
    fn authenticated_decrypt(
        &self,
        wallet_handle: WalletHandle,
        my_vk: &str,
        msg: &[u8],
    ) -> IndyResult<(String, Vec<u8>)> {
        trace!("authenticated_decrypt >>> wallet_handle: {:?}, my_vk: {:?}, msg: {:?}", wallet_handle, my_vk, msg);

        self.crypto_service.validate_key(my_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            my_vk,
            &RecordOptions::id_value(),
        )?;

        let decrypted_msg = self.crypto_service.crypto_box_seal_open(&my_key, &msg)?;

        let parsed_msg = ComboBox::from_msg_pack(decrypted_msg.as_slice())
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't deserialize ComboBox: {:?}", err)))?;

        let doc: Vec<u8> = base64::decode(&parsed_msg.msg)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't decode internal msg filed from base64 {}", err)))?;

        let nonce: Vec<u8> = base64::decode(&parsed_msg.nonce)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't decode nonce from base64 {}", err)))?;

        let decrypted_msg = self.crypto_service.crypto_box_open(&my_key, &parsed_msg.sender, &doc, &nonce)?;

        let res = (parsed_msg.sender, decrypted_msg);

        trace!("authenticated_decrypt <<< res: {:?}", res);

        Ok(res)
    }

    fn anonymous_encrypt(&self,
                         their_vk: &str,
                         msg: &[u8]) -> IndyResult<Vec<u8>> {
        trace!(
            "anonymous_encrypt >>> their_vk: {:?}, msg: {:?}",
            their_vk, msg
        );

        self.crypto_service.validate_key(their_vk)?;

        let res = self.crypto_service.crypto_box_seal(their_vk, &msg)?;

        trace!("anonymous_encrypt <<< res: {:?}", res);

        Ok(res)
    }

    fn anonymous_decrypt(&self,
                         wallet_handle: WalletHandle,
                         my_vk: &str,
                         encrypted_msg: &[u8]) -> IndyResult<Vec<u8>> {
        trace!(
            "anonymous_decrypt >>> wallet_handle: {:?}, my_vk: {:?}, encrypted_msg: {:?}",
            wallet_handle, my_vk, encrypted_msg
        );

        self.crypto_service.validate_key(&my_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            &my_vk,
            &RecordOptions::id_value(),
        )?;

        let res = self
            .crypto_service
            .crypto_box_seal_open(&my_key, &encrypted_msg)?;

        trace!("anonymous_decrypt <<< res: {:?}", res);

        Ok(res)
    }

    fn set_key_metadata(&self, wallet_handle: WalletHandle, verkey: &str, metadata: &str) -> IndyResult<()> {
        debug!(
            "set_key_metadata >>> wallet_handle: {:?}, verkey: {:?}, metadata: {:?}",
            wallet_handle, verkey, metadata
        );

        self.crypto_service.validate_key(verkey)?;

        let metadata = KeyMetadata {
            value: metadata.to_string(),
        };

        self.wallet_service
            .upsert_indy_object(wallet_handle, &verkey, &metadata)?;

        debug!("set_key_metadata <<<");

        Ok(())
    }

    fn get_key_metadata(&self, wallet_handle: WalletHandle, verkey: &str) -> IndyResult<String> {
        debug!(
            "get_key_metadata >>> wallet_handle: {:?}, verkey: {:?}",
            wallet_handle, verkey
        );

        self.crypto_service.validate_key(verkey)?;

        let metadata = self.wallet_service.get_indy_object::<KeyMetadata>(
            wallet_handle,
            &verkey,
            &RecordOptions::id_value(),
        )?;

        let res = metadata.value;

        debug!("get_key_metadata <<< res: {:?}", res);

        Ok(res)
    }

    //TODO: Refactor pack to be more modular to version changes or crypto_scheme changes
    //this match statement is super messy, but the easiest way to comply with current architecture
    pub fn pack_msg(
        &self,
        message: Vec<u8>,
        receiver_list: Vec<String>,
        sender_vk: Option<String>,
        wallet_handle: WalletHandle,
    ) -> IndyResult<Vec<u8>> {

        //break early and error out if no receivers keys are provided
        if receiver_list.is_empty() {
            return Err(err_msg(IndyErrorKind::InvalidStructure, "No receiver keys found".to_string()));
        }

        //generate content encryption key that will encrypt `message`
        let cek = chacha20poly1305_ietf::gen_key();

        let base64_protected = if let Some(sender_vk) = sender_vk {
            self.crypto_service.validate_key(&sender_vk)?;

            //returns authcrypted pack_message format. See Wire message format HIPE for details
            self._prepare_protected_authcrypt(&cek, receiver_list, &sender_vk, wallet_handle)?
        } else {
            //returns anoncrypted pack_message format. See Wire message format HIPE for details
            self._prepare_protected_anoncrypt(&cek, receiver_list)?
        };

        // Use AEAD to encrypt `message` with "protected" data as "associated data"
        let (ciphertext, iv, tag) =
            self.crypto_service
                .encrypt_plaintext(message, &base64_protected, &cek);

        self._format_pack_message(&base64_protected, &ciphertext, &iv, &tag)
    }

    fn _prepare_protected_anoncrypt(&self,
                                    cek: &chacha20poly1305_ietf::Key,
                                    receiver_list: Vec<String>,
    ) -> IndyResult<String> {
        let mut encrypted_recipients_struct : Vec<Recipient> = Vec::with_capacity(receiver_list.len());

        for their_vk in receiver_list {
            //encrypt sender verkey
            let enc_cek = self.crypto_service.crypto_box_seal(&their_vk, &cek[..])?;

            //create recipient struct and push to encrypted list
            encrypted_recipients_struct.push(Recipient {
                encrypted_key: base64::encode_urlsafe(enc_cek.as_slice()),
                header: Header {
                    kid: their_vk,
                    sender: None,
                    iv: None
                },
            });
        } // end for-loop
        Ok(self._base64_encode_protected(encrypted_recipients_struct, false)?)
    }

    fn _prepare_protected_authcrypt(&self,
                                    cek: &chacha20poly1305_ietf::Key,
                                    receiver_list: Vec<String>, sender_vk: &str,
                                    wallet_handle: WalletHandle,
    ) -> IndyResult<String> {
        let mut encrypted_recipients_struct : Vec<Recipient> = vec![];

        //get my_key from my wallet
        let my_key = self.wallet_service.get_indy_object(
            wallet_handle,
            sender_vk,
            &RecordOptions::id_value()
        )?;

        //encrypt cek for recipient
        for their_vk in receiver_list {
            let (enc_cek, iv) = self.crypto_service.crypto_box(&my_key, &their_vk, &cek[..])?;

            let enc_sender = self.crypto_service.crypto_box_seal(&their_vk, sender_vk.as_bytes())?;

            //create recipient struct and push to encrypted list
            encrypted_recipients_struct.push(Recipient {
                encrypted_key: base64::encode_urlsafe(enc_cek.as_slice()),
                header: Header {
                    kid: their_vk,
                    sender: Some(base64::encode_urlsafe(enc_sender.as_slice())),
                    iv: Some(base64::encode_urlsafe(iv.as_slice()))
                },
            });
        } // end for-loop

        Ok(self._base64_encode_protected(encrypted_recipients_struct, true)?)
    }

    fn _base64_encode_protected(&self, encrypted_recipients_struct: Vec<Recipient>, alg_is_authcrypt: bool) -> IndyResult<String> {
        let alg_val = if alg_is_authcrypt { String::from(PROTECTED_HEADER_ALG_AUTH) } else { String::from(PROTECTED_HEADER_ALG_ANON) };

        //structure protected and base64URL encode it
        let protected_struct = Protected {
            enc: PROTECTED_HEADER_ENC.to_string(),
            typ: PROTECTED_HEADER_TYP.to_string(),
            alg: alg_val,
            recipients: encrypted_recipients_struct,
        };
        let protected_encoded = serde_json::to_string(&protected_struct).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to serialize protected field {}",
                err
            ))
        })?;

        Ok(base64::encode_urlsafe(protected_encoded.as_bytes()))
    }

    fn _format_pack_message(
        &self,
        base64_protected: &str,
        ciphertext: &str,
        iv: &str,
        tag: &str
    ) -> IndyResult<Vec<u8>> {

        //serialize pack message and return as vector of bytes
        let jwe_struct = JWE {
            protected: base64_protected.to_string(),
            iv: iv.to_string(),
            ciphertext: ciphertext.to_string(),
            tag: tag.to_string()
        };

        serde_json::to_vec(&jwe_struct).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to serialize JWE {}",
                err
            ))
        })
    }

    pub fn unpack_msg(&self, jwe_struct: JWE, wallet_handle: WalletHandle) -> IndyResult<Vec<u8>> {
        //decode protected data
        let protected_decoded_vec = base64::decode_urlsafe(&jwe_struct.protected)?;
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

        //extract recipient that matches a key in the wallet
        let (recipient, is_auth_recipient) = self._find_correct_recipient(protected_struct, wallet_handle)?;

        //get cek and sender data
        let (sender_verkey_option, cek) = if is_auth_recipient {
            self._unpack_cek_authcrypt(recipient.clone(), wallet_handle)
        } else {
            self._unpack_cek_anoncrypt(recipient.clone(), wallet_handle)
        }?; //close cek and sender_data match statement

        //decrypt message
        let message = self.crypto_service.decrypt_ciphertext(
            &jwe_struct.ciphertext,
            &jwe_struct.protected,
            &jwe_struct.iv,
            &jwe_struct.tag,
            &cek,
        )?;

        //serialize and return decrypted message
        let res = UnpackMessage {
            message,
            sender_verkey: sender_verkey_option,
            recipient_verkey: recipient.header.kid
        };

        serde_json::to_vec(&res).map_err(|err| {
            err_msg(IndyErrorKind::InvalidStructure, format!(
                "Failed to serialize message {}",
                err
            ))
        })
    }

    fn _find_correct_recipient(&self, protected_struct: Protected, wallet_handle: WalletHandle) -> IndyResult<(Recipient, bool)>{
        for recipient in protected_struct.recipients {
            let my_key_res = self.wallet_service.get_indy_object::<Key>(
                wallet_handle,
                &recipient.header.kid,
                &RecordOptions::id_value()
            );


            if my_key_res.is_ok() {
                return Ok((recipient.clone(), recipient.header.sender.is_some()))
            }
        }
        Err(IndyError::from(IndyErrorKind::WalletItemNotFound))
    }

    fn _unpack_cek_authcrypt(&self, recipient: Recipient, wallet_handle: WalletHandle) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
        let encrypted_key_vec = base64::decode_urlsafe(&recipient.encrypted_key)?;
        let iv = base64::decode_urlsafe(&recipient.header.iv.unwrap())?;
        let enc_sender_vk = base64::decode_urlsafe(&recipient.header.sender.unwrap())?;

        //get my private key
        let my_key = self.wallet_service.get_indy_object(
            wallet_handle,
            &recipient.header.kid,
            &RecordOptions::id_value(),
        )?;

        //decrypt sender_vk
        let sender_vk_vec = self.crypto_service.crypto_box_seal_open(&my_key, enc_sender_vk.as_slice())?;
        let sender_vk = String::from_utf8(sender_vk_vec)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Failed to utf-8 encode sender_vk {}", err)))?;

        //decrypt cek
        let cek_as_vec = self.crypto_service.crypto_box_open(
            &my_key,
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

    fn _unpack_cek_anoncrypt(&self, recipient: Recipient, wallet_handle: WalletHandle) -> IndyResult<(Option<String>, chacha20poly1305_ietf::Key)> {
        let encrypted_key_vec = base64::decode_urlsafe(&recipient.encrypted_key)?;

        //get my private key
        let my_key : Key = self.wallet_service.get_indy_object(
            wallet_handle,
            &recipient.header.kid,
            &RecordOptions::id_value(),
        )?;

        //decrypt cek
        let cek_as_vec = self.crypto_service
            .crypto_box_seal_open(&my_key, encrypted_key_vec.as_slice())?;

        //convert cek to chacha Key struct
        let cek: chacha20poly1305_ietf::Key =
            chacha20poly1305_ietf::Key::from_slice(&cek_as_vec[..]).map_err(
                |err| {
                    err_msg(IndyErrorKind::InvalidStructure, format!("Failed to decrypt cek {}", err))
                },
            )?;

        Ok((None, cek))
    }

}
