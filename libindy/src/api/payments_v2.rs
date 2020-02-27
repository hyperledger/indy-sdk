use indy_api_types::CommandHandle;
use indy_api_types::WalletHandle;
use indy_api_types::ErrorCode;
use libc::c_char;
use crate::commands::CommandExecutor;
use crate::commands::Command;
use crate::commands::payments::PaymentsCommand;
use indy_utils::ctypes;
use indy_api_types::errors::prelude::*;
use indy_api_types::domain::crypto::did::DidValue;
use indy_api_types::validation::Validatable;

/// Builds Indy request for getting sources list for payment address
/// according to this payment method.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// payment_address: target payment address
/// from: shift to the next slice of payment sources
///
/// #Returns
/// get_sources_txn_json - Indy request for getting sources list for payment address
/// payment_method - used payment method
#[no_mangle]
pub extern fn indy_build_get_payment_sources_with_from_request(command_handle: CommandHandle,
                                                               wallet_handle: WalletHandle,
                                                               submitter_did: *const c_char,
                                                               payment_address: *const c_char,
                                                               from: i64,
                                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                    err: ErrorCode,
                                                                                    get_sources_txn_json: *const c_char,
                                                                                    payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_payment_sources_with_from_request: >>> wallet_handle: {:?}, submitter_did: {:?}, payment_address: {:?}", wallet_handle, submitter_did, payment_address);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(payment_address, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let from: Option<i64> = if from == -1 { None } else { Some(from) };

    trace!("indy_build_get_payment_sources_with_from_request: entities >>> wallet_handle: {:?}, submitter_did: {:?}, payment_address: {:?}, from: {:?}", wallet_handle, submitter_did, payment_address, from);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildGetPaymentSourcesRequest(
                    wallet_handle,
                    submitter_did,
                    payment_address,
                    from,
                    Box::new(move |result| {
                        let (err, get_sources_txn_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_build_get_payment_sources_with_from_request: get_sources_txn_json: {:?}, payment_method: {:?}", get_sources_txn_json, payment_method);
                        let get_sources_txn_json = ctypes::string_to_cstring(get_sources_txn_json);
                        let payment_method = ctypes::string_to_cstring(payment_method);
                        cb(command_handle, err, get_sources_txn_json.as_ptr(), payment_method.as_ptr());
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_build_get_payment_sources_with_from_request: <<< res: {:?}", res);

    res
}

/// Parses response for Indy request for getting sources list.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use.
/// resp_json: response for Indy request for getting sources list
///
/// #Returns
/// next - pointer to the next slice of payment sources
/// sources_json - parsed (payment method and node version agnostic) sources info as json:
///   [{
///      source: <str>, // source input
///      paymentAddress: <str>, //payment address for this source
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
#[no_mangle]
pub extern fn indy_parse_get_payment_sources_with_from_response(command_handle: CommandHandle,
                                                                payment_method: *const c_char,
                                                                resp_json: *const c_char,
                                                                cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                     err: ErrorCode,
                                                                                     sources_json: *const c_char,
                                                                                     next: i64)>) -> ErrorCode {
    trace!("indy_parse_get_payment_sources_with_from_response: >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_get_payment_sources_with_from_response: entities >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::ParseGetPaymentSourcesResponse(
                    payment_method,
                    resp_json,
                    Box::new(move |result| {
                        let (err, sources_json, next) = prepare_result_2!(result, String::new(), -1);
                        trace!("indy_parse_get_payment_sources_with_from_response: sources_json: {:?}", sources_json);
                        let sources_json = ctypes::string_to_cstring(sources_json);
                        cb(command_handle, err, sources_json.as_ptr(), next);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_parse_get_payment_sources_with_from_response: <<< res: {:?}", res);

    res
}