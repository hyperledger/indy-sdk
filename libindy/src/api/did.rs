use indy_api_types::{ErrorCode, CommandHandle, WalletHandle, PoolHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::did::DidCommand;
use crate::domain::crypto::did::{TheirDidInfo, DidValue, MyDidInfo, DidMethod};
use crate::domain::crypto::key::KeyInfo;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;
use indy_api_types::validation::Validatable;
use indy_vdr::utils::validation::Validatable as VdrValidatable;

use serde_json;
use libc::c_char;

use std::ptr;
use crate::services::ledger::parsers::attrib::Endpoint;


/// Creates keys (signing and encryption keys) for a new
/// DID (owned by the caller of the library).
/// Identity's DID must be either explicitly provided, or taken as the first 16 bit of verkey.
/// Saves the Identity DID with keys in a secured Wallet, so that it can be used to sign
/// and encrypt transactions.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// did_info: Identity information as json. See domain::crypto::did::MyDidInfo
/// Example:
/// {
///     "did": string, (optional;
///             if not provided and cid param is false then the first 16 bit of the verkey will be used as a new DID;
///             if not provided and cid is true then the full verkey will be used as a new DID;
///             if provided, then keys will be replaced - key rotation use case)
///     "seed": string, (optional) Seed that allows deterministic did creation (if not set random one will be created).
///                                Can be UTF-8, base64 or hex string.
///     "crypto_type": string, (optional; if not set then ed25519 curve is used;
///               currently only 'ed25519' value is supported for this field)
///     "cid": bool, (optional; if not set then false is used;)
///     "method_name": string, (optional) method name to create fully qualified did.
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///   did: DID generated and stored in the wallet
///   verkey: The DIDs verification key
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_create_and_store_my_did(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            did_info: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 did: *const c_char,
                                                                 verkey: *const c_char)>) -> ErrorCode {
    trace!("indy_create_and_store_my_did: >>> wallet_handle: {:?}, did_json: {:?}", wallet_handle, did_info);

    check_useful_validatable_json!(did_info, ErrorCode::CommonInvalidParam3, MyDidInfo); // redefine to MyDidInfo if valid
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_create_and_store_my_did: entities >>> wallet_handle: {:?}, did_json: {:?}", wallet_handle, secret!(&did_info));

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::CreateAndStoreMyDid(
            wallet_handle,
            did_info,
            Box::new(move |result| {
                let (err, did, verkey) = prepare_result_2!(result, String::new(), String::new());
                trace!("indy_create_and_store_my_did: did: {:?}, verkey: {:?}", did, verkey);
                let did = ctypes::string_to_cstring(did);
                let verkey = ctypes::string_to_cstring(verkey);
                cb(command_handle, err, did.as_ptr(), verkey.as_ptr())
            }),
        )));

    let res = prepare_result!(result);

    trace!("indy_create_and_store_my_did: <<< res: {:?}", res);

    res
}

/// Generated temporary keys (signing and encryption keys) for an existing
/// DID (owned by the caller of the library).
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// did: target did to rotate keys.
/// key_info: key information as json. Example:
/// {
///     "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
///                                Can be UTF-8, base64 or hex string.
///     "crypto_type": string, (optional; if not set then ed25519 curve is used;
///               currently only 'ed25519' value is supported for this field)
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///   verkey: The DIDs verification key
///
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_replace_keys_start(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       did: *const c_char,
                                       key_info: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode,
                                                            verkey: *const c_char)>) -> ErrorCode {
    trace!("indy_replace_keys_start: >>> wallet_handle: {:?}, did: {:?}, identity_json: {:?}", wallet_handle, did, key_info);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_json!(key_info, ErrorCode::CommonInvalidParam4, KeyInfo);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_replace_keys_start: entities>>> wallet_handle: {:?}, did: {:?}, key_info: {:?}", wallet_handle, did, secret!(&key_info));

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::ReplaceKeysStart(
            wallet_handle,
            key_info,
            did,
            boxed_callback_string!("indy_replace_keys_start", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_replace_keys_start: <<< res: {:?}", res);

    res
}

/// Apply temporary keys as main for an existing DID (owned by the caller of the library).
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// did: DID stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_replace_keys_apply(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       did: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode)>) -> ErrorCode {
    trace!("indy_replace_keys_apply: >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_replace_keys_apply: entities >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::ReplaceKeysApply(
            wallet_handle,
            did,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_replace_keys_apply:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_replace_keys_apply: <<< res: {:?}", res);

    res
}

/// Saves their DID for a pairwise connection in a secured Wallet,
/// so that it can be used to verify transaction.
/// Updates DID associated verkey in case DID already exists in the Wallet.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// identity_json: Identity information as json. Example:
///     {
///        "did": string, (required)
///        "verkey": string
///             - optional is case of adding a new DID, and DID is cryptonym: did == verkey,
///             - mandatory in case of updating an existing DID
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_store_their_did(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    identity_json: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_store_their_did: >>> wallet_handle: {:?}, identity_json: {:?}", wallet_handle, identity_json);

    check_useful_validatable_json!(identity_json, ErrorCode::CommonInvalidParam3, TheirDidInfo);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_store_their_did: entities >>> wallet_handle: {:?}, identity_json: {:?}", wallet_handle, identity_json);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::StoreTheirDid(
            wallet_handle,
            identity_json,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_store_their_did:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_store_their_did: <<< res: {:?}", res);

    res
}

/// Returns ver key (key id) for the given DID.
///
/// "indy_key_for_did" call follow the idea that we resolve information about their DID from
/// the ledger with cache in the local wallet. The "indy_open_wallet" call has freshness parameter
/// that is used for checking the freshness of cached pool value.
///
/// Note if you don't want to resolve their DID info from the ledger you can use
/// "indy_key_for_local_did" call instead that will look only to the local wallet and skip
/// freshness checking.
///
/// Note that "indy_create_and_store_my_did" makes similar wallet record as "indy_create_key".
/// As result we can use returned ver key in all generic crypto and messaging functions.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// pool_handle:   Pool handle (created by open_pool).
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to resolve key.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
/// - key - The DIDs ver key (key id).
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_key_for_did(command_handle: CommandHandle,
                               pool_handle: PoolHandle,
                               wallet_handle: WalletHandle,
                               did: *const c_char,
                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                    err: ErrorCode,
                                                    key: *const c_char)>) -> ErrorCode {
    trace!("indy_key_for_did: >>> pool_handle: {:?}, wallet_handle: {:?}, did: {:?}", pool_handle, wallet_handle, did);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam4, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_key_for_did: entities >>> pool_handle: {:?}, wallet_handle: {:?}, did: {:?}", pool_handle, wallet_handle, did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::KeyForDid(
            pool_handle,
            wallet_handle,
            did,
            boxed_callback_string!("indy_key_for_did", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_key_for_did: <<< res: {:?}", res);

    res
}

/// Returns ver key (key id) for the given DID.
///
/// "indy_key_for_local_did" call looks data stored in the local wallet only and skips freshness
/// checking.
///
/// Note if you want to get fresh data from the ledger you can use "indy_key_for_did" call
/// instead.
///
/// Note that "indy_create_and_store_my_did" makes similar wallet record as "indy_create_key".
/// As result we can use returned ver key in all generic crypto and messaging functions.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to resolve key.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
/// - key - The DIDs ver key (key id).
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_key_for_local_did(command_handle: CommandHandle,
                                     wallet_handle: WalletHandle,
                                     did: *const c_char,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode,
                                                          key: *const c_char)>) -> ErrorCode {
    trace!("indy_key_for_local_did: >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_key_for_local_did: entities >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::KeyForLocalDid(
            wallet_handle,
            did,
            boxed_callback_string!("indy_key_for_local_did", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_key_for_local_did: <<< res: {:?}", res);

    res
}

/// Set/replaces endpoint information for the given DID.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to resolve endpoint.
/// address -  The DIDs endpoint address. indy-node and indy-plenum restrict this to ip_address:port
/// transport_key - The DIDs transport key (ver key, key id).
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_set_endpoint_for_did(command_handle: CommandHandle,
                                        wallet_handle: WalletHandle,
                                        did: *const c_char,
                                        address: *const c_char,
                                        transport_key: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode)>) -> ErrorCode {
    trace!("indy_set_endpoint_for_did: >>> wallet_handle: {:?}, did: {:?}, address: {:?}, transport_key: {:?}", wallet_handle, did, address, transport_key);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(address, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(transport_key, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_set_endpoint_for_did: entities >>> wallet_handle: {:?}, did: {:?}, address: {:?}, transport_key: {:?}",
           wallet_handle, did, address, transport_key);

    let endpoint = Endpoint::new(address, Some(transport_key));

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::SetEndpointForDid(
            wallet_handle,
            did,
            endpoint,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_set_endpoint_for_did:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_set_endpoint_for_did: <<< res: {:?}", res);

    res
}

/// Returns endpoint information for the given DID.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to resolve endpoint.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
/// - endpoint - The DIDs endpoint.
/// - transport_vk - The DIDs transport key (ver key, key id).
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_get_endpoint_for_did(command_handle: CommandHandle,
                                        wallet_handle: WalletHandle,
                                        pool_handle: PoolHandle,
                                        did: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             address: *const c_char,
                                                             transport_vk: *const c_char)>) -> ErrorCode {
    trace!("indy_get_endpoint_for_did: >>> wallet_handle: {:?}, pool_handle: {:?}, did: {:?}", wallet_handle, pool_handle, did);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_get_endpoint_for_did: entities >>> wallet_handle: {:?}, pool_handle: {:?}, did: {:?}", wallet_handle, pool_handle, did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::GetEndpointForDid(
            wallet_handle,
            pool_handle,
            did,
            Box::new(move |result| {
                let (err, address, transport_vk) = prepare_result_2!(result, String::new(), None);
                trace!("indy_get_endpoint_for_did: address: {:?}, transport_vk: {:?}", address, transport_vk);
                let address = ctypes::string_to_cstring(address);
                let transport_vk = transport_vk.map(ctypes::string_to_cstring);
                cb(command_handle, err, address.as_ptr(),
                   transport_vk.as_ref().map(|vk| vk.as_ptr()).unwrap_or(ptr::null()));
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_get_endpoint_for_did: <<< res: {:?}", res);

    res
}

/// Saves/replaces the meta information for the giving DID in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - the DID to store metadata.
/// metadata - the meta information that will be store with the DID.
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
pub extern fn indy_set_did_metadata(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    did: *const c_char,
                                    metadata: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_set_did_metadata: >>> wallet_handle: {:?}, did: {:?}, metadata: {:?}", wallet_handle, did, metadata);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str_empty_accepted!(metadata, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_set_did_metadata: entities >>> wallet_handle: {:?}, did: {:?}, metadata: {:?}", wallet_handle, did, metadata);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::SetDidMetadata(
            wallet_handle,
            did,
            metadata,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_set_did_metadata:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_set_did_metadata: <<< res: {:?}", res);

    res
}

/// Retrieves the meta information for the giving DID in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to retrieve metadata.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
/// - metadata - The meta information stored with the DID; Can be null if no metadata was saved for this DID.
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_get_did_metadata(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    did: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode,
                                                         metadata: *const c_char)>) -> ErrorCode {
    trace!("indy_get_did_metadata: >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_get_did_metadata: entities >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::GetDidMetadata(
            wallet_handle,
            did,
            boxed_callback_string!("indy_get_did_metadata", cb, command_handle))));

    let res = prepare_result!(result);

    trace!("indy_get_did_metadata: <<< res: {:?}", res);

    res
}

/// Retrieves the information about the giving DID in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did - The DID to retrieve information.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///   did_with_meta:  {
///     "did": string - DID stored in the wallet,
///     "verkey": string - The DIDs transport key (ver key, key id),
///     "tempVerkey": string - Temporary DIDs transport key (ver key, key id), exist only during the rotation of the keys.
///                            After rotation is done, it becomes a new verkey.
///     "metadata": string - The meta information stored with the DID
///   }
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_get_my_did_with_meta(command_handle: CommandHandle,
                                        wallet_handle: WalletHandle,
                                        my_did: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             did_with_meta: *const c_char)>) -> ErrorCode {
    trace!("indy_get_my_did_with_meta: >>> wallet_handle: {:?}, my_did: {:?}", wallet_handle, my_did);

    check_useful_validatable_string!(my_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_get_my_did_with_meta: entities >>> wallet_handle: {:?}, my_did: {:?}", wallet_handle, my_did);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::GetMyDidWithMeta(
            wallet_handle,
            my_did,
            boxed_callback_string!("indy_get_my_did_with_meta", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_get_my_did_with_meta: <<< res: {:?}", res);

    res
}

/// Retrieves the information about all DIDs stored in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///   dids:  [{
///     "did": string - DID stored in the wallet,
///     "verkey": string - The DIDs transport key (ver key, key id).,
///     "metadata": string - The meta information stored with the DID
///   }]
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_list_my_dids_with_meta(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               dids: *const c_char)>) -> ErrorCode {
    trace!("indy_list_my_dids_with_meta: >>> wallet_handle: {:?}", wallet_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_list_my_dids_with_meta: entities >>> wallet_handle: {:?}", wallet_handle);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::ListMyDidsWithMeta(
            wallet_handle,
            boxed_callback_string!("indy_list_my_dids_with_meta", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_list_my_dids_with_meta: <<< res: {:?}", res);

    res
}

/// Retrieves abbreviated verkey if it is possible otherwise return full verkey.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// did: DID.
/// full_verkey: The DIDs verification key,
///
/// #Returns
/// Error Code
/// cb:
/// - command_handle_: Command handle to map callback to caller context.
/// - err: Error code.
///   verkey: The DIDs verification key in either abbreviated or full form
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub  extern fn indy_abbreviate_verkey(command_handle: CommandHandle,
                                      did: *const c_char,
                                      full_verkey: *const c_char,
                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                           err: ErrorCode,
                                                           verkey: *const c_char)>) -> ErrorCode {
    trace!("indy_abbreviate_verkey: >>> did: {:?}, full_verkey: {:?}", did, full_verkey);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(full_verkey, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_abbreviate_verkey: entities >>> did: {:?}, full_verkey: {:?}", did, full_verkey);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::AbbreviateVerkey(
            did,
            full_verkey,
            boxed_callback_string!("indy_abbreviate_verkey", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_abbreviate_verkey: <<< res: {:?}", res);

    res
}

/// Update DID stored in the wallet to make fully qualified, or to do other DID maintenance.
///     - If the DID has no method, a method will be appended (prepend did:peer to a legacy did)
///     - If the DID has a method, a method will be updated (migrate did:peer to did:peer-new)
///
/// Update DID related entities stored in the wallet.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: Wallet handle (created by open_wallet).
/// did: target DID stored in the wallet.
/// method: method to apply to the DID.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error Code
/// cb:
/// - did: fully qualified form of did
///
/// #Errors
/// Common*
/// Wallet*
/// Crypto*
#[no_mangle]
pub extern fn indy_qualify_did(command_handle: CommandHandle,
                               wallet_handle: WalletHandle,
                               did: *const c_char,
                               method: *const c_char,
                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 full_qualified_did: *const c_char)>) -> ErrorCode {
    trace!("indy_qualify_did: >>> wallet_handle: {:?}, did: {:?}, method: {:?}", wallet_handle, did, method);

    check_useful_validatable_string!(did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_string!(method, ErrorCode::CommonInvalidParam4, DidMethod);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_qualify_did: entities >>> wallet_handle: {:?}, did: {:?}, method: {:?}", wallet_handle, did, method);

    let result = CommandExecutor::instance()
        .send(Command::Did(DidCommand::QualifyDid(
            wallet_handle,
            did,
            method,
            boxed_callback_string!("indy_qualify_did", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_qualify_did: <<< res: {:?}", res);

    res
}