use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::string::ToString;

use indy_api_types::{CommandHandle, PoolHandle, WalletHandle};
use indy_api_types::errors::prelude::*;
use indy_utils::next_command_handle;
use rust_base58::ToBase58;
use serde_json;
use serde_json::Value;

use indy_wallet::{RecordOptions, WalletService};

use crate::api::ledger::{CustomFree, CustomTransactionParser};
use crate::commands::{BoxedCallbackStringStringSend, Command, CommandExecutor};
use crate::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionId, CredentialDefinitionV1};
use crate::domain::anoncreds::revocation_registry_definition::{RevocationRegistryDefinition, RevocationRegistryDefinitionV1, RevocationRegistryId};
use crate::domain::anoncreds::revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1};
use crate::domain::anoncreds::schema::{Schema, SchemaId, SchemaV1};
use crate::domain::crypto::did::{Did, DidValue};
use crate::domain::crypto::key::Key;
use crate::domain::ledger::auth_rule::{AuthRules, Constraint};
use crate::domain::ledger::author_agreement::{AcceptanceMechanisms, GetTxnAuthorAgreementData};
use crate::domain::ledger::node::NodeOperationData;
use crate::domain::ledger::pool::Schedule;
use crate::domain::ledger::request::Request;
use crate::services::crypto::CryptoService;
use crate::services::ledger::LedgerService;
use crate::services::pool::{
    parse_response_metadata,
    PoolService
};
use crate::utils::crypto::signature_serializer::serialize_signature;
use crate::services::metrics::MetricsService;

pub enum LedgerCommand {
    SignAndSubmitRequest(
        PoolHandle, // pool handle
        WalletHandle,
        DidValue, // submitter did
        String, // request json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    SubmitRequest(
        PoolHandle, // pool handle
        String, // request json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    SubmitAck(
        CommandHandle,
        IndyResult<String>, // result json or error
    ),
    SubmitAction(
        PoolHandle, // pool handle
        String, // request json
        Option<String>, // nodes
        Option<i32>, // timeout
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    SignRequest(
        WalletHandle,
        DidValue, // submitter did
        String, // request json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    MultiSignRequest(
        WalletHandle,
        DidValue, // submitter did
        String, // request json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetDdoRequest(
        Option<DidValue>, // submitter did
        DidValue, // target did
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildNymRequest(
        DidValue, // submitter did
        DidValue, // target did
        Option<String>, // verkey
        Option<String>, // alias
        Option<String>, // role
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildAttribRequest(
        DidValue, // submitter did
        DidValue, // target did
        Option<String>, // hash
        Option<serde_json::Value>, // raw
        Option<String>, // enc
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetAttribRequest(
        Option<DidValue>, // submitter did
        DidValue, // target did
        Option<String>, // raw
        Option<String>, // hash
        Option<String>, // enc
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetNymRequest(
        Option<DidValue>, // submitter did
        DidValue, // target did
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetNymResponse(
        String, // get nym response json
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildSchemaRequest(
        DidValue, // submitter did
        Schema, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetSchemaRequest(
        Option<DidValue>, // submitter did
        SchemaId, // id
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetSchemaResponse(
        String, // get schema response json
        BoxedCallbackStringStringSend),
    BuildCredDefRequest(
        DidValue, // submitter did
        CredentialDefinition, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetCredDefRequest(
        Option<DidValue>, // submitter did
        CredentialDefinitionId, // id
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetCredDefResponse(
        String, // get cred definition response
        BoxedCallbackStringStringSend),
    BuildNodeRequest(
        DidValue, // submitter did
        DidValue, // target_did
        NodeOperationData, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetValidatorInfoRequest(
        DidValue, // submitter did
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetTxnRequest(
        Option<DidValue>, // submitter did
        Option<String>, // ledger type
        i32, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildPoolConfigRequest(
        DidValue, // submitter did
        bool, // writes
        bool, // force
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildPoolRestartRequest(
        DidValue, //submitter did
        String, //action
        Option<String>, //datetime
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildPoolUpgradeRequest(
        DidValue, // submitter did
        String, // name
        String, // version
        String, // action
        String, // sha256
        Option<u32>, // timeout
        Option<Schedule>, // schedule
        Option<String>, // justification
        bool, // reinstall
        bool, // force
        Option<String>, // package
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildRevocRegDefRequest(
        DidValue, // submitter did
        RevocationRegistryDefinition, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetRevocRegDefRequest(
        Option<DidValue>, // submitter did
        RevocationRegistryId, // revocation registry definition id
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetRevocRegDefResponse(
        String, // get revocation registry definition response
        BoxedCallbackStringStringSend),
    BuildRevocRegEntryRequest(
        DidValue, // submitter did
        RevocationRegistryId, // revocation registry definition id
        String, // revocation registry definition type
        RevocationRegistryDelta, // value
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetRevocRegRequest(
        Option<DidValue>, // submitter did
        RevocationRegistryId, // revocation registry definition id
        i64, // timestamp
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetRevocRegResponse(
        String, // get revocation registry response
        Box<dyn Fn(IndyResult<(String, String, u64)>, Rc<MetricsService>) + Send>),
    BuildGetRevocRegDeltaRequest(
        Option<DidValue>, // submitter did
        RevocationRegistryId, // revocation registry definition id
        Option<i64>, // from
        i64, // to
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    ParseGetRevocRegDeltaResponse(
        String, // get revocation registry delta response
        Box<dyn Fn(IndyResult<(String, String, u64)>, Rc<MetricsService>) + Send>),
    RegisterSPParser(
        String, // txn type
        CustomTransactionParser,
        CustomFree,
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    GetResponseMetadata(
        String, // response
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildAuthRuleRequest(
        DidValue, // submitter did
        String, // auth type
        String, // auth action
        String, // field
        Option<String>, // old value
        Option<String>, // new value
        Constraint, // constraint
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildAuthRulesRequest(
        DidValue, // submitter did
        AuthRules, // auth rules
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetAuthRuleRequest(
        Option<DidValue>, // submitter did
        Option<String>, // auth type
        Option<String>, // auth action
        Option<String>, // field
        Option<String>, // old value
        Option<String>, // new value
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    GetSchema(
        PoolHandle,
        Option<DidValue>,
        SchemaId,
        BoxedCallbackStringStringSend,
    ),
    GetSchemaContinue(
        SchemaId,
        IndyResult<String>,
        CommandHandle,
    ),
    GetCredDef(
        PoolHandle,
        Option<DidValue>,
        CredentialDefinitionId,
        BoxedCallbackStringStringSend,
    ),
    GetCredDefContinue(
        CredentialDefinitionId,
        IndyResult<String>,
        CommandHandle,
    ),
    BuildTxnAuthorAgreementRequest(
        DidValue, // submitter did
        Option<String>, // text
        String, // version
        Option<u64>,   // ratification date
        Option<u64>,   // retirement date
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildDisableAllTxnAuthorAgreementsRequest(
        DidValue, // submitter did
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetTxnAuthorAgreementRequest(
        Option<DidValue>, // submitter did
        Option<GetTxnAuthorAgreementData>, // data
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildAcceptanceMechanismRequests(
        DidValue, // submitter did
        AcceptanceMechanisms, // aml
        String, // version
        Option<String>, // aml context
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    BuildGetAcceptanceMechanismsRequest(
        Option<DidValue>, // submitter did
        Option<u64>, // timestamp
        Option<String>, // version
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    AppendTxnAuthorAgreementAcceptanceToRequest(
        String, // request json
        Option<String>, // text
        Option<String>, // version
        Option<String>, // hash
        String, // acceptance mechanism type
        u64, // time of acceptance
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    AppendRequestEndorser(
        String, // request json
        DidValue, // endorser did
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
}

pub struct LedgerCommandExecutor {
    pool_service: Rc<PoolService>,
    crypto_service: Rc<CryptoService>,
    wallet_service: Rc<WalletService>,
    ledger_service: Rc<LedgerService>,
    metrics_service: Rc<MetricsService>,

    send_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<String>, Rc<MetricsService>)>>>,
    pending_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<(String, String)>, Rc<MetricsService>)>>>,
}

impl LedgerCommandExecutor {
    pub fn new(pool_service: Rc<PoolService>,
               crypto_service: Rc<CryptoService>,
               wallet_service: Rc<WalletService>,
               ledger_service: Rc<LedgerService>,
               metrics_service: Rc<MetricsService>) -> LedgerCommandExecutor {
        LedgerCommandExecutor {
            pool_service,
            crypto_service,
            wallet_service,
            ledger_service,
            metrics_service,
            send_callbacks: RefCell::new(HashMap::new()),
            pending_callbacks: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: LedgerCommand) {
        match command {
            LedgerCommand::SignAndSubmitRequest(pool_handle, wallet_handle, submitter_did, request_json, cb) => {
                debug!(target: "ledger_command_executor", "SignAndSubmitRequest command received");
                self.sign_and_submit_request(pool_handle, wallet_handle, &submitter_did, &request_json, cb);
            }
            LedgerCommand::SubmitRequest(handle, request_json, cb) => {
                debug!(target: "ledger_command_executor", "SubmitRequest command received");
                self.submit_request(handle, &request_json, cb);
            }
            LedgerCommand::SubmitAck(handle, result) => {
                debug!(target: "ledger_command_executor", "SubmitAck command received");
                match self.send_callbacks.borrow_mut().remove(&handle) {
                    Some(cb) => cb(result.map_err(IndyError::from), self.metrics_service.clone()),
                    None => {
                        error!("Can't process LedgerCommand::SubmitAck for handle {:?} with result {:?} - appropriate callback not found!",
                               handle, result);
                    }
                }
            }
            LedgerCommand::SubmitAction(handle, request_json, nodes, timeout, cb) => {
                debug!(target: "ledger_command_executor", "SubmitRequest command received");
                self.submit_action(handle, &request_json, nodes.as_ref().map(String::as_str), timeout, cb);
            }
            LedgerCommand::RegisterSPParser(txn_type, parser, free, cb) => {
                debug!(target: "ledger_command_executor", "RegisterSPParser command received");
                cb(self.register_sp_parser(&txn_type, parser, free), self.metrics_service.clone());
            }
            LedgerCommand::SignRequest(wallet_handle, submitter_did, request_json, cb) => {
                debug!(target: "ledger_command_executor", "SignRequest command received");
                cb(self.sign_request(wallet_handle, &submitter_did, &request_json), self.metrics_service.clone());
            }
            LedgerCommand::MultiSignRequest(wallet_handle, submitter_did, request_json, cb) => {
                debug!(target: "ledger_command_executor", "MultiSignRequest command received");
                cb(self.multi_sign_request(wallet_handle, &submitter_did, &request_json), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetDdoRequest(submitter_did, target_did, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetDdoRequest command received");
                cb(self.build_get_ddo_request(submitter_did.as_ref(), &target_did), self.metrics_service.clone());
            }
            LedgerCommand::BuildNymRequest(submitter_did, target_did, verkey, alias, role, cb) => {
                debug!(target: "ledger_command_executor", "BuildNymRequest command received");
                cb(self.build_nym_request(&submitter_did, &target_did,
                                          verkey.as_ref().map(String::as_str),
                                          alias.as_ref().map(String::as_str),
                                          role.as_ref().map(String::as_str)),
                   self.metrics_service.clone());
            }
            LedgerCommand::BuildAttribRequest(submitter_did, target_did, hash, raw, enc, cb) => {
                debug!(target: "ledger_command_executor", "BuildAttribRequest command received");
                cb(self.build_attrib_request(&submitter_did, &target_did,
                                             hash.as_ref().map(String::as_str),
                                             raw.as_ref(),
                                             enc.as_ref().map(String::as_str)),
                    self.metrics_service.clone());
            }
            LedgerCommand::BuildGetAttribRequest(submitter_did, target_did, raw, hash, enc, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetAttribRequest command received");
                cb(self.build_get_attrib_request(submitter_did.as_ref(), &target_did,
                                                 raw.as_ref().map(String::as_str),
                                                 hash.as_ref().map(String::as_str),
                                                 enc.as_ref().map(String::as_str)),
                    self.metrics_service.clone());
            }
            LedgerCommand::BuildGetNymRequest(submitter_did, target_did, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetNymRequest command received");
                cb(self.build_get_nym_request(submitter_did.as_ref(), &target_did), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetNymResponse(get_nym_response, cb) => {
                info!(target: "ledger_command_executor", "ParseGetNymResponse command received");
                cb(self.parse_get_nym_response(&get_nym_response), self.metrics_service.clone());
            }
            LedgerCommand::BuildSchemaRequest(submitter_did, data, cb) => {
                debug!(target: "ledger_command_executor", "BuildSchemaRequest command received");
                cb(self.build_schema_request(&submitter_did, SchemaV1::from(data)), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetSchemaRequest(submitter_did, id, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetSchemaRequest command received");
                cb(self.build_get_schema_request(submitter_did.as_ref(), &id), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetSchemaResponse(get_schema_response, cb) => {
                debug!(target: "ledger_command_executor", "ParseGetSchemaResponse command received");
                cb(self.parse_get_schema_response(&get_schema_response), self.metrics_service.clone());
            }
            LedgerCommand::BuildCredDefRequest(submitter_did, data, cb) => {
                debug!(target: "ledger_command_executor", "BuildCredDefRequest command received");
                cb(self.build_cred_def_request(&submitter_did, CredentialDefinitionV1::from(data)), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetCredDefRequest(submitter_did, id, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetCredDefRequest command received");
                cb(self.build_get_cred_def_request(submitter_did.as_ref(), &id), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetCredDefResponse(get_cred_def_response, cb) => {
                debug!(target: "ledger_command_executor", "ParseGetCredDefResponse command received");
                cb(self.parse_get_cred_def_response(&get_cred_def_response), self.metrics_service.clone());
            }
            LedgerCommand::BuildNodeRequest(submitter_did, target_did, data, cb) => {
                debug!(target: "ledger_command_executor", "BuildNodeRequest command received");
                cb(self.build_node_request(&submitter_did, &target_did, data), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetValidatorInfoRequest(submitter_did, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetValidatorInfoRequest command received");
                cb(self.build_get_validator_info_request(&submitter_did), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetTxnRequest(submitter_did, ledger_type, seq_no, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetTxnRequest command received");
                cb(self.build_get_txn_request(submitter_did.as_ref(), ledger_type.as_ref().map(String::as_str), seq_no), self.metrics_service.clone());
            }
            LedgerCommand::BuildPoolConfigRequest(submitter_did, writes, force, cb) => {
                debug!(target: "ledger_command_executor", "BuildPoolConfigRequest command received");
                cb(self.build_pool_config_request(&submitter_did, writes, force), self.metrics_service.clone());
            }
            LedgerCommand::BuildPoolRestartRequest(submitter_did, action, datetime, cb) => {
                debug!(target: "ledger_command_executor", "BuildPoolRestartRequest command received");
                cb(self.build_pool_restart_request(&submitter_did, &action, datetime.as_ref().map(String::as_str)), self.metrics_service.clone());
            }
            LedgerCommand::BuildPoolUpgradeRequest(submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package, cb) => {
                debug!(target: "ledger_command_executor", "BuildPoolUpgradeRequest command received");
                cb(self.build_pool_upgrade_request(&submitter_did, &name, &version, &action, &sha256, timeout,
                                                   schedule,
                                                   justification.as_ref().map(String::as_str),
                                                   reinstall, force, package.as_ref().map(String::as_str)),
                    self.metrics_service.clone());
            }
            LedgerCommand::BuildRevocRegDefRequest(submitter_did, data, cb) => {
                debug!(target: "ledger_command_executor", "BuildRevocRegDefRequest command received");
                cb(self.build_revoc_reg_def_request(&submitter_did, RevocationRegistryDefinitionV1::from(data)), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetRevocRegDefRequest(submitter_did, id, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetRevocRegDefRequest command received");
                cb(self.build_get_revoc_reg_def_request(submitter_did.as_ref(), &id), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetRevocRegDefResponse(get_revoc_ref_def_response, cb) => {
                debug!(target: "ledger_command_executor", "ParseGetRevocRegDefDefResponse command received");
                cb(self.parse_revoc_reg_def_response(&get_revoc_ref_def_response), self.metrics_service.clone());
            }
            LedgerCommand::BuildRevocRegEntryRequest(submitter_did, revoc_reg_def_id, rev_def_type, value, cb) => {
                debug!(target: "ledger_command_executor", "BuildRevocRegEntryRequest command received");
                cb(self.build_revoc_reg_entry_request(&submitter_did, &revoc_reg_def_id, &rev_def_type, RevocationRegistryDeltaV1::from(value)), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetRevocRegRequest(submitter_did, revoc_reg_def_id, timestamp, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetRevocRegRequest command received");
                cb(self.build_get_revoc_reg_request(submitter_did.as_ref(), &revoc_reg_def_id, timestamp), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetRevocRegResponse(get_revoc_reg_response, cb) => {
                debug!(target: "ledger_command_executor", "ParseGetRevocRegResponse command received");
                cb(self.parse_revoc_reg_response(&get_revoc_reg_response), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetRevocRegDeltaRequest(submitter_did, revoc_reg_def_id, from, to, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetRevocRegDeltaRequest command received");
                cb(self.build_get_revoc_reg_delta_request(submitter_did.as_ref(), &revoc_reg_def_id, from, to), self.metrics_service.clone());
            }
            LedgerCommand::ParseGetRevocRegDeltaResponse(get_revoc_reg_delta_response, cb) => {
                debug!(target: "ledger_command_executor", "ParseGetRevocRegDeltaResponse command received");
                cb(self.parse_revoc_reg_delta_response(&get_revoc_reg_delta_response), self.metrics_service.clone());
            }
            LedgerCommand::GetResponseMetadata(response, cb) => {
                debug!(target: "ledger_command_executor", "GetResponseMetadata command received");
                cb(self.get_response_metadata(&response), self.metrics_service.clone());
            }
            LedgerCommand::BuildAuthRuleRequest(submitter_did, txn_type, action, field, old_value, new_value, constraint, cb) => {
                debug!(target: "ledger_command_executor", "BuildAuthRuleRequest command received");
                cb(self.build_auth_rule_request(&submitter_did, &txn_type, &action, &field, old_value.as_ref().map(String::as_str), new_value.as_ref().map(String::as_str), constraint), self.metrics_service.clone());
            }
            LedgerCommand::BuildAuthRulesRequest(submitter_did, rules, cb) => {
                debug!(target: "ledger_command_executor", "BuildAuthRulesRequest command received");
                cb(self.build_auth_rules_request(&submitter_did, rules), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetAuthRuleRequest(submitter_did, txn_type, action, field, old_value, new_value, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetAuthRuleRequest command received");
                cb(self.build_get_auth_rule_request(submitter_did.as_ref(),
                                                    txn_type.as_ref().map(String::as_str),
                                                    action.as_ref().map(String::as_str),
                                                    field.as_ref().map(String::as_str),
                                                    old_value.as_ref().map(String::as_str),
                                                    new_value.as_ref().map(String::as_str)),
                    self.metrics_service.clone());
            }
            LedgerCommand::GetSchema(pool_handle, submitter_did, id, cb) => {
                debug!(target: "ledger_command_executor", "GetSchema command received");
                self.get_schema(pool_handle, submitter_did.as_ref(), &id, cb);
            }
            LedgerCommand::GetSchemaContinue(id, pool_response, cb_id) => {
                debug!(target: "ledger_command_executor", "GetSchemaContinue command received");
                self._get_schema_continue(id, pool_response, cb_id);
            }
            LedgerCommand::GetCredDef(pool_handle, submitter_did, id, cb) => {
                debug!(target: "ledger_command_executor", "GetCredDef command received");
                self.get_cred_def(pool_handle, submitter_did.as_ref(), &id, cb);
            }
            LedgerCommand::GetCredDefContinue(id, pool_response, cb_id) => {
                debug!(target: "ledger_command_executor", "GetCredDefContinue command received");
                self._get_cred_def_continue(id, pool_response, cb_id);
            }
            LedgerCommand::BuildTxnAuthorAgreementRequest(submitter_did, text, version, ratification_ts, retirement_ts, cb) => {
                debug!(target: "ledger_command_executor", "BuildTxnAuthorAgreementRequest command received");
                cb(self.build_txn_author_agreement_request(&submitter_did, text.as_ref().map(String::as_str), &version, ratification_ts, retirement_ts), self.metrics_service.clone());
            }
            LedgerCommand::BuildDisableAllTxnAuthorAgreementsRequest(submitter_did, cb) => {
                debug!(target: "ledger_command_executor", "BuildDisableAllTxnAuthorAgreementsRequest command received");
                cb(self.build_disable_all_txn_author_agreements_request(&submitter_did), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetTxnAuthorAgreementRequest(submitter_did, data, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetTxnAuthorAgreementRequest command received");
                cb(self.build_get_txn_author_agreement_request(submitter_did.as_ref(), data.as_ref()), self.metrics_service.clone());
            }
            LedgerCommand::BuildAcceptanceMechanismRequests(submitter_did, aml, version, aml_context, cb) => {
                debug!(target: "ledger_command_executor", "BuildAcceptanceMechanismRequests command received");
                cb(self.build_acceptance_mechanisms_request(&submitter_did, aml, &version, aml_context.as_ref().map(String::as_str)), self.metrics_service.clone());
            }
            LedgerCommand::BuildGetAcceptanceMechanismsRequest(submitter_did, timestamp, version, cb) => {
                debug!(target: "ledger_command_executor", "BuildGetAcceptanceMechanismsRequest command received");
                cb(self.build_get_acceptance_mechanisms_request(submitter_did.as_ref(),
                                                                timestamp,
                                                                version.as_ref().map(String::as_str)),
                    self.metrics_service.clone());
            }
            LedgerCommand::AppendTxnAuthorAgreementAcceptanceToRequest(request_json, text, version, hash, acc_mech_type, time_of_acceptance, cb) => {
                debug!(target: "ledger_command_executor", "AppendTxnAuthorAgreementAcceptanceToRequest command received");
                cb(self.append_txn_author_agreement_acceptance_to_request(&request_json,
                                                                          text.as_ref().map(String::as_str),
                                                                          version.as_ref().map(String::as_str),
                                                                          hash.as_ref().map(String::as_str),
                                                                          &acc_mech_type,
                                                                          time_of_acceptance),
                    self.metrics_service.clone());
            }
            LedgerCommand::AppendRequestEndorser(request_json, endorser_did, cb) => {
                debug!(target: "ledger_command_executor", "AppendRequestEndorser command received");
                cb(self.append_request_endorser(&request_json, &endorser_did), self.metrics_service.clone());
            }
        };
    }

    fn register_sp_parser(&self, txn_type: &str,
                          parser: CustomTransactionParser, free: CustomFree) -> IndyResult<()> {
        debug!("register_sp_parser >>> txn_type: {:?}, parser: {:?}, free: {:?}",
               txn_type, parser, free);

        PoolService::register_sp_parser(txn_type, parser, free)
            .map_err(IndyError::from)
    }

    fn sign_and_submit_request(&self,
                               pool_handle: PoolHandle,
                               wallet_handle: WalletHandle,
                               submitter_did: &DidValue,
                               request_json: &str,
                               cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        debug!("sign_and_submit_request >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}",
               pool_handle, wallet_handle, submitter_did, request_json);

        match self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Single) {
            Ok(signed_request) => self.submit_request(pool_handle, signed_request.as_str(), cb),
            Err(err) => cb(Err(err), self.metrics_service.clone())
        }
    }

    fn _sign_request(&self,
                     wallet_handle: WalletHandle,
                     submitter_did: &DidValue,
                     request_json: &str,
                     signature_type: SignatureType) -> IndyResult<String> {
        debug!("_sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let my_did: Did = self.wallet_service.get_indy_object(wallet_handle, &submitter_did.0, &RecordOptions::id_value())?;

        let my_key: Key = self.wallet_service.get_indy_object(wallet_handle, &my_did.verkey, &RecordOptions::id_value())?;

        let mut request: Value = serde_json::from_str(request_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Message is invalid json")?;

        if !request.is_object() {
            return Err(err_msg(IndyErrorKind::InvalidStructure, "Message isn't json object"));
        }

        let serialized_request = serialize_signature(request.clone())?;
        let signature = self.crypto_service.sign(&my_key, &serialized_request.as_bytes().to_vec())?;
        let did = my_did.did.to_short();

        match signature_type {
            SignatureType::Single => { request["signature"] = Value::String(signature.to_base58()); }
            SignatureType::Multi => {
                request.as_object_mut()
                    .map(|request| {
                        if !request.contains_key("signatures") {
                            request.insert("signatures".to_string(), Value::Object(serde_json::Map::new()));
                        }
                        request["signatures"].as_object_mut().unwrap().insert(did.0, Value::String(signature.to_base58()));

                        if let (Some(identifier), Some(signature)) = (request.get("identifier").and_then(Value::as_str).map(str::to_owned),
                                                                      request.remove("signature")) {
                            request["signatures"].as_object_mut().unwrap().insert(identifier, signature);
                        }
                    });
            }
        }

        let res: String = serde_json::to_string(&request)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize message after signing")?;

        debug!("_sign_request <<< res: {:?}", res);

        Ok(res)
    }

    fn submit_request(&self,
                      handle: PoolHandle,
                      request_json: &str,
                      cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        debug!("submit_request >>> handle: {:?}, request_json: {:?}", handle, request_json);

        if let Err(err) = serde_json::from_str::<Request<serde_json::Value>>(&request_json) {
            return cb(Err(IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Request is invalid json: {:?}", err))), self.metrics_service.clone());
        }

        let x: IndyResult<CommandHandle> = self.pool_service.send_tx(handle, request_json);
        match x {
            Ok(cmd_id) => { self.send_callbacks.borrow_mut().insert(cmd_id, cb); }
            Err(err) => { cb(Err(err), self.metrics_service.clone()); }
        };
    }

    fn submit_action(&self,
                     handle: PoolHandle,
                     request_json: &str,
                     nodes: Option<&str>,
                     timeout: Option<i32>,
                     cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        debug!("submit_action >>> handle: {:?}, request_json: {:?}, nodes: {:?}, timeout: {:?}", handle, request_json, nodes, timeout);

        if let Err(err) = self.ledger_service.validate_action(request_json) {
            return cb(Err(err), self.metrics_service.clone());
        }

        let x: IndyResult<CommandHandle> = self.pool_service.send_action(handle, request_json, nodes, timeout);
        match x {
            Ok(cmd_id) => { self.send_callbacks.borrow_mut().insert(cmd_id, cb); }
            Err(err) => { cb(Err(err), self.metrics_service.clone()); }
        };
    }

    fn sign_request(&self,
                    wallet_handle: WalletHandle,
                    submitter_did: &DidValue,
                    request_json: &str) -> IndyResult<String> {
        debug!("sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let res = self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Single)?;

        debug!("sign_request <<< res: {:?}", res);

        Ok(res)
    }

    fn multi_sign_request(&self,
                          wallet_handle: WalletHandle,
                          submitter_did: &DidValue,
                          request_json: &str) -> IndyResult<String> {
        debug!("multi_sign_request >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

        let res = self._sign_request(wallet_handle, submitter_did, request_json, SignatureType::Multi)?;

        debug!("multi_sign_request <<< res: {:?}", res);

        Ok(res)
    }
    fn build_get_ddo_request(&self,
                             submitter_did: Option<&DidValue>,
                             target_did: &DidValue) -> IndyResult<String> {
        debug!("build_get_ddo_request >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

        let res = self.ledger_service.build_get_ddo_request(submitter_did, target_did)?;

        debug!("build_get_ddo_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_nym_request(&self,
                         submitter_did: &DidValue,
                         target_did: &DidValue,
                         verkey: Option<&str>,
                         alias: Option<&str>,
                         role: Option<&str>) -> IndyResult<String> {
        debug!("build_nym_request >>> submitter_did: {:?}, target_did: {:?}, verkey: {:?}, alias: {:?}, role: {:?}",
               submitter_did, target_did, verkey, alias, role);

        self.crypto_service.validate_did(submitter_did)?;
        self.crypto_service.validate_did(target_did)?;
        if let Some(vk) = verkey {
            self.crypto_service.validate_key(vk)?;
        }

        let res = self.ledger_service.build_nym_request(submitter_did,
                                                        target_did,
                                                        verkey,
                                                        alias,
                                                        role)?;

        debug!("build_nym_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_attrib_request(&self,
                            submitter_did: &DidValue,
                            target_did: &DidValue,
                            hash: Option<&str>,
                            raw: Option<&serde_json::Value>,
                            enc: Option<&str>) -> IndyResult<String> {
        debug!("build_attrib_request >>> submitter_did: {:?}, target_did: {:?}, hash: {:?}, raw: {:?}, enc: {:?}",
               submitter_did, target_did, hash, raw, enc);

        self.crypto_service.validate_did(submitter_did)?;
        self.crypto_service.validate_did(target_did)?;

        let res = self.ledger_service.build_attrib_request(submitter_did,
                                                           target_did,
                                                           hash,
                                                           raw,
                                                           enc)?;

        debug!("build_attrib_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_attrib_request(&self,
                                submitter_did: Option<&DidValue>,
                                target_did: &DidValue,
                                raw: Option<&str>,
                                hash: Option<&str>,
                                enc: Option<&str>) -> IndyResult<String> {
        debug!("build_get_attrib_request >>> submitter_did: {:?}, target_did: {:?}, raw: {:?}, hash: {:?}, enc: {:?}",
               submitter_did, target_did, raw, hash, enc);

        self.validate_opt_did(submitter_did)?;
        self.crypto_service.validate_did(target_did)?;

        let res = self.ledger_service.build_get_attrib_request(submitter_did,
                                                               target_did,
                                                               raw,
                                                               hash,
                                                               enc)?;

        debug!("build_get_attrib_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_nym_request(&self,
                             submitter_did: Option<&DidValue>,
                             target_did: &DidValue) -> IndyResult<String> {
        debug!("build_get_nym_request >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

        self.validate_opt_did(submitter_did)?;
        self.crypto_service.validate_did(target_did)?;

        let res = self.ledger_service.build_get_nym_request(submitter_did,
                                                            target_did)?;

        debug!("build_get_attrib_request <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_get_nym_response(&self,
                             get_nym_response: &str) -> IndyResult<String> {
        debug!("parse_get_nym_response >>> get_nym_response: {:?}", get_nym_response);

        let res = self.ledger_service.parse_get_nym_response(get_nym_response)?;

        debug!("parse_get_nym_response <<< res: {:?}", res);

        Ok(res)
    }

    fn build_schema_request(&self,
                            submitter_did: &DidValue,
                            schema: SchemaV1) -> IndyResult<String> {
        debug!("build_schema_request >>> submitter_did: {:?}, schema: {:?}", submitter_did, schema);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_schema_request(submitter_did, schema)?;

        debug!("build_schema_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_schema_request(&self,
                                submitter_did: Option<&DidValue>,
                                id: &SchemaId) -> IndyResult<String> {
        debug!("build_get_schema_request >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_schema_request(submitter_did, id)?;

        debug!("build_get_schema_request <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_get_schema_response(&self,
                                 get_schema_response: &str) -> IndyResult<(String, String)> {
        debug!("parse_get_schema_response >>> get_schema_response: {:?}", get_schema_response);

        let res = self.ledger_service.parse_get_schema_response(get_schema_response, None)?;

        debug!("parse_get_schema_response <<< res: {:?}", res);

        Ok(res)
    }

    fn build_cred_def_request(&self,
                              submitter_did: &DidValue,
                              cred_def: CredentialDefinitionV1) -> IndyResult<String> {
        debug!("build_cred_def_request >>> submitter_did: {:?}, cred_def: {:?}",
               submitter_did, cred_def);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_cred_def_request(submitter_did, cred_def)?;

        debug!("build_cred_def_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_cred_def_request(&self,
                                  submitter_did: Option<&DidValue>,
                                  id: &CredentialDefinitionId) -> IndyResult<String> {
        debug!("build_get_cred_def_request >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_cred_def_request(submitter_did, id)?;

        debug!("build_get_cred_def_request <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_get_cred_def_response(&self,
                                   get_cred_def_response: &str) -> IndyResult<(String, String)> {
        debug!("parse_get_cred_def_response >>> get_cred_def_response: {:?}", get_cred_def_response);

        let res = self.ledger_service.parse_get_cred_def_response(get_cred_def_response, None)?;

        debug!("parse_get_cred_def_response <<< res: {:?}", res);

        Ok(res)
    }

    fn build_node_request(&self,
                          submitter_did: &DidValue,
                          target_did: &DidValue,
                          data: NodeOperationData) -> IndyResult<String> {
        debug!("build_node_request >>> submitter_did: {:?}, target_did: {:?}, data: {:?}",
               submitter_did, target_did, data);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_node_request(submitter_did, target_did, data)?;

        debug!("build_node_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_validator_info_request(&self,
                                        submitter_did: &DidValue) -> IndyResult<String> {
        info!("build_get_validator_info_request >>> submitter_did: {:?}", submitter_did);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_get_validator_info_request(submitter_did)?;

        info!("build_get_validator_info_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_txn_request(&self,
                             submitter_did: Option<&DidValue>,
                             ledger_type: Option<&str>,
                             seq_no: i32) -> IndyResult<String> {
        debug!("build_get_txn_request >>> submitter_did: {:?}, ledger_type: {:?}, seq_no: {:?}",
               submitter_did, ledger_type, seq_no);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_txn_request(submitter_did, ledger_type, seq_no)?;

        debug!("build_get_txn_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_config_request(&self,
                                 submitter_did: &DidValue,
                                 writes: bool,
                                 force: bool) -> IndyResult<String> {
        debug!("build_pool_config_request >>> submitter_did: {:?}, writes: {:?}, force: {:?}",
               submitter_did, writes, force);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_pool_config(submitter_did, writes, force)?;

        debug!("build_pool_config_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_restart_request(&self, submitter_did: &DidValue, action: &str,
                                  datetime: Option<&str>) -> IndyResult<String> {
        debug!("build_pool_restart_request >>> submitter_did: {:?}, action: {:?}, datetime: {:?}", submitter_did, action, datetime);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_pool_restart(&submitter_did, action, datetime)?;

        debug!("build_pool_config_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_pool_upgrade_request(&self,
                                  submitter_did: &DidValue,
                                  name: &str,
                                  version: &str,
                                  action: &str,
                                  sha256: &str,
                                  timeout: Option<u32>,
                                  schedule: Option<Schedule>,
                                  justification: Option<&str>,
                                  reinstall: bool,
                                  force: bool,
                                  package: Option<&str>) -> IndyResult<String> {
        debug!("build_pool_upgrade_request >>> submitter_did: {:?}, name: {:?}, version: {:?}, action: {:?}, sha256: {:?},\
         timeout: {:?}, schedule: {:?}, justification: {:?}, reinstall: {:?}, force: {:?}, package: {:?}",
               submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_pool_upgrade(&submitter_did, name, version, action, sha256,
                                                         timeout, schedule, justification, reinstall, force, package)?;

        debug!("build_pool_upgrade_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_revoc_reg_def_request(&self,
                                   submitter_did: &DidValue,
                                   data: RevocationRegistryDefinitionV1) -> IndyResult<String> {
        debug!("build_revoc_reg_def_request >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_revoc_reg_def_request(&submitter_did, data)?;

        debug!("build_revoc_reg_def_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_revoc_reg_def_request(&self,
                                       submitter_did: Option<&DidValue>,
                                       id: &RevocationRegistryId) -> IndyResult<String> {
        debug!("build_get_revoc_reg_def_request >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_revoc_reg_def_request(submitter_did, id)?;

        debug!("build_get_revoc_reg_def_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_revoc_reg_def_response(&self,
                                    get_revoc_reg_def_response: &str) -> IndyResult<(String, String)> {
        debug!("parse_revoc_reg_def_response >>> get_revoc_reg_def_response: {:?}", get_revoc_reg_def_response);

        let res = self.ledger_service.parse_get_revoc_reg_def_response(get_revoc_reg_def_response)?;

        debug!("parse_revoc_reg_def_response <<< res: {:?}", res);

        Ok(res)
    }

    fn build_revoc_reg_entry_request(&self,
                                     submitter_did: &DidValue,
                                     revoc_reg_def_id: &RevocationRegistryId,
                                     revoc_def_type: &str,
                                     value: RevocationRegistryDeltaV1) -> IndyResult<String> {
        debug!("build_revoc_reg_entry_request >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, revoc_def_type: {:?}, value: {:?}",
               submitter_did, revoc_reg_def_id, revoc_def_type, value);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_revoc_reg_entry_request(&submitter_did, revoc_reg_def_id, revoc_def_type, value)?;

        debug!("build_revoc_reg_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_revoc_reg_request(&self,
                                   submitter_did: Option<&DidValue>,
                                   revoc_reg_def_id: &RevocationRegistryId,
                                   timestamp: i64) -> IndyResult<String> {
        debug!("build_get_revoc_reg_request >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, timestamp: {:?}", submitter_did, revoc_reg_def_id, timestamp);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_revoc_reg_request(submitter_did, revoc_reg_def_id, timestamp)?;

        debug!("build_get_revoc_reg_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_revoc_reg_response(&self,
                                get_revoc_reg_response: &str) -> IndyResult<(String, String, u64)> {
        debug!("parse_revoc_reg_response >>> get_revoc_reg_response: {:?}", get_revoc_reg_response);

        let res = self.ledger_service.parse_get_revoc_reg_response(get_revoc_reg_response)?;

        debug!("parse_revoc_reg_response <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_revoc_reg_delta_request(&self,
                                         submitter_did: Option<&DidValue>,
                                         revoc_reg_def_id: &RevocationRegistryId,
                                         from: Option<i64>,
                                         to: i64) -> IndyResult<String> {
        debug!("build_get_revoc_reg_delta_request >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, from: {:?}, to: {:?}", submitter_did, revoc_reg_def_id, from, to);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_revoc_reg_delta_request(submitter_did, revoc_reg_def_id, from, to)?;

        debug!("build_get_revoc_reg_delta_request  <<< res: {:?}", res);

        Ok(res)
    }

    fn parse_revoc_reg_delta_response(&self,
                                      get_revoc_reg_delta_response: &str) -> IndyResult<(String, String, u64)> {
        debug!("parse_revoc_reg_delta_response >>> get_revoc_reg_delta_response: {:?}", get_revoc_reg_delta_response);

        let res = self.ledger_service.parse_get_revoc_reg_delta_response(get_revoc_reg_delta_response)?;

        debug!("parse_revoc_reg_delta_response <<< res: {:?}", res);

        Ok(res)
    }

    fn get_response_metadata(&self,
                             response: &str) -> IndyResult<String> {
        debug!("get_response_metadata >>> response: {:?}", response);

        let metadata = parse_response_metadata(response)?;

        let res = serde_json::to_string(&metadata)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize ResponseMetadata")?;

        debug!("get_response_metadata <<< res: {:?}", res);

        Ok(res)
    }

    fn build_auth_rule_request(&self,
                               submitter_did: &DidValue,
                               txn_type: &str,
                               action: &str,
                               field: &str,
                               old_value: Option<&str>,
                               new_value: Option<&str>,
                               constraint: Constraint) -> IndyResult<String> {
        debug!("build_auth_rule_request >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
            old_value: {:?}, new_value: {:?}, constraint: {:?}", submitter_did, txn_type, action, field, old_value, new_value, constraint);

        self.crypto_service.validate_did(&submitter_did)?;

        let res = self.ledger_service.build_auth_rule_request(&submitter_did, txn_type, action, field, old_value, new_value, constraint)?;

        debug!("build_auth_rule_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_auth_rules_request(&self,
                                submitter_did: &DidValue,
                                rules: AuthRules) -> IndyResult<String> {
        debug!("build_auth_rules_request >>> submitter_did: {:?}, rules: {:?}", submitter_did, rules);

        self.validate_opt_did(Some(submitter_did))?;

        let res = self.ledger_service.build_auth_rules_request(submitter_did, rules)?;

        debug!("build_auth_rules_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_auth_rule_request(&self,
                                   submitter_did: Option<&DidValue>,
                                   txn_type: Option<&str>,
                                   action: Option<&str>,
                                   field: Option<&str>,
                                   old_value: Option<&str>,
                                   new_value: Option<&str>) -> IndyResult<String> {
        debug!("build_get_auth_rule_request >>> submitter_did: {:?}, auth_type: {:?}, auth_action: {:?}, field: {:?}, \
            old_value: {:?}, new_value: {:?}", submitter_did, txn_type, action, field, old_value, new_value);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value)?;

        debug!("build_get_auth_rule_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_txn_author_agreement_request(&self,
                                          submitter_did: &DidValue,
                                          text: Option<&str>,
                                          version: &str,
                                          ratification_ts: Option<u64>,
                                          retirement_ts: Option<u64>) -> IndyResult<String> {
        debug!("build_txn_author_agreement_request >>> submitter_did: {:?}, text: {:?}, version: {:?}, ratification_ts {:?}, retirement_ts {:?}",
               submitter_did, text, version, ratification_ts, retirement_ts);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_txn_author_agreement_request(submitter_did, text, version, ratification_ts, retirement_ts)?;

        debug!("build_txn_author_agreement_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_disable_all_txn_author_agreements_request(&self, submitter_did: &DidValue) -> IndyResult<String> {
        debug!("build_disable_all_txn_author_agreements_request >>> submitter_did: {:?}", submitter_did);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_disable_all_txn_author_agreements_request(submitter_did)?;

        debug!("build_disable_all_txn_author_agreements_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_txn_author_agreement_request(&self,
                                              submitter_did: Option<&DidValue>,
                                              data: Option<&GetTxnAuthorAgreementData>) -> IndyResult<String> {
        debug!("build_get_txn_author_agreement_request >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_txn_author_agreement_request(submitter_did, data)?;

        debug!("build_get_txn_author_agreement_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_acceptance_mechanisms_request(&self,
                                           submitter_did: &DidValue,
                                           aml: AcceptanceMechanisms,
                                           version: &str,
                                           aml_context: Option<&str>) -> IndyResult<String> {
        debug!("build_acceptance_mechanisms_request >>> submitter_did: {:?}, aml: {:?}, version: {:?}, aml_context: {:?}", submitter_did, aml, version, aml_context);

        self.crypto_service.validate_did(submitter_did)?;

        let res = self.ledger_service.build_acceptance_mechanisms_request(submitter_did, aml, version, aml_context)?;

        debug!("build_acceptance_mechanisms_request <<< res: {:?}", res);

        Ok(res)
    }

    fn build_get_acceptance_mechanisms_request(&self,
                                               submitter_did: Option<&DidValue>,
                                               timestamp: Option<u64>,
                                               version: Option<&str>) -> IndyResult<String> {
        debug!("build_get_acceptance_mechanisms_request >>> submitter_did: {:?}, timestamp: {:?}, version: {:?}", submitter_did, timestamp, version);

        self.validate_opt_did(submitter_did)?;

        let res = self.ledger_service.build_get_acceptance_mechanisms_request(submitter_did, timestamp, version)?;

        debug!("build_get_acceptance_mechanisms_request <<< res: {:?}", res);

        Ok(res)
    }

    fn append_txn_author_agreement_acceptance_to_request(&self,
                                                         request_json: &str,
                                                         text: Option<&str>,
                                                         version: Option<&str>,
                                                         taa_digest: Option<&str>,
                                                         acc_mech_type: &str,
                                                         time: u64) -> IndyResult<String> {
        debug!("append_txn_author_agreement_acceptance_to_request >>> request_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, acc_mech_type: {:?}, time: {:?}",
               request_json, text, version, taa_digest, acc_mech_type, time);

        let mut request: serde_json::Value = serde_json::from_str(request_json)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot deserialize request: {:?}", err)))?;

        request["taaAcceptance"] = json!(self.ledger_service.prepare_acceptance_data(text, version, taa_digest, acc_mech_type, time)?);

        let res: String = serde_json::to_string(&request)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize request after adding author agreement acceptance data")?;

        debug!("append_txn_author_agreement_acceptance_to_request <<< res: {:?}", res);

        Ok(res)
    }

    fn append_request_endorser(&self,
                               request_json: &str,
                               endorser_did: &DidValue) -> IndyResult<String> {
        debug!("append_request_endorser >>> request_json: {:?}, endorser_did: {:?}", request_json, endorser_did);

        self.crypto_service.validate_did(endorser_did)?;

        let endorser_did = endorser_did.to_short();

        let mut request: serde_json::Value = serde_json::from_str(request_json)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot deserialize request: {:?}", err)))?;

        request["endorser"] = json!(endorser_did);

        let res: String = serde_json::to_string(&request)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize request after adding endorser")?;

        debug!("append_request_endorser <<< res: {:?}", res);

        Ok(res)
    }

    fn validate_opt_did(&self, did: Option<&DidValue>) -> IndyResult<()> {
        match did {
            Some(did) => Ok(self.crypto_service.validate_did(did)?),
            None => Ok(())
        }
    }

    fn get_schema(&self, pool_handle: i32, submitter_did: Option<&DidValue>, id: &SchemaId, cb: BoxedCallbackStringStringSend) {
        let request_json = try_cb!(self.build_get_schema_request(submitter_did, id), cb, self.metrics_service.clone());

        let cb_id = next_command_handle();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);
        let id = id.clone();

        self.submit_request(pool_handle, &request_json, Box::new(move |response, _| {
            CommandExecutor::instance().send(
                Command::Ledger(
                    LedgerCommand::GetSchemaContinue(
                        id.clone(),
                        response,
                        cb_id
                    )
                )
            ).unwrap();
        }));
    }

    fn _get_schema_continue(&self, id: SchemaId, pool_response: IndyResult<String>, cb_id: CommandHandle) {
        let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");
        let pool_response = try_cb!(pool_response, cb, self.metrics_service.clone());
        cb(self.ledger_service.parse_get_schema_response(&pool_response, id.get_method().as_ref().map(String::as_str)), self.metrics_service.clone())
    }

    fn get_cred_def(&self, pool_handle: i32, submitter_did: Option<&DidValue>, id: &CredentialDefinitionId, cb: BoxedCallbackStringStringSend) {
        let request_json = try_cb!(self.build_get_cred_def_request(submitter_did, id), cb, self.metrics_service.clone());

        let cb_id = next_command_handle();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);
        let id = id.clone();

        self.submit_request(pool_handle, &request_json, Box::new(move |response, _| {
            CommandExecutor::instance().send(
                Command::Ledger(
                    LedgerCommand::GetCredDefContinue(
                        id.clone(),
                        response,
                        cb_id
                    )
                )
            ).unwrap();
        }));
    }

    fn _get_cred_def_continue(&self, id: CredentialDefinitionId, pool_response: IndyResult<String>, cb_id: CommandHandle) {
        let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");
        let pool_response = try_cb!(pool_response, cb, self.metrics_service.clone());
        cb(self.ledger_service.parse_get_cred_def_response(&pool_response, id.get_method().as_ref().map(String::as_str)), self.metrics_service.clone())
    }
}

enum SignatureType {
    Single,
    Multi
}
