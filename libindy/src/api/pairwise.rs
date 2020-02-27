use indy_api_types::{ErrorCode, CommandHandle, WalletHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::pairwise::PairwiseCommand;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;
use indy_api_types::validation::Validatable;
use indy_api_types::domain::crypto::did::DidValue;

use libc::c_char;


/// Check if pairwise is exists.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encrypted DID
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// exists: true - if pairwise is exists, false - otherwise
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_is_pairwise_exists(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       their_did: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode, exists: bool)>) -> ErrorCode {
    trace!("indy_is_pairwise_exists: >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_is_pairwise_exists: entities >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

    let result = CommandExecutor::instance()
        .send(Command::Pairwise(PairwiseCommand::PairwiseExists(
            wallet_handle,
            their_did,
            Box::new(move |result| {
                let (err, exists) = prepare_result_1!(result, false);
                trace!("indy_is_pairwise_exists: exists: {:?}", exists);
                cb(command_handle, err, exists)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_is_pairwise_exists: <<< res: {:?}", res);

    res
}


/// Creates pairwise.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encrypted DID
/// my_did: encrypted DID
/// metadata Optional: extra information for pairwise
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_create_pairwise(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    their_did: *const c_char,
                                    my_did: *const c_char,
                                    metadata: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode)>) -> ErrorCode {
    trace!("indy_create_pairwise: >>> wallet_handle: {:?}, their_did: {:?}, my_did: {:?}, metadata: {:?}", wallet_handle, their_did, my_did, metadata);

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_string!(my_did, ErrorCode::CommonInvalidParam4, DidValue);
    check_useful_opt_c_str!(metadata, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_create_pairwise: entities >>> wallet_handle: {:?}, their_did: {:?}, my_did: {:?}, metadata: {:?}", wallet_handle, their_did, my_did, metadata);

    let result = CommandExecutor::instance()
        .send(Command::Pairwise(PairwiseCommand::CreatePairwise(
            wallet_handle,
            their_did,
            my_did,
            metadata,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_create_pairwise:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_create_pairwise: <<< res: {:?}", res);

    res
}

/// Get list of saved pairwise.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// list_pairwise: list of saved pairwise
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_list_pairwise(command_handle: CommandHandle,
                                  wallet_handle: WalletHandle,
                                  cb: Option<extern fn(command_handle_: CommandHandle,
                                                       err: ErrorCode,
                                                       list_pairwise: *const c_char)>) -> ErrorCode {
    trace!("indy_list_pairwise: >>> wallet_handle: {:?}", wallet_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_list_pairwise: entities >>> wallet_handle: {:?}", wallet_handle);

    let result = CommandExecutor::instance()
        .send(Command::Pairwise(PairwiseCommand::ListPairwise(
            wallet_handle,
            boxed_callback_string!("indy_list_pairwise", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_list_pairwise: <<< res: {:?}", res);

    res
}

/// Gets pairwise information for specific their_did.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encoded Did
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// pairwise_info_json: did info associated with their did
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_get_pairwise(command_handle: CommandHandle,
                                 wallet_handle: WalletHandle,
                                 their_did: *const c_char,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode,
                                                      pairwise_info_json: *const c_char)>) -> ErrorCode {
    trace!("indy_get_pairwise: >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_get_pairwise: entities >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

    let result = CommandExecutor::instance()
        .send(Command::Pairwise(PairwiseCommand::GetPairwise(
            wallet_handle,
            their_did,
            boxed_callback_string!("indy_get_pairwise", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_get_pairwise: <<< res: {:?}", res);

    res
}

/// Save some data in the Wallet for pairwise associated with Did.
///
/// #Params
/// wallet_handle: wallet handler (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// their_did: encoded Did
/// metadata: some extra information for pairwise
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_set_pairwise_metadata(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          their_did: *const c_char,
                                          metadata: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode)>) -> ErrorCode {
    trace!("indy_set_pairwise_metadata: >>> wallet_handle: {:?}, their_did: {:?}, metadata: {:?}", wallet_handle, their_did, metadata);

    check_useful_validatable_string!(their_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(metadata, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_set_pairwise_metadata: entities >>> wallet_handle: {:?}, their_did: {:?}, metadata: {:?}", wallet_handle, their_did, metadata);

    let result = CommandExecutor::instance()
        .send(Command::Pairwise(PairwiseCommand::SetPairwiseMetadata(
            wallet_handle,
            their_did,
            metadata,
            Box::new(move |result| {
                let err = prepare_result!(result);
                trace!("indy_set_pairwise_metadata:");
                cb(command_handle, err)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_set_pairwise_metadata: <<< res: {:?}", res);

    res
}
