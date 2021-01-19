use libc::c_char;
use indy_api_types::{ErrorCode, CommandHandle, WalletHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::payments::PaymentsCommand;
use crate::services::payments::PaymentsMethodCBs;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;
use crate::services::payments::{RequesterInfo, Fees};
use crate::domain::crypto::did::DidValue;
use indy_api_types::validation::Validatable;
use std::rc::Rc;
use crate::services::metrics::MetricsService;
use crate::utils::time::get_cur_time;
use crate::services::metrics::command_metrics::CommandMetric;

/// Create the payment address for this payment method.
///
/// This method generates private part of payment address
/// and stores it in a secure place. Ideally it should be
/// secret in libindy wallet (see crypto module).
///
/// Note that payment method should be able to resolve this
/// secret by fully resolvable payment address format.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle where keys for signature are stored
/// config: payment address config as json:
///   {
///     seed: <str>, // allows deterministic creation of payment address
///   }
///
/// #Returns
/// payment_address - public identifier of payment address in fully resolvable payment address format
pub type CreatePaymentAddressCB = extern fn(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            config: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 payment_address: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Modifies Indy request by adding information how to pay fees for this transaction
/// according to this payment method.
///
/// This method consumes set of inputs and outputs. The difference between inputs balance
/// and outputs balance is the fee for this transaction.
///
/// Not that this method also produces correct fee signatures.
///
/// Format of inputs is specific for payment method. Usually it should reference payment transaction
/// with at least one output that corresponds to payment address that user owns.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// req_json: initial transaction request as json
/// inputs_json: The list of payment sources as json array:
///   ["source1", ...]
///   Note that each source should reference payment address
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for payment operation
///
/// #Returns
/// req_with_fees_json - modified Indy request with added fees info
pub type AddRequestFeesCB = extern fn(command_handle: CommandHandle,
                                      wallet_handle: WalletHandle,
                                      submitter_did: *const c_char,
                                      req_json: *const c_char,
                                      inputs_json: *const c_char,
                                      outputs_json: *const c_char,
                                      extra: *const c_char,
                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                           err: ErrorCode,
                                                           req_with_fees_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Parses response for Indy request with fees.
///
/// #Params
/// command_handle: command handle to map callback to context
/// resp_json: response for Indy request with fees
///
/// #Returns
/// receipts_json - parsed (payment method and node version agnostic) receipts info as json:
///   [{
///      receipt: <str>, // receipt that can be used for payment referencing and verification
///      recipient: <str>, //payment address for this recipient
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
pub type ParseResponseWithFeesCB = extern fn(command_handle: CommandHandle,
                                             resp_json: *const c_char,
                                             cb: Option<extern fn(command_handle_: CommandHandle,
                                                                  err: ErrorCode,
                                                                  receipts_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Builds Indy request for getting sources list for payment address
/// according to this payment method.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// payment_address: target payment address
/// from: shift to the next slice of payment sources
///
/// #Returns
/// get_sources_txn_json - Indy request for getting sources list for payment address
pub type BuildGetPaymentSourcesRequestCB = extern fn(command_handle: CommandHandle,
                                                     wallet_handle: WalletHandle,
                                                     submitter_did: *const c_char,
                                                     payment_address: *const c_char,
                                                     from: i64,
                                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                                          err: ErrorCode,
                                                                          get_sources_txn_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Parses response for Indy request for getting sources list.
///
/// #Params
/// command_handle: command handle to map callback to context
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
pub type ParseGetPaymentSourcesResponseCB = extern fn(command_handle: CommandHandle,
                                                      resp_json: *const c_char,
                                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                                           err: ErrorCode,
                                                                           sources_json: *const c_char,
                                                                           next: i64) -> ErrorCode>) -> ErrorCode;

/// Builds Indy request for doing payment
/// according to this payment method.
///
/// This method consumes set of inputs and outputs.
///
/// Format of inputs is specific for payment method. Usually it should reference payment transaction
/// with at least one output that corresponds to payment address that user owns.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// inputs_json: The list of payment sources as json array:
///   ["source1", ...]
///   Note that each source should reference payment address
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for payment operation
///
/// #Returns
/// payment_req_json - Indy request for doing payment
pub type BuildPaymentReqCB = extern fn(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       submitter_did: *const c_char,
                                       inputs_json: *const c_char,
                                       outputs_json: *const c_char,
                                       extra: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode,
                                                            payment_req_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Parses response for Indy request for payment txn.
///
/// #Params
/// command_handle: command handle to map callback to context
/// resp_json: response for Indy request for payment txn
///
/// #Returns
/// receipts_json - parsed (payment method and node version agnostic) receipts info as json:
///   [{
///      receipt: <str>, // receipt that can be used for payment referencing and verification
///      recipient: <str>, //payment address for this receipt
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
pub type ParsePaymentResponseCB = extern fn(command_handle: CommandHandle,
                                            resp_json: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 receipts_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Builds Indy request for doing minting
/// according to this payment method.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for payment operation
///
/// #Returns
/// mint_req_json - Indy request for doing minting
pub type BuildMintReqCB = extern fn(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    submitter_did: *const c_char,
                                    outputs_json: *const c_char,
                                    extra: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode,
                                                         mint_req_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Builds Indy request for setting fees for transactions in the ledger
///
/// # Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// fees_json {
///   txnType1: amount1,
///   txnType2: amount2,
///   .................
///   txnTypeN: amountN,
/// }
///
/// # Return
/// set_txn_fees_json - Indy request for setting fees for transactions in the ledger
pub type BuildSetTxnFeesReqCB = extern fn(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          submitter_did: *const c_char,
                                          fees_json: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               set_txn_fees_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Builds Indy get request for getting fees for transactions in the ledger
///
/// # Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
///
/// # Return
/// get_txn_fees_json - Indy request for getting fees for transactions in the ledger
pub type BuildGetTxnFeesReqCB = extern fn(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          submitter_did: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               get_txn_fees_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Parses response for Indy request for getting fees
///
/// # Params
/// command_handle: command handle to map callback to context
/// resp_json: response for Indy request for getting fees
///
/// # Return
/// fees_json {
///   txnType1: amount1,
///   txnType2: amount2,
///   .................
///   txnTypeN: amountN,
/// }
pub type ParseGetTxnFeesResponseCB = extern fn(command_handle: CommandHandle,
                                               resp_json: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    fees_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Builds Indy request for getting information to verify the payment receipt
///
/// # Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// receipt: payment receipt to verify
///
/// # Return
/// verify_txn_json -- request to be sent to ledger
pub type BuildVerifyPaymentReqCB = extern fn(command_handle: CommandHandle,
                                             wallet_handle: WalletHandle,
                                             submitter_did: *const c_char,
                                             receipt: *const c_char,
                                             cb: Option<extern fn(command_handle_: CommandHandle,
                                                                  err: ErrorCode,
                                                                  verify_txn_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Parses Indy response with information to verify receipt
///
/// # Params
/// command_handle: command handle to map callback to context
/// resp_json: response for Indy request for information to verify the payment receipt
///
/// # Return
/// txn_json: {
///     sources: [<str>, ]
///     receipts: [ {
///         recipient: <str>, // payment address of recipient
///         receipt: <str>, // receipt that can be used for payment referencing and verification
///         amount: <int>, // amount
///     }, ]
///     extra: <str>, //optional data
/// }
pub type ParseVerifyPaymentResponseCB = extern fn(command_handle: CommandHandle,
                                                  resp_json: *const c_char,
                                                  cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode,
                                                                txn_json: *const c_char) -> ErrorCode>) -> ErrorCode;

/// Sign a message using the private key from a public payment address
///
/// # Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle
/// address: the public payment address to use to sign a message
/// message_raw: a pointer to first byte of message to be signed
/// message_len: a message length
/// cb: Callback that takes command result as parameter.
///
/// # Return
/// a signature byte array
pub type SignWithAddressCB = extern fn (command_handle: CommandHandle, wallet_handle: WalletHandle,
                                        address: *const c_char,
                                        message_raw: *const u8, message_len: u32,
                                        cb: Option<extern fn(command_handle: CommandHandle, err: ErrorCode, raw: *const u8, len: u32) -> ErrorCode>) -> ErrorCode;

/// Verify a signature with a public payment address.
///
/// # Params
/// command_handle: command handle to map callback to user context.
/// address: public payment address of the message signer
/// message_raw: a pointer to first byte of message that has been signed
/// message_len: a message length
/// signature_raw: a pointer to first byte of signature to be verified
/// signature_len: a signature length
/// cb: Callback that takes command result as parameter.
///
/// # Returns
/// valid: true - if signature is valid, false - otherwise
pub type VerifyWithAddressCB = extern fn (command_handle: CommandHandle, address: *const c_char,
                                          message_raw: *const u8, message_len: u32,
                                          signature_raw: *const u8, signature_len: u32,
                                          cb: Option<extern fn(command_handle: CommandHandle, err: ErrorCode, result: u8) -> ErrorCode>) -> ErrorCode;

/// Register custom payment implementation.
///
/// It allows library user to provide custom payment method implementation as set of handlers.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: The type of payment method also used as sub-prefix for fully resolvable payment address format ("sov" - for example)
/// create_payment_address: "create_payment_address" operation handler
/// add_request_fees: "add_request_fees" operation handler
/// parse_response_with_fees: "parse_response_with_fees" operation handler
/// build_get_payment_sources_request: "build_get_payment_sources_request" operation handler
/// parse_get_payment_sources_response: "parse_get_payment_sources_response" operation handler
/// build_payment_req: "build_payment_req" operation handler
/// parse_payment_response: "parse_payment_response" operation handler
/// build_mint_req: "build_mint_req" operation handler
/// build_set_txn_fees_req: "build_set_txn_fees_req" operation handler
/// build_get_txn_fees_req: "build_get_txn_fees_req" operation handler
/// parse_get_txn_fees_response: "parse_get_txn_fees_response" operation handler
/// build_verify_payment_req: "build_verify_payment_req" operation handler
/// parse_verify_payment_response: "parse_verify_payment_response" operation handler
///
/// #Returns
/// Error code
#[no_mangle]
pub extern fn indy_register_payment_method(command_handle: CommandHandle,
                                           payment_method: *const c_char,
                                           create_payment_address: Option<CreatePaymentAddressCB>,
                                           add_request_fees: Option<AddRequestFeesCB>,
                                           parse_response_with_fees: Option<ParseResponseWithFeesCB>,
                                           build_get_payment_sources_request: Option<BuildGetPaymentSourcesRequestCB>,
                                           parse_get_payment_sources_response: Option<ParseGetPaymentSourcesResponseCB>,
                                           build_payment_req: Option<BuildPaymentReqCB>,
                                           parse_payment_response: Option<ParsePaymentResponseCB>,
                                           build_mint_req: Option<BuildMintReqCB>,
                                           build_set_txn_fees_req: Option<BuildSetTxnFeesReqCB>,
                                           build_get_txn_fees_req: Option<BuildGetTxnFeesReqCB>,
                                           parse_get_txn_fees_response: Option<ParseGetTxnFeesResponseCB>,
                                           build_verify_payment_req: Option<BuildVerifyPaymentReqCB>,
                                           parse_verify_payment_response: Option<ParseVerifyPaymentResponseCB>,
                                           sign_with_address: Option<SignWithAddressCB>,
                                           verify_with_address: Option<VerifyWithAddressCB>,
                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode)>) -> ErrorCode {
    trace!("indy_register_payment_method: >>> payment_method: {:?}", payment_method);

    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(create_payment_address, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(add_request_fees, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(parse_response_with_fees, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(build_get_payment_sources_request, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(parse_get_payment_sources_response, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(build_payment_req, ErrorCode::CommonInvalidParam8);
    check_useful_c_callback!(parse_payment_response, ErrorCode::CommonInvalidParam9);
    check_useful_c_callback!(build_mint_req, ErrorCode::CommonInvalidParam10);
    check_useful_c_callback!(build_set_txn_fees_req, ErrorCode::CommonInvalidParam11);
    check_useful_c_callback!(build_get_txn_fees_req, ErrorCode::CommonInvalidParam12);
    check_useful_c_callback!(parse_get_txn_fees_response, ErrorCode::CommonInvalidParam13);
    check_useful_c_callback!(build_verify_payment_req, ErrorCode::CommonInvalidParam14);
    check_useful_c_callback!(parse_verify_payment_response, ErrorCode::CommonInvalidParam15);
    check_useful_c_callback!(sign_with_address, ErrorCode::CommonInvalidParam16);
    check_useful_c_callback!(verify_with_address, ErrorCode::CommonInvalidParam17);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam16);

    trace!("indy_register_payment_method: entities >>> payment_method: {:?}", payment_method);

    let cbs = PaymentsMethodCBs::new(
        create_payment_address,
        add_request_fees,
        parse_response_with_fees,
        build_get_payment_sources_request,
        parse_get_payment_sources_response,
        build_payment_req,
        parse_payment_response,
        build_mint_req,
        build_set_txn_fees_req,
        build_get_txn_fees_req,
        parse_get_txn_fees_response,
        build_verify_payment_req,
        parse_verify_payment_response,
        sign_with_address,
        verify_with_address
    );
    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::RegisterMethod(
                    payment_method,
                    cbs,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, result.into());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandRegisterMethod, get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_register_payment_method: <<< res: {:?}", res);

    res
}

/// Create the payment address for specified payment method
///
///
/// This method generates private part of payment address
/// and stores it in a secure place. Ideally it should be
/// secret in libindy wallet (see crypto module).
///
/// Note that payment method should be able to resolve this
/// secret by fully resolvable payment address format.
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet handle where to save new address
/// payment_method: payment method to use (for example, 'sov')
/// config: payment address config as json:
///   {
///     seed: <str>, // allows deterministic creation of payment address
///   }
///
/// #Returns
/// payment_address - public identifier of payment address in fully resolvable payment address format
#[no_mangle]
pub extern fn indy_create_payment_address(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          payment_method: *const c_char,
                                          config: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               payment_address: *const c_char)>) -> ErrorCode {
    trace!("indy_create_payment_address: >>> wallet_handle: {:?}, payment_method: {:?}, config: {:?}", wallet_handle, payment_method, config);

    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(config, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_create_payment_address: entities >>> wallet_handle: {:?}, payment_method: {:?}, config: {:?}", wallet_handle, payment_method, config);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::CreateAddress(
                    wallet_handle,
                    payment_method,
                    config,
                    boxed_callback_string!("indy_create_payment_address", cb, command_handle, CommandMetric::PaymentsCommandCreateAddress)
                )
            ));

    let res = prepare_result!(result);

    trace!("indy_create_payment_address: <<< res: {:?}", res);

    res
}

/// Lists all payment addresses that are stored in the wallet
///
/// #Params
/// command_handle: command handle to map callback to context
/// wallet_handle: wallet to search for payment_addresses in
///
/// #Returns
/// payment_addresses_json - json array of string with json addresses
#[no_mangle]
pub extern fn indy_list_payment_addresses(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               payment_addresses_json: *const c_char)>) -> ErrorCode {
    trace!("indy_list_payment_address: >>> wallet_handle: {:?}", wallet_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_list_payment_address: entities >>> wallet_handle: {:?}", wallet_handle);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::ListAddresses(
                    wallet_handle,
                    boxed_callback_string!("indy_list_payment_address", cb, command_handle, CommandMetric::PaymentsCommandListAddresses)
                )
            )
        );

    let res = prepare_result!(result);

    trace!("indy_list_payment_address: <<< res: {:?}", res);

    res
}

/// Modifies Indy request by adding information how to pay fees for this transaction
/// according to this payment method.
///
/// This method consumes set of inputs and outputs. The difference between inputs balance
/// and outputs balance is the fee for this transaction.
///
/// Not that this method also produces correct fee signatures.
///
/// Format of inputs is specific for payment method. Usually it should reference payment transaction
/// with at least one output that corresponds to payment address that user owns.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// req_json: initial transaction request as json
/// inputs_json: The list of payment sources as json array:
///   ["source1", ...]
///     - each input should reference paymentAddress
///     - this param will be used to determine payment_method
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for payment operation
///
/// #Returns
/// req_with_fees_json - modified Indy request with added fees info
/// payment_method - used payment method
#[no_mangle]
pub extern fn indy_add_request_fees(command_handle: CommandHandle,
                                    wallet_handle: WalletHandle,
                                    submitter_did: *const c_char,
                                    req_json: *const c_char,
                                    inputs_json: *const c_char,
                                    outputs_json: *const c_char,
                                    extra: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode,
                                                         req_with_fees_json: *const c_char,
                                                         payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_add_request_fees: >>> wallet_handle: {:?}, submitter_did: {:?}, req_json: {:?}, inputs_json: {:?}, outputs_json: {:?}, extra: {:?}",
           wallet_handle, submitter_did, req_json, inputs_json, outputs_json, extra);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(req_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(inputs_json, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(outputs_json, ErrorCode::CommonInvalidParam6);
    check_useful_opt_c_str!(extra, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_add_request_fees: entities >>> wallet_handle: {:?}, submitter_did: {:?}, req_json: {:?}, inputs_json: {:?}, outputs_json: {:?}, extra: {:?}",
           wallet_handle, submitter_did, req_json, inputs_json, outputs_json, extra);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::AddRequestFees(
                    wallet_handle,
                    submitter_did,
                    req_json,
                    inputs_json,
                    outputs_json,
                    extra,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let (err, req_with_fees_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_add_request_fees: req_with_fees_json: {:?}, payment_method: {:?}", req_with_fees_json, payment_method);
                        let req_with_fees_json = ctypes::string_to_cstring(req_with_fees_json);
                        let payment_method = ctypes::string_to_cstring(payment_method);
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, err, req_with_fees_json.as_ptr(), payment_method.as_ptr());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandAddRequestFees,get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_add_request_fees: <<< res: {:?}", res);

    res
}

/// Parses response for Indy request with fees.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use
/// resp_json: response for Indy request with fees
///
/// #Returns
/// receipts_json - parsed (payment method and node version agnostic) receipts info as json:
///   [{
///      receipt: <str>, // receipt that can be used for payment referencing and verification
///      recipient: <str>, //payment address of recipient
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
#[no_mangle]
pub extern fn indy_parse_response_with_fees(command_handle: CommandHandle,
                                            payment_method: *const c_char,
                                            resp_json: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 receipts_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_response_with_fees: >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_response_with_fees: entities >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::ParseResponseWithFees(
                    payment_method,
                    resp_json,
                    boxed_callback_string!("indy_parse_response_with_fees", cb, command_handle, CommandMetric::PaymentsCommandParseResponseWithFees))));
    let res = prepare_result!(result);

    trace!("indy_parse_response_with_fees: <<< res: {:?}", res);

    res
}

/// Builds Indy request for getting sources list for payment address
/// according to this payment method.
/// Deprecated. This function will be most likely be removed with Indy SDK 2.0 version
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// payment_address: target payment address
///
/// #Returns
/// get_sources_txn_json - Indy request for getting sources list for payment address
/// payment_method - used payment method
#[no_mangle]
pub extern fn indy_build_get_payment_sources_request(command_handle: CommandHandle,
                                                     wallet_handle: WalletHandle,
                                                     submitter_did: *const c_char,
                                                     payment_address: *const c_char,
                                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                                          err: ErrorCode,
                                                                          get_sources_txn_json: *const c_char,
                                                                          payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_payment_sources_request: >>> wallet_handle: {:?}, submitter_did: {:?}, payment_address: {:?}", wallet_handle, submitter_did, payment_address);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(payment_address, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_get_payment_sources_request: entities >>> wallet_handle: {:?}, submitter_did: {:?}, payment_address: {:?}", wallet_handle, submitter_did, payment_address);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildGetPaymentSourcesRequest(
                    wallet_handle,
                    submitter_did,
                    payment_address,
                    None,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let (err, get_sources_txn_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_build_get_payment_sources_request: get_sources_txn_json: {:?}, payment_method: {:?}", get_sources_txn_json, payment_method);
                        let get_sources_txn_json = ctypes::string_to_cstring(get_sources_txn_json);
                        let payment_method = ctypes::string_to_cstring(payment_method);
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, err, get_sources_txn_json.as_ptr(), payment_method.as_ptr());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandBuildGetPaymentSourcesRequest, get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_build_get_payment_sources_request: <<< res: {:?}", res);

    res
}

/// Parses response for Indy request for getting sources list.
/// Deprecated. This function will be most likely be removed with Indy SDK 2.0 version
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use.
/// resp_json: response for Indy request for getting sources list
///
/// #Returns
/// sources_json - parsed (payment method and node version agnostic) sources info as json:
///   [{
///      source: <str>, // source input
///      paymentAddress: <str>, //payment address for this source
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
#[no_mangle]
pub extern fn indy_parse_get_payment_sources_response(command_handle: CommandHandle,
                                                      payment_method: *const c_char,
                                                      resp_json: *const c_char,
                                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                                           err: ErrorCode,
                                                                           sources_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_payment_sources_response: >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_get_payment_sources_response: entities >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::ParseGetPaymentSourcesResponse(
                    payment_method,
                    resp_json,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let (err, sources_json, _) = prepare_result_2!(result, String::new(), -1);
                        trace!("indy_parse_get_payment_sources_response: sources_json: {:?}", sources_json);
                        let sources_json = ctypes::string_to_cstring(sources_json);
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, err, sources_json.as_ptr());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandParseGetPaymentSourcesResponse, get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_parse_get_payment_sources_response: <<< res: {:?}", res);

    res
}

/// Builds Indy request for doing payment
/// according to this payment method.
///
/// This method consumes set of inputs and outputs.
///
/// Format of inputs is specific for payment method. Usually it should reference payment transaction
/// with at least one output that corresponds to payment address that user owns.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// inputs_json: The list of payment sources as json array:
///   ["source1", ...]
///   Note that each source should reference payment address
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for payment operation
///
/// #Returns
/// payment_req_json - Indy request for doing payment
/// payment_method - used payment method
#[no_mangle]
pub extern fn indy_build_payment_req(command_handle: CommandHandle,
                                     wallet_handle: WalletHandle,
                                     submitter_did: *const c_char,
                                     inputs_json: *const c_char,
                                     outputs_json: *const c_char,
                                     extra: *const c_char,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode,
                                                          payment_req_json: *const c_char,
                                                          payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_build_payment_req: >>> wallet_handle: {:?}, submitter_did: {:?}, inputs_json: {:?}, outputs_json: {:?}, extra: {:?}",
           wallet_handle, submitter_did, inputs_json, outputs_json, extra);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(inputs_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(outputs_json, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(extra, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_payment_req: entities >>> wallet_handle: {:?}, submitter_did: {:?}, inputs_json: {:?}, outputs_json: {:?}, extra: {:?}",
           wallet_handle, submitter_did, inputs_json, outputs_json, extra);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildPaymentReq(
                    wallet_handle,
                    submitter_did,
                    inputs_json,
                    outputs_json,
                    extra,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let (err, payment_req_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_build_payment_req: payment_req_json: {:?}, payment_method: {:?}", payment_req_json, payment_method);
                        let payment_req_json = ctypes::string_to_cstring(payment_req_json);
                        let payment_method = ctypes::string_to_cstring(payment_method);
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, err, payment_req_json.as_ptr(), payment_method.as_ptr());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandBuildPaymentReq,get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_build_payment_req: <<< res: {:?}", res);

    res
}

/// Parses response for Indy request for payment txn.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use
/// resp_json: response for Indy request for payment txn
///
/// #Returns
/// receipts_json - parsed (payment method and node version agnostic) receipts info as json:
///   [{
///      receipt: <str>, // receipt that can be used for payment referencing and verification
///      recipient: <str>, // payment address of recipient
///      amount: <int>, // amount
///      extra: <str>, // optional data from payment transaction
///   }]
#[no_mangle]
pub extern fn indy_parse_payment_response(command_handle: CommandHandle,
                                          payment_method: *const c_char,
                                          resp_json: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               receipts_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_payment_response: >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_payment_response: entities >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::ParsePaymentResponse(
                    payment_method,
                    resp_json,
                    boxed_callback_string!("indy_parse_payment_response", cb, command_handle, CommandMetric::PaymentsCommandParsePaymentResponse))));

    let res = prepare_result!(result);

    trace!("indy_parse_payment_response: <<< res: {:?}", res);

    res
}

/// Prepare payment extra JSON with TAA acceptance data
///
/// EXPERIMENTAL
///
/// This function may calculate digest by itself or consume it as a parameter.
/// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// extra_json: (optional) original extra json.
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if taa_digest parameter is omitted.
/// taa_digest - (optional) digest on text and version.
///     Digest is sha256 hash calculated on concatenated strings: version || text.
///     This parameter is required if text and version parameters are omitted.
/// mechanism - mechanism how user has accepted the TAA
/// time - UTC timestamp when user has accepted the TAA
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Updated request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_prepare_payment_extra_with_acceptance_data(command_handle: CommandHandle,
                                                              extra_json: *const c_char,
                                                              text: *const c_char,
                                                              version: *const c_char,
                                                              taa_digest: *const c_char,
                                                              mechanism: *const c_char,
                                                              time: u64,
                                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                   err: ErrorCode,
                                                                                   extra_with_acceptance: *const c_char)>) -> ErrorCode {
    trace!("indy_prepare_payment_extra_with_acceptance_data: >>> extra_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, \
        mechanism: {:?}, time: {:?}",
           extra_json, text, version, taa_digest, mechanism, time);

    check_useful_opt_c_str!(extra_json, ErrorCode::CommonInvalidParam2);
    check_useful_opt_c_str!(text, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(taa_digest, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(mechanism, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_prepare_payment_extra_with_acceptance_data: entities >>> extra_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, \
        mechanism: {:?}, time: {:?}",
           extra_json, text, version, taa_digest, mechanism, time);

    let result = CommandExecutor::instance()
        .send(Command::Payments(
            PaymentsCommand::AppendTxnAuthorAgreementAcceptanceToExtra(
                extra_json,
                text,
                version,
                taa_digest,
                mechanism,
                time,
                boxed_callback_string!("indy_prepare_payment_extra_with_acceptance_data", cb, command_handle, CommandMetric::PaymentsCommandAppendTxnAuthorAgreementAcceptanceToExtra)
            )));

    let res = prepare_result!(result);

    trace!("indy_prepare_payment_extra_with_acceptance_data: <<< res: {:?}", res);

    res
}

/// Builds Indy request for doing minting
/// according to this payment method.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// outputs_json: The list of outputs as json array:
///   [{
///     recipient: <str>, // payment address of recipient
///     amount: <int>, // amount
///   }]
/// extra: // optional information for mint operation
///
/// #Returns
/// mint_req_json - Indy request for doing minting
/// payment_method - used payment method
#[no_mangle]
pub extern fn indy_build_mint_req(command_handle: CommandHandle,
                                  wallet_handle: WalletHandle,
                                  submitter_did: *const c_char,
                                  outputs_json: *const c_char,
                                  extra: *const c_char,
                                  cb: Option<extern fn(command_handle_: CommandHandle,
                                                       err: ErrorCode,
                                                       mint_req_json: *const c_char,
                                                       payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_build_mint_req: >>> wallet_handle: {:?}, submitter_did: {:?}, outputs_json: {:?}, extra: {:?}", wallet_handle, submitter_did, outputs_json, extra);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(outputs_json, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(extra, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_build_mint_req: entities >>> wallet_handle: {:?}, submitter_did: {:?}, outputs_json: {:?}, extra: {:?}", wallet_handle, submitter_did, outputs_json, extra);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildMintReq(
                    wallet_handle,
                    submitter_did,
                    outputs_json,
                    extra,
                    Box::new(move |result, metrics_service: Rc<MetricsService>| {
                        let (err, mint_req_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_build_mint_req: mint_req_json: {:?}, payment_method: {:?}", mint_req_json, payment_method);
                        let mint_req_json = ctypes::string_to_cstring(mint_req_json);
                        let payment_method = ctypes::string_to_cstring(payment_method);
                        let start_execution_ts = get_cur_time();
                        cb(command_handle, err, mint_req_json.as_ptr(), payment_method.as_ptr());
                        metrics_service.cmd_callback(CommandMetric::PaymentsCommandBuildMintReq,get_cur_time() - start_execution_ts);
                    }))
            ));

    let res = prepare_result!(result);

    trace!("indy_build_mint_req: <<< res: {:?}", res);

    res
}

/// Builds Indy request for setting fees for transactions in the ledger
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// payment_method: payment method to use
/// fees_json {
///   txnType1: amount1,
///   txnType2: amount2,
///   .................
///   txnTypeN: amountN,
/// }
/// # Return
/// set_txn_fees_json - Indy request for setting fees for transactions in the ledger
#[no_mangle]
pub extern fn indy_build_set_txn_fees_req(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          submitter_did: *const c_char,
                                          payment_method: *const c_char,
                                          fees_json: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               set_txn_fees_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_set_txn_fees_req: >>> wallet_handle: {:?}, submitter_did: {:?}, payment_method: {:?}, fees_json: {:?}", wallet_handle, submitter_did, payment_method, fees_json);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(fees_json, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_build_set_txn_fees_req: entitites >>> wallet_handle: {:?}, submitter_did: {:?}, payment_method: {:?}, fees_json: {:?}", wallet_handle, submitter_did, payment_method, fees_json);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildSetTxnFeesReq(
                    wallet_handle,
                    submitter_did,
                    payment_method,
                    fees_json,
                    boxed_callback_string!("indy_build_set_txn_fees_req", cb, command_handle, CommandMetric::PaymentsCommandBuildSetTxnFeesReq))));

    let res = prepare_result!(result);

    trace!("indy_build_set_txn_fees_req: <<< res: {:?}", res);

    res
}

/// Builds Indy get request for getting fees for transactions in the ledger
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// payment_method: payment method to use
///
/// # Return
/// get_txn_fees_json - Indy request for getting fees for transactions in the ledger
#[no_mangle]
pub extern fn indy_build_get_txn_fees_req(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          submitter_did: *const c_char,
                                          payment_method: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               get_txn_fees_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_txn_fees_req: >>> wallet_handle: {:?}, submitter_did: {:?}, payment_method: {:?}", wallet_handle, submitter_did, payment_method);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_get_txn_fees_req: entities >>> wallet_handle: {:?}, submitter_did: {:?}, payment_method: {:?}", wallet_handle, submitter_did, payment_method);

    let result =
        CommandExecutor::instance().send(
            Command::Payments(
                PaymentsCommand::BuildGetTxnFeesReq(
                    wallet_handle,
                    submitter_did,
                    payment_method,
                    boxed_callback_string!("indy_build_get_txn_fees_req", cb, command_handle, CommandMetric::PaymentsCommandBuildGetTxnFeesReq))));

    let res = prepare_result!(result);

    trace!("indy_build_get_txn_fees_req: <<< res: {:?}", res);

    res
}

/// Parses response for Indy request for getting fees
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use
/// resp_json: response for Indy request for getting fees
///
/// # Return
/// fees_json {
///   txnType1: amount1,
///   txnType2: amount2,
///   .................
///   txnTypeN: amountN,
/// }
#[no_mangle]
pub extern fn indy_parse_get_txn_fees_response(command_handle: CommandHandle,
                                               payment_method: *const c_char,
                                               resp_json: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    fees_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_txn_fees_response: >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_get_txn_fees_response: entities >>> payment_method: {:?}, resp_json: {:?}", payment_method, resp_json);

    let result =
        CommandExecutor::instance()
            .send(Command::Payments(
                PaymentsCommand::ParseGetTxnFeesResponse(
                    payment_method,
                    resp_json,
                    boxed_callback_string!("indy_parse_get_txn_fees_response", cb, command_handle, CommandMetric::PaymentsCommandParseGetTxnFeesResponse))));

    let res = prepare_result!(result);

    trace!("indy_parse_get_txn_fees_response: <<< res: {:?}", res);

    res
}

/// Builds Indy request for information to verify the payment receipt
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// wallet_handle: wallet handle
/// submitter_did: (Optional) DID of request sender
/// receipt: payment receipt to verify
///
/// # Return
/// verify_txn_json: Indy request for verification receipt
/// payment_method: used payment method
#[no_mangle]
pub extern fn indy_build_verify_payment_req(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            submitter_did: *const c_char,
                                            receipt: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 verify_txn_json: *const c_char,
                                                                 payment_method: *const c_char)>) -> ErrorCode {
    trace!("indy_build_verify_payment_req: >>> wallet_handle {:?}, submitter_did: {:?}, receipt: {:?}", wallet_handle, submitter_did, receipt);
    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(receipt, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_verify_payment_req: entities >>> wallet_handle {:?}, submitter_did: {:?}, receipt: {:?}", wallet_handle, submitter_did, receipt);

    let result = CommandExecutor::instance()
        .send(Command::Payments(
            PaymentsCommand::BuildVerifyPaymentReq(
                wallet_handle,
                submitter_did,
                receipt,
                Box::new(move |result, metrics_service: Rc<MetricsService>| {
                    let (err, verify_txn_json, payment_method) = prepare_result_2!(result, String::new(), String::new());
                    trace!("indy_build_verify_payment_req: verify_txn_json: {:?}, payment_method: {:?}", verify_txn_json, payment_method);
                    let verify_txn_json = ctypes::string_to_cstring(verify_txn_json);
                    let payment_method = ctypes::string_to_cstring(payment_method);
                    let start_execution_ts = get_cur_time();
                    cb(command_handle, err, verify_txn_json.as_ptr(), payment_method.as_ptr());
                    metrics_service.cmd_callback(CommandMetric::PaymentsCommandBuildVerifyPaymentReq,get_cur_time() - start_execution_ts);
                })
            )));

    let result = prepare_result!(result);

    trace!("indy_build_verify_payment_req: <<< result: {:?}", result);

    result
}

/// Parses Indy response with information to verify receipt
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// payment_method: payment method to use
/// resp_json: response of the ledger for verify txn
///
/// # Return
/// txn_json: {
///     sources: [<str>, ]
///     receipts: [ {
///         recipient: <str>, // payment address of recipient
///         receipt: <str>, // receipt that can be used for payment referencing and verification
///         amount: <int>, // amount
///     } ],
///     extra: <str>, //optional data
/// }
#[no_mangle]
pub extern fn indy_parse_verify_payment_response(command_handle: CommandHandle,
                                                 payment_method: *const c_char,
                                                 resp_json: *const c_char,
                                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                                      err: ErrorCode,
                                                                      txn_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_verify_payment_response: >>> resp_json: {:?}", resp_json);
    check_useful_c_str!(payment_method, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(resp_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_parse_verify_payment_response: entities >>> resp_json: {:?}", resp_json);

    let result = CommandExecutor::instance()
        .send(Command::Payments(
            PaymentsCommand::ParseVerifyPaymentResponse(
                payment_method,
                resp_json,
                boxed_callback_string!("indy_parse_verify_payment_response", cb, command_handle, CommandMetric::PaymentsCommandParseVerifyPaymentResponse)
            )));

    let result = prepare_result!(result);

    trace!("indy_parse_verify_payment_response: <<< result: {:?}", result);

    result
}

/// Gets request requirements (with minimal price) correspondent to specific auth rule
/// in case the requester can perform this action.
///
/// EXPERIMENTAL
///
/// If the requester does not match to the request constraints `TransactionNotAllowed` error will be thrown.
///
/// # Params
/// command_handle: Command handle to map callback to caller context.
/// get_auth_rule_response_json: response on GET_AUTH_RULE request returning action constraints set on the ledger.
/// requester_info_json: {
///     "role": string (optional) - role of a user which can sign a transaction.
///     "sig_count": u64 - number of signers.
///     "is_owner": bool (optional) - if user is an owner of transaction (false by default).
///     "is_off_ledger_signature": bool (optional) - if user did is unknow for ledger (false by default).
/// }
/// fees_json: fees set on the ledger (result of `indy_parse_get_txn_fees_response`).
///
/// # Return
/// request_info_json: request info if a requester match to the action constraints.
/// {
///     "price": u64 - fee required for the action performing,
///     "requirements": [{
///         "role": string (optional) - role of users who should sign,
///         "sig_count": u64 - number of signers,
///         "need_to_be_owner": bool - if requester need to be owner,
///         "off_ledger_signature": bool - allow signature of unknow for ledger did (false by default).
///     }]
/// }
///
#[no_mangle]
pub extern fn indy_get_request_info(command_handle: CommandHandle,
                                    get_auth_rule_response_json: *const c_char,
                                    requester_info_json: *const c_char,
                                    fees_json: *const c_char,
                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                         err: ErrorCode,
                                                         request_info_json: *const c_char)>) -> ErrorCode {
    trace!("indy_get_request_info: >>> get_auth_rule_response_json: {:?}, requester_info_json: {:?}, fees_json: {:?}",
           get_auth_rule_response_json, requester_info_json, fees_json);

    check_useful_c_str!(get_auth_rule_response_json, ErrorCode::CommonInvalidParam2);
    check_useful_json!(requester_info_json, ErrorCode::CommonInvalidParam3, RequesterInfo);
    check_useful_json!(fees_json, ErrorCode::CommonInvalidParam4, Fees);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_get_request_info: entities >>> get_auth_rule_response_json: {:?}, requester_info_json: {:?}, fees_json: {:?}",
           get_auth_rule_response_json, requester_info_json, fees_json);

    let result = CommandExecutor::instance()
        .send(Command::Payments(
            PaymentsCommand::GetRequestInfo(
                get_auth_rule_response_json,
                requester_info_json,
                fees_json,
                boxed_callback_string!("indy_get_request_info", cb, command_handle, CommandMetric::PaymentsCommandGetRequestInfo)
            )));

    let result = prepare_result!(result);

    trace!("indy_get_request_info: <<< result: {:?}", result);

    result
}

/// Signs a message with a payment address.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handler (created by open_wallet).
/// address: payment address of message signer. The key must be created by calling indy_create_address
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
pub extern fn indy_sign_with_address(command_handle: CommandHandle,
                                     wallet_handle: WalletHandle,
                                     address: *const c_char,
                                     message_raw: *const u8,
                                     message_len: u32,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode,
                                                          signature_raw: *const u8,
                                                          signature_len: u32)>) -> ErrorCode {
    trace!("indy_sign_with_address: >>> wallet_handle: {:?}, address: {:?}, message_raw: {:?}, message_len: {:?}",
           wallet_handle, address, message_raw, message_len);
    check_useful_c_str!(address, ErrorCode::CommonInvalidParam3);
    check_useful_c_byte_array!(message_raw, message_len, ErrorCode::CommonInvalidParam4, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_sign_with_address: entities >>> wallet_handle: {:?}, address: {:?}, message_raw: {:?}, message_len: {:?}",
           wallet_handle, address, message_raw, message_len);

    let result = CommandExecutor::instance()
        .send(Command::Payments(
            PaymentsCommand::SignWithAddressReq(wallet_handle,
                                                address,
                                                message_raw,
                                                Box::new(move |result, metrics_service: Rc<MetricsService>| {
                                                    let (err, signature) = prepare_result_1!(result, Vec::new());
                                                    trace!("indy_sign_with_address: signature: {:?}", signature);
                                                    let (signature_raw, signature_len) = ctypes::vec_to_pointer(&signature);
                                                    let start_execution_ts = get_cur_time();
                                                    let result = cb(command_handle, err, signature_raw, signature_len);
                                                    metrics_service.cmd_callback(CommandMetric::PaymentsCommandSignWithAddressReq,get_cur_time() - start_execution_ts);

                                                    result
                                        }))
        ));


    let res = prepare_result!(result);

    trace!("indy_sign_with_address: <<< res: {:?}", res);

    res
}

/// Verify a signature with a payment address.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// address: payment address of the message signer
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
pub extern fn indy_verify_with_address(command_handle: CommandHandle,
                                       address: *const c_char,
                                       message_raw: *const u8,
                                       message_len: u32,
                                       signature_raw: *const u8,
                                       signature_len: u32,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode,
                                                            result: bool)>) -> ErrorCode {
    trace!("indy_verify_with_address: >>> address: {:?}, message_raw: {:?}, message_len: {:?}, signature_raw: {:?}, signature_len: {:?}",
           address, message_raw, message_len, signature_raw, signature_len);

    check_useful_c_str!(address, ErrorCode::CommonInvalidParam2);
    check_useful_c_byte_array!(message_raw, message_len, ErrorCode::CommonInvalidParam3, ErrorCode::CommonInvalidParam4);
    check_useful_c_byte_array!(signature_raw, signature_len, ErrorCode::CommonInvalidParam5, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_verify_with_address: entities >>> address: {:?}, message_raw: {:?}, message_len: {:?}, signature_raw: {:?}, signature_len: {:?}",
           address, message_raw, message_len, signature_raw, signature_len);

    let result = CommandExecutor::instance()
        .send(Command::Payments(PaymentsCommand::VerifyWithAddressReq(
            address,
            message_raw,
            signature_raw,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, valid) = prepare_result_1!(result, false);
                trace!("indy_verify_with_address: valid: {:?}", valid);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, valid);
                metrics_service.cmd_callback(CommandMetric::PaymentsCommandVerifyWithAddressReq,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_verify_with_address: <<< res: {:?}", res);

    res
}
