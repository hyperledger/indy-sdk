use indy_api_types::{CommandHandle, ErrorCode, PoolHandle, WalletHandle};
use indy_api_types::errors::prelude::*;
use indy_api_types::validation::Validatable;
use indy_utils::ctypes;
use libc::c_char;
use serde_json;

use crate::commands::{Command, CommandExecutor};
use crate::commands::ledger::LedgerCommand;
use crate::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionId};
use crate::domain::anoncreds::revocation_registry_definition::{RevocationRegistryDefinition, RevocationRegistryId};
use crate::domain::anoncreds::revocation_registry_delta::RevocationRegistryDelta;
use crate::domain::anoncreds::schema::{Schema, SchemaId};
use indy_api_types::domain::crypto::did::DidValue;
use crate::domain::ledger::auth_rule::{AuthRules, Constraint};
use crate::domain::ledger::author_agreement::{AcceptanceMechanisms, GetTxnAuthorAgreementData};
use crate::domain::ledger::node::NodeOperationData;
use crate::domain::ledger::pool::Schedule;

/// Signs and submits request message to validator pool.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign), and sends signed request message
/// to validator pool (see write_request).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_handle: pool handle (created by open_pool_ledger).
/// wallet_handle: wallet handle (created by open_wallet).
/// submitter_did: Id of Identity stored in secured Wallet.
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_sign_and_submit_request(command_handle: CommandHandle,
                                           pool_handle: PoolHandle,
                                           wallet_handle: WalletHandle,
                                           submitter_did: *const c_char,
                                           request_json: *const c_char,
                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode,
                                                                request_result_json: *const c_char)>) -> ErrorCode {
    trace!("indy_sign_and_submit_request: >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}",
           pool_handle, wallet_handle, submitter_did, request_json);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_sign_and_submit_request: entities >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}",
           pool_handle, wallet_handle, submitter_did, request_json);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SignAndSubmitRequest(
            pool_handle,
            wallet_handle,
            submitter_did,
            request_json,
            boxed_callback_string!("indy_sign_and_submit_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_sign_and_submit_request: <<< res: {:?}", res);

    res
}

/// Publishes request message to validator pool (no signing, unlike sign_and_submit_request).
///
/// The request is sent to the validator pool as is. It's assumed that it's already prepared.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_handle: pool handle (created by open_pool_ledger).
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_submit_request(command_handle: CommandHandle,
                                  pool_handle: PoolHandle,
                                  request_json: *const c_char,
                                  cb: Option<extern fn(command_handle_: CommandHandle,
                                                       err: ErrorCode,
                                                       request_result_json: *const c_char)>) -> ErrorCode {
    trace!("indy_submit_request: >>> pool_handle: {:?}, request_json: {:?}", pool_handle, request_json);

    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_submit_request: entities >>> pool_handle: {:?}, request_json: {:?}", pool_handle, request_json);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SubmitRequest(
            pool_handle,
            request_json,
            boxed_callback_string!("indy_submit_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_submit_request: <<< res: {:?}", res);

    res
}

/// Send action to particular nodes of validator pool.
///
/// The list of requests can be send:
///     POOL_RESTART
///     GET_VALIDATOR_INFO
///
/// The request is sent to the nodes as is. It's assumed that it's already prepared.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// pool_handle: pool handle (created by open_pool_ledger).
/// request_json: Request data json.
/// nodes: (Optional) List of node names to send the request.
///        ["Node1", "Node2",...."NodeN"]
/// timeout: (Optional) Time to wait respond from nodes (override the default timeout) (in sec).
///                     Pass -1 to use default timeout
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_submit_action(command_handle: CommandHandle,
                                 pool_handle: PoolHandle,
                                 request_json: *const c_char,
                                 nodes: *const c_char,
                                 timeout: i32,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode,
                                                      request_result_json: *const c_char)>) -> ErrorCode {
    trace!("indy_submit_action: >>> pool_handle: {:?}, request_json: {:?}, nodes: {:?}, timeout: {:?}", pool_handle, request_json, nodes, timeout);

    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(nodes, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let timeout = if timeout != -1 { Some(timeout) } else { None };

    trace!("indy_submit_action: entities >>> pool_handle: {:?}, request_json: {:?}, nodes: {:?}, timeout: {:?}", pool_handle, request_json, nodes, timeout);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::SubmitAction(
                pool_handle,
                request_json,
                nodes,
                timeout,
                boxed_callback_string!("indy_submit_action", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_submit_action: <<< res: {:?}", res);

    res
}

/// Signs request message.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// submitter_did: Id of Identity stored in secured Wallet.
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Signed request json.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_sign_request(command_handle: CommandHandle,
                                wallet_handle: WalletHandle,
                                submitter_did: *const c_char,
                                request_json: *const c_char,
                                cb: Option<extern fn(command_handle_: CommandHandle,
                                                     err: ErrorCode,
                                                     signed_request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_sign_request: >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_sign_request: entities >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::SignRequest(
            wallet_handle,
            submitter_did,
            request_json,
            boxed_callback_string!("indy_sign_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_sign_request: <<< res: {:?}", res);

    res
}

/// Multi signs request message.
///
/// Adds submitter information to passed request json, signs it with submitter
/// sign key (see wallet_sign).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// wallet_handle: wallet handle (created by open_wallet).
/// submitter_did: Id of Identity stored in secured Wallet.
/// request_json: Request data json.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Signed request json.
///
/// #Errors
/// Common*
/// Wallet*
/// Ledger*
/// Crypto*
#[no_mangle]
pub extern fn indy_multi_sign_request(command_handle: CommandHandle,
                                      wallet_handle: WalletHandle,
                                      submitter_did: *const c_char,
                                      request_json: *const c_char,
                                      cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                           signed_request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_multi_sign_request: >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_multi_sign_request: entities >>> wallet_handle: {:?}, submitter_did: {:?}, request_json: {:?}", wallet_handle, submitter_did, request_json);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::MultiSignRequest(
            wallet_handle,
            submitter_did,
            request_json,
            boxed_callback_string!("indy_multi_sign_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_multi_sign_request: <<< res: {:?}", res);

    res
}


/// Builds a request to get a DDO.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_ddo_request(command_handle: CommandHandle,
                                         submitter_did: *const c_char,
                                         target_did: *const c_char,
                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                              err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_ddo_request: >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_ddo_request: entities >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetDdoRequest(
            submitter_did,
            target_did,
            boxed_callback_string!("indy_build_get_ddo_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_ddo_request: <<< res: {:?}", res);

    res
}


/// Builds a NYM request. Request to create a new NYM record for a specific user.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
/// verkey: Target identity verification key as base58-encoded string.
/// alias: NYM's alias.
/// role: Role of a user NYM record:
///                             null (common USER)
///                             TRUSTEE
///                             STEWARD
///                             TRUST_ANCHOR
///                             ENDORSER - equal to TRUST_ANCHOR that will be removed soon
///                             NETWORK_MONITOR
///                             empty string to reset role
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_nym_request(command_handle: CommandHandle,
                                     submitter_did: *const c_char,
                                     target_did: *const c_char,
                                     verkey: *const c_char,
                                     alias: *const c_char,
                                     role: *const c_char,
                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                          err: ErrorCode,
                                                          request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_nym_request: >>> submitter_did: {:?}, target_did: {:?}, verkey: {:?}, alias: {:?}, role: {:?}",
           submitter_did, target_did, verkey, alias, role);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(verkey, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(alias, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(role, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_nym_request: entities >>> submitter_did: {:?}, target_did: {:?}, verkey: {:?}, alias: {:?}, role: {:?}",
           submitter_did, target_did, verkey, alias, role);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildNymRequest(
            submitter_did,
            target_did,
            verkey,
            alias,
            role,
            boxed_callback_string!("indy_build_nym_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_nym_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_NYM request. Request to get information about a DID (NYM).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_nym_request(command_handle: CommandHandle,
                                         submitter_did: *const c_char,
                                         target_did: *const c_char,
                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                              err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_nym_request: >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_nym_request: entities >>> submitter_did: {:?}, target_did: {:?}", submitter_did, target_did);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetNymRequest(
            submitter_did,
            target_did,
            boxed_callback_string!("indy_build_get_nym_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_nym_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_NYM response to get NYM data.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_nym_response: response on GET_NYM request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// NYM data
/// {
///     did: DID as base58-encoded string for 16 or 32 bit DID value.
///     verkey: verification key as base58-encoded string.
///     role: Role associated number
///                             null (common USER)
///                             0 - TRUSTEE
///                             2 - STEWARD
///                             101 - TRUST_ANCHOR
///                             101 - ENDORSER - equal to TRUST_ANCHOR that will be removed soon
///                             201 - NETWORK_MONITOR
/// }
///
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_nym_response(command_handle: CommandHandle,
                                          get_nym_response: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               nym_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_nym_response: >>> get_nym_response: {:?}", get_nym_response);

    check_useful_c_str!(get_nym_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_nym_response: entities >>> get_nym_response: {:?}", get_nym_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetNymResponse(
            get_nym_response,
            boxed_callback_string!("indy_parse_get_nym_response", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_nym_response: <<< res: {:?}", res);

    res
}

/// Builds an ATTRIB request. Request to add attribute to a NYM record.
///
/// Note: one of the fields `hash`, `raw`, `enc` must be specified.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
/// hash: (Optional) Hash of attribute data.
/// raw: (Optional) Json, where key is attribute name and value is attribute value.
/// enc: (Optional) Encrypted value attribute data.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_attrib_request(command_handle: CommandHandle,
                                        submitter_did: *const c_char,
                                        target_did: *const c_char,
                                        hash: *const c_char,
                                        raw: *const c_char,
                                        enc: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_attrib_request: >>> submitter_did: {:?}, target_did: {:?}, hash: {:?}, raw: {:?}, enc: {:?}",
           submitter_did, target_did, hash, raw, enc);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(hash, ErrorCode::CommonInvalidParam4);
    check_useful_opt_json!(raw, ErrorCode::CommonInvalidParam5, serde_json::Value);
    check_useful_opt_c_str!(enc, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_attrib_request: entities >>> submitter_did: {:?}, target_did: {:?}, hash: {:?}, raw: {:?}, enc: {:?}",
           submitter_did, target_did, hash, raw, enc);

    if raw.is_none() && hash.is_none() && enc.is_none() {
        return IndyError::from_msg(IndyErrorKind::InvalidStructure, "Either raw or hash or enc must be specified").into();
    }

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildAttribRequest(
            submitter_did,
            target_did,
            hash,
            raw,
            enc,
            boxed_callback_string!("indy_build_attrib_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_attrib_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_ATTRIB request. Request to get information about an Attribute for the specified DID.
///
/// Note: one of the fields `hash`, `raw`, `enc` must be specified.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// target_did: Target DID as base58-encoded string for 16 or 32 bit DID value.
/// raw: (Optional) Requested attribute name.
/// hash: (Optional) Requested attribute hash.
/// enc: (Optional) Requested attribute encrypted value.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_attrib_request(command_handle: CommandHandle,
                                            submitter_did: *const c_char,
                                            target_did: *const c_char,
                                            raw: *const c_char,
                                            hash: *const c_char,
                                            enc: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_attrib_request: >>> submitter_did: {:?}, target_did: {:?}, hash: {:?}, raw: {:?}, enc: {:?}",
           submitter_did, target_did, hash, raw, enc);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(raw, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(hash, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(enc, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_get_attrib_request: entities >>> submitter_did: {:?}, target_did: {:?}, hash: {:?}, raw: {:?}, enc: {:?}",
           submitter_did, target_did, hash, raw, enc);

    if raw.is_none() && hash.is_none() && enc.is_none() {
        return IndyError::from_msg(IndyErrorKind::InvalidStructure, "Either raw or hash or enc must be specified").into();
    }

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetAttribRequest(
            submitter_did,
            target_did,
            raw,
            hash,
            enc,
            boxed_callback_string!("indy_build_get_attrib_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_attrib_request: <<< res: {:?}", res);

    res
}

/// Builds a SCHEMA request. Request to add Credential's schema.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// data: Credential schema.
/// {
///     id: identifier of schema
///     attrNames: array of attribute name strings (the number of attributes should be less or equal than 125)
///     name: Schema's name string
///     version: Schema's version string,
///     ver: Version of the Schema json
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_schema_request(command_handle: CommandHandle,
                                        submitter_did: *const c_char,
                                        data: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle,
                                                             err: ErrorCode,
                                                             request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_schema_request: >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_json!(data, ErrorCode::CommonInvalidParam3, Schema);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_schema_request: entities >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildSchemaRequest(
            submitter_did,
            data,
            boxed_callback_string!("indy_build_schema_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_schema_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_SCHEMA request. Request to get Credential's Schema.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// id: Schema ID in ledger
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_schema_request(command_handle: CommandHandle,
                                            submitter_did: *const c_char,
                                            id: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_schema_request: >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(id, ErrorCode::CommonInvalidParam3, SchemaId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_schema_request: entities >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetSchemaRequest(
            submitter_did,
            id,
            boxed_callback_string!("indy_build_get_schema_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_schema_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_SCHEMA response to get Schema in the format compatible with Anoncreds API.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_schema_response: response of GET_SCHEMA request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Schema Id and Schema json.
/// {
///     id: identifier of schema
///     attrNames: array of attribute name strings
///     name: Schema's name string
///     version: Schema's version string
///     ver: Version of the Schema json
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_schema_response(command_handle: CommandHandle,
                                             get_schema_response: *const c_char,
                                             cb: Option<extern fn(command_handle_: CommandHandle,
                                                                  err: ErrorCode,
                                                                  schema_id: *const c_char,
                                                                  schema_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_schema_response: >>> get_schema_response: {:?}", get_schema_response);

    check_useful_c_str!(get_schema_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_schema_response: entities >>> get_schema_response: {:?}", get_schema_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetSchemaResponse(
            get_schema_response,
            Box::new(move |result| {
                let (err, schema_id, schema_json) = prepare_result_2!(result, String::new(), String::new());
                trace!("indy_parse_get_schema_response: schema_id: {:?}, schema_json: {:?}", schema_id, schema_json);
                let schema_id = ctypes::string_to_cstring(schema_id);
                let schema_json = ctypes::string_to_cstring(schema_json);
                cb(command_handle, err, schema_id.as_ptr(), schema_json.as_ptr())
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_schema_response: <<< res: {:?}", res);

    res
}

/// Builds an CRED_DEF request. Request to add a Credential Definition (in particular, public key),
/// that Issuer creates for a particular Credential Schema.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// data: credential definition json
/// {
///     id: string - identifier of credential definition
///     schemaId: string - identifier of stored in ledger schema
///     type: string - type of the credential definition. CL is the only supported type now.
///     tag: string - allows to distinct between credential definitions for the same issuer and schema
///     value: Dictionary with Credential Definition's data: {
///         primary: primary credential public key,
///         Optional<revocation>: revocation credential public key
///     },
///     ver: Version of the CredDef json
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_cred_def_request(command_handle: CommandHandle,
                                          submitter_did: *const c_char,
                                          data: *const c_char,
                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                               err: ErrorCode,
                                                               request_result_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_cred_def_request: >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_json!(data, ErrorCode::CommonInvalidParam3, CredentialDefinition);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_cred_def_request: entities >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildCredDefRequest(
            submitter_did,
            data,
            boxed_callback_string!("indy_build_cred_def_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_cred_def_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_CRED_DEF request. Request to get a Credential Definition (in particular, public key),
/// that Issuer creates for a particular Credential Schema.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// id: Credential Definition ID in ledger.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_cred_def_request(command_handle: CommandHandle,
                                              submitter_did: *const c_char,
                                              id: *const c_char,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: ErrorCode,
                                                                   request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_cred_def_request: >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_cred_def_request: entities >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetCredDefRequest(
            submitter_did,
            id,
            boxed_callback_string!("indy_build_get_cred_def_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_cred_def_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_CRED_DEF response to get Credential Definition in the format compatible with Anoncreds API.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_cred_def_response: response of GET_CRED_DEF request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Credential Definition Id and Credential Definition json.
/// {
///     id: string - identifier of credential definition
///     schemaId: string - identifier of stored in ledger schema
///     type: string - type of the credential definition. CL is the only supported type now.
///     tag: string - allows to distinct between credential definitions for the same issuer and schema
///     value: Dictionary with Credential Definition's data: {
///         primary: primary credential public key,
///         Optional<revocation>: revocation credential public key
///     },
///     ver: Version of the Credential Definition json
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_cred_def_response(command_handle: CommandHandle,
                                               get_cred_def_response: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    cred_def_id: *const c_char,
                                                                    cred_def_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_cred_def_response: >>> get_cred_def_response: {:?}", get_cred_def_response);

    check_useful_c_str!(get_cred_def_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_cred_def_response: entities >>> get_cred_def_response: {:?}", get_cred_def_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetCredDefResponse(
            get_cred_def_response,
            Box::new(move |result| {
                let (err, cred_def_id, cred_def_json) = prepare_result_2!(result, String::new(), String::new());
                trace!("indy_parse_get_cred_def_response: cred_def_id: {:?}, cred_def_json: {:?}", cred_def_id, cred_def_json);
                let cred_def_id = ctypes::string_to_cstring(cred_def_id);
                let cred_def_json = ctypes::string_to_cstring(cred_def_json);
                cb(command_handle, err, cred_def_id.as_ptr(), cred_def_json.as_ptr())
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_cred_def_response: <<< res: {:?}", res);

    res
}

/// Builds a NODE request. Request to add a new node to the pool, or updates existing in the pool.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// target_did: Target Node's DID.  It differs from submitter_did field.
/// data: Data associated with the Node: {
///     alias: string - Node's alias
///     blskey: string - (Optional) BLS multi-signature key as base58-encoded string.
///     blskey_pop: string - (Optional) BLS key proof of possession as base58-encoded string.
///     client_ip: string - (Optional) Node's client listener IP address.
///     client_port: string - (Optional) Node's client listener port.
///     node_ip: string - (Optional) The IP address other Nodes use to communicate with this Node.
///     node_port: string - (Optional) The port other Nodes use to communicate with this Node.
///     services: array<string> - (Optional) The service of the Node. VALIDATOR is the only supported one now.
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_node_request(command_handle: CommandHandle,
                                      submitter_did: *const c_char,
                                      target_did: *const c_char,
                                      data: *const c_char,
                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                           err: ErrorCode,
                                                           request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_node_request: >>> submitter_did: {:?}, target_did: {:?}, data: {:?}", submitter_did, target_did, data);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(target_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_json!(data, ErrorCode::CommonInvalidParam4, NodeOperationData);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_node_request: entities >>> submitter_did: {:?}, target_did: {:?}, data: {:?}", submitter_did, target_did, data);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildNodeRequest(
            submitter_did,
            target_did,
            data,
            boxed_callback_string!("indy_build_node_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_node_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_VALIDATOR_INFO request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: DID of the read request sender.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_validator_info_request(command_handle: CommandHandle,
                                                    submitter_did: *const c_char,
                                                    cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                         request_json: *const c_char)>) -> ErrorCode {
    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetValidatorInfoRequest(
            submitter_did,
            boxed_callback_string!("indy_build_get_validator_info_request", cb, command_handle)
        )));

    prepare_result!(result)
}

/// Builds a GET_TXN request. Request to get any transaction by its seq_no.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// ledger_type: (Optional) type of the ledger the requested transaction belongs to:
///     DOMAIN - used default,
///     POOL,
///     CONFIG
///     any number
/// seq_no: requested transaction sequence number as it's stored on Ledger.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_txn_request(command_handle: CommandHandle,
                                         submitter_did: *const c_char,
                                         ledger_type: *const c_char,
                                         seq_no: i32,
                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                              err: ErrorCode,
                                                              request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_txn_request: >>> submitter_did: {:?}, ledger_type: {:?}, seq_no: {:?}", submitter_did, ledger_type, seq_no);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_opt_c_str!(ledger_type, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_get_txn_request: entities >>> submitter_did: {:?}, ledger_type: {:?}, seq_no: {:?}", submitter_did, ledger_type, seq_no);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetTxnRequest(
            submitter_did,
            ledger_type,
            seq_no,
            boxed_callback_string!("indy_build_get_txn_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_txn_request: <<< res: {:?}", res);

    res
}

/// Builds a POOL_CONFIG request. Request to change Pool's configuration.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// writes: Whether any write requests can be processed by the pool
///         (if false, then pool goes to read-only state). True by default.
/// force: Whether we should apply transaction (for example, move pool to read-only state)
///        without waiting for consensus of this transaction.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_config_request(command_handle: CommandHandle,
                                             submitter_did: *const c_char,
                                             writes: bool,
                                             force: bool,
                                             cb: Option<extern fn(command_handle_: CommandHandle,
                                                                  err: ErrorCode,
                                                                  request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_pool_config_request: >>> submitter_did: {:?}, writes: {:?}, force: {:?}", submitter_did, writes, force);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_pool_config_request: entities >>> submitter_did: {:?}, writes: {:?}, force: {:?}", submitter_did, writes, force);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildPoolConfigRequest(
            submitter_did,
            writes,
            force,
            boxed_callback_string!("indy_build_pool_config_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_pool_config_request: <<< res: {:?}", res);

    res
}

/// Builds a POOL_RESTART request.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
/// action:        Action that pool has to do after received transaction. Either `start` or `cancel`.
/// datetime:      <Optional> Restart time in datetime format. Skip to restart as early as possible.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_restart_request(command_handle: CommandHandle,
                                              submitter_did: *const c_char,
                                              action: *const c_char,
                                              datetime: *const c_char,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: ErrorCode,
                                                                   request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_pool_restart_request: >>> submitter_did: {:?}, action: {:?}, datetime: {:?}", submitter_did, action, datetime);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(action, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(datetime, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_pool_restart_request: entities >>> submitter_did: {:?}, action: {:?}, datetime: {:?}", submitter_did, action, datetime);

    if action != "start" && action != "cancel" {
        return IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Unsupported action: {}. Must be either `start` or `cancel`", action)).into();
    }

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildPoolRestartRequest(
                submitter_did,
                action,
                datetime,
                boxed_callback_string!("indy_build_pool_restart_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_pool_restart_request: <<< res: {:?}", res);

    res
}


/// Builds a POOL_UPGRADE request. Request to upgrade the Pool (sent by Trustee).
/// It upgrades the specified Nodes (either all nodes in the Pool, or some specific ones).
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// name: Human-readable name for the upgrade.
/// version: The version of indy-node package we perform upgrade to.
///          Must be greater than existing one (or equal if reinstall flag is True).
/// action: Either start or cancel.
/// sha256: sha256 hash of the package.
/// timeout: (Optional) Limits upgrade time on each Node.
/// schedule: (Optional) Schedule of when to perform upgrade on each node. Map Node DIDs to upgrade time.
/// justification: (Optional) justification string for this particular Upgrade.
/// reinstall: Whether it's allowed to re-install the same version. False by default.
/// force: Whether we should apply transaction (schedule Upgrade) without waiting
///        for consensus of this transaction.
/// package: (Optional) Package to be upgraded.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_pool_upgrade_request(command_handle: CommandHandle,
                                              submitter_did: *const c_char,
                                              name: *const c_char,
                                              version: *const c_char,
                                              action: *const c_char,
                                              sha256: *const c_char,
                                              timeout: i32,
                                              schedule: *const c_char,
                                              justification: *const c_char,
                                              reinstall: bool,
                                              force: bool,
                                              package: *const c_char,
                                              cb: Option<extern fn(command_handle_: CommandHandle,
                                                                   err: ErrorCode,
                                                                   request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_pool_upgrade_request: >>> submitter_did: {:?}, name: {:?}, version: {:?}, action: {:?}, sha256: {:?}, timeout: {:?}, \
    schedule: {:?}, justification: {:?}, reinstall: {:?}, force: {:?}, package: {:?}",
           submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(name, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(action, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(sha256, ErrorCode::CommonInvalidParam6);
    check_useful_opt_json!(schedule, ErrorCode::CommonInvalidParam8, Schedule);
    check_useful_opt_c_str!(justification, ErrorCode::CommonInvalidParam9);
    check_useful_opt_c_str!(package, ErrorCode::CommonInvalidParam12);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam13);

    let timeout = if timeout != -1 { Some(timeout as u32) } else { None };

    trace!("indy_build_pool_upgrade_request: entities >>> submitter_did: {:?}, name: {:?}, version: {:?}, action: {:?}, sha256: {:?}, timeout: {:?}, \
    schedule: {:?}, justification: {:?}, reinstall: {:?}, force: {:?}, package: {:?}",
           submitter_did, name, version, action, sha256, timeout, schedule, justification, reinstall, force, package);

    if action != "start" && action != "cancel" {
        return IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Invalid action: {}", action)).into();
    }

    if action == "start" && schedule.is_none() {
        return IndyError::from_msg(IndyErrorKind::InvalidStructure, format!("Schedule is required for `{}` action", action)).into();
    }

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildPoolUpgradeRequest(
                submitter_did,
                name,
                version,
                action,
                sha256,
                timeout,
                schedule,
                justification,
                reinstall,
                force,
                package,
                boxed_callback_string!("indy_build_pool_upgrade_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_pool_upgrade_request: <<< res: {:?}", res);

    res
}

/// Builds a REVOC_REG_DEF request. Request to add the definition of revocation registry
/// to an exists credential definition.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// data: Revocation Registry data:
///     {
///         "id": string - ID of the Revocation Registry,
///         "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
///         "tag": string - Unique descriptive ID of the Registry,
///         "credDefId": string - ID of the corresponding CredentialDefinition,
///         "value": Registry-specific data {
///             "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
///             "maxCredNum": number - Maximum number of credentials the Registry can serve.
///             "tailsHash": string - Hash of tails.
///             "tailsLocation": string - Location of tails file.
///             "publicKeys": <public_keys> - Registry's public key.
///         },
///         "ver": string - version of revocation registry definition json.
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_revoc_reg_def_request(command_handle: CommandHandle,
                                               submitter_did: *const c_char,
                                               data: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    rev_reg_def_req: *const c_char)>) -> ErrorCode {
    trace!("indy_build_revoc_reg_def_request: >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_json!(data, ErrorCode::CommonInvalidParam3, RevocationRegistryDefinition);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_revoc_reg_def_request: entities >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildRevocRegDefRequest(
            submitter_did,
            data,
            boxed_callback_string!("indy_build_revoc_reg_def_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_revoc_reg_def_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_REVOC_REG_DEF request. Request to get a revocation registry definition,
/// that Issuer creates for a particular Credential Definition.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// id:  ID of Revocation Registry Definition in ledger.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_revoc_reg_def_request(command_handle: CommandHandle,
                                                   submitter_did: *const c_char,
                                                   id: *const c_char,
                                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                                        err: ErrorCode,
                                                                        request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_revoc_reg_def_request: >>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(id, ErrorCode::CommonInvalidParam3, RevocationRegistryId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_revoc_reg_def_request: entities>>> submitter_did: {:?}, id: {:?}", submitter_did, id);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetRevocRegDefRequest(
            submitter_did,
            id,
            boxed_callback_string!("indy_build_get_revoc_reg_def_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_revoc_reg_def_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_REVOC_REG_DEF response to get Revocation Registry Definition in the format
/// compatible with Anoncreds API.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_revoc_reg_def_response: response of GET_REVOC_REG_DEF request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revocation Registry Definition Id and Revocation Registry Definition json.
/// {
///     "id": string - ID of the Revocation Registry,
///     "revocDefType": string - Revocation Registry type (only CL_ACCUM is supported for now),
///     "tag": string - Unique descriptive ID of the Registry,
///     "credDefId": string - ID of the corresponding CredentialDefinition,
///     "value": Registry-specific data {
///         "issuanceType": string - Type of Issuance(ISSUANCE_BY_DEFAULT or ISSUANCE_ON_DEMAND),
///         "maxCredNum": number - Maximum number of credentials the Registry can serve.
///         "tailsHash": string - Hash of tails.
///         "tailsLocation": string - Location of tails file.
///         "publicKeys": <public_keys> - Registry's public key.
///     },
///     "ver": string - version of revocation registry definition json.
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_revoc_reg_def_response(command_handle: CommandHandle,
                                                    get_revoc_reg_def_response: *const c_char,
                                                    cb: Option<extern fn(command_handle_: CommandHandle,
                                                                         err: ErrorCode,
                                                                         revoc_reg_def_id: *const c_char,
                                                                         revoc_reg_def_json: *const c_char)>) -> ErrorCode {
    trace!("indy_parse_get_revoc_reg_def_response: >>> get_revoc_reg_def_response: {:?}", get_revoc_reg_def_response);

    check_useful_c_str!(get_revoc_reg_def_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_revoc_reg_def_response: entities >>> get_revoc_reg_def_response: {:?}", get_revoc_reg_def_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetRevocRegDefResponse(
            get_revoc_reg_def_response,
            Box::new(move |result| {
                let (err, revoc_reg_def_id, revoc_reg_def_json) = prepare_result_2!(result, String::new(), String::new());
                trace!("indy_parse_get_revoc_reg_def_response: revoc_reg_def_id: {:?}, revoc_reg_def_json: {:?}", revoc_reg_def_id, revoc_reg_def_json);

                let revoc_reg_def_id = ctypes::string_to_cstring(revoc_reg_def_id);
                let revoc_reg_def_json = ctypes::string_to_cstring(revoc_reg_def_json);
                cb(command_handle, err, revoc_reg_def_id.as_ptr(), revoc_reg_def_json.as_ptr())
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_revoc_reg_def_response: <<< res: {:?}", res);

    res
}

/// Builds a REVOC_REG_ENTRY request.  Request to add the RevocReg entry containing
/// the new accumulator value and issued/revoked indices.
/// This is just a delta of indices, not the whole list.
/// So, it can be sent each time a new credential is issued/revoked.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// revoc_reg_def_id: ID of the corresponding RevocRegDef.
/// rev_def_type: Revocation Registry type (only CL_ACCUM is supported for now).
/// value: Registry-specific data: {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> - an array of issued indices.
///         revoked: array<number> an array of revoked indices.
///     },
///     ver: string - version revocation registry entry json
/// }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_revoc_reg_entry_request(command_handle: CommandHandle,
                                                 submitter_did: *const c_char,
                                                 revoc_reg_def_id: *const c_char,
                                                 rev_def_type: *const c_char,
                                                 value: *const c_char,
                                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                                      err: ErrorCode,
                                                                      request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_revoc_reg_entry_request: >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, rev_def_type: {:?}, value: {:?}",
           submitter_did, revoc_reg_def_id, rev_def_type, value);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(revoc_reg_def_id, ErrorCode::CommonInvalidParam3, RevocationRegistryId);
    check_useful_c_str!(rev_def_type, ErrorCode::CommonInvalidParam4);
    check_useful_json!(value, ErrorCode::CommonInvalidParam5, RevocationRegistryDelta);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_build_revoc_reg_entry_request: entities >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, rev_def_type: {:?}, value: {:?}",
           submitter_did, revoc_reg_def_id, rev_def_type, value);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildRevocRegEntryRequest(
            submitter_did,
            revoc_reg_def_id,
            rev_def_type,
            value,
            boxed_callback_string!("indy_build_revoc_reg_entry_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_revoc_reg_entry_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_REVOC_REG request. Request to get the accumulated state of the Revocation Registry
/// by ID. The state is defined by the given timestamp.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
/// timestamp: Requested time represented as a total number of seconds from Unix Epoch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_revoc_reg_request(command_handle: CommandHandle,
                                               submitter_did: *const c_char,
                                               revoc_reg_def_id: *const c_char,
                                               timestamp: i64,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_revoc_reg_request: >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, timestamp: {:?}", submitter_did, revoc_reg_def_id, timestamp);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(revoc_reg_def_id, ErrorCode::CommonInvalidParam3, RevocationRegistryId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_build_get_revoc_reg_request: entities >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, timestamp: {:?}", submitter_did, revoc_reg_def_id, timestamp);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetRevocRegRequest(
            submitter_did,
            revoc_reg_def_id,
            timestamp,
            boxed_callback_string!("indy_build_get_revoc_reg_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_revoc_reg_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_REVOC_REG response to get Revocation Registry in the format compatible with Anoncreds API.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_revoc_reg_response: response of GET_REVOC_REG request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revocation Registry Definition Id, Revocation Registry json and Timestamp.
/// {
///     "value": Registry-specific data {
///         "accum": string - current accumulator value.
///     },
///     "ver": string - version revocation registry json
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_revoc_reg_response(command_handle: CommandHandle,
                                                get_revoc_reg_response: *const c_char,
                                                cb: Option<extern fn(command_handle_: CommandHandle,
                                                                     err: ErrorCode,
                                                                     revoc_reg_def_id: *const c_char,
                                                                     revoc_reg_json: *const c_char,
                                                                     timestamp: u64)>) -> ErrorCode {
    trace!("indy_parse_get_revoc_reg_response: >>> get_revoc_reg_response: {:?}", get_revoc_reg_response);

    check_useful_c_str!(get_revoc_reg_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_revoc_reg_response: entities >>> get_revoc_reg_response: {:?}", get_revoc_reg_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetRevocRegResponse(
            get_revoc_reg_response,
            Box::new(move |result| {
                let (err, revoc_reg_def_id, revoc_reg_json, timestamp) = prepare_result_3!(result, String::new(), String::new(), 0);
                trace!("indy_parse_get_revoc_reg_response: revoc_reg_def_id: {:?}, revoc_reg_json: {:?}, timestamp: {:?}",
                       revoc_reg_def_id, revoc_reg_json, timestamp);

                let revoc_reg_def_id = ctypes::string_to_cstring(revoc_reg_def_id);
                let revoc_reg_json = ctypes::string_to_cstring(revoc_reg_json);
                cb(command_handle, err, revoc_reg_def_id.as_ptr(), revoc_reg_json.as_ptr(), timestamp)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_revoc_reg_response: <<< res: {:?}", res);

    res
}

/// Builds a GET_REVOC_REG_DELTA request. Request to get the delta of the accumulated state of the Revocation Registry.
/// The Delta is defined by from and to timestamp fields.
/// If from is not specified, then the whole state till to will be returned.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// revoc_reg_def_id:  ID of the corresponding Revocation Registry Definition in ledger.
/// from: Requested time represented as a total number of seconds from Unix Epoch
/// to: Requested time represented as a total number of seconds from Unix Epoch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_revoc_reg_delta_request(command_handle: CommandHandle,
                                                     submitter_did: *const c_char,
                                                     revoc_reg_def_id: *const c_char,
                                                     from: i64,
                                                     to: i64,
                                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                                          err: ErrorCode,
                                                                          request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_revoc_reg_request: >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, from: {:?}, to: {:?}",
           submitter_did, revoc_reg_def_id, from, to);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_string!(revoc_reg_def_id, ErrorCode::CommonInvalidParam3, RevocationRegistryId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let from = if from != -1 { Some(from) } else { None };

    trace!("indy_build_get_revoc_reg_request: entities >>> submitter_did: {:?}, revoc_reg_def_id: {:?}, from: {:?}, to: {:?}",
           submitter_did, revoc_reg_def_id, from, to);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetRevocRegDeltaRequest(
            submitter_did,
            revoc_reg_def_id,
            from,
            to,
            boxed_callback_string!("indy_build_get_revoc_reg_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_revoc_reg_request: <<< res: {:?}", res);

    res
}

/// Parse a GET_REVOC_REG_DELTA response to get Revocation Registry Delta in the format compatible with Anoncreds API.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// get_revoc_reg_response: response of GET_REVOC_REG_DELTA request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Revocation Registry Definition Id, Revocation Registry Delta json and Timestamp.
/// {
///     "value": Registry-specific data {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> - an array of issued indices.
///         revoked: array<number> an array of revoked indices.
///     },
///     "ver": string - version revocation registry delta json
/// }
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_parse_get_revoc_reg_delta_response(command_handle: CommandHandle,
                                                      get_revoc_reg_delta_response: *const c_char,
                                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                                           err: ErrorCode,
                                                                           revoc_reg_def_id: *const c_char,
                                                                           revoc_reg_delta_json: *const c_char,
                                                                           timestamp: u64)>) -> ErrorCode {
    trace!("indy_parse_get_revoc_reg_delta_response: >>> get_revoc_reg_delta_response: {:?}", get_revoc_reg_delta_response);

    check_useful_c_str!(get_revoc_reg_delta_response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_parse_get_revoc_reg_delta_response: entities >>> get_revoc_reg_delta_response: {:?}", get_revoc_reg_delta_response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::ParseGetRevocRegDeltaResponse(
            get_revoc_reg_delta_response,
            Box::new(move |result| {
                let (err, revoc_reg_def_id, revoc_reg_delta_json, timestamp) = prepare_result_3!(result, String::new(), String::new(), 0);
                trace!("indy_parse_get_revoc_reg_delta_response: revoc_reg_def_id: {:?}, revoc_reg_delta_json: {:?}, timestamp: {:?}",
                       revoc_reg_def_id, revoc_reg_delta_json, timestamp);

                let revoc_reg_def_id = ctypes::string_to_cstring(revoc_reg_def_id);
                let revoc_reg_delta_json = ctypes::string_to_cstring(revoc_reg_delta_json);
                cb(command_handle, err, revoc_reg_def_id.as_ptr(), revoc_reg_delta_json.as_ptr(), timestamp)
            })
        )));

    let res = prepare_result!(result);

    trace!("indy_parse_get_revoc_reg_delta_response: <<< res: {:?}", res);

    res
}

/// Callback type for parsing Reply from Node to specific StateProof format
///
/// # params
/// reply_from_node: string representation of node's reply ("as is")
/// parsed_sp: out param to return serialized as string JSON with array of ParsedSP
///
/// # return
/// result ErrorCode
///
/// Note: this method allocate memory for result string `CustomFree` should be called to deallocate it
pub type CustomTransactionParser = extern fn(reply_from_node: *const c_char, parsed_sp: *mut *const c_char) -> ErrorCode;

/// Callback type to deallocate result buffer `parsed_sp` from `CustomTransactionParser`
pub type CustomFree = extern fn(data: *const c_char) -> ErrorCode;


/// Register callbacks (see type description for `CustomTransactionParser` and `CustomFree`
///
/// # params
/// command_handle: command handle to map callback to caller context.
/// txn_type: type of transaction to apply `parse` callback.
/// parse: required callback to parse reply for state proof.
/// free: required callback to deallocate memory.
/// cb: Callback that takes command result as parameter.
///
/// # returns
/// Status of callbacks registration.
///
/// # errors
/// Common*
#[no_mangle]
pub extern fn indy_register_transaction_parser_for_sp(command_handle: CommandHandle,
                                                      txn_type: *const c_char,
                                                      parser: Option<CustomTransactionParser>,
                                                      free: Option<CustomFree>,
                                                      cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_register_transaction_parser_for_sp: >>> txn_type {:?}, parser {:?}, free {:?}",
           txn_type, parser, free);

    check_useful_c_str!(txn_type, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(parser, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(free, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_register_transaction_parser_for_sp: entities: txn_type {}, parser {:?}, free {:?}",
           txn_type, parser, free);

    let res = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::RegisterSPParser(
            txn_type,
            parser,
            free,
            Box::new(move |res| {
                let res = prepare_result!(res);
                trace!("indy_register_transaction_parser_for_sp: res: {:?}", res);
                cb(command_handle, res)
            }),
        )));

    let res = prepare_result!(res);

    trace!("indy_register_transaction_parser_for_sp: <<< res: {:?}", res);

    res
}

/// Parse transaction response to fetch metadata.
/// The important use case for this method is validation of Node's response freshens.
///
/// Distributed Ledgers can reply with outdated information for consequence read request after write.
/// To reduce pool load libindy sends read requests to one random node in the pool.
/// Consensus validation is performed based on validation of nodes multi signature for current ledger Merkle Trie root.
/// This multi signature contains information about the latest ldeger's transaction ordering time and sequence number that this method returns.
///
/// If node that returned response for some reason is out of consensus and has outdated ledger
/// it can be caught by analysis of the returned latest ledger's transaction ordering time and sequence number.
///
/// There are two ways to filter outdated responses:
///     1) based on "seqNo" - sender knows the sequence number of transaction that he consider as a fresh enough.
///     2) based on "txnTime" - sender knows the timestamp that he consider as a fresh enough.
///
/// Note: response of GET_VALIDATOR_INFO request isn't supported
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// response: response of write or get request.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// response metadata.
/// {
///     "seqNo": Option<u64> - transaction sequence number,
///     "txnTime": Option<u64> - transaction ordering time,
///     "lastSeqNo": Option<u64> - the latest transaction seqNo for particular Node,
///     "lastTxnTime": Option<u64> - the latest transaction ordering time for particular Node
/// }
///
/// #Errors
/// Common*
/// Ledger*
#[no_mangle]
pub extern fn indy_get_response_metadata(command_handle: CommandHandle,
                                         response: *const c_char,
                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                              err: ErrorCode,
                                                              response_metadata: *const c_char)>) -> ErrorCode {
    trace!("indy_get_response_metadata: >>> response: {:?}", response);

    check_useful_c_str!(response, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_get_response_metadata: entities >>> response: {:?}", response);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::GetResponseMetadata(
            response,
            boxed_callback_string!("indy_get_response_metadata", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_get_response_metadata: <<< res: {:?}", res);

    res
}

/// Builds a AUTH_RULE request. Request to change authentication rules for a ledger transaction.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// txn_type: ledger transaction alias or associated value.
/// action: type of an action.
///     Can be either "ADD" (to add a new rule) or "EDIT" (to edit an existing one).
/// field: transaction field.
/// old_value: (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action).
/// new_value: (Optional) new value that can be used to fill the field.
/// constraint: set of constraints required for execution of an action in the following format:
///     {
///         constraint_id - <string> type of a constraint.
///             Can be either "ROLE" to specify final constraint or  "AND"/"OR" to combine constraints.
///         role - <string> (optional) role of a user which satisfy to constrain.
///         sig_count - <u32> the number of signatures required to execution action.
///         need_to_be_owner - <bool> (optional) if user must be an owner of transaction (false by default).
///         off_ledger_signature - <bool> (optional) allow signature of unknow for ledger did (false by default).
///         metadata - <object> (optional) additional parameters of the constraint.
///     }
/// can be combined by
///     {
///         'constraint_id': <"AND" or "OR">
///         'auth_constraints': [<constraint_1>, <constraint_2>]
///     }
///
/// Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md
///
/// More about AUTH_RULE request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rule
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_auth_rule_request(command_handle: CommandHandle,
                                           submitter_did: *const c_char,
                                           txn_type: *const c_char,
                                           action: *const c_char,
                                           field: *const c_char,
                                           old_value: *const c_char,
                                           new_value: *const c_char,
                                           constraint: *const c_char,
                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode,
                                                                request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_auth_rule_request: >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
    old_value: {:?}, new_value: {:?}, constraint: {:?}",
           submitter_did, txn_type, action, field, old_value, new_value, constraint);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(txn_type, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(action, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(field, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(old_value, ErrorCode::CommonInvalidParam6);
    check_useful_opt_c_str!(new_value, ErrorCode::CommonInvalidParam7);
    check_useful_json!(constraint, ErrorCode::CommonInvalidParam8, Constraint);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam9);

    trace!("indy_build_auth_rule_request: entities >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
    old_value: {:?}, new_value: {:?}, constraint: {:?}",
           submitter_did, txn_type, action, field, old_value, new_value, constraint);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildAuthRuleRequest(
            submitter_did,
            txn_type,
            action,
            field,
            old_value,
            new_value,
            constraint,
            boxed_callback_string!("indy_build_auth_rule_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_auth_rule_request: <<< res: {:?}", res);

    res
}

/// Builds a AUTH_RULES request. Request to change multiple authentication rules for a ledger transaction.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// rules: a list of auth rules: [
///     {
///         "auth_type": ledger transaction alias or associated value,
///         "auth_action": type of an action,
///         "field": transaction field,
///         "old_value": (Optional) old value of a field, which can be changed to a new_value (mandatory for EDIT action),
///         "new_value": (Optional) new value that can be used to fill the field,
///         "constraint": set of constraints required for execution of an action in the format described above for `indy_build_auth_rule_request` function.
///     },
///     ...
/// ]
///
/// Default ledger auth rules: https://github.com/hyperledger/indy-node/blob/master/docs/source/auth_rules.md
///
/// More about AUTH_RULES request: https://github.com/hyperledger/indy-node/blob/master/docs/source/requests.md#auth_rules
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_auth_rules_request(command_handle: CommandHandle,
                                            submitter_did: *const c_char,
                                            rules: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_auth_rules_request: >>> submitter_did: {:?}, rules: {:?}", submitter_did, rules);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_json!(rules, ErrorCode::CommonInvalidParam3, AuthRules);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    if rules.is_empty() {
        return err_msg(IndyErrorKind::InvalidStructure, "Empty list of Auth Rules has been passed").into();
    }

    trace!("indy_build_auth_rules_request: entities >>> submitter_did: {:?}, rules: {:?}", submitter_did, rules);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildAuthRulesRequest(
            submitter_did,
            rules,
            boxed_callback_string!("indy_build_auth_rules_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_auth_rules_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_AUTH_RULE request. Request to get authentication rules for ledger transactions.
///
/// NOTE: Either none or all transaction related parameters must be specified (`old_value` can be skipped for `ADD` action).
///     * none - to get all authentication rules for all ledger transactions
///     * all - to get authentication rules for specific action (`old_value` can be skipped for `ADD` action)
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// txn_type: (Optional) target ledger transaction alias or associated value.
/// action: (Optional) target action type. Can be either "ADD" or "EDIT".
/// field: (Optional) target transaction field.
/// old_value: (Optional) old value of field, which can be changed to a new_value (mandatory for EDIT action).
/// new_value: (Optional) new value that can be used to fill the field.
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_auth_rule_request(command_handle: CommandHandle,
                                               submitter_did: *const c_char,
                                               txn_type: *const c_char,
                                               action: *const c_char,
                                               field: *const c_char,
                                               old_value: *const c_char,
                                               new_value: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                                    err: ErrorCode,
                                                                    request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_auth_rule_request: >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
    old_value: {:?}, new_value: {:?}",
           submitter_did, txn_type, action, field, old_value, new_value);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_opt_c_str!(txn_type, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(action, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(field, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(old_value, ErrorCode::CommonInvalidParam6);
    check_useful_opt_c_str!(new_value, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_build_get_auth_rule_request: entities >>> submitter_did: {:?}, txn_type: {:?}, action: {:?}, field: {:?}, \
    old_value: {:?}, new_value: {:?}",
           submitter_did, txn_type, action, field, old_value, new_value);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(LedgerCommand::BuildGetAuthRuleRequest(
            submitter_did,
            txn_type,
            action,
            field,
            old_value,
            new_value,
            boxed_callback_string!("indy_build_get_auth_rule_request", cb, command_handle)
        )));

    let res = prepare_result!(result);

    trace!("indy_build_get_auth_rule_request: <<< res: {:?}", res);

    res
}

/// Builds a TXN_AUTHR_AGRMT request. Request to add a new version of Transaction Author Agreement to the ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// text: (Optional) a content of the TTA.
///             Mandatory in case of adding a new TAA. An existing TAA text can not be changed.
///             for Indy Node version <= 1.12.0:
///                 Use empty string to reset TAA on the ledger
///             for Indy Node version > 1.12.0
///                 Should be omitted in case of updating an existing TAA (setting `retirement_ts`)
/// version: a version of the TTA (unique UTF-8 string).
/// ratification_ts: (Optional) the date (timestamp) of TAA ratification by network government. (-1 to omit)
///              for Indy Node version <= 1.12.0:
///                 Must be omitted
///              for Indy Node version > 1.12.0:
///                 Must be specified in case of adding a new TAA
///                 Can be omitted in case of updating an existing TAA
/// retirement_ts: (Optional) the date (timestamp) of TAA retirement. (-1 to omit)
///              for Indy Node version <= 1.12.0:
///                 Must be omitted
///              for Indy Node version > 1.12.0:
///                 Must be omitted in case of adding a new (latest) TAA.
///                 Should be used for updating (deactivating) non-latest TAA on the ledger.
///
/// Note: Use `indy_build_disable_all_txn_author_agreements_request` to disable all TAA's on the ledger.
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_txn_author_agreement_request(command_handle: CommandHandle,
                                                      submitter_did: *const c_char,
                                                      text: *const c_char,
                                                      version: *const c_char,
                                                      ratification_ts: i64,
                                                      retirement_ts: i64,
                                                      cb: Option<extern fn(command_handle_: CommandHandle,
                                                                           err: ErrorCode,
                                                                           request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_txn_author_agreement_request: >>> submitter_did: {:?}, text: {:?}, version: {:?}, ratification_ts {:?}, retirement_ts {:?}",
           submitter_did, text, version, ratification_ts, retirement_ts);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_opt_c_str!(text, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_opt_u64!(ratification_ts, ErrorCode::CommonInvalidParam5);
    check_useful_opt_u64!(retirement_ts, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_txn_author_agreement_request: entities >>> submitter_did: {:?}, text: {:?}, version: {:?}, ratification_ts {:?}, retirement_ts {:?}",
           submitter_did, text, version, ratification_ts, retirement_ts);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildTxnAuthorAgreementRequest(
                submitter_did,
                text,
                version,
                ratification_ts,
                retirement_ts,
                boxed_callback_string!("indy_build_txn_author_agreement_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_txn_author_agreement_request: <<< res: {:?}", res);

    res
}


/// Builds a DISABLE_ALL_TXN_AUTHR_AGRMTS request. Request to disable all Transaction Author Agreement on the ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_disable_all_txn_author_agreements_request(command_handle: CommandHandle,
                                                                   submitter_did: *const c_char,
                                                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                        err: ErrorCode,
                                                                                        request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_disable_all_txn_author_agreements_request: >>> submitter_did: {:?}", submitter_did);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_build_disable_all_txn_author_agreements_request: entities >>> submitter_did: {:?}", submitter_did);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildDisableAllTxnAuthorAgreementsRequest(
                submitter_did,
                boxed_callback_string!("indy_build_disable_all_txn_author_agreements_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_disable_all_txn_author_agreements_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_TXN_AUTHR_AGRMT request. Request to get a specific Transaction Author Agreement from the ledger.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// data: (Optional) specifies a condition for getting specific TAA.
/// Contains 3 mutually exclusive optional fields:
/// {
///     hash: Optional<str> - hash of requested TAA,
///     version: Optional<str> - version of requested TAA.
///     timestamp: Optional<u64> - ledger will return TAA valid at requested timestamp.
/// }
/// Null data or empty JSON are acceptable here. In this case, ledger will return the latest version of TAA.
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_txn_author_agreement_request(command_handle: CommandHandle,
                                                          submitter_did: *const c_char,
                                                          data: *const c_char,
                                                          cb: Option<extern fn(command_handle_: CommandHandle,
                                                                               err: ErrorCode,
                                                                               request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_txn_author_agreement_request: >>> submitter_did: {:?}, data: {:?}?", submitter_did, data);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_opt_validatable_json!(data, ErrorCode::CommonInvalidParam3, GetTxnAuthorAgreementData);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_build_get_txn_author_agreement_request: entities >>> submitter_did: {:?}, data: {:?}", submitter_did, data);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildGetTxnAuthorAgreementRequest(
                submitter_did,
                data,
                boxed_callback_string!("indy_build_get_txn_author_agreement_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_get_txn_author_agreement_request: <<< res: {:?}", res);

    res
}

/// Builds a SET_TXN_AUTHR_AGRMT_AML request. Request to add a new list of acceptance mechanisms for transaction author agreement.
/// Acceptance Mechanism is a description of the ways how the user may accept a transaction author agreement.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: Identifier (DID) of the transaction author as base58-encoded string.
///                Actual request sender may differ if Endorser is used (look at `indy_append_request_endorser`)
/// aml: a set of new acceptance mechanisms:
/// {
///     “<acceptance mechanism label 1>”: { acceptance mechanism description 1},
///     “<acceptance mechanism label 2>”: { acceptance mechanism description 2},
///     ...
/// }
/// version: a version of new acceptance mechanisms. (Note: unique on the Ledger)
/// aml_context: (Optional) common context information about acceptance mechanisms (may be a URL to external resource).
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_acceptance_mechanisms_request(command_handle: CommandHandle,
                                                       submitter_did: *const c_char,
                                                       aml: *const c_char,
                                                       version: *const c_char,
                                                       aml_context: *const c_char,
                                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                                            err: ErrorCode,
                                                                            request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_acceptance_mechanisms_request: >>> submitter_did: {:?}, aml: {:?}, version: {:?}, aml_context: {:?}",
           submitter_did, aml, version, aml_context);

    check_useful_validatable_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_validatable_json!(aml, ErrorCode::CommonInvalidParam3, AcceptanceMechanisms);
    check_useful_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(aml_context, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_build_acceptance_mechanisms_request: entities >>> submitter_did: {:?}, aml: {:?}, version: {:?}, aml_context: {:?}",
           submitter_did, aml, version, aml_context);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildAcceptanceMechanismRequests(
                submitter_did,
                aml,
                version,
                aml_context,
                boxed_callback_string!("indy_build_acceptance_mechanisms_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_acceptance_mechanisms_request: <<< res: {:?}", res);

    res
}

/// Builds a GET_TXN_AUTHR_AGRMT_AML request. Request to get a list of  acceptance mechanisms from the ledger
/// valid for specified time or the latest one.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// submitter_did: (Optional) DID of the read request sender (if not provided then default Libindy DID will be used).
/// timestamp: i64 - time to get an active acceptance mechanisms. Pass -1 to get the latest one.
/// version: (Optional) version of acceptance mechanisms.
/// cb: Callback that takes command result as parameter.
///
/// NOTE: timestamp and version cannot be specified together.
///
/// #Returns
/// Request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_build_get_acceptance_mechanisms_request(command_handle: CommandHandle,
                                                           submitter_did: *const c_char,
                                                           timestamp: i64,
                                                           version: *const c_char,
                                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                err: ErrorCode,
                                                                                request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_build_get_acceptance_mechanisms_request: >>> submitter_did: {:?}, timestamp: {:?}, version: {:?}", submitter_did, timestamp, version);

    check_useful_validatable_opt_string!(submitter_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_opt_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    let timestamp = if timestamp != -1 { Some(timestamp as u64) } else { None };

    trace!("indy_build_get_acceptance_mechanisms_request: entities >>> submitter_did: {:?}, timestamp: {:?}, version: {:?}", submitter_did, timestamp, version);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::BuildGetAcceptanceMechanismsRequest(
                submitter_did,
                timestamp,
                version,
                boxed_callback_string!("indy_build_get_acceptance_mechanisms_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_build_get_acceptance_mechanisms_request: <<< res: {:?}", res);

    res
}

/// Append transaction author agreement acceptance data to a request.
/// This function should be called before signing and sending a request
/// if there is any transaction author agreement set on the Ledger.
///
/// EXPERIMENTAL
///
/// This function may calculate digest by itself or consume it as a parameter.
/// If all text, version and taa_digest parameters are specified, a check integrity of them will be done.
///
/// #Params
/// command_handle: command handle to map callback to caller context.
/// request_json: original request data json.
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if taa_digest parameter is omitted.
/// taa_digest - (optional) digest on text and version.
///     Digest is sha256 hash calculated on concatenated strings: version || text.
///     This parameter is required if text and version parameters are omitted.
/// mechanism - mechanism how user has accepted the TAA
/// time - UTC timestamp when user has accepted the TAA. Note that the time portion will be discarded to avoid a privacy risk.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// Updated request result as json.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_append_txn_author_agreement_acceptance_to_request(command_handle: CommandHandle,
                                                                     request_json: *const c_char,
                                                                     text: *const c_char,
                                                                     version: *const c_char,
                                                                     taa_digest: *const c_char,
                                                                     mechanism: *const c_char,
                                                                     time: u64,
                                                                     cb: Option<extern fn(command_handle_: CommandHandle,
                                                                                          err: ErrorCode,
                                                                                          request_with_meta_json: *const c_char)>) -> ErrorCode {
    trace!("indy_append_txn_author_agreement_acceptance_to_request: >>> request_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, \
        mechanism: {:?}, time: {:?}",
           request_json, text, version, taa_digest, mechanism, time);

    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam2);
    check_useful_opt_c_str!(text, ErrorCode::CommonInvalidParam3);
    check_useful_opt_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_opt_c_str!(taa_digest, ErrorCode::CommonInvalidParam5);
    check_useful_c_str!(mechanism, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_append_txn_author_agreement_acceptance_to_request: entities >>> request_json: {:?}, text: {:?}, version: {:?}, taa_digest: {:?}, \
        mechanism: {:?}, time: {:?}",
           request_json, text, version, taa_digest, mechanism, time);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::AppendTxnAuthorAgreementAcceptanceToRequest(
                request_json,
                text,
                version,
                taa_digest,
                mechanism,
                time,
                boxed_callback_string!("indy_append_txn_author_agreement_acceptance_to_request", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_append_txn_author_agreement_acceptance_to_request: <<< res: {:?}", res);

    res
}

/// Append Endorser to an existing request.
///
/// An author of request still is a `DID` used as a `submitter_did` parameter for the building of the request.
/// But it is expecting that the transaction will be sent by the specified Endorser.
///
/// Note: Both Transaction Author and Endorser must sign output request after that.
///
/// More about Transaction Endorser: https://github.com/hyperledger/indy-node/blob/master/design/transaction_endorser.md
///                                  https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
///
/// #Params
/// request_json: original request
/// endorser_did: DID of the Endorser that will submit the transaction.
///               The Endorser's DID must be present on the ledger.
/// cb: Callback that takes command result as parameter.
///     The command result is a request JSON with Endorser field appended.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_append_request_endorser(command_handle: CommandHandle,
                                           request_json: *const c_char,
                                           endorser_did: *const c_char,
                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode,
                                                                out_request_json: *const c_char)>) -> ErrorCode {
    trace!("indy_append_request_endorser: >>> request_json: {:?}, endorser_did: {:?}",
           request_json, endorser_did);

    check_useful_c_str!(request_json, ErrorCode::CommonInvalidParam2);
    check_useful_validatable_string!(endorser_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_append_request_endorser: entities >>> request_json: {:?},endorser_did: {:?}", request_json, endorser_did);

    let result = CommandExecutor::instance()
        .send(Command::Ledger(
            LedgerCommand::AppendRequestEndorser(
                request_json,
                endorser_did,
                boxed_callback_string!("indy_append_request_endorser", cb, command_handle)
            )));

    let res = prepare_result!(result);

    trace!("indy_append_request_endorser: <<< res: {:?}", res);

    res
}