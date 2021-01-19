
use indy_api_types::{ErrorCode, CommandHandle, WalletHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::crypto::CryptoCommand;
use crate::domain::crypto::pack::JWE;
use crate::domain::crypto::key::KeyInfo;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;

use serde_json;
use libc::c_char;
use crate::services::metrics::MetricsService;
use std::rc::Rc;
use crate::utils::time::get_cur_time;
use crate::services::metrics::command_metrics::CommandMetric;


/// Creates keys pair and stores in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// key_json: Key information as json. Example:
/// {
///     "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
///                                Can be UTF-8, base64 or hex string.
///     "crypto_type": string, // Optional (if not set then ed25519 curve is used); Currently only 'ed25519' value is supported for this field.
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
/// - verkey: Ver key of generated key pair, also used as key identifier
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_create_key(command_handle: CommandHandle,
                              wallet_handle: WalletHandle,
                              key_json: *const c_char,
                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                   err: ErrorCode,
                                                   verkey: *const c_char)>) -> ErrorCode {
    trace!("indy_create_key: >>> wallet_handle: {:?}, key_json: {:?}", wallet_handle, key_json);

    check_useful_json!(key_json, ErrorCode::CommonInvalidParam3, KeyInfo);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_create_key: entities >>> wallet_handle: {:?}, key_json: {:?}", wallet_handle, secret!(&key_json));

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::CreateKey(
            wallet_handle,
            key_json,
            boxed_callback_string!("indy_create_key", cb, command_handle, CommandMetric::CryptoCommandCreateKey)
        )));

    let res = prepare_result!(result);

    trace!("indy_create_key: <<< res: {:?}", res);

    res
}

/// Saves/replaces the meta information for the giving key in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// verkey - the key (verkey, key id) to store metadata.
/// metadata - the meta information that will be store with the key.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: command handle to map callback to caller context.
/// - err: Error code.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_set_key_metadata(command_handle: CommandHandle,
                                     wallet_handle: WalletHandle,
                                     verkey: *const c_char,
                                     metadata: *const c_char,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode)>) -> ErrorCode {
    trace!("indy_set_key_metadata: >>> wallet_handle: {:?}, verkey: {:?}, metadata: {:?}", wallet_handle, verkey, metadata);

    check_useful_c_str!(verkey, ErrorCode::CommonInvalidParam3);
    check_useful_c_str_empty_accepted!(metadata, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_set_key_metadata: entities >>> wallet_handle: {:?}, verkey: {:?}, metadata: {:?}", wallet_handle, verkey, metadata);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::SetKeyMetadata(
            wallet_handle,
            verkey,
            metadata,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_set_key_metadata: ");
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandSetKeyMetadata,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_set_key_metadata: <<< res: {:?}", res);

    res
}

/// Retrieves the meta information for the giving key in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// verkey - The key (verkey, key id) to retrieve metadata.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
/// - metadata - The meta information stored with the key; Can be null if no metadata was saved for this key.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_get_key_metadata(command_handle: CommandHandle,
                                     wallet_handle: WalletHandle,
                                     verkey: *const c_char,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode,
                                                          metadata: *const c_char)>) -> ErrorCode {
    trace!("indy_get_key_metadata: >>> wallet_handle: {:?}, verkey: {:?}", wallet_handle, verkey);

    check_useful_c_str!(verkey, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_get_key_metadata: entities >>> wallet_handle: {:?}, verkey: {:?}", wallet_handle, verkey);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::GetKeyMetadata(
            wallet_handle,
            verkey,
            boxed_callback_string!("indy_get_key_metadata", cb, command_handle, CommandMetric::CryptoCommandGetKeyMetadata)
        )));

    let res = prepare_result!(result);

    trace!("indy_get_key_metadata: <<< res: {:?}", res);

    res
}

/// Signs a message with a key.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handler (created by open_wallet).
/// signer_vk: id (verkey) of message signer. The key must be created by calling indy_create_key or indy_create_and_store_my_did
/// message_raw: a pointer to first byte of message to be signed
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// a signature string
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_sign(command_handle: CommandHandle,
                                wallet_handle: WalletHandle,
                                signer_vk: *const c_char,
                                message_raw: *const u8,
                                message_len: u32,
                                cb: Option<extern fn(command_handle_: CommandHandle,
                                                     err: ErrorCode,
                                                     signature_raw: *const u8,
                                                     signature_len: u32)>) -> ErrorCode {
    trace!("indy_crypto_sign: >>> wallet_handle: {:?}, signer_vk: {:?}, message_raw: {:?}, message_len: {:?}",
           wallet_handle, signer_vk, message_raw, message_len);

    check_useful_c_str!(signer_vk, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(message_raw, message_len, ErrorCode::CommonInvalidParam4, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_crypto_sign: entities >>> wallet_handle: {:?}, signer_vk: {:?}, message_raw: {:?}, message_len: {:?}",
           wallet_handle, signer_vk, message_raw, message_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::CryptoSign(
            wallet_handle,
            signer_vk,
            message_raw,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, signature) = prepare_result_1!(result, Vec::new());
                trace!("indy_crypto_sign: signature: {:?}", signature);
                let (signature_raw, signature_len) = ctypes::vec_to_pointer(&signature);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, signature_raw, signature_len);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandCryptoSign,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_sign: <<< res: {:?}", res);

    res
}

/// Verify a signature with a verkey.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// signer_vk: verkey of the message signer
/// message_raw: a pointer to first byte of message that has been signed
/// message_len: a message length
/// signature_raw: a pointer to first byte of signature to be verified
/// signature_len: a signature length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// valid: true - if signature is valid, false - otherwise
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_verify(command_handle: CommandHandle,
                                  signer_vk: *const c_char,
                                  message_raw: *const u8,
                                  message_len: u32,
                                  signature_raw: *const u8,
                                  signature_len: u32,
                                  cb: Option<extern fn(command_handle_: CommandHandle,
                                                       err: ErrorCode,
                                                       valid: bool)>) -> ErrorCode {
    trace!("indy_crypto_verify: >>> signer_vk: {:?}, message_raw: {:?}, message_len: {:?}, signature_raw: {:?}, signature_len: {:?}",
           signer_vk, message_raw, message_len, signature_raw, signature_len);

    check_useful_c_str!(signer_vk, ErrorCode::CommonInvalidParam2);
    check_useful_c_byte_array!(message_raw, message_len, ErrorCode::CommonInvalidParam3, ErrorCode::CommonInvalidParam4);
    check_useful_c_byte_array!(signature_raw, signature_len, ErrorCode::CommonInvalidParam5, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_crypto_verify: entities >>> signer_vk: {:?}, message_raw: {:?}, message_len: {:?}, signature_raw: {:?}, signature_len: {:?}",
           signer_vk, message_raw, message_len, signature_raw, signature_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::CryptoVerify(
            signer_vk,
            message_raw,
            signature_raw,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, valid) = prepare_result_1!(result, false);
                trace!("indy_crypto_verify: valid: {:?}", valid);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, valid);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandCryptoVerify,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_verify: <<< res: {:?}", res);

    res
}

/// **** THIS FUNCTION WILL BE DEPRECATED USE indy_pack_message() INSTEAD ****
/// Encrypt a message by authenticated-encryption scheme.
///
/// Sender can encrypt a confidential message specifically for Recipient, using Sender's public key.
/// Using Recipient's public key, Sender can compute a shared secret key.
/// Using Sender's public key and his secret key, Recipient can compute the exact same shared secret key.
/// That shared secret key can be used to verify that the encrypted message was not tampered with,
/// before eventually decrypting it.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// sender_vk: id (verkey) of message sender. The key must be created by calling indy_create_key or indy_create_and_store_my_did
/// recipient_vk: id (verkey) of message recipient
/// message_raw: a pointer to first byte of message that to be encrypted
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// an encrypted message as a pointer to array of bytes.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_auth_crypt(command_handle: CommandHandle,
                                      wallet_handle: WalletHandle,
                                      sender_vk: *const c_char,
                                      recipient_vk: *const c_char,
                                      msg_data: *const u8,
                                      msg_len: u32,
                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                           err: ErrorCode,
                                                           encrypted_msg: *const u8,
                                                           encrypted_len: u32)>) -> ErrorCode {
    trace!("indy_crypto_auth_crypt: >>> wallet_handle: {:?}, sender_vk: {:?}, recipient_vk: {:?}, msg_data: {:?}, msg_len: {:?}",
           wallet_handle, sender_vk, recipient_vk, msg_data, msg_len);

    check_useful_c_str!(sender_vk, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(recipient_vk, ErrorCode::CommonInvalidParam4);
    check_useful_c_byte_array!(msg_data, msg_len, ErrorCode::CommonInvalidParam5, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_crypto_auth_crypt: entities >>> wallet_handle: {:?}, sender_vk: {:?}, recipient_vk: {:?}, msg_data: {:?}, msg_len: {:?}",
           wallet_handle, sender_vk, recipient_vk, msg_data, msg_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::AuthenticatedEncrypt(
            wallet_handle,
            sender_vk,
            recipient_vk,
            msg_data,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, encrypted_msg) = prepare_result_1!(result, Vec::new());
                trace!("indy_crypto_auth_crypt: encrypted_msg: {:?}", encrypted_msg);
                let (encrypted_msg_raw, encrypted_msg_len) = ctypes::vec_to_pointer(&encrypted_msg);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, encrypted_msg_raw, encrypted_msg_len);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandAuthenticatedEncrypt,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_auth_crypt: <<< res: {:?}", res);

    res
}

/// **** THIS FUNCTION WILL BE DEPRECATED USE indy_unpack_message() INSTEAD ****
/// Decrypt a message by authenticated-encryption scheme.
///
/// Sender can encrypt a confidential message specifically for Recipient, using Sender's public key.
/// Using Recipient's public key, Sender can compute a shared secret key.
/// Using Sender's public key and his secret key, Recipient can compute the exact same shared secret key.
/// That shared secret key can be used to verify that the encrypted message was not tampered with,
/// before eventually decrypting it.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handler (created by open_wallet).
/// recipient_vk: id (verkey) of message recipient. The key must be created by calling indy_create_key or indy_create_and_store_my_did
/// encrypted_msg_raw: a pointer to first byte of message that to be decrypted
/// encrypted_msg_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// sender verkey and decrypted message as a pointer to array of bytes
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_auth_decrypt(command_handle: CommandHandle,
                                        wallet_handle: WalletHandle,
                                        recipient_vk: *const c_char,
                                        encrypted_msg: *const u8,
                                        encrypted_len: u32,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             sender_vk: *const c_char,
                                                             msg_data: *const u8,
                                                             msg_len: u32)>) -> ErrorCode {
    trace!("indy_crypto_auth_decrypt: >>> wallet_handle: {:?}, recipient_vk: {:?}, encrypted_msg: {:?}, encrypted_len: {:?}",
           wallet_handle, recipient_vk, encrypted_msg, encrypted_len);

    check_useful_c_str!(recipient_vk, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(encrypted_msg, encrypted_len, ErrorCode::CommonInvalidParam4, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_crypto_auth_decrypt: entities >>> wallet_handle: {:?}, recipient_vk: {:?}, encrypted_msg: {:?}, encrypted_len: {:?}",
           wallet_handle, recipient_vk, encrypted_msg, encrypted_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::AuthenticatedDecrypt(
            wallet_handle,
            recipient_vk,
            encrypted_msg,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, sender_vk, msg) = prepare_result_2!(result, String::new(), Vec::new());
                trace!("indy_crypto_auth_decrypt: sender_vk: {:?}, msg: {:?}", sender_vk, msg);
                let (msg_data, msg_len) = ctypes::vec_to_pointer(&msg);
                let sender_vk = ctypes::string_to_cstring(sender_vk);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, sender_vk.as_ptr(), msg_data, msg_len);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandAuthenticatedDecrypt,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_auth_decrypt: <<< res: {:?}", res);

    res
}

/// Encrypts a message by anonymous-encryption scheme.
///
/// Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
/// Only the Recipient can decrypt these messages, using its private key.
/// While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// Note: use indy_pack_message() function for A2A goals.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// recipient_vk: verkey of message recipient
/// message_raw: a pointer to first byte of message that to be encrypted
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// an encrypted message as a pointer to array of bytes
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_anon_crypt(command_handle: CommandHandle,
                                      recipient_vk: *const c_char,
                                      msg_data: *const u8,
                                      msg_len: u32,
                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                           err: ErrorCode,
                                                           encrypted_msg: *const u8,
                                                           encrypted_len: u32)>) -> ErrorCode {
    trace!("indy_crypto_anon_crypt: >>> recipient_vk: {:?}, msg_data: {:?}, msg_len: {:?}", recipient_vk, msg_data, msg_len);

    check_useful_c_str!(recipient_vk, ErrorCode::CommonInvalidParam2);
    check_useful_c_byte_array!(msg_data, msg_len, ErrorCode::CommonInvalidParam3, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_crypto_anon_crypt: entities >>> recipient_vk: {:?}, msg_data: {:?}, msg_len: {:?}", recipient_vk, msg_data, msg_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::AnonymousEncrypt(
            recipient_vk,
            msg_data,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, encrypted_msg) = prepare_result_1!(result, Vec::new());
                trace!("indy_crypto_anon_crypt: encrypted_msg: {:?}", encrypted_msg);
                let (encrypted_msg_raw, encrypted_msg_len) = ctypes::vec_to_pointer(&encrypted_msg);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, encrypted_msg_raw, encrypted_msg_len);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandAnonymousEncrypt,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_anon_crypt: <<< res: {:?}", res);

    res
}

/// Decrypts a message by anonymous-encryption scheme.
///
/// Sealed boxes are designed to anonymously send messages to a Recipient given its public key.
/// Only the Recipient can decrypt these messages, using its private key.
/// While the Recipient can verify the integrity of the message, it cannot verify the identity of the Sender.
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// Note: use indy_unpack_message() function for A2A goals.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handler (created by open_wallet).
/// recipient_vk: id (verkey) of my key. The key must be created by calling indy_create_key or indy_create_and_store_my_did
/// encrypted_msg_raw: a pointer to first byte of message that to be decrypted
/// encrypted_msg_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// decrypted message as a pointer to an array of bytes
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_crypto_anon_decrypt(command_handle: CommandHandle,
                                        wallet_handle: WalletHandle,
                                        recipient_vk: *const c_char,
                                        encrypted_msg: *const u8,
                                        encrypted_len: u32,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             msg_data: *const u8,
                                                             msg_len: u32)>) -> ErrorCode {
    trace!("indy_crypto_anon_decrypt: >>> wallet_handle: {:?}, recipient_vk: {:?}, encrypted_msg: {:?}, encrypted_len: {:?}",
           wallet_handle, recipient_vk, encrypted_msg, encrypted_len);

    check_useful_c_str!(recipient_vk, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(encrypted_msg, encrypted_len, ErrorCode::CommonInvalidParam4, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_crypto_anon_decrypt: entities >>> wallet_handle: {:?}, recipient_vk: {:?}, encrypted_msg: {:?}, encrypted_len: {:?}",
           wallet_handle, recipient_vk, encrypted_msg, encrypted_len);

    let result = CommandExecutor::instance()
        .send(Command::Crypto(CryptoCommand::AnonymousDecrypt(
            wallet_handle,
            recipient_vk,
            encrypted_msg,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, msg) = prepare_result_1!(result, Vec::new());
                trace!("indy_crypto_anon_decrypt: msg: {:?}", msg);
                let (msg_data, msg_len) = ctypes::vec_to_pointer(&msg);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, msg_data, msg_len);
                metrics_service.cmd_callback(CommandMetric::CryptoCommandAnonymousDecrypt,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_crypto_anon_decrypt: <<< res: {:?}", res);

    res
}

/// Packs a message by encrypting the message and serializes it in a JWE-like format (Experimental)
///
/// Note to use DID keys with this function you can call indy_key_for_did to get key id (verkey)
/// for specific DID.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// message: a pointer to the first byte of the message to be packed
/// message_len: the length of the message
/// receivers: a string in the format of a json list which will contain the list of receiver's keys
///                the message is being encrypted for.
///                Example:
///                "[<receiver edge_agent_1 verkey>, <receiver edge_agent_2 verkey>]"
/// sender: the sender's verkey as a string When null pointer is used in this parameter, anoncrypt is used
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// a JWE using authcrypt alg is defined below:
/// {
///     "protected": "b64URLencoded({
///        "enc": "xsalsa20poly1305",
///        "typ": "JWM/1.0",
///        "alg": "Authcrypt",
///        "recipients": [
///            {
///                "encrypted_key": base64URLencode(libsodium.crypto_box(my_key, their_vk, cek, cek_iv))
///                "header": {
///                     "kid": "base58encode(recipient_verkey)",
///                     "sender" : base64URLencode(libsodium.crypto_box_seal(their_vk, base58encode(sender_vk)),
///                     "iv" : base64URLencode(cek_iv)
///                }
///            },
///        ],
///     })",
///     "iv": <b64URLencode(iv)>,
///     "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv, cek),
///     "tag": <b64URLencode(tag)>
/// }
///
/// Alternative example in using anoncrypt alg is defined below:
/// {
///     "protected": "b64URLencoded({
///        "enc": "xsalsa20poly1305",
///        "typ": "JWM/1.0",
///        "alg": "Anoncrypt",
///        "recipients": [
///            {
///                "encrypted_key": base64URLencode(libsodium.crypto_box_seal(their_vk, cek)),
///                "header": {
///                    "kid": base58encode(recipient_verkey),
///                }
///            },
///        ],
///     })",
///     "iv": b64URLencode(iv),
///     "ciphertext": b64URLencode(encrypt_detached({'@type'...}, protected_value_encoded, iv, cek),
///     "tag": b64URLencode(tag)
/// }
///
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_pack_message(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    message: *const u8,
    message_len: u32,
    receiver_keys: *const c_char,
    sender: *const c_char,
    cb: Option<extern fn(xcommand_handle: CommandHandle, err: ErrorCode, jwe_data: *const u8, jwe_len: u32)>,
) -> ErrorCode {
    trace!("indy_pack_message: >>> wallet_handle: {:?}, message: {:?}, message_len {:?},\
            receiver_keys: {:?}, sender: {:?}", wallet_handle, message, message_len, receiver_keys, sender);

    check_useful_c_byte_array!(message, message_len, ErrorCode::CommonInvalidParam2, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(receiver_keys, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(sender, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_pack_message: entities >>> wallet_handle: {:?}, message: {:?}, message_len {:?},\
            receiver_keys: {:?}, sender: {:?}", wallet_handle, message, message_len, receiver_keys, sender);

    //parse json array of keys
    let receiver_list = match serde_json::from_str::<Vec<String>>(&receiver_keys) {
        Ok(x) => x,
        Err(_) => {
            return IndyError::from_msg(IndyErrorKind::InvalidParam(4), "Invalid RecipientKeys has been passed").into();
        },
    };

    //break early and error out if no receivers keys are provided
    if receiver_list.is_empty() {
        return IndyError::from_msg(IndyErrorKind::InvalidParam(4), "Empty RecipientKeys has been passed").into();
    }

    let result = CommandExecutor::instance().send(Command::Crypto(CryptoCommand::PackMessage(
        message,
        receiver_list,
        sender,
        wallet_handle,
        Box::new(move |result, metrics_service: Rc<MetricsService>| {
            let (err, jwe) = prepare_result_1!(result, Vec::new());
            trace!("indy_auth_pack_message: jwe: {:?}", jwe);
            let (jwe_data, jwe_len) = ctypes::vec_to_pointer(&jwe);
            let start_execution_ts = get_cur_time();
            let result = cb(command_handle, err, jwe_data, jwe_len);
            metrics_service.cmd_callback(CommandMetric::CryptoCommandPackMessage,get_cur_time() - start_execution_ts);

            result
        }),
    )));

    let res = prepare_result!(result);

    trace!("indy_auth_pack_message: <<< res: {:?}", res);

    res
}


/// Unpacks a JWE-like formatted message outputted by indy_pack_message (Experimental)
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// jwe_data: a pointer to the first byte of the JWE to be unpacked
/// jwe_len: the length of the JWE message in bytes
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// if authcrypt was used to pack the message returns this json structure:
/// {
///     message: <decrypted message>,
///     sender_verkey: <sender_verkey>,
///     recipient_verkey: <recipient_verkey>
/// }
///
/// OR
///
/// if anoncrypt was used to pack the message returns this json structure:
/// {
///     message: <decrypted message>,
///     recipient_verkey: <recipient_verkey>
/// }
///
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_unpack_message(
    command_handle: CommandHandle,
    wallet_handle: WalletHandle,
    jwe_data: *const u8,
    jwe_len: u32,
    cb: Option<
        extern fn(
            xcommand_handle: CommandHandle,
            err: ErrorCode,
            res_json_data : *const u8,
            res_json_len : u32
        ),
    >,
) -> ErrorCode {
    trace!(
        "indy_unpack_message: >>> wallet_handle: {:?}, jwe_data: {:?}, jwe_len {:?}",
        wallet_handle,
        jwe_data,
        jwe_len
    );

    check_useful_c_byte_array!(jwe_data, jwe_len, ErrorCode::CommonInvalidParam2, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!(
        "indy_unpack_message: entities >>> wallet_handle: {:?}, jwe_data: {:?}, jwe_len {:?}",
        wallet_handle,
        jwe_data,
        jwe_len
    );

    //serialize JWE to struct
    let jwe_struct: JWE = match serde_json::from_slice(jwe_data.as_slice()) {
        Ok(x) => x,
        Err(_) => return ErrorCode::CommonInvalidParam3
    };

    let result = CommandExecutor::instance().send(Command::Crypto(CryptoCommand::UnpackMessage(
        jwe_struct,
        wallet_handle,
        Box::new(move |result, metrics_service: Rc<MetricsService>| {
            let (err, res_json) = prepare_result_1!(result, Vec::new());
            trace!("indy_unpack_message: cb command_handle: {:?}, err: {:?}, res_json: {:?}",
                command_handle, err, res_json
            );
            let (res_json_data, res_json_len) = ctypes::vec_to_pointer(&res_json);
            let start_execution_ts = get_cur_time();
            let result = cb(command_handle, err, res_json_data, res_json_len);
            metrics_service.cmd_callback(CommandMetric::CryptoCommandUnpackMessage,get_cur_time() - start_execution_ts);

            result
        }),
    )));

    let res = prepare_result!(result);

    trace!("indy_unpack_message: <<< res: {:?}", res);

    res
}
