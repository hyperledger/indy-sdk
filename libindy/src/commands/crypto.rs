use std::collections::HashMap;

use indy_api_types::domain::crypto::key::{Key, KeyInfo, KeyMetadata};
use indy_api_types::errors::prelude::*;
use indy_wallet::{RecordOptions, WalletService};
use indy_utils::crypto::pack::{pack_msg, unpack_msg, jwe::JWE, jwe::Recipient};

use std::rc::Rc;
use std::str;
use indy_utils::crypto::{base64, create_key, sign, verify, create_combo_box, crypto_box_seal, crypto_box_seal_open, crypto_box_open, validate_key};
use indy_api_types::domain::crypto::combo_box::ComboBox;
use indy_api_types::WalletHandle;

pub enum CryptoCommand {
    CreateKey(
        WalletHandle,
        KeyInfo, // key info
        Box<dyn Fn(IndyResult<String /*verkey*/>) + Send>,
    ),
    SetKeyMetadata(
        WalletHandle,
        String, // verkey
        String, // metadata
        Box<dyn Fn(IndyResult<()>) + Send>,
    ),
    GetKeyMetadata(
        WalletHandle,
        String, // verkey
        Box<dyn Fn(IndyResult<String>) + Send>,
    ),
    CryptoSign(
        WalletHandle,
        String, // my vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
    CryptoVerify(
        String, // their vk
        Vec<u8>, // msg
        Vec<u8>, // signature
        Box<dyn Fn(IndyResult<bool>) + Send>,
    ),
    AuthenticatedEncrypt(
        WalletHandle,
        String, // my vk
        String, // their vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
    AuthenticatedDecrypt(
        WalletHandle,
        String, // my vk
        Vec<u8>, // encrypted msg
        Box<dyn Fn(IndyResult<(String, Vec<u8>)>) + Send>,
    ),
    AnonymousEncrypt(
        String, // their vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
    AnonymousDecrypt(
        WalletHandle,
        String, // my vk
        Vec<u8>, // msg
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
    PackMessage(
        Vec<u8>, // plaintext message
        Vec<String>, // list of receiver's keys
        Option<String>, // senders verkey
        WalletHandle,
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
    UnpackMessage(
        JWE,
        WalletHandle,
        Box<dyn Fn(IndyResult<Vec<u8>>) + Send>,
    ),
}

pub struct CryptoCommandExecutor {
    wallet_service: Rc<WalletService>
}

impl CryptoCommandExecutor {
    pub fn new(
        wallet_service: Rc<WalletService>,
    ) -> CryptoCommandExecutor {
        CryptoCommandExecutor {
            wallet_service,
        }
    }

    pub fn execute(&self, command: CryptoCommand) {
        match command {
            CryptoCommand::CreateKey(wallet_handle, key_info, cb) => {
                debug!("CreateKey command received");
                cb(self.create_key(wallet_handle, &key_info));
            }
            CryptoCommand::SetKeyMetadata(wallet_handle, verkey, metadata, cb) => {
                debug!("SetKeyMetadata command received");
                cb(self.set_key_metadata(wallet_handle, &verkey, &metadata));
            }
            CryptoCommand::GetKeyMetadata(wallet_handle, verkey, cb) => {
                debug!("GetKeyMetadata command received");
                cb(self.get_key_metadata(wallet_handle, &verkey));
            }
            CryptoCommand::CryptoSign(wallet_handle, my_vk, msg, cb) => {
                debug!("CryptoSign command received");
                cb(self.crypto_sign(wallet_handle, &my_vk, &msg));
            }
            CryptoCommand::CryptoVerify(their_vk, msg, signature, cb) => {
                debug!("CryptoVerify command received");
                cb(self.crypto_verify(&their_vk, &msg, &signature));
            }
            CryptoCommand::AuthenticatedEncrypt(wallet_handle, my_vk, their_vk, msg, cb) => {
                debug!("AuthenticatedEncrypt command received");
                cb(self.authenticated_encrypt(wallet_handle, &my_vk, &their_vk, &msg));
            }
            CryptoCommand::AuthenticatedDecrypt(wallet_handle, my_vk, encrypted_msg, cb) => {
                debug!("AuthenticatedDecrypt command received");
                cb(self.authenticated_decrypt(wallet_handle, &my_vk, &encrypted_msg));
            }
            CryptoCommand::AnonymousEncrypt(their_vk, msg, cb) => {
                debug!("AnonymousEncrypt command received");
                cb(self.anonymous_encrypt(&their_vk, &msg));
            }
            CryptoCommand::AnonymousDecrypt(wallet_handle, my_vk, encrypted_msg, cb) => {
                debug!("AnonymousDecrypt command received");
                cb(self.anonymous_decrypt(wallet_handle, &my_vk, &encrypted_msg));
            }
            CryptoCommand::PackMessage(message, receivers, sender_vk, wallet_handle, cb) => {
                debug!("PackMessage command received");
                cb(self.pack_msg(message, receivers, sender_vk, wallet_handle));
            }
            CryptoCommand::UnpackMessage(jwe_json, wallet_handle, cb) => {
                debug!("UnpackMessage command received");
                cb(self.unpack_msg(jwe_json, wallet_handle));
            }
        };
    }

    fn create_key(&self, wallet_handle: WalletHandle, key_info: &KeyInfo) -> IndyResult<String> {
        debug!(
            "create_key >>> wallet_handle: {:?}, key_info: {:?}",
            wallet_handle,
            secret!(key_info)
        );

        let key = create_key(key_info)?;
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

        validate_key(my_vk)?;

        let key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            &my_vk,
            &RecordOptions::id_value(),
        )?;

        let res = sign(&key, msg)?;

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

        validate_key(their_vk)?;

        let res = verify(their_vk, msg, signature)?;

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

        validate_key(my_vk)?;
        validate_key(their_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            my_vk,
            &RecordOptions::id_value(),
        )?;

        let msg = create_combo_box(&my_key, &their_vk, msg)?;

        let msg = msg.to_msg_pack()
            .map_err(|e| err_msg(IndyErrorKind::InvalidState, format!("Can't serialize ComboBox: {:?}", e)))?;

        let res = crypto_box_seal(&their_vk, &msg)?;

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

        validate_key(my_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            my_vk,
            &RecordOptions::id_value(),
        )?;

        let decrypted_msg = crypto_box_seal_open(&my_key, &msg)?;

        let parsed_msg = ComboBox::from_msg_pack(decrypted_msg.as_slice())
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't deserialize ComboBox: {:?}", err)))?;

        let doc: Vec<u8> = base64::decode(&parsed_msg.msg)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't decode internal msg filed from base64 {}", err)))?;

        let nonce: Vec<u8> = base64::decode(&parsed_msg.nonce)
            .map_err(|err| err_msg(IndyErrorKind::InvalidStructure, format!("Can't decode nonce from base64 {}", err)))?;

        let decrypted_msg = crypto_box_open(&my_key, &parsed_msg.sender, &doc, &nonce)?;

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

        validate_key(their_vk)?;

        let res = crypto_box_seal(their_vk, &msg)?;

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

        validate_key(&my_vk)?;

        let my_key: Key = self.wallet_service.get_indy_object(
            wallet_handle,
            &my_vk,
            &RecordOptions::id_value(),
        )?;

        let res = crypto_box_seal_open(&my_key, &encrypted_msg)?;

        trace!("anonymous_decrypt <<< res: {:?}", res);

        Ok(res)
    }

    fn set_key_metadata(&self, wallet_handle: WalletHandle, verkey: &str, metadata: &str) -> IndyResult<()> {
        debug!(
            "set_key_metadata >>> wallet_handle: {:?}, verkey: {:?}, metadata: {:?}",
            wallet_handle, verkey, metadata
        );

        validate_key(verkey)?;

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

        validate_key(verkey)?;

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
        let sender_key = match sender_vk {
            Some(sender_key_) => {
                validate_key(&sender_key_)?;

                //get my_key from my wallet
                Some(self.wallet_service.get_indy_object::<Key>(
                    wallet_handle,
                    &sender_key_,
                    &RecordOptions::id_value(),
                )?)
            }
            None => None
        };

        pack_msg(message, sender_key, receiver_list)
    }

    pub fn unpack_msg(&self, jwe_struct: JWE, wallet_handle: WalletHandle) -> IndyResult<Vec<u8>> {
        let recipients = jwe_struct.get_recipients()?;

        //extract recipient that matches a key in the wallet
        let (recipient, my_key) = self._find_correct_recipient(recipients, wallet_handle)?;

        unpack_msg(jwe_struct, recipient, my_key)
    }

    fn _find_correct_recipient(&self, recipients: Vec<Recipient>, wallet_handle: WalletHandle) -> IndyResult<(Recipient, Key)> {
        for recipient in recipients.into_iter() {
            if let Ok(my_key) = self.wallet_service.get_indy_object::<Key>(
                wallet_handle,
                &recipient.header.kid,
                &RecordOptions::id_value(),
            ) {
                return Ok((recipient, my_key));
            }
        }
        Err(IndyError::from(IndyErrorKind::WalletItemNotFound))
    }
}
