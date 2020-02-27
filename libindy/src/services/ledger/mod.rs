use hex::FromHex;
use ursa::cl::RevocationRegistryDelta as CryproRevocationRegistryDelta;
use serde::de::DeserializeOwned;
use serde_json;
use serde_json::Value;
use log_derive::logfn;

use crate::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionV1, CredentialDefinitionId};
use crate::domain::anoncreds::revocation_registry::RevocationRegistry;
use crate::domain::anoncreds::revocation_registry_definition::{RevocationRegistryDefinition, RevocationRegistryDefinitionV1, RevocationRegistryId};
use crate::domain::anoncreds::revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1};
use crate::domain::anoncreds::schema::{Schema, SchemaV1, SchemaId};
use indy_api_types::domain::crypto::did::DidValue;
use crate::domain::ledger::attrib::{AttribOperation, GetAttribOperation};
use crate::domain::ledger::constants::{GET_VALIDATOR_INFO, POOL_RESTART, ROLE_REMOVE, STEWARD, ENDORSER, TRUSTEE, NETWORK_MONITOR, ROLES, txn_name_to_code};
use crate::domain::ledger::cred_def::{CredDefOperation, GetCredDefOperation, GetCredDefReplyResult};
use crate::domain::ledger::ddo::GetDdoOperation;
use crate::domain::ledger::node::{NodeOperation, NodeOperationData};
use crate::domain::ledger::nym::{GetNymOperation, GetNymReplyResult, GetNymResultDataV0, NymData, NymOperation};
use crate::domain::ledger::pool::{PoolConfigOperation, PoolRestartOperation, PoolUpgradeOperation, Schedule};
use crate::domain::ledger::request::{TxnAuthrAgrmtAcceptanceData, Request};
use crate::domain::ledger::response::{Message, Reply, ReplyType};
use crate::domain::ledger::rev_reg::{GetRevocRegDeltaReplyResult, GetRevocRegReplyResult, GetRevRegDeltaOperation, GetRevRegOperation, RevRegEntryOperation};
use crate::domain::ledger::rev_reg_def::{GetRevocRegDefReplyResult, GetRevRegDefOperation, RevRegDefOperation};
use crate::domain::ledger::schema::{GetSchemaOperation, GetSchemaOperationData, GetSchemaReplyResult, SchemaOperation, SchemaOperationData};
use crate::domain::ledger::txn::{GetTxnOperation, LedgerType};
use crate::domain::ledger::validator_info::GetValidatorInfoOperation;
use crate::domain::ledger::auth_rule::*;
use crate::domain::ledger::author_agreement::*;
use indy_api_types::errors::prelude::*;
use indy_utils::crypto::hash::hash as openssl_hash;

pub mod merkletree;

macro_rules! build_result {
        ($operation:ident, $submitter_did:expr) => ({
            let operation = $operation::new();

            Request::build_request($submitter_did, operation)
                .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, err))
        });
        ($operation:ident, $submitter_did:expr, $($params:tt)*) => ({
            let operation = $operation::new($($params)*);

            Request::build_request($submitter_did, operation)
                .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, err))
        })
    }

pub struct LedgerService {}

impl LedgerService {
    pub fn new() -> LedgerService {
        LedgerService {}
    }

    #[logfn(Info)]
    pub fn build_nym_request(&self, identifier: &DidValue, dest: &DidValue, verkey: Option<&str>,
                             alias: Option<&str>, role: Option<&str>) -> IndyResult<String> {
        let role = if let Some(r) = role {
            Some(
                if r == ROLE_REMOVE {
                    Value::Null
                } else {
                    json!(
                    match r {
                        "STEWARD" => STEWARD,
                        "TRUSTEE" => TRUSTEE,
                        "TRUST_ANCHOR" | "ENDORSER" => ENDORSER,
                        "NETWORK_MONITOR" => NETWORK_MONITOR,
                        role if ROLES.contains(&role) => role,
                        role => return Err(err_msg(IndyErrorKind::InvalidStructure, format!("Invalid role: {}", role)))
                    }
                )
                }
            )
        } else { None };

        build_result!(NymOperation, Some(identifier), dest.to_short(),
                                                      verkey.map(String::from),
                                                      alias.map(String::from),
                                                      role)
    }

    #[logfn(Info)]
    pub fn build_get_nym_request(&self, identifier: Option<&DidValue>, dest: &DidValue) -> IndyResult<String> {
        build_result!(GetNymOperation, identifier, dest.to_short())
    }

    #[logfn(Info)]
    pub fn parse_get_nym_response(&self, get_nym_response: &str) -> IndyResult<String> {
        let reply: Reply<GetNymReplyResult> = LedgerService::parse_response(get_nym_response)?;

        let nym_data = match reply.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                let data: GetNymResultDataV0 = res.data
                    .ok_or(IndyError::from_msg(IndyErrorKind::LedgerItemNotFound, format!("Nym not found")))
                    .and_then(|data| serde_json::from_str(&data)
                        .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot parse GET_NYM response: {}", err)))
                    )?;

                NymData {
                    did: data.dest,
                    verkey: data.verkey,
                    role: data.role,
                }
            }
            GetNymReplyResult::GetNymReplyResultV1(res) => {
                NymData {
                    did: res.txn.data.did,
                    verkey: res.txn.data.verkey,
                    role: res.txn.data.role
                }
            }
        };

        let res = serde_json::to_string(&nym_data)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot serialize NYM data: {}", err)))?;

        Ok(res)
    }

    #[logfn(Info)]
    pub fn build_get_ddo_request(&self, identifier: Option<&DidValue>, dest: &DidValue) -> IndyResult<String> {
        build_result!(GetDdoOperation, identifier, dest.to_short())
    }

    #[logfn(Info)]
    pub fn build_attrib_request(&self, identifier: &DidValue, dest: &DidValue, hash: Option<&str>,
                                raw: Option<&serde_json::Value>, enc: Option<&str>) -> IndyResult<String> {
        build_result!(AttribOperation, Some(identifier), dest.to_short(),
                                                         hash.map(String::from),
                                                         raw.map(serde_json::Value::to_string),
                                                         enc.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_get_attrib_request(&self, identifier: Option<&DidValue>, dest: &DidValue, raw: Option<&str>, hash: Option<&str>,
                                    enc: Option<&str>) -> IndyResult<String> {
        build_result!(GetAttribOperation, identifier, dest.to_short(), raw, hash, enc)
    }

    #[logfn(Info)]
    pub fn build_schema_request(&self, identifier: &DidValue, schema: SchemaV1) -> IndyResult<String> {
        let schema_data = SchemaOperationData::new(schema.name, schema.version, schema.attr_names.into());
        build_result!(SchemaOperation, Some(identifier), schema_data)
    }

    #[logfn(Info)]
    pub fn build_get_schema_request(&self, identifier: Option<&DidValue>, id: &SchemaId) -> IndyResult<String> {
        let id = id.to_unqualified();
        let (dest, name, version) = id.parts()
            .ok_or(IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Schema ID `{}` cannot be used to build request: invalid number of parts", id.0)))?;

        let data = GetSchemaOperationData::new(name, version);
        build_result!(GetSchemaOperation, identifier, dest.to_short(), data)
    }

    #[logfn(Info)]
    pub fn build_cred_def_request(&self, identifier: &DidValue, cred_def: CredentialDefinitionV1) -> IndyResult<String> {
        let cred_def: CredentialDefinitionV1 = CredentialDefinitionV1 {
            id: cred_def.id.to_unqualified(),
            schema_id: cred_def.schema_id.to_unqualified(),
            signature_type: cred_def.signature_type,
            tag: cred_def.tag,
            value: cred_def.value,
        };
        build_result!(CredDefOperation, Some(identifier), cred_def)
    }

    #[logfn(Info)]
    pub fn build_get_cred_def_request(&self, identifier: Option<&DidValue>, id: &CredentialDefinitionId) -> IndyResult<String> {
        let id = id.to_unqualified();
        let (origin, signature_type, schema_id, tag) = id.parts()
            .ok_or(IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Credential Definition ID `{}` cannot be used to build request: invalid number of parts", id.0)))?;

        let ref_ = schema_id.0
            .parse::<i32>()
            .to_indy(IndyErrorKind::InvalidStructure, format!("Schema ID is invalid number in: {:?}", id))?;

        build_result!(GetCredDefOperation, identifier, ref_, signature_type, origin.to_short(), Some(tag))
    }

    #[logfn(Info)]
    pub fn build_node_request(&self, identifier: &DidValue, dest: &DidValue, data: NodeOperationData) -> IndyResult<String> {
        build_result!(NodeOperation, Some(identifier), dest.to_short(), data)
    }

    #[logfn(Info)]
    pub fn build_get_validator_info_request(&self, identifier: &DidValue) -> IndyResult<String> {
        let operation = GetValidatorInfoOperation::new();

        Request::build_request(Some(identifier), operation)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, err))
    }

    #[logfn(Info)]
    pub fn build_get_txn_request(&self, identifier: Option<&DidValue>, ledger_type: Option<&str>, seq_no: i32) -> IndyResult<String> {
        let ledger_id = match ledger_type {
            Some(type_) =>
                serde_json::from_str::<LedgerType>(&format!(r#""{}""#, type_))
                    .map(|type_| type_.to_id())
                    .or_else(|_| type_.parse::<i32>())
                    .to_indy(IndyErrorKind::InvalidStructure, format!("Invalid Ledger type: {}", type_))?,
            None => LedgerType::DOMAIN.to_id()
        };

        build_result!(GetTxnOperation, identifier, seq_no, ledger_id)
    }

    #[logfn(Info)]
    pub fn build_pool_config(&self, identifier: &DidValue, writes: bool, force: bool) -> IndyResult<String> {
        build_result!(PoolConfigOperation, Some(identifier), writes, force)
    }

    #[logfn(Info)]
    pub fn build_pool_restart(&self, identifier: &DidValue, action: &str, datetime: Option<&str>) -> IndyResult<String> {
        build_result!(PoolRestartOperation, Some(identifier),action, datetime.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_pool_upgrade(&self, identifier: &DidValue, name: &str, version: &str, action: &str,
                              sha256: &str, timeout: Option<u32>, schedule: Option<Schedule>,
                              justification: Option<&str>, reinstall: bool, force: bool, package: Option<&str>) -> IndyResult<String> {
        build_result!(PoolUpgradeOperation, Some(identifier), name, version, action, sha256, timeout, schedule, justification, reinstall, force, package)
    }

    #[logfn(Info)]
    pub fn build_revoc_reg_def_request(&self, identifier: &DidValue, mut rev_reg_def: RevocationRegistryDefinitionV1) -> IndyResult<String> {
        rev_reg_def.id = rev_reg_def.id.to_unqualified();
        rev_reg_def.cred_def_id = rev_reg_def.cred_def_id.to_unqualified();
        build_result!(RevRegDefOperation, Some(identifier), rev_reg_def)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_def_request(&self, identifier: Option<&DidValue>, id: &RevocationRegistryId) -> IndyResult<String> {
        let id = id.to_unqualified();
        build_result!(GetRevRegDefOperation, identifier, &id)
    }

    #[logfn(Info)]
    pub fn build_revoc_reg_entry_request(&self, identifier: &DidValue, revoc_reg_def_id: &RevocationRegistryId,
                                         revoc_def_type: &str, rev_reg_entry: RevocationRegistryDeltaV1) -> IndyResult<String> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        build_result!(RevRegEntryOperation, Some(identifier), revoc_def_type, &revoc_reg_def_id, rev_reg_entry)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_request(&self, identifier: Option<&DidValue>, revoc_reg_def_id: &RevocationRegistryId, timestamp: i64) -> IndyResult<String> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        build_result!(GetRevRegOperation, identifier, &revoc_reg_def_id, timestamp)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_delta_request(&self, identifier: Option<&DidValue>, revoc_reg_def_id: &RevocationRegistryId, from: Option<i64>, to: i64) -> IndyResult<String> {
        let revoc_reg_def_id = revoc_reg_def_id.to_unqualified();
        build_result!(GetRevRegDeltaOperation, identifier, &revoc_reg_def_id, from, to)
    }

    #[logfn(Info)]
    pub fn parse_get_schema_response(&self, get_schema_response: &str, method_name: Option<&str>) -> IndyResult<(String, String)> {
        let reply: Reply<GetSchemaReplyResult> = LedgerService::parse_response(get_schema_response)?;

        let schema = match reply.result() {
            GetSchemaReplyResult::GetSchemaReplyResultV0(res) => SchemaV1 {
                id: SchemaId::new(&DidValue::new(&res.dest.0, method_name), &res.data.name, &res.data.version),
                name: res.data.name,
                version: res.data.version,
                attr_names: res.data.attr_names.into(),
                seq_no: Some(res.seq_no),
            },
            GetSchemaReplyResult::GetSchemaReplyResultV1(res) => {
                SchemaV1 {
                    name: res.txn.data.schema_name,
                    version: res.txn.data.schema_version,
                    attr_names: res.txn.data.value.attr_names.into(),
                    id: match method_name {
                        Some(method) => res.txn.data.id.qualify(method),
                        None => res.txn.data.id
                    },
                    seq_no: Some(res.txn_metadata.seq_no),
                }
            }
        };

        let res = (schema.id.0.clone(),
                   serde_json::to_string(&Schema::SchemaV1(schema))
                       .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Schema")?);

        Ok(res)
    }

    #[logfn(Info)]
    pub fn parse_get_cred_def_response(&self, get_cred_def_response: &str, method_name: Option<&str>) -> IndyResult<(String, String)> {
        let reply: Reply<GetCredDefReplyResult> = LedgerService::parse_response(get_cred_def_response)?;

        let cred_def = match reply.result() {
            GetCredDefReplyResult::GetCredDefReplyResultV0(res) => CredentialDefinitionV1 {
                id: CredentialDefinitionId::new(
                    &DidValue::new(&res.origin.0, method_name),
                    &SchemaId(res.ref_.to_string()),
                    &res.signature_type.to_str(),
                    &res.tag.clone().unwrap_or_default()),
                schema_id: SchemaId(res.ref_.to_string()),
                signature_type: res.signature_type,
                tag: res.tag.unwrap_or_default(),
                value: res.data,
            },
            GetCredDefReplyResult::GetCredDefReplyResultV1(res) => CredentialDefinitionV1 {
                id: match method_name {
                    Some(method) => res.txn.data.id.qualify(method),
                    None => res.txn.data.id
                },
                schema_id: res.txn.data.schema_ref,
                signature_type: res.txn.data.type_,
                tag: res.txn.data.tag,
                value: res.txn.data.public_keys,
            }
        };

        let res = (cred_def.id.0.clone(),
                   serde_json::to_string(&CredentialDefinition::CredentialDefinitionV1(cred_def))
                       .to_indy(IndyErrorKind::InvalidState, "Cannot serialize CredentialDefinition")?);

        Ok(res)
    }

    #[logfn(Info)]
    pub fn parse_get_revoc_reg_def_response(&self, get_revoc_reg_def_response: &str) -> IndyResult<(String, String)> {
        let reply: Reply<GetRevocRegDefReplyResult> = LedgerService::parse_response(get_revoc_reg_def_response)?;

        let revoc_reg_def = match reply.result() {
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV0(res) => res.data,
            GetRevocRegDefReplyResult::GetRevocRegDefReplyResultV1(res) => res.txn.data,
        };

        let res = (revoc_reg_def.id.0.clone(),
                   serde_json::to_string(&RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def))
                       .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDefinition")?);

        Ok(res)
    }

    #[logfn(Info)]
    pub fn parse_get_revoc_reg_response(&self, get_revoc_reg_response: &str) -> IndyResult<(String, String, u64)> {
        let reply: Reply<GetRevocRegReplyResult> = LedgerService::parse_response(get_revoc_reg_response)?;

        let (revoc_reg_def_id, revoc_reg, txn_time) = match reply.result() {
            GetRevocRegReplyResult::GetRevocRegReplyResultV0(res) => (res.revoc_reg_def_id, res.data, res.txn_time),
            GetRevocRegReplyResult::GetRevocRegReplyResultV1(res) => (res.txn.data.revoc_reg_def_id, res.txn.data.value, res.txn_metadata.creation_time),
        };

        let res = (revoc_reg_def_id.0,
                   serde_json::to_string(&RevocationRegistry::RevocationRegistryV1(revoc_reg))
                       .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistry")?,
                   txn_time);

        Ok(res)
    }

    #[logfn(Info)]
    pub fn parse_get_revoc_reg_delta_response(&self, get_revoc_reg_delta_response: &str) -> IndyResult<(String, String, u64)> {
        let reply: Reply<GetRevocRegDeltaReplyResult> = LedgerService::parse_response(get_revoc_reg_delta_response)?;

        let (revoc_reg_def_id, revoc_reg) = match reply.result() {
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV0(res) => (res.revoc_reg_def_id, res.data),
            GetRevocRegDeltaReplyResult::GetRevocRegDeltaReplyResultV1(res) => (res.txn.data.revoc_reg_def_id, res.txn.data.value),
        };

        let res = (revoc_reg_def_id.0,
                   serde_json::to_string(&RevocationRegistryDelta::RevocationRegistryDeltaV1(
                       RevocationRegistryDeltaV1 {
                           value: CryproRevocationRegistryDelta::from_parts(revoc_reg.value.accum_from.map(|accum| accum.value).as_ref(),
                                                                            &revoc_reg.value.accum_to.value,
                                                                            &revoc_reg.value.issued,
                                                                            &revoc_reg.value.revoked)
                       }))
                       .to_indy(IndyErrorKind::InvalidState, "Cannot serialize RevocationRegistryDelta")?,
                   revoc_reg.value.accum_to.txn_time);

        Ok(res)
    }

    #[logfn(Info)]
    pub fn build_auth_rule_request(&self, submitter_did: &DidValue, txn_type: &str, action: &str, field: &str,
                                   old_value: Option<&str>, new_value: Option<&str>, constraint: Constraint) -> IndyResult<String> {
        let txn_type = txn_name_to_code(&txn_type)
            .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, format!("Unsupported `txn_type`: {}", txn_type)))?;

        let action = serde_json::from_str::<AuthAction>(&format!("\"{}\"", action))
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot parse auth action: {}", err)))?;

        build_result!(AuthRuleOperation, Some(submitter_did), txn_type.to_string(), field.to_string(), action,
                                                              old_value.map(String::from), new_value.map(String::from), constraint)
    }

    #[logfn(Info)]
    pub fn build_auth_rules_request(&self, submitter_did: &DidValue, rules: AuthRules) -> IndyResult<String> {
        build_result!(AuthRulesOperation, Some(submitter_did), rules)
    }

    #[logfn(Info)]
    pub fn build_get_auth_rule_request(&self, submitter_did: Option<&DidValue>, auth_type: Option<&str>, auth_action: Option<&str>,
                                       field: Option<&str>, old_value: Option<&str>, new_value: Option<&str>) -> IndyResult<String> {
        let operation = match (auth_type, auth_action, field) {
            (None, None, None) => GetAuthRuleOperation::get_all(),
            (Some(auth_type), Some(auth_action), Some(field)) => {
                let type_ = txn_name_to_code(&auth_type)
                    .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, format!("Unsupported `auth_type`: {}", auth_type)))?;

                let action = serde_json::from_str::<AuthAction>(&format!("\"{}\"", auth_action))
                    .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot parse auth action: {}", err)))?;

                GetAuthRuleOperation::get_one(type_.to_string(),
                                              field.to_string(),
                                              action,
                                              old_value.map(String::from),
                                              new_value.map(String::from))
            }
            _ => return Err(err_msg(IndyErrorKind::InvalidStructure, "Either none or all transaction related parameters must be specified."))
        };

        let request = Request::build_request(submitter_did, operation)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, err))?;

        Ok(request)
    }

    #[logfn(Info)]
    pub fn build_txn_author_agreement_request(&self, identifier: &DidValue, text: Option<&str>, version: &str, ratification_ts: Option<u64>, retirement_ts: Option<u64>) -> IndyResult<String> {
        build_result!(TxnAuthorAgreementOperation, Some(identifier), text.map(str::to_string), version.to_string(), ratification_ts, retirement_ts)
    }

    #[logfn(Info)]
    pub fn build_disable_all_txn_author_agreements_request(&self, identifier: &DidValue) -> IndyResult<String> {
        build_result!(DisableAllTxnAuthorAgreementsOperation, Some(identifier))
    }

    #[logfn(Info)]
    pub fn build_get_txn_author_agreement_request(&self, identifier: Option<&DidValue>, data: Option<&GetTxnAuthorAgreementData>) -> IndyResult<String> {
        build_result!(GetTxnAuthorAgreementOperation, identifier, data)
    }

    #[logfn(Info)]
    pub fn build_acceptance_mechanisms_request(&self, identifier: &DidValue, aml: AcceptanceMechanisms, version: &str, aml_context: Option<&str>) -> IndyResult<String> {
        build_result!(SetAcceptanceMechanismOperation, Some(identifier), aml, version.to_string(), aml_context.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_get_acceptance_mechanisms_request(&self, identifier: Option<&DidValue>, timestamp: Option<u64>, version: Option<&str>) -> IndyResult<String> {
        if timestamp.is_some() && version.is_some() {
            return Err(err_msg(IndyErrorKind::InvalidStructure, "timestamp and version cannot be specified together."));
        }

        build_result!(GetAcceptanceMechanismOperation, identifier, timestamp, version.map(String::from))
    }

    #[logfn(Info)]
    pub fn parse_response<T>(response: &str) -> IndyResult<Reply<T>> where T: DeserializeOwned + ReplyType + ::std::fmt::Debug {
        let message: serde_json::Value = serde_json::from_str(&response)
            .to_indy(IndyErrorKind::InvalidTransaction, "Response is invalid json")?;

        if message["op"] == json!("REPLY") && message["result"]["type"] != json!(T::get_type()) {
            return Err(err_msg(IndyErrorKind::InvalidTransaction, "Invalid response type"));
        }

        let message: Message<T> = serde_json::from_value(message)
            .to_indy(IndyErrorKind::LedgerItemNotFound, "Structure doesn't correspond to type. Most probably not found")?; // FIXME: Review how we handle not found

        match message {
            Message::Reject(response) | Message::ReqNACK(response) =>
                Err(err_msg(IndyErrorKind::InvalidTransaction, format!("Transaction has been failed: {:?}", response.reason))),
            Message::Reply(reply) =>
                Ok(reply)
        }
    }

    #[logfn(Info)]
    pub fn validate_action(&self, request: &str) -> IndyResult<()> {
        let request: Request<serde_json::Value> = serde_json::from_str(request)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Request is invalid json: {:?}", err)))?;

        match request.operation["type"].as_str() {
            Some(POOL_RESTART) | Some(GET_VALIDATOR_INFO) => Ok(()),
            Some(_) => Err(err_msg(IndyErrorKind::InvalidStructure, "Request does not match any type of Actions: POOL_RESTART, GET_VALIDATOR_INFO")),
            None => Err(err_msg(IndyErrorKind::InvalidStructure, "No valid type field in request"))
        }
    }

    #[logfn(Info)]
    pub fn prepare_acceptance_data(&self, text: Option<&str>, version: Option<&str>, hash: Option<&str>, mechanism: &str, time: u64) -> IndyResult<TxnAuthrAgrmtAcceptanceData> {
        let taa_digest = match (text, version, hash) {
            (None, None, None) => {
                return Err(err_msg(IndyErrorKind::InvalidStructure, "Invalid combination of params: Either combination `text` + `version` or `taa_digest` must be passed."));
            }
            (None, None, Some(hash_)) => {
                hash_.to_string()
            }
            (Some(_), None, _) | (None, Some(_), _) => {
                return Err(err_msg(IndyErrorKind::InvalidStructure, "Invalid combination of params: `text` and `version` should be passed or skipped together."));
            }
            (Some(text_), Some(version_), None) => {
                hex::encode(self._calculate_hash(text_, version_)?)
            }
            (Some(text_), Some(version_), Some(hash_)) => {
                self._compare_hash(text_, version_, hash_)?;
                hash_.to_string()
            }
        };

        let acceptance_data = TxnAuthrAgrmtAcceptanceData {
            mechanism: mechanism.to_string(),
            taa_digest,
            time: LedgerService::datetime_to_date_timestamp(time),
        };

        Ok(acceptance_data)
    }

    fn datetime_to_date_timestamp(time: u64) -> u64 {
        const SEC_IN_DAY: u64 = 86400;
        time / SEC_IN_DAY * SEC_IN_DAY
    }

    fn _calculate_hash(&self, text: &str, version: &str) -> IndyResult<Vec<u8>> {
        let content: String = version.to_string() + text;
        openssl_hash(content.as_bytes())
    }

    fn _compare_hash(&self, text: &str, version: &str, hash: &str) -> IndyResult<()> {
        let calculated_hash = self._calculate_hash(text, version)?;

        let passed_hash = Vec::from_hex(hash)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Cannot decode `hash`: {:?}", err)))?;

        if calculated_hash != passed_hash {
            return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure,
                                           format!("Calculated hash of concatenation `version` and `text` doesn't equal to passed `hash` value. \n\
                                           Calculated hash value: {:?}, \n Passed hash value: {:?}", calculated_hash, passed_hash)));
        }
        Ok(())
    }

    pub fn parse_get_auth_rule_response(&self, response: &str) -> IndyResult<Vec<AuthRule>> {
        trace!("parse_get_auth_rule_response >>> response: {:?}", response);

        let response: Reply<GetAuthRuleResult> = serde_json::from_str(&response)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidTransaction, format!("Cannot parse GetAuthRule response: {:?}", err)))?;

        let res = response.result().data;

        trace!("parse_get_auth_rule_response <<< {:?}", res);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::anoncreds::schema::AttributeNames;
    use crate::domain::ledger::constants::*;
    use crate::domain::ledger::node::Services;
    use crate::domain::ledger::request::ProtocolVersion;

    use super::*;

    const IDENTIFIER: &str = "NcYxiDXkpYi6ov5FcYDi1e";
    const DEST: &str = "VsKV7grR1BUE29mG2Fm2kX";
    const VERKEY: &str = "CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW";

    fn identifier() -> DidValue {
        DidValue(IDENTIFIER.to_string())
    }

    fn dest() -> DidValue {
        DidValue(DEST.to_string())
    }

    #[test]
    fn build_nym_request_works_for_only_required_fields() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST
        });

        let request = ledger_service.build_nym_request(&identifier(), &dest(), None, None, None).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_nym_request_works_for_empty_role() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST,
            "role": serde_json::Value::Null,
        });

        let request = ledger_service.build_nym_request(&identifier(), &dest(), None, None, Some("")).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_nym_request_works_for_optional_fields() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": NYM,
            "dest": DEST,
            "role": serde_json::Value::Null,
            "alias": "some_alias",
            "verkey": VERKEY,
        });

        let request = ledger_service.build_nym_request(&identifier(), &dest(), Some(VERKEY), Some("some_alias"), Some("")).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_nym_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_NYM,
            "dest": DEST
        });

        let request = ledger_service.build_get_nym_request(Some(&identifier()), &dest()).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_ddo_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_DDO,
            "dest": DEST
        });

        let request = ledger_service.build_get_ddo_request(Some(&identifier()), &dest()).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_attrib_request_works_for_hash_field() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": ATTRIB,
            "dest": DEST,
            "hash": "hash"
        });

        let request = ledger_service.build_attrib_request(&identifier(), &dest(), Some("hash"), None, None).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_raw_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "raw": "raw"
        });

        let request = ledger_service.build_get_attrib_request(Some(&identifier()), &dest(), Some("raw"), None, None).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_hash_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "hash": "hash"
        });

        let request = ledger_service.build_get_attrib_request(Some(&identifier()), &dest(), None, Some("hash"), None).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_attrib_request_works_for_enc_value() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_ATTR,
            "dest": DEST,
            "enc": "enc"
        });

        let request = ledger_service.build_get_attrib_request(Some(&identifier()), &dest(), None, None, Some("enc")).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_schema_request_works() {
        let ledger_service = LedgerService::new();

        let mut attr_names: AttributeNames = AttributeNames::new();
        attr_names.0.insert("male".to_string());

        let data = SchemaV1 {
            id: SchemaId::new(&identifier(), "name", "1.0"),
            name: "name".to_string(),
            version: "1.0".to_string(),
            attr_names,
            seq_no: None,
        };

        let expected_result = json!({
            "type": SCHEMA,
            "data": {
                "name": "name",
                "version": "1.0",
                "attr_names": ["male"]
            }
        });

        let request = ledger_service.build_schema_request(&identifier(), data).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_schema_request_works_for_valid_id() {
        let ledger_service = LedgerService::new();

        let id = SchemaId::new(&identifier(), "name", "1.0");

        let expected_result = json!({
            "type": GET_SCHEMA,
            "dest": IDENTIFIER,
            "data": {
                "name": "name",
                "version": "1.0"
            }
        });

        let request = ledger_service.build_get_schema_request(Some(&identifier()), &id).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_cred_def_request_works() {
        ProtocolVersion::set(2);

        let ledger_service = LedgerService::new();

        let id = CredentialDefinitionId::new(&identifier(), &SchemaId("1".to_string()), "signature_type", "tag");

        let expected_result = json!({
            "type": GET_CRED_DEF,
            "ref": 1,
            "signature_type": "signature_type",
            "origin": IDENTIFIER,
            "tag":"tag"
        });

        let request = ledger_service.build_get_cred_def_request(Some(&identifier()), &id).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_node_request_works() {
        let ledger_service = LedgerService::new();

        let data = NodeOperationData {
            node_ip: Some("ip".to_string()),
            node_port: Some(1),
            client_ip: Some("ip".to_string()),
            client_port: Some(1),
            alias: "some".to_string(),
            services: Some(vec![Services::VALIDATOR]),
            blskey: Some("blskey".to_string()),
            blskey_pop: Some("pop".to_string()),
        };

        let expected_result = json!({
            "type": NODE,
            "dest": DEST,
            "data": {
                "node_ip": "ip",
                "node_port": 1,
                "client_ip": "ip",
                "client_port": 1,
                "alias": "some",
                "services": ["VALIDATOR"],
                "blskey": "blskey",
                "blskey_pop": "pop"
            }
        });

        let request = ledger_service.build_node_request(&identifier(), &dest(), data).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 1
        });

        let request = ledger_service.build_get_txn_request(Some(&identifier()), None, 1).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_ledger_type_as_predefined_string_constant() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 0
        });

        let request = ledger_service.build_get_txn_request(Some(&identifier()), Some("POOL"), 1).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_ledger_type_as_number() {
        let ledger_service = LedgerService::new();

        let expected_result = json!({
            "type": GET_TXN,
            "data": 1,
            "ledgerId": 10
        });

        let request = ledger_service.build_get_txn_request(Some(&identifier()), Some("10"), 1).unwrap();
        check_request(&request, expected_result);
    }

    #[test]
    fn build_get_txn_request_works_for_invalid_type() {
        let ledger_service = LedgerService::new();

        let res = ledger_service.build_get_txn_request(Some(&identifier()), Some("type"), 1);
        assert_kind!(IndyErrorKind::InvalidStructure, res);
    }

    #[test]
    fn validate_action_works_for_pool_restart() {
        let ledger_service = LedgerService::new();
        let request = ledger_service.build_pool_restart(&identifier(), "start", None).unwrap();
        ledger_service.validate_action(&request).unwrap();
    }

    #[test]
    fn validate_action_works_for_get_validator_info() {
        let ledger_service = LedgerService::new();
        let request = ledger_service.build_get_validator_info_request(&identifier()).unwrap();
        ledger_service.validate_action(&request).unwrap();
    }

    mod auth_rule {
        use super::*;

        const ADD_AUTH_ACTION: &str = "ADD";
        const EDIT_AUTH_ACTION: &str = "EDIT";
        const FIELD: &str = "role";
        const OLD_VALUE: &str = "0";
        const NEW_VALUE: &str = "101";

        fn _role_constraint() -> Constraint {
            Constraint::RoleConstraint(RoleConstraint {
                sig_count: 0,
                metadata: None,
                role: Some(String::new()),
                need_to_be_owner: false,
                off_ledger_signature: false,
            })
        }

        fn _role_constraint_json() -> String {
            serde_json::to_string(&_role_constraint()).unwrap()
        }

        #[test]
        fn build_auth_rule_request_works_for_role_constraint() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
                "constraint": _role_constraint(),
            });

            let request = ledger_service.build_auth_rule_request(&identifier(), NYM, ADD_AUTH_ACTION, FIELD,
                                                                 None, Some(NEW_VALUE),
                                                                 _role_constraint()).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_combination_constraints() {
            let ledger_service = LedgerService::new();

            let constraint = Constraint::AndConstraint(
                CombinationConstraint {
                    auth_constraints: vec![
                        _role_constraint(),
                        Constraint::OrConstraint(
                            CombinationConstraint {
                                auth_constraints: vec![
                                    _role_constraint(), _role_constraint(), ],
                            }
                        )
                    ],
                });

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
                "constraint": constraint,
            });

            let request = ledger_service.build_auth_rule_request(&identifier(), NYM, ADD_AUTH_ACTION, FIELD,
                                                                 None, Some(NEW_VALUE),
                                                                 constraint).unwrap();

            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_edit_auth_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "old_value": OLD_VALUE,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::EDIT,
                "constraint": _role_constraint(),
            });

            let request = ledger_service.build_auth_rule_request(&identifier(), NYM, EDIT_AUTH_ACTION, FIELD,
                                                                 Some(OLD_VALUE), Some(NEW_VALUE),
                                                                 _role_constraint()).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_auth_rule_request_works_for_invalid_auth_action() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_auth_rule_request(&identifier(), NYM, "WRONG", FIELD, None, Some(NEW_VALUE), _role_constraint());
            assert_kind!(IndyErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_add_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::ADD,
            });

            let request = ledger_service.build_get_auth_rule_request(Some(&identifier()), Some(NYM),
                                                                     Some(ADD_AUTH_ACTION), Some(FIELD),
                                                                     None, Some(NEW_VALUE)).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_edit_action() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
                "auth_type": NYM,
                "field": FIELD,
                "old_value": OLD_VALUE,
                "new_value": NEW_VALUE,
                "auth_action": AuthAction::EDIT,
            });

            let request = ledger_service.build_get_auth_rule_request(Some(&identifier()), Some(NYM),
                                                                     Some(EDIT_AUTH_ACTION), Some(FIELD),
                                                                     Some(OLD_VALUE), Some(NEW_VALUE)).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_none_params() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_AUTH_RULE,
            });

            let request = ledger_service.build_get_auth_rule_request(Some(&identifier()), None,
                                                                     None, None,
                                                                     None, None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_some_fields_are_specified() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(Some(&identifier()), Some(NYM),
                                                                 None, Some(FIELD),
                                                                 None, None);
            assert_kind!(IndyErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_invalid_auth_action() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(Some(&identifier()), None, Some("WRONG"), None, None, None);
            assert_kind!(IndyErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_get_auth_rule_request_works_for_invalid_auth_type() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_auth_rule_request(Some(&identifier()), Some("WRONG"), None, None, None, None);
            assert_kind!(IndyErrorKind::InvalidStructure, res);
        }

        #[test]
        fn build_auth_rules_request_works() {
            let ledger_service = LedgerService::new();

            let mut data = AuthRules::new();
            data.push(AuthRuleData::Add(AddAuthRuleData {
                auth_type: NYM.to_string(),
                field: FIELD.to_string(),
                new_value: Some(NEW_VALUE.to_string()),
                constraint: _role_constraint(),
            }));

            data.push(AuthRuleData::Edit(EditAuthRuleData {
                auth_type: NYM.to_string(),
                field: FIELD.to_string(),
                old_value: Some(OLD_VALUE.to_string()),
                new_value: Some(NEW_VALUE.to_string()),
                constraint: _role_constraint(),
            }));

            let expected_result = json!({
                "type": AUTH_RULES,
                "rules": data.clone(),
            });

            let request = ledger_service.build_auth_rules_request(&identifier(), data).unwrap();
            check_request(&request, expected_result);
        }
    }

    mod author_agreement {
        use super::*;

        const TEXT: &str = "indy agreement";
        const VERSION: &str = "1.0.0";

        #[test]
        fn build_txn_author_agreement_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT,
                "text": TEXT,
                "version": VERSION
            });

            let request = ledger_service.build_txn_author_agreement_request(&identifier(), Some(TEXT), VERSION, None, None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_txn_author_agreement_request_works_for_retired_wo_text() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT,
                "version": VERSION,
                "ratification_ts": 12345,
                "retirement_ts": 54321,
            });

            let request = ledger_service.build_txn_author_agreement_request(&identifier(), None, VERSION, Some(12345), Some(54321)).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_txn_author_agreement_request_works() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT
            });

            let request = ledger_service.build_get_txn_author_agreement_request(Some(&identifier()), None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_txn_author_agreement_request_for_specific_version() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT,
                "version": VERSION
            });

            let data = GetTxnAuthorAgreementData {
                digest: None,
                version: Some(VERSION.to_string()),
                timestamp: None,
            };

            let request = ledger_service.build_get_txn_author_agreement_request(Some(&identifier()), Some(&data)).unwrap();
            check_request(&request, expected_result);
        }
    }

    mod acceptance_mechanism {
        use super::*;

        const LABEL: &str = "label";
        const VERSION: &str = "1.0.0";
        const CONTEXT: &str = "some context";
        const TIMESTAMP: u64 = 123456789;

        fn _aml() -> AcceptanceMechanisms {
            let mut aml: AcceptanceMechanisms = AcceptanceMechanisms::new();
            aml.0.insert(LABEL.to_string(), json!({"text": "This is description for acceptance mechanism"}));
            aml
        }

        #[test]
        fn build_acceptance_mechanisms_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT_AML,
                "aml":  _aml(),
                "version":  VERSION,
            });

            let request = ledger_service.build_acceptance_mechanisms_request(&identifier(), _aml(), VERSION, None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_acceptance_mechanisms_request_with_context() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": TXN_AUTHR_AGRMT_AML,
                "aml":  _aml(),
                "version":  VERSION,
                "amlContext": CONTEXT.to_string(),
            });

            let request = ledger_service.build_acceptance_mechanisms_request(&identifier(), _aml(), VERSION, Some(CONTEXT)).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
            });

            let request = ledger_service.build_get_acceptance_mechanisms_request(None, None, None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_timestamp() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
                "timestamp": TIMESTAMP,
            });

            let request = ledger_service.build_get_acceptance_mechanisms_request(None, Some(TIMESTAMP), None).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_version() {
            let ledger_service = LedgerService::new();

            let expected_result = json!({
                "type": GET_TXN_AUTHR_AGRMT_AML,
                "version": VERSION,
            });

            let request = ledger_service.build_get_acceptance_mechanisms_request(None, None, Some(VERSION)).unwrap();
            check_request(&request, expected_result);
        }

        #[test]
        fn build_get_acceptance_mechanisms_request_for_timestamp_and_version() {
            let ledger_service = LedgerService::new();

            let res = ledger_service.build_get_acceptance_mechanisms_request(None, Some(TIMESTAMP), Some(VERSION));
            assert_kind!(IndyErrorKind::InvalidStructure, res);
        }
    }

    #[test]
    fn datetime_to_date() {
        assert_eq!(0, LedgerService::datetime_to_date_timestamp(0));
        assert_eq!(0, LedgerService::datetime_to_date_timestamp(20));
        assert_eq!(1562284800, LedgerService::datetime_to_date_timestamp(1562367600));
        assert_eq!(1562284800, LedgerService::datetime_to_date_timestamp(1562319963));
        assert_eq!(1562284800, LedgerService::datetime_to_date_timestamp(1562284800));
    }

    fn check_request(request: &str, expected_result: serde_json::Value) {
        let request: serde_json::Value = serde_json::from_str(request).unwrap();
        assert_eq!(request["operation"], expected_result);
    }
}
