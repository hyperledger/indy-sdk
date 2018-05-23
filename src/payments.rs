use super::ErrorCode;

use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use utils;
use ffi::payments;
use callbacks::payments as callbacks;

pub struct Payment {}

impl Payment {
//    pub fn make_payment_response_c_callback(f: &callbacks::PaymentResponseCallback) -> payments::PaymentResponseCB {
//        extern "C" fn _callback(xcommand_handle: i32, err: ErrorCode, c_str: *const c_char) {
//            let payment_address = unsafe { CStr::from_ptr(c_str).to_str().unwrap().to_string() };
//            f(xcommand_handle, err, payment_address)
//        }
//        _callback
//    }
//
//    pub fn make_create_payment_address_c_callback(f: &callbacks::CreatePaymentAddressCallback) -> payments::CreatePaymentAddressCB {
//        extern "C" fn _callback(command_handle: i32, wallet_handle: i32, c_str: *const c_char, cb: Option<payments::PaymentResponseCB>) -> ErrorCode {
//            let config = unsafe { CStr::from_ptr(c_str).to_str().unwrap().to_string() };
//            f(command_handle, wallet_handle, config, cb)
//        }
//        _callback
//    }

//    pub fn register(payment_method: &str,
//                    create_payment_address: callbacks::CreatePaymentAddressCallback,
//                    add_request_fees: callbacks::AddRequestFeesCallback,
//                    parse_response_with_fees: callbacks::ParseResponseWithFeesCallback,
//                    build_get_utxo_request: callbacks::BuildGetUTXORequestCallback,
//                    parse_get_utxo_response: callbacks::ParseGetUTXOResponseCallback,
//                    build_payment_req: callbacks::BuildPaymentRequestCallback,
//                    parse_payment_response: callbacks::ParsePaymentResponseCallback,
//                    build_mint_req: callbacks::BuildMintRequestCallback,
//                    build_set_txn_fees_req: callbacks::BuildSetTxnFeesRequestCallback,
//                    build_get_txn_fees_req: callbacks::BuildGetTxnFeesRequestCallback,
//                    parse_get_txn_fees_response: callbacks::ParseGetTxnFeesResponseCallback) -> Result<(), ErrorCode> {
//
//    }

//    pub fn sign_multi_request(wallet_handle: i32, submitter_did: &str, resp_json: &str) -> Result<String, ErrorCode> {
//        let (receiver, command_handle, cb) = utils::callbacks::_closure_to_cb_ec_string();
//
//        let submitter_did = CString::new(submitter_did).unwrap();
//        let resp_json = CString::new(resp_json).unwrap();
//
//        let err = unsafe {
//            payments::indy_sign_multi_request(command_handle,
//                                              wallet_handle,
//                                              submitter_did.as_ptr(),
//                                              resp_json.as_ptr(), cb)
//        };
//
//        utils::results::result_to_one(err, receiver)
//    }

    pub fn create_payment_address(wallet_handle: i32, payment_method: &str, config: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = utils::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let config = CString::new(config).unwrap();

        let err = unsafe {
            payments::indy_create_payment_address(command_handle,
                                        wallet_handle,
                                        payment_method.as_ptr(),
                                        config.as_ptr(),
                                        cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn list_payment_addresses(wallet_handle: i32) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) = utils::callbacks::_closure_to_cb_ec_string();

        let err = unsafe {
            payments::indy_list_payment_addresses(command_handle,
                                        wallet_handle,
                                        cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn add_request_fees(wallet_handle: i32, submitter_did: &str, req_json: &str, inputs_json: &str, outputs_json: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) = utils::callbacks::_closure_to_cb_ec_string_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let req_json = CString::new(req_json).unwrap();
        let inputs_json = CString::new(inputs_json).unwrap();
        let outputs_json = CString::new(outputs_json).unwrap();

        let err = unsafe {
            payments::indy_add_request_fees(command_handle,
                                  wallet_handle,
                                  submitter_did.as_ptr(),
                                  req_json.as_ptr(),
                                  inputs_json.as_ptr(),
                                  outputs_json.as_ptr(),
                                  cb)
        };

        utils::results::result_to_two(err, receiver)
    }

    pub fn build_get_utxo_request(wallet_handle: i32, submitter_did: &str, payment_address: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let payment_address = CString::new(payment_address).unwrap();

        let err = unsafe {
            payments::indy_build_get_utxo_request(command_handle,
                                        wallet_handle,
                                        submitter_did.as_ptr(),
                                        payment_address.as_ptr(),
                                        cb)
        };

        utils::results::result_to_two(err, receiver)
    }

    pub fn parse_get_utxo_response(payment_method: &str, resp_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let resp_json = CString::new(resp_json).unwrap();

        let err = unsafe {
            payments::indy_parse_get_utxo_response(command_handle,
                                         payment_method.as_ptr(),
                                         resp_json.as_ptr(),
                                         cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn build_payment_req(wallet_handle: i32, submitter_did: &str, inputs: &str, outputs: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let inputs = CString::new(inputs).unwrap();
        let outputs = CString::new(outputs).unwrap();

        let err = unsafe {
            payments::indy_build_payment_req(command_handle,
                                   wallet_handle,
                                   submitter_did.as_ptr(),
                                   inputs.as_ptr(),
                                   outputs.as_ptr(),
                                   cb)
        };

        utils::results::result_to_two(err, receiver)
    }

    pub fn parse_payment_response(payment_method: &str, resp_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let resp_json = CString::new(resp_json).unwrap();

        let err = unsafe {
            payments::indy_parse_payment_response(command_handle,
                                        payment_method.as_ptr(),
                                        resp_json.as_ptr(),
                                        cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn build_mint_req(wallet_handle: i32, submitter_did: &str, outputs_json: &str) -> Result<(String, String), ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let outputs_json = CString::new(outputs_json).unwrap();

        let err = unsafe {
            payments::indy_build_mint_req(command_handle,
                                wallet_handle,
                                submitter_did.as_ptr(),
                                outputs_json.as_ptr(),
                                cb)
        };

        utils::results::result_to_two(err, receiver)
    }

    pub fn build_set_txn_fees_req(wallet_handle: i32, submitter_did: &str, payment_method: &str, fees_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let payment_method = CString::new(payment_method).unwrap();
        let fees_json = CString::new(fees_json).unwrap();

        let err = unsafe {
            payments::indy_build_set_txn_fees_req(command_handle,
                                        wallet_handle,
                                        submitter_did.as_ptr(),
                                        payment_method.as_ptr(),
                                        fees_json.as_ptr(),
                                        cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn build_get_txn_fees_req(wallet_handle: i32, submitter_did: &str, payment_method: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string();

        let submitter_did = CString::new(submitter_did).unwrap();
        let payment_method = CString::new(payment_method).unwrap();

        let err = unsafe {
            payments::indy_build_get_txn_fees_req(command_handle,
                                        wallet_handle,
                                        submitter_did.as_ptr(),
                                        payment_method.as_ptr(),
                                        cb)
        };

        utils::results::result_to_one(err, receiver)
    }

    pub fn parse_get_txn_fees_response(payment_method: &str, resp_json: &str) -> Result<String, ErrorCode> {
        let (receiver, command_handle, cb) =
            utils::callbacks::_closure_to_cb_ec_string();

        let payment_method = CString::new(payment_method).unwrap();
        let resp_json = CString::new(resp_json).unwrap();

        let err = unsafe {
            payments::indy_parse_get_txn_fees_response(command_handle,
                                             payment_method.as_ptr(),
                                             resp_json.as_ptr(),
                                             cb)
        };

        utils::results::result_to_one(err, receiver)
    }
}
