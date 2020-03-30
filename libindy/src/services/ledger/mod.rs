use serde::de::DeserializeOwned;
use serde_json;
use log_derive::logfn;

use ursa::cl::RevocationRegistryDelta as CryproRevocationRegistryDelta;

use crate::services::ledger::parsers::rev_reg_def::GetRevocRegDefReplyResult;
use crate::services::ledger::parsers::nym::{GetNymReplyResult, GetNymResultDataV0, NymData};
use crate::services::ledger::parsers::response::{Message, Reply};
use crate::services::ledger::parsers::rev_reg::GetRevocRegReplyResult;
use crate::services::ledger::parsers::schema::GetSchemaReplyResult;
use crate::services::ledger::parsers::cred_def::GetCredDefReplyResult;
use crate::services::ledger::parsers::rev_reg::GetRevocRegDeltaReplyResult;

use indy_api_types::errors::prelude::*;

use indy_vdr::ledger::{RequestBuilder, TxnAuthrAgrmtAcceptanceData};
use indy_vdr::common::did::DidValue;
use indy_vdr::ledger::requests::schema::{Schema, GetSchemaOperation, SchemaV1};
use indy_vdr::ledger::requests::cred_def::{CredentialDefinition, GetCredDefOperation, CredentialDefinitionV1};
use indy_vdr::ledger::requests::rev_reg_def::{RevocationRegistryDefinition, GetRevRegDefOperation, RegistryType};
use indy_vdr::ledger::requests::rev_reg::{RevocationRegistryDelta, GetRevRegOperation, GetRevRegDeltaOperation, RevocationRegistryDeltaV1, RevocationRegistry};
use indy_vdr::ledger::requests::node::NodeOperationData;
use indy_vdr::ledger::requests::auth_rule::{Constraint, AuthRules, AuthRule, GetAuthRuleResult};
use indy_vdr::ledger::requests::author_agreement::{GetTxnAuthorAgreementData, AcceptanceMechanisms};
use indy_vdr::ledger::requests::pool::Schedule;
use indy_vdr::ledger::identifiers::schema::SchemaId;
use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;
use indy_vdr::ledger::identifiers::rev_reg::RevocationRegistryId;
use indy_vdr::config::ProtocolVersion;
use std::cell::RefCell;
use indy_vdr::ledger::requests::{RequestType, Request};
use indy_vdr::ledger::requests::nym::GetNymOperation;
use indy_vdr::ledger::requests::constants::{GET_VALIDATOR_INFO, POOL_RESTART};
use indy_vdr::pool::LedgerType;
use indy_vdr::utils::qualifier::Qualifiable;

pub mod parsers;

macro_rules! build_result {
    ($self_:ident, $builder:ident, $($params:tt)*) => ({
        let request_json = $self_.request_builder()?
            .$builder($($params)*)?
            .req_json
            .to_string();

        Ok(request_json)
    })
}

pub struct LedgerService {
    protocol_version: RefCell<ProtocolVersion>,
}

impl LedgerService {
    pub fn new() -> LedgerService {
        LedgerService {
            protocol_version: RefCell::new(ProtocolVersion::Node1_4),
        }
    }

    pub fn request_builder(&self) -> IndyResult<RequestBuilder> {
        let protocol_version = self.get_protocol_version()?;
        Ok(RequestBuilder::new(protocol_version))
    }

    #[logfn(Info)]
    pub fn set_protocol_version(&self, version: usize) -> IndyResult<()> {
        let protocol_version = ProtocolVersion::from_id(version as u64)
            .map_err(|_err| IndyError::from_msg(IndyErrorKind::PoolIncompatibleProtocolVersion, format!("Unsupported Protocol version: {}", version)))?;

        *self.protocol_version.try_borrow_mut()? = protocol_version;

        Ok(())
    }

    #[logfn(Info)]
    pub fn get_protocol_version(&self) -> IndyResult<ProtocolVersion> {
        let protocol_version = self.protocol_version.try_borrow()?;
        Ok(protocol_version.clone())
    }

    #[logfn(Info)]
    pub fn build_nym_request(&self, identifier: &DidValue, dest: &DidValue, verkey: Option<&str>,
                             alias: Option<&str>, role: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_nym_request, identifier,
                                               dest,
                                               verkey.map(String::from),
                                               alias.map(String::from),
                                               role.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_get_nym_request(&self, identifier: Option<&DidValue>, dest: &DidValue) -> IndyResult<String> {
        build_result!(self, build_get_nym_request, identifier,
                                                   dest)
    }

    #[logfn(Info)]
    pub fn parse_get_nym_response(&self, get_nym_response: &str) -> IndyResult<String> {
        let reply: Reply<GetNymReplyResult> = LedgerService::parse_response::<GetNymOperation, GetNymReplyResult>(get_nym_response)?;

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
                    role: res.txn.data.role,
                }
            }
        };

        let res = serde_json::to_string(&nym_data)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot serialize NYM data: {}", err)))?;

        Ok(res)
    }

    #[logfn(Info)]
    pub fn build_get_ddo_request(&self, _identifier: Option<&DidValue>, _dest: &DidValue) -> IndyResult<String> {
        Ok(String::new())
    }

    #[logfn(Info)]
    pub fn build_attrib_request(&self, identifier: &DidValue, dest: &DidValue, hash: Option<&str>,
                                raw: Option<&serde_json::Value>, enc: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_attrib_request, identifier,
                                                  dest,
                                                  hash.map(String::from),
                                                  raw,
                                                  enc.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_get_attrib_request(&self, identifier: Option<&DidValue>, dest: &DidValue, raw: Option<&str>, hash: Option<&str>,
                                    enc: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_get_attrib_request, identifier,
                                                      dest,
                                                      raw.map(String::from),
                                                      hash.map(String::from),
                                                      enc.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_schema_request(&self, identifier: &DidValue, schema: Schema) -> IndyResult<String> {
        build_result!(self, build_schema_request, identifier,
                                                  schema)
    }

    #[logfn(Info)]
    pub fn build_get_schema_request(&self, identifier: Option<&DidValue>, id: &SchemaId) -> IndyResult<String> {
        build_result!(self, build_get_schema_request, identifier,
                                                      id)
    }

    #[logfn(Info)]
    pub fn build_cred_def_request(&self, identifier: &DidValue, cred_def: CredentialDefinition) -> IndyResult<String> {
        build_result!(self, build_cred_def_request, identifier,
                                                    cred_def)
    }

    #[logfn(Info)]
    pub fn build_get_cred_def_request(&self, identifier: Option<&DidValue>, id: &CredentialDefinitionId) -> IndyResult<String> {
        build_result!(self, build_get_cred_def_request, identifier,
                                                        id)
    }

    #[logfn(Info)]
    pub fn build_node_request(&self, identifier: &DidValue, dest: &DidValue, data: NodeOperationData) -> IndyResult<String> {
        build_result!(self, build_node_request, identifier,
                                                dest,
                                                data)
    }

    #[logfn(Info)]
    pub fn build_get_validator_info_request(&self, identifier: &DidValue) -> IndyResult<String> {
        build_result!(self, build_get_validator_info_request, identifier)
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

        build_result!(self, build_get_txn_request, identifier,
                                                   ledger_id,
                                                   seq_no)
    }

    #[logfn(Info)]
    pub fn build_pool_config(&self, identifier: &DidValue, writes: bool, force: bool) -> IndyResult<String> {
        build_result!(self, build_pool_config, identifier,
                                               writes,
                                               force)
    }

    #[logfn(Info)]
    pub fn build_pool_restart(&self, identifier: &DidValue, action: &str, datetime: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_pool_restart, identifier,
                                                action,
                                                datetime)
    }

    #[logfn(Info)]
    pub fn build_pool_upgrade(&self, identifier: &DidValue, name: &str, version: &str, action: &str,
                              sha256: &str, timeout: Option<u32>, schedule: Option<Schedule>,
                              justification: Option<&str>, reinstall: bool, force: bool, package: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_pool_upgrade, identifier,
                                                name,
                                                version,
                                                action,
                                                sha256,
                                                timeout,
                                                schedule,
                                                justification,
                                                reinstall,
                                                force,
                                                package)
    }

    #[logfn(Info)]
    pub fn build_revoc_reg_def_request(&self, identifier: &DidValue, rev_reg_def: RevocationRegistryDefinition) -> IndyResult<String> {
        build_result!(self, build_revoc_reg_def_request, identifier,
                                                         rev_reg_def)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_def_request(&self, identifier: Option<&DidValue>, id: &RevocationRegistryId) -> IndyResult<String> {
        build_result!(self, build_get_revoc_reg_def_request, identifier,
                                                             id)
    }

    #[logfn(Info)]
    pub fn build_revoc_reg_entry_request(&self, identifier: &DidValue, revoc_reg_def_id: &RevocationRegistryId,
                                         revoc_def_type: &str, rev_reg_entry: RevocationRegistryDelta) -> IndyResult<String> {
        let revoc_def_type = serde_json::from_str::<RegistryType>(&format!(r#""{}""#, revoc_def_type))
            .to_indy(IndyErrorKind::InvalidStructure, format!("Invalid Revocation Definition Type type: {}", revoc_def_type))?;

        build_result!(self, build_revoc_reg_entry_request, identifier,
                                                           revoc_reg_def_id,
                                                           &revoc_def_type,
                                                           rev_reg_entry)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_request(&self, identifier: Option<&DidValue>, revoc_reg_def_id: &RevocationRegistryId, timestamp: i64) -> IndyResult<String> {
        build_result!(self, build_get_revoc_reg_request, identifier,
                                                         revoc_reg_def_id,
                                                         timestamp)
    }

    #[logfn(Info)]
    pub fn build_get_revoc_reg_delta_request(&self, identifier: Option<&DidValue>, revoc_reg_def_id: &RevocationRegistryId, from: Option<i64>, to: i64) -> IndyResult<String> {
        build_result!(self, build_get_revoc_reg_delta_request, identifier,
                                                               revoc_reg_def_id,
                                                               from,
                                                               to)
    }

    #[logfn(Info)]
    pub fn parse_get_schema_response(&self, get_schema_response: &str, method_name: Option<&str>) -> IndyResult<(String, String)> {
        let reply: Reply<GetSchemaReplyResult> = LedgerService::parse_response::<GetSchemaOperation, GetSchemaReplyResult>(get_schema_response)?;

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
                        Some(method) => res.txn.data.id.to_qualified(method)?,
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
        let reply: Reply<GetCredDefReplyResult> = LedgerService::parse_response::<GetCredDefOperation, GetCredDefReplyResult>(get_cred_def_response)?;

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
                    Some(method) => res.txn.data.id.to_qualified(method)?,
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
        let reply: Reply<GetRevocRegDefReplyResult> = LedgerService::parse_response::<GetRevRegDefOperation, GetRevocRegDefReplyResult>(get_revoc_reg_def_response)?;

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
        let reply: Reply<GetRevocRegReplyResult> = LedgerService::parse_response::<GetRevRegOperation, GetRevocRegReplyResult>(get_revoc_reg_response)?;

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
        let reply: Reply<GetRevocRegDeltaReplyResult> = LedgerService::parse_response::<GetRevRegDeltaOperation, GetRevocRegDeltaReplyResult>(get_revoc_reg_delta_response)?;

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
        build_result!(self, build_auth_rule_request, submitter_did,
                                                     txn_type.to_string(),
                                                     action.to_string(),
                                                     field.to_string(),
                                                     old_value.map(String::from),
                                                     new_value.map(String::from),
                                                     constraint)
    }

    #[logfn(Info)]
    pub fn build_auth_rules_request(&self, submitter_did: &DidValue, rules: AuthRules) -> IndyResult<String> {
        build_result!(self, build_auth_rules_request, submitter_did,
                                                      rules)
    }

    #[logfn(Info)]
    pub fn build_get_auth_rule_request(&self, submitter_did: Option<&DidValue>, auth_type: Option<&str>, auth_action: Option<&str>,
                                       field: Option<&str>, old_value: Option<&str>, new_value: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_get_auth_rule_request, submitter_did,
                                                         auth_type.map(String::from),
                                                         auth_action.map(String::from),
                                                         field.map(String::from),
                                                         old_value.map(String::from),
                                                         new_value.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_txn_author_agreement_request(&self, identifier: &DidValue, text: Option<&str>, version: &str, ratification_ts: Option<u64>, retirement_ts: Option<u64>) -> IndyResult<String> {
        build_result!(self, build_txn_author_agreement_request, identifier,
                                                                text.map(String::from),
                                                                version.to_string(),
                                                                ratification_ts,
                                                                retirement_ts)
    }

    #[logfn(Info)]
    pub fn build_disable_all_txn_author_agreements_request(&self, identifier: &DidValue) -> IndyResult<String> {
        build_result!(self, build_disable_all_txn_author_agreements_request, identifier)
    }

    #[logfn(Info)]
    pub fn build_get_txn_author_agreement_request(&self, identifier: Option<&DidValue>, data: Option<&GetTxnAuthorAgreementData>) -> IndyResult<String> {
        build_result!(self, build_get_txn_author_agreement_request, identifier,
                                                                    data)
    }

    #[logfn(Info)]
    pub fn build_acceptance_mechanisms_request(&self, identifier: &DidValue, aml: AcceptanceMechanisms, version: &str, aml_context: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_acceptance_mechanisms_request, identifier,
                                                                 aml,
                                                                 version.to_string(),
                                                                 aml_context.map(String::from))
    }

    #[logfn(Info)]
    pub fn build_get_acceptance_mechanisms_request(&self, identifier: Option<&DidValue>, timestamp: Option<u64>, version: Option<&str>) -> IndyResult<String> {
        build_result!(self, build_get_acceptance_mechanisms_request, identifier,
                                                                     timestamp,
                                                                     version.map(String::from))
    }

    #[logfn(Info)]
    pub fn parse_response<T, M>(response: &str) -> IndyResult<Reply<M>> where T: RequestType, M: DeserializeOwned + ::std::fmt::Debug {
        let message: serde_json::Value = serde_json::from_str(&response)
            .to_indy(IndyErrorKind::InvalidTransaction, "Response is invalid json")?;

        if message["op"] == json!("REPLY") && message["result"]["type"] != json!(T::get_txn_type()) {
            return Err(err_msg(IndyErrorKind::InvalidTransaction, "Invalid response type"));
        }

        let message: Message<M> = serde_json::from_value(message)
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
        self.request_builder()?
            .prepare_txn_author_agreement_acceptance_data(text,
                                                          version,
                                                          hash,
                                                          mechanism,
                                                          time)
            .map_err(IndyError::from)
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

    use indy_vdr::ledger::constants::*;
    use indy_vdr::ledger::requests::node::Services;
    use indy_vdr::ledger::requests::auth_rule::*;

    use super::*;
    use indy_vdr::ledger::requests::schema::SchemaV1;

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

        let data = Schema::SchemaV1(SchemaV1 {
            id: SchemaId::new(&identifier(), "name", "1.0"),
            name: "name".to_string(),
            version: "1.0".to_string(),
            attr_names,
            seq_no: None,
        });

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
        let ledger_service = LedgerService::new();
        ledger_service.set_protocol_version(2).unwrap();

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


    fn check_request(request: &str, expected_result: serde_json::Value) {
        let request: serde_json::Value = serde_json::from_str(request).unwrap();
        assert_eq!(request["operation"], expected_result);
    }
}
