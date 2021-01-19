use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::string::String;
use std::vec::Vec;

use hex;

use serde_json;

use indy_api_types::errors::prelude::*;
use crate::services::crypto::CryptoService;
use crate::services::ledger::LedgerService;
use crate::services::payments::{PaymentsMethodCBs, PaymentsService, RequesterInfo, Fees};
use indy_wallet::{RecordOptions, WalletService};
use indy_api_types::{WalletHandle, CommandHandle};
use crate::domain::ledger::auth_rule::AuthRule;
use crate::domain::crypto::did::DidValue;

use indy_utils::next_command_handle;
use crate::commands::BoxedCallbackStringStringSend;
use crate::services::metrics::MetricsService;

pub enum PaymentsCommand {
    RegisterMethod(
        String, //type
        PaymentsMethodCBs, //method callbacks
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    CreateAddress(
        WalletHandle,
        String, //type
        String, //config
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    CreateAddressAck(
        CommandHandle,
        WalletHandle,
        IndyResult<String /* address */>),
    ListAddresses(
        WalletHandle,
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    AddRequestFees(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //req
        String, //inputs
        String, //outputs
        Option<String>, //extra
        BoxedCallbackStringStringSend),
    AddRequestFeesAck(
        CommandHandle, //handle
        IndyResult<String>),
    ParseResponseWithFees(
        String, //type
        String, //response
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseResponseWithFeesAck(
        CommandHandle, //handle
        IndyResult<String>),
    BuildGetPaymentSourcesRequest(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //payment address
        Option<i64>, //from
        BoxedCallbackStringStringSend),
    BuildGetPaymentSourcesRequestAck(
        CommandHandle,
        IndyResult<String>),
    ParseGetPaymentSourcesResponse(
        String, //type
        String, //response
        Box<dyn Fn(IndyResult<(String, i64)>, Rc<MetricsService>) + Send>),
    ParseGetPaymentSourcesResponseAck(
        CommandHandle,
        IndyResult<(String, i64)>),
    BuildPaymentReq(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //inputs
        String, //outputs
        Option<String>, //extra
        BoxedCallbackStringStringSend),
    BuildPaymentReqAck(
        CommandHandle,
        IndyResult<String>),
    ParsePaymentResponse(
        String, //payment_method
        String, //response
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParsePaymentResponseAck(
        CommandHandle,
        IndyResult<String>),
    AppendTxnAuthorAgreementAcceptanceToExtra(
        Option<String>, // extra json
        Option<String>, // text
        Option<String>, // version
        Option<String>, // hash
        String, // acceptance mechanism type
        u64, // time of acceptance
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildMintReq(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //outputs
        Option<String>, //extra
        BoxedCallbackStringStringSend),
    BuildMintReqAck(
        CommandHandle,
        IndyResult<String>),
    BuildSetTxnFeesReq(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //method
        String, //fees
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildSetTxnFeesReqAck(
        CommandHandle,
        IndyResult<String>),
    BuildGetTxnFeesReq(
        WalletHandle,
        Option<DidValue>, //submitter did
        String, //method
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetTxnFeesReqAck(
        CommandHandle,
        IndyResult<String>),
    ParseGetTxnFeesResponse(
        String, //method
        String, //response
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetTxnFeesResponseAck(
        CommandHandle,
        IndyResult<String>),
    BuildVerifyPaymentReq(
        WalletHandle,
        Option<DidValue>, //submitter_did
        String, //receipt
        BoxedCallbackStringStringSend),
    BuildVerifyPaymentReqAck(
        CommandHandle,
        IndyResult<String>),
    ParseVerifyPaymentResponse(
        String, //payment_method
        String, //resp_json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseVerifyPaymentResponseAck(
        CommandHandle,
        IndyResult<String>),
    GetRequestInfo(
        String, // get auth rule response json
        RequesterInfo, //requester info
        Fees, //fees
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    SignWithAddressReq(
        WalletHandle,
        String, //address
        Vec<u8>, //message
        Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>),
    SignWithAddressAck(
        CommandHandle,
        IndyResult<Vec<u8>>),
    VerifyWithAddressReq(
        String, //address
        Vec<u8>, //message
        Vec<u8>, //signature
        Box<dyn Fn(IndyResult<bool>, Rc<MetricsService>) + Send>),
    VerifyWithAddressAck(
        CommandHandle,
        IndyResult<bool>)
}

pub struct PaymentsCommandExecutor {
    payments_service: Rc<PaymentsService>,
    wallet_service: Rc<WalletService>,
    crypto_service: Rc<CryptoService>,
    ledger_service: Rc<LedgerService>,
    metrics_service: Rc<MetricsService>,
    pending_callbacks_str: RefCell<HashMap<i32, Box<dyn Fn(IndyResult<String>, Rc<MetricsService>)>>>,
    pending_callbacks_str_i64: RefCell<HashMap<i32, Box<dyn Fn(IndyResult<(String, i64)>, Rc<MetricsService>)>>>,
    pending_array_callbacks: RefCell<HashMap<i32, Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>)>>>,
    pending_bool_callbacks: RefCell<HashMap<i32, Box<dyn Fn(IndyResult<bool>, Rc<MetricsService>)>>>,
}

impl PaymentsCommandExecutor {
    pub fn new(payments_service: Rc<PaymentsService>, wallet_service: Rc<WalletService>, crypto_service: Rc<CryptoService>, ledger_service: Rc<LedgerService>, metrics_service: Rc<MetricsService>) -> PaymentsCommandExecutor {
        PaymentsCommandExecutor {
            payments_service,
            wallet_service,
            crypto_service,
            ledger_service,
            metrics_service,
            pending_callbacks_str: RefCell::new(HashMap::new()),
            pending_callbacks_str_i64: RefCell::new(HashMap::new()),
            pending_array_callbacks: RefCell::new(HashMap::new()),
            pending_bool_callbacks: RefCell::new(HashMap::new())
        }
    }

    pub fn execute<'a>(&'a self, command: PaymentsCommand) {
        match command {
            PaymentsCommand::RegisterMethod(type_, method_cbs, cb) => {
                debug!(target: "payments_command_executor", "RegisterMethod command received");
                cb(self.register_method(&type_, method_cbs), self.metrics_service.clone());
            }
            PaymentsCommand::CreateAddress(wallet_handle, type_, config, cb) => {
                debug!(target: "payments_command_executor", "CreateAddress command received");
                self.create_address(wallet_handle, &type_, &config, cb);
            }
            PaymentsCommand::CreateAddressAck(handle, wallet_handle, result) => {
                debug!(target: "payments_command_executor", "CreateAddressAck command received");
                self.create_address_ack(handle, wallet_handle, result);
            }
            PaymentsCommand::ListAddresses(wallet_handle, cb) => {
                debug!(target: "payments_command_executor", "ListAddresses command received");
                self.list_addresses(wallet_handle, cb);
            }
            PaymentsCommand::AddRequestFees(wallet_handle, submitter_did, req, inputs, outputs, extra, cb) => {
                debug!(target: "payments_command_executor", "AddRequestFees command received");
                self.add_request_fees(wallet_handle, submitter_did.as_ref(), &req, &inputs, &outputs, extra.as_ref().map(String::as_str), cb);
            }
            PaymentsCommand::AddRequestFeesAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "AddRequestFeesAck command received");
                self.add_request_fees_ack(cmd_handle, result);
            }
            PaymentsCommand::ParseResponseWithFees(type_, response, cb) => {
                debug!(target: "payments_command_executor", "ParseResponseWithFees command received");
                self.parse_response_with_fees(&type_, &response, cb);
            }
            PaymentsCommand::ParseResponseWithFeesAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "ParseResponseWithFeesAck command received");
                self.parse_response_with_fees_ack(cmd_handle, result);
            }
            PaymentsCommand::BuildGetPaymentSourcesRequest(wallet_handle, submitter_did, payment_address, from, cb) => {
                debug!(target: "payments_command_executor", "BuildGetPaymentSourcesRequest command received");
                self.build_get_payment_sources_request(wallet_handle, submitter_did.as_ref(), &payment_address, from, cb);
            }
            PaymentsCommand::BuildGetPaymentSourcesRequestAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "BuildGetPaymentSourcesRequestAck command received");
                self.build_get_payment_sources_request_ack(cmd_handle, result);
            }
            PaymentsCommand::ParseGetPaymentSourcesResponse(type_, response, cb) => {
                debug!(target: "payments_command_executor", "ParseGetPaymentSourcesResponse command received");
                self.parse_get_payment_sources_response(&type_, &response, cb);
            }
            PaymentsCommand::ParseGetPaymentSourcesResponseAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "ParseGetPaymentSourcesResponseAck command received");
                self.parse_get_payment_sources_response_ack(cmd_handle, result);
            }
            PaymentsCommand::BuildPaymentReq(wallet_handle, submitter_did, inputs, outputs, extra, cb) => {
                debug!(target: "payments_command_executor", "BuildPaymentReq command received");
                self.build_payment_req(wallet_handle, submitter_did.as_ref(), &inputs, &outputs, extra.as_ref().map(String::as_str), cb);
            }
            PaymentsCommand::BuildPaymentReqAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "BuildPaymentReqAck command received");
                self.build_payment_req_ack(cmd_handle, result);
            }
            PaymentsCommand::ParsePaymentResponse(payment_method, response, cb) => {
                debug!(target: "payments_command_executor", "ParsePaymentResponse command received");
                self.parse_payment_response(&payment_method, &response, cb);
            }
            PaymentsCommand::ParsePaymentResponseAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "ParsePaymentResponseAck command received");
                self.parse_payment_response_ack(cmd_handle, result);
            }
            PaymentsCommand::AppendTxnAuthorAgreementAcceptanceToExtra(extra, text, version, taa_digest, mechanism, time, cb) => {
                debug!(target: "payments_command_executor", "AppendTxnAuthorAgreementAcceptanceToExtra command received");
                cb(self.append_txn_author_agreement_acceptance_to_extra(extra.as_ref().map(String::as_str),
                                                                        text.as_ref().map(String::as_str),
                                                                        version.as_ref().map(String::as_str),
                                                                        taa_digest.as_ref().map(String::as_str),
                                                                        &mechanism,
                                                                        time),
                    self.metrics_service.clone());
            }
            PaymentsCommand::BuildMintReq(wallet_handle, submitter_did, outputs, extra, cb) => {
                debug!(target: "payments_command_executor", "BuildMintReq command received");
                self.build_mint_req(wallet_handle, submitter_did.as_ref(), &outputs, extra.as_ref().map(String::as_str), cb);
            }
            PaymentsCommand::BuildMintReqAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "BuildMintReqAck command received");
                self.build_mint_req_ack(cmd_handle, result);
            }
            PaymentsCommand::BuildSetTxnFeesReq(wallet_handle, submitter_did, type_, fees, cb) => {
                debug!(target: "payments_command_executor", "BuildSetTxnFeesReq command received");
                self.build_set_txn_fees_req(wallet_handle, submitter_did.as_ref(), &type_, &fees, cb);
            }
            PaymentsCommand::BuildSetTxnFeesReqAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "BuildSetTxnFeesReqAck command received");
                self.build_set_txn_fees_req_ack(cmd_handle, result);
            }
            PaymentsCommand::BuildGetTxnFeesReq(wallet_handle, submitter_did, type_, cb) => {
                debug!(target: "payments_command_executor", "BuildGetTxnFeesReq command received");
                self.build_get_txn_fees_req(wallet_handle, submitter_did.as_ref(), &type_, cb);
            }
            PaymentsCommand::BuildGetTxnFeesReqAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "BuildGetTxnFeesReqAck command received");
                self.build_get_txn_fees_req_ack(cmd_handle, result);
            }
            PaymentsCommand::ParseGetTxnFeesResponse(type_, response, cb) => {
                debug!(target: "payments_command_executor", "ParseGetTxnFeesResponse command received");
                self.parse_get_txn_fees_response(&type_, &response, cb);
            }
            PaymentsCommand::ParseGetTxnFeesResponseAck(cmd_handle, result) => {
                debug!(target: "payments_command_executor", "ParseGetTxnFeesResponseAck command received");
                self.parse_get_txn_fees_response_ack(cmd_handle, result);
            }
            PaymentsCommand::BuildVerifyPaymentReq(wallet_handle, submitter_did, receipt, cb) => {
                debug!(target: "payments_command_executor", "BuildVerifyPaymentReq command received");
                self.build_verify_payment_request(wallet_handle, submitter_did.as_ref(), &receipt, cb);
            }
            PaymentsCommand::BuildVerifyPaymentReqAck(command_handle, result) => {
                debug!(target: "payments_command_executor", "BuildVerifyReqAck command received");
                self.build_verify_payment_request_ack(command_handle, result);
            }
            PaymentsCommand::ParseVerifyPaymentResponse(payment_method, resp_json, cb) => {
                debug!(target: "payments_command_executor", "ParseVerifyPaymentResponse command received");
                self.parse_verify_payment_response(&payment_method, &resp_json, cb);
            }
            PaymentsCommand::ParseVerifyPaymentResponseAck(command_handle, result) => {
                debug!(target: "payments_command_executor", "ParseVerifyResponseAck command received");
                self.parse_verify_payment_response_ack(command_handle, result);
            }
            PaymentsCommand::GetRequestInfo(get_auth_rule_response_json, requester_info, fees_json, cb) => {
                debug!(target: "payments_command_executor", "GetRequestInfo command received");
                cb(self.get_request_info(&get_auth_rule_response_json, requester_info, &fees_json), self.metrics_service.clone());
	        },
            PaymentsCommand::SignWithAddressReq(wallet_handle, address, message, cb) => {
                debug!(target: "payments_command_executor", "SignWithAddressReq command received");
                self.sign_with_address(wallet_handle, &address, message.as_slice(), cb);
            },
            PaymentsCommand::SignWithAddressAck(command_handle, result) => {
                debug!(target: "payments_command_executor", "SignWithAddressAck command received");
                self.sign_with_address_ack(command_handle, result);
            },
            PaymentsCommand::VerifyWithAddressReq(address, message, signature, cb) => {
                debug!(target: "payments_command_executor", "VerifyWithAddressReq command received");
                self.verify_with_address(&address, message.as_slice(), signature.as_slice(), cb);
            },
            PaymentsCommand::VerifyWithAddressAck(command_handle, result) => {
                 debug!(target: "payments_command_executor", "VerifyWithAddressAck command received");
                self.verify_with_address_ack(command_handle, result);
            }
        }
    }

    fn register_method(&self, type_: &str, methods: PaymentsMethodCBs) -> IndyResult<()> {
        trace!("register_method >>> type_: {:?}, methods: {:?}", type_, methods);

        self.payments_service.register_payment_method(type_, methods);
        let res = Ok(());

        trace!("register_method << res: {:?}", res);

        res
    }

    fn create_address(&self, wallet_handle: WalletHandle, type_: &str, config: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("create_address >>> wallet_handle: {:?}, type_: {:?}, config: {:?}", wallet_handle, type_, config);

        match self.wallet_service.check(wallet_handle).map_err(map_err_err!()) {
            Err(err) => return cb(Err(err), self.metrics_service.clone()),
            _ => ()
        };
        self._process_method_str(cb, &|i| self.payments_service.create_address(i, wallet_handle, type_, config));

        trace!("create_address <<<");
    }

    fn create_address_ack(&self, handle: CommandHandle, wallet_handle: WalletHandle, result: IndyResult<String>) {
        trace!("create_address_ack >>> wallet_handle: {:?}, result: {:?}", wallet_handle, result);
        let total_result: IndyResult<String> = match result {
            Ok(res) => {
                //TODO: think about deleting payment_address on wallet save failure
                self.wallet_service.add_record(wallet_handle, &self.wallet_service.add_prefix("PaymentAddress"), &res, &res, &HashMap::new()).map(|_| res)
                    .map_err(IndyError::from)
            }
            Err(err) => Err(err)
        };
        self._common_ack_str(handle, total_result, "CreateAddressAck");
        trace!("create_address_ack <<<");
    }

    fn list_addresses(&self, wallet_handle: WalletHandle, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("list_addresses >>> wallet_handle: {:?}", wallet_handle);

        match self.wallet_service.search_records(wallet_handle, &self.wallet_service.add_prefix("PaymentAddress"), "{}", &RecordOptions::id_value()) {
            Ok(mut search) => {
                let mut list_addresses: Vec<String> = Vec::new();

                while let Ok(Some(payment_address)) = search.fetch_next_record() {
                    match payment_address.get_value() {
                        Some(value) => list_addresses.push(value.to_string()),
                        None => cb(Err(err_msg(IndyErrorKind::InvalidState, "Record value not found")), self.metrics_service.clone())
                    }
                }

                let json_string = serde_json::to_string(&list_addresses)
                    .to_indy(IndyErrorKind::InvalidState, "Cannot deserialize List of Payment Addresses");

                cb(json_string, self.metrics_service.clone());
            }
            Err(err) => cb(Err(err), self.metrics_service.clone())
        }
        trace!("list_addresses <<<");
    }

    fn add_request_fees(&self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, req: &str, inputs: &str, outputs: &str, extra: Option<&str>, cb: BoxedCallbackStringStringSend) {
        trace!("add_request_fees >>> wallet_handle: {:?}, submitter_did: {:?}, req: {:?}, inputs: {:?}, outputs: {:?}, extra: {:?}",
               wallet_handle, submitter_did, req, inputs, outputs, extra);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        let method_from_inputs = self.payments_service.parse_method_from_inputs(inputs);

        let method = if outputs == "[]" {
            method_from_inputs
        } else {
            let method_from_outputs = self.payments_service.parse_method_from_outputs(outputs);
            PaymentsCommandExecutor::_merge_parse_result(method_from_inputs, method_from_outputs)
        };

        match method {
            Ok(type_) => {
                let type_copy = type_.to_string();
                self._process_method_str(
                    Box::new(move |result, metrics_service| cb(result.map(|e| (e, type_.to_string())), metrics_service)),
                    &|i| self.payments_service.add_request_fees(i, &type_copy, wallet_handle, submitter_did, req, inputs, outputs, extra),
                );
            }
            Err(error) => {
                cb(Err(error), self.metrics_service.clone())
            }
        };
        trace!("add_request_fees <<<");
    }

    fn add_request_fees_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("add_request_fees_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "AddRequestFeesAck");
        trace!("add_request_fees_ack <<<");
    }

    fn parse_response_with_fees(&self, type_: &str, response: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("parse_response_with_fees >>> type_: {:?}, response: {:?}", type_, response);
        self._process_method_str(cb, &|i| self.payments_service.parse_response_with_fees(i, type_, response));
        trace!("parse_response_with_fees <<<");
    }

    fn parse_response_with_fees_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("parse_response_with_fees_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "ParseResponseWithFeesFeesAck");
        trace!("parse_response_with_fees_ack <<<");
    }

    fn build_get_payment_sources_request<'a>(&'a self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, payment_address: &str, next: Option<i64>, cb: BoxedCallbackStringStringSend) {
        trace!("build_get_payment_sources_request >>> wallet_handle: {:?}, submitter_did: {:?}, payment_address: {:?}", wallet_handle, submitter_did, payment_address);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        let method = match self.payments_service.parse_method_from_payment_address(payment_address) {
            Ok(method) => method,
            Err(err) => {
                cb(Err(err), self.metrics_service.clone());
                return;
            }
        };
        let method_copy = method.to_string();

        self._process_method_str(
            Box::new(move |get_sources_txn_json, metrics_service: Rc<MetricsService>| cb(get_sources_txn_json.map(|s| (s, method.to_string())), metrics_service)),
            &|i| self.payments_service.build_get_payment_sources_request(i, &method_copy, wallet_handle, submitter_did, payment_address, next),
        );
        trace!("build_get_payment_sources_request <<<");
    }

    fn build_get_payment_sources_request_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_get_payment_sources_request_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildGetSourcesRequestAck");
        trace!("build_get_payment_sources_request_ack <<<");
    }

    fn parse_get_payment_sources_response(&self, type_: &str, response: &str, cb: Box<dyn Fn(IndyResult<(String, i64)>, Rc<MetricsService>) + Send>) {
        trace!("parse_get_payment_sources_response >>> response: {:?}", response);
        self._process_method_str_i64(cb, &|i| self.payments_service.parse_get_payment_sources_response(i, type_, response));
        trace!("parse_get_payment_sources_response <<<");
    }

    fn parse_get_payment_sources_response_ack(&self, cmd_handle: CommandHandle, result: IndyResult<(String, i64)>) {
        trace!("parse_get_payment_sources_response_ack >>> result: {:?}", result);
        self._common_ack_payments_str_i64(cmd_handle, result, "ParseGetSourcesResponseAck");
        trace!("parse_get_payment_sources_response_ack <<<");
    }

    fn build_payment_req<'a>(&'a self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, inputs: &str, outputs: &str, extra: Option<&str>, cb: BoxedCallbackStringStringSend) {
        trace!("build_payment_req >>> wallet_handle: {:?}, submitter_did: {:?}, inputs: {:?}, outputs: {:?}, extra: {:?}", wallet_handle, submitter_did, inputs, outputs, extra);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        let method_from_inputs = self.payments_service.parse_method_from_inputs(inputs);
        let method_from_outputs = self.payments_service.parse_method_from_outputs(outputs);
        let method = PaymentsCommandExecutor::_merge_parse_result(method_from_inputs, method_from_outputs);

        match method {
            Ok(type_) => {
                let type_copy = type_.to_string();
                self._process_method_str(
                    Box::new(move |result, metrics_service| cb(result.map(|s| (s, type_.to_string())), metrics_service)),
                    &|i| self.payments_service.build_payment_req(i, &type_copy, wallet_handle, submitter_did, inputs, outputs, extra),
                );
            }
            Err(error) => {
                cb(Err(error), self.metrics_service.clone())
            }
        }
        trace!("build_payment_req <<<");
    }

    fn append_txn_author_agreement_acceptance_to_extra(&self,
                                                       extra: Option<&str>,
                                                       text: Option<&str>,
                                                       version: Option<&str>,
                                                       taa_digest: Option<&str>,
                                                       mechanism: &str,
                                                       time: u64) -> IndyResult<String> {
        debug!("append_txn_author_agreement_acceptance_to_extra >>> extra: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, mechanism: {:?}, time: {:?}",
               extra, text, version, taa_digest, mechanism, time);

        let mut extra: serde_json::Value = serde_json::from_str(extra.unwrap_or("{}"))
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot deserialize extra: {:?}", err)))?;

        let acceptance_data = self.ledger_service.prepare_acceptance_data(text, version, taa_digest, mechanism, time)?;

        extra["taaAcceptance"] = serde_json::to_value(acceptance_data)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize author agreement acceptance data")?;

        let res: String = extra.to_string();

        debug!("append_txn_author_agreement_acceptance_to_extra <<< res: {:?}", res);

        Ok(res)
    }

    fn build_payment_req_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_payment_req_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildPaymentReqAck");
        trace!("build_payment_req_ack <<<");
    }

    fn parse_payment_response(&self, payment_method: &str, response: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("parse_payment_response >>> response: {:?}", response);
        self._process_method_str(cb, &|i| self.payments_service.parse_payment_response(i, payment_method, response));
        trace!("parse_payment_response <<<");
    }

    fn parse_payment_response_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("parse_payment_response_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "ParsePaymentResponseAck");
        trace!("parse_payment_response_ack <<<");
    }

    fn build_mint_req<'a>(&'a self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, outputs: &str, extra: Option<&str>, cb: BoxedCallbackStringStringSend) {
        trace!("build_mint_req >>> wallet_handle: {:?}, submitter_did: {:?}, outputs: {:?}, extra: {:?}", wallet_handle, submitter_did, outputs, extra);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        match self.payments_service.parse_method_from_outputs(outputs) {
            Ok(type_) => {
                let type_copy = type_.to_string();
                self._process_method_str(
                    Box::new(move |result, metrics_service| cb(result.map(|s| (s, type_.to_string())), metrics_service)),
                    &|i| self.payments_service.build_mint_req(i, &type_copy, wallet_handle, submitter_did, outputs, extra),
                );
            }
            Err(error) => cb(Err(error), self.metrics_service.clone())
        }
        trace!("build_mint_req <<<");
    }

    fn build_mint_req_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_mint_req_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildMintReqAck");
        trace!("build_mint_req_ack <<<");
    }

    fn build_set_txn_fees_req(&self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, type_: &str, fees: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("build_set_txn_fees_req >>> wallet_handle: {:?}, submitter_did: {:?}, type_: {:?}, fees: {:?}", wallet_handle, submitter_did, type_, fees);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        match serde_json::from_str::<HashMap<String, i64>>(fees) {
            Err(err) => {
                error!("Cannot deserialize Fees: {:?}", err);
                cb(Err(err.to_indy(IndyErrorKind::InvalidStructure, "Cannot deserialize Fees")), self.metrics_service.clone())
            }
            _ => self._process_method_str(cb, &|i| self.payments_service.build_set_txn_fees_req(i, type_, wallet_handle, submitter_did, fees)),
        };
        trace!("build_set_txn_fees_req <<<");
    }

    fn build_set_txn_fees_req_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_set_txn_fees_req_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildSetTxnFeesReq");
        trace!("build_set_txn_fees_req_ack <<<");
    }

    fn build_get_txn_fees_req(&self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, type_: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("build_get_txn_fees_req >>> wallet_handle: {:?}, submitter_did: {:?}, type_: {:?}", wallet_handle, submitter_did, type_);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        self._process_method_str(cb, &|i| self.payments_service.build_get_txn_fees_req(i, type_, wallet_handle, submitter_did));
        trace!("build_get_txn_fees_req <<<");
    }

    fn build_get_txn_fees_req_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_get_txn_fees_req_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildGetTxnFeesReqAck");
        trace!("build_get_txn_fees_req_ack <<<");
    }

    fn parse_get_txn_fees_response(&self, type_: &str, response: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("parse_get_txn_fees_response >>> response: {:?}", response);
        self._process_method_str(cb, &|i| self.payments_service.parse_get_txn_fees_response(i, type_, response));
        trace!("parse_get_txn_fees_response <<<");
    }

    fn parse_get_txn_fees_response_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("parse_get_txn_fees_response_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "ParseGetTxnFeesResponseAck");
        trace!("parse_get_txn_fees_response_ack <<<");
    }

    fn build_verify_payment_request(&self, wallet_handle: WalletHandle, submitter_did: Option<&DidValue>, receipt: &str, cb: BoxedCallbackStringStringSend) {
        trace!("build_verify_payment_request >>> wallet_handle: {:?}, submitter_did: {:?}, receipt: {:?}", wallet_handle, submitter_did, receipt);
        if let Some(ref did) = submitter_did {
            match self.crypto_service.validate_did(did).map_err(map_err_err!()) {
                Err(err) => return cb(Err(err), self.metrics_service.clone()),
                _ => ()
            }
        }

        let method = match self.payments_service.parse_method_from_payment_address(receipt) {
            Ok(method) => method,
            Err(err) => {
                cb(Err(err), self.metrics_service.clone());
                return;
            }
        };
        let method_copy = method.to_string();
        self._process_method_str(
            Box::new(move |result, metrics_service| cb(result.map(|s| (s, method.to_string())), metrics_service)),
            &|i| self.payments_service.build_verify_payment_req(i, &method_copy, wallet_handle, submitter_did, receipt),
        );
        trace!("build_verify_payment_request <<<");
    }

    fn build_verify_payment_request_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("build_verify_payment_request_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "BuildVerifyPaymentReqAck");
        trace!("build_verify_payment_request_ack <<<");
    }

    fn parse_verify_payment_response(&self, type_: &str, resp_json: &str, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("parse_verify_payment_response >>> response: {:?}", resp_json);
        self._process_method_str(cb, &|i| self.payments_service.parse_verify_payment_response(i, type_, resp_json));
        trace!("parse_verify_payment_response <<<");
    }

    fn parse_verify_payment_response_ack(&self, cmd_handle: CommandHandle, result: IndyResult<String>) {
        trace!("parse_verify_payment_response_ack >>> result: {:?}", result);
        self._common_ack_payments_str(cmd_handle, result, "ParseVerifyPaymentResponseAck");
        trace!("parse_verify_payment_response_ack <<<");
    }

    fn sign_with_address(&self, wallet_handle: WalletHandle, address: &str, message: &[u8], cb: Box<dyn Fn(IndyResult<Vec<u8>>, Rc<MetricsService>) + Send>) {
        trace!("sign_with_address >>> address: {:?}, message: {:?}", address, hex::encode(message));
        let method = match self.payments_service.parse_method_from_payment_address(address) {
            Ok(method) => method,
            Err(err) => {
                cb(Err(err), self.metrics_service.clone());
                return;
            }
        };
        let cmd_handle = next_command_handle();

        if let Err(err) = self.payments_service.sign_with_address(cmd_handle, &method, wallet_handle, address, message) {
            cb(Err(err), self.metrics_service.clone());
        } else {
            self.pending_array_callbacks.borrow_mut().insert(cmd_handle, cb);
        }
    }

    fn sign_with_address_ack(&self, command_handle: CommandHandle, result: IndyResult<Vec<u8>>) {
        trace!("sign_with_address_ack >>> result: {:?}", result);
        match self.pending_array_callbacks.borrow_mut().remove(&command_handle) {
            Some(cb) => cb(result, self.metrics_service.clone()),
            None => error!("Can't process PaymentsCommand::SignWithAddressAck for handle {} with result {:?} - appropriate callback not found!", command_handle, result)
        };
        trace!("sign_with_address_ack <<<");
    }

    fn verify_with_address(&self, address: &str, message: &[u8], signature: &[u8], cb: Box<dyn Fn(IndyResult<bool>, Rc<MetricsService>) + Send>) {
        trace!("sign_with_address >>> address: {:?}, message: {:?}, signature: {:?}", address, hex::encode(message), hex::encode(signature));

        let method = match self.payments_service.parse_method_from_payment_address(address) {
            Ok(method) => method,
            Err(err) => {
                cb(Err(err), self.metrics_service.clone());
                return;
            }
        };

        let cmd_handle = next_command_handle();

        if let Err(err) = self.payments_service.verify_with_address(cmd_handle, &method, address, message, signature) {
            cb(Err(err), self.metrics_service.clone())
        } else {
            self.pending_bool_callbacks.borrow_mut().insert(cmd_handle, cb);
        }
    }

    fn verify_with_address_ack(&self, command_handle: CommandHandle, result: IndyResult<bool>) {
        trace!("verify_with_address_ack >>> result: {:?}", result);
        match self.pending_bool_callbacks.borrow_mut().remove(&command_handle) {
            Some(cb) => cb(result, self.metrics_service.clone()),
            None => error!("Can't process PaymentsCommand::VerifyWithAddressAck for handle {} with result {:?} - appropriate callback not found!", command_handle, result)
        };
        trace!("verify_with_address_ack <<<");
    }

    // HELPERS

    fn _process_method_str(&self, cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>)>,
                           method: &dyn Fn(CommandHandle) -> IndyResult<()>) {
        let cmd_handle = next_command_handle();
        match method(cmd_handle) {
            Ok(()) => {
                self.pending_callbacks_str.borrow_mut().insert(cmd_handle, cb);
            }
            Err(err) => cb(Err(err), self.metrics_service.clone())
        }
    }

    fn _process_method_str_i64(&self, cb: Box<dyn Fn(IndyResult<(String, i64)>, Rc<MetricsService>) + Send>,
                           method: &dyn Fn(CommandHandle) -> IndyResult<()>) {
        let cmd_handle = next_command_handle();
        match method(cmd_handle) {
            Ok(()) => {
                self.pending_callbacks_str_i64.borrow_mut().insert(cmd_handle, cb);
            }
            Err(err) => cb(Err(err), self.metrics_service.clone())
        }
    }

    fn _common_ack_payments_str(&self, cmd_handle: CommandHandle, result: IndyResult<String>, name: &str) {
        self._common_ack_str(cmd_handle, result.map_err(IndyError::from), name)
    }

    fn _common_ack_str(&self, cmd_handle: CommandHandle, result: IndyResult<String>, name: &str) {
        match self.pending_callbacks_str.borrow_mut().remove(&cmd_handle) {
            Some(cb) => {
                cb(result, self.metrics_service.clone())
            }
            None => error!("Can't process PaymentsCommand::{} for handle {} with result {:?} - appropriate callback not found!",
                           name, cmd_handle, result),
        }
    }

    fn _common_ack_payments_str_i64(&self, cmd_handle: CommandHandle, result: IndyResult<(String, i64)>, name: &str) {
        self._common_ack_str_i64(cmd_handle, result.map_err(IndyError::from), name)
    }

    fn _common_ack_str_i64(&self, cmd_handle: CommandHandle, result: IndyResult<(String, i64)>, name: &str) {
        match self.pending_callbacks_str_i64.borrow_mut().remove(&cmd_handle) {
            Some(cb) => {
                cb(result, self.metrics_service.clone())
            }
            None => error!("Can't process PaymentsCommand::{} for handle {:?} with result {:?} - appropriate callback not found!",
                           name, cmd_handle, result),
        }
    }

    fn _merge_parse_result(method_from_inputs: IndyResult<String>, method_from_outputs: IndyResult<String>) -> IndyResult<String> {
        match (method_from_inputs, method_from_outputs) {
            (Err(err), _) | (_, Err(err)) => Err(err),
            (Ok(ref mth1), Ok(ref mth2)) if mth1 != mth2 => {
                error!("Different payment method in inputs and outputs");
                Err(err_msg(IndyErrorKind::IncompatiblePaymentMethods, "Different payment method in inputs and outputs"))
            }
            (Ok(mth1), Ok(_)) => Ok(mth1)
        }
    }

    pub fn get_request_info(&self, get_auth_rule_response_json: &str, requester_info: RequesterInfo, fees: &Fees) -> IndyResult<String> {
        trace!("get_request_info >>> get_auth_rule_response_json: {:?}, requester_info: {:?}, fees: {:?}", get_auth_rule_response_json, requester_info, fees);

        let auth_rule = self._parse_get_auth_rule_response(get_auth_rule_response_json)?;

        let req_info = self.payments_service.get_request_info_with_min_price(&auth_rule.constraint, &requester_info, &fees)?;

        let res = serde_json::to_string(&req_info)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RequestInfo")?;

        trace!("get_request_info <<< {:?}", res);

        Ok(res)
    }

    fn _parse_get_auth_rule_response(&self, get_auth_rule_response_json: &str) -> IndyResult<AuthRule> {
        trace!("_parse_get_auth_rule_response >>> get_auth_rule_response_json: {:?}", get_auth_rule_response_json);

        let mut auth_rules: Vec<AuthRule> = self.ledger_service.parse_get_auth_rule_response(get_auth_rule_response_json)?;

        if auth_rules.len() != 1 {
            return Err(IndyError::from_msg(IndyErrorKind::InvalidTransaction, "GetAuthRule response must contain one auth rule"));
        }

        let res = auth_rules.pop().unwrap();

        trace!("_parse_get_auth_rule_response <<< {:?}", res);

        Ok(res)
    }
}
