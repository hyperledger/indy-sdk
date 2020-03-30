use indy_api_types::{ErrorCode, IndyHandle, CommandHandle, WalletHandle, SearchHandle, INVALID_SEARCH_HANDLE};
use indy_api_types::errors::prelude::*;
use crate::commands::{Command, CommandExecutor};
use crate::commands::anoncreds::AnoncredsCommand;
use crate::commands::anoncreds::issuer::IssuerCommand;
use crate::commands::anoncreds::prover::ProverCommand;
use crate::commands::anoncreds::verifier::VerifierCommand;
use crate::domain::anoncreds::schema::{Schema, AttributeNames, Schemas};
use crate::domain::crypto::did::DidValue;
use crate::domain::anoncreds::credential_definition::{CredentialDefinition, CredentialDefinitionConfig, CredentialDefinitionId, CredentialDefinitions};
use crate::domain::anoncreds::credential_offer::CredentialOffer;
use crate::domain::anoncreds::credential_request::{CredentialRequest, CredentialRequestMetadata};
use crate::domain::anoncreds::credential_attr_tag_policy::CredentialAttrTagPolicy;
use crate::domain::anoncreds::credential::{Credential, CredentialValues};
use crate::domain::anoncreds::revocation_registry_definition::{RevocationRegistryConfig, RevocationRegistryDefinition, RevocationRegistryId, RevocationRegistryDefinitions};
use crate::domain::anoncreds::revocation_registry_delta::RevocationRegistryDelta;
use crate::domain::anoncreds::proof::Proof;
use crate::domain::anoncreds::proof_request::{ProofRequest, ProofRequestExtraQuery};
use crate::domain::anoncreds::requested_credential::RequestedCredentials;
use crate::domain::anoncreds::revocation_registry::RevocationRegistries;
use crate::domain::anoncreds::revocation_state::{RevocationState, RevocationStates};
use indy_utils::ctypes;

use libc::c_char;
use std::ptr;

use crate::indy_api_types::validation::Validatable;
use indy_vdr::utils::validation::Validatable as VdrValidatable;

/*
These functions wrap the Ursa algorithm as documented in this paper:
https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf

And is documented in this HIPE:
https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
*/


/// Create credential schema entity that describes credential attributes list and allows credentials
/// interoperability.
///
/// Schema is public and intended to be shared with all anoncreds workflow actors usually by publishing SCHEMA transaction
/// to Indy distributed ledger.
///
/// It is IMPORTANT for current version POST Schema in Ledger and after that GET it from Ledger
/// with correct seq_no to save compatibility with Ledger.
/// After that can call indy_issuer_create_and_store_credential_def to build corresponding Credential Definition.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// issuer_did: DID of schema issuer
/// name: a name the schema
/// version: a version of the schema
/// attrs: a list of schema attributes descriptions (the number of attributes should be less or equal than 125)
///     `["attr1", "attr2"]`
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// schema_id: identifier of created schema
/// schema_json: schema as json:
/// {
///     id: identifier of schema
///     attrNames: array of attribute name strings
///     name: schema's name string
///     version: schema's version string,
///     ver: version of the Schema json
/// }
///
/// #Errors
/// Common*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_create_schema(command_handle: CommandHandle,
                                        issuer_did: *const c_char,
                                        name: *const c_char,
                                        version: *const c_char,
                                        attrs: *const c_char,
                                        cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                             schema_id: *const c_char, schema_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_create_schema: >>> issuer_did: {:?}, name: {:?}, version: {:?}, attrs: {:?}", issuer_did, name, version, attrs);

    check_useful_validatable_string!(issuer_did, ErrorCode::CommonInvalidParam2, DidValue);
    check_useful_c_str!(name, ErrorCode::CommonInvalidParam3);
    check_useful_c_str!(version, ErrorCode::CommonInvalidParam4);
    check_useful_validatable_json!(attrs, ErrorCode::CommonInvalidParam5, AttributeNames);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_issuer_create_schema: entity >>> issuer_did: {:?}, name: {:?}, version: {:?}, attrs: {:?}", issuer_did, name, version, attrs);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::CreateSchema(
                    issuer_did,
                    name,
                    version,
                    attrs,
                    Box::new(move |result| {
                        let (err, id, schema_json) = prepare_result_2!(result, String::new(), String::new());
                        trace!("ursa_cl_credential_public_key_to_json: id: {:?}, schema_json: {:?}", id, schema_json);
                        let id = ctypes::string_to_cstring(id);
                        let schema_json = ctypes::string_to_cstring(schema_json);
                        cb(command_handle, err, id.as_ptr(), schema_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_create_schema: <<< res: {:?}", res);

    res
}

/// Create credential definition entity that encapsulates credentials issuer DID, credential schema, secrets used for signing credentials
/// and secrets used for credentials revocation.
///
/// Credential definition entity contains private and public parts. Private part will be stored in the wallet. Public part
/// will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing CRED_DEF transaction
/// to Indy distributed ledger.
///
/// It is IMPORTANT for current version GET Schema from Ledger with correct seq_no to save compatibility with Ledger.
///
/// Note: Use combination of `indy_issuer_rotate_credential_def_start` and `indy_issuer_rotate_credential_def_apply` functions
/// to generate new keys for an existing credential definition.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// issuer_did: a DID of the issuer
/// schema_json: credential schema as a json: {
///     id: identifier of schema
///     attrNames: array of attribute name strings
///     name: schema's name string
///     version: schema's version string,
///     seqNo: (Optional) schema's sequence number on the ledger,
///     ver: version of the Schema json
/// }
/// tag: any string that allows to distinguish between credential definitions for the same issuer and schema
/// signature_type: credential definition type (optional, 'CL' by default) that defines credentials signature and revocation math.
/// Supported signature types:
/// - 'CL': Camenisch-Lysyanskaya credential signature type that is implemented according to the algorithm in this paper:
///             https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
///         And is documented in this HIPE:
///             https://github.com/hyperledger/indy-hipe/blob/c761c583b1e01c1e9d3ceda2b03b35336fdc8cc1/text/anoncreds-protocol/README.md
/// config_json: (optional) type-specific configuration of credential definition as json:
/// - 'CL':
///     {
///         "support_revocation" - bool (optional, default false) whether to request non-revocation credential
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// cred_def_id: identifier of created credential definition
/// cred_def_json: public part of created credential definition
/// {
///     id: string - identifier of credential definition
///     schemaId: string - identifier of stored in ledger schema
///     type: string - type of the credential definition. CL is the only supported type now.
///     tag: string - allows to distinct between credential definitions for the same issuer and schema
///     value: Dictionary with Credential Definition's data is depended on the signature type: {
///         primary: primary credential public key,
///         Optional<revocation>: revocation credential public key
///     },
///     ver: Version of the CredDef json
/// }
///
/// Note: `primary` and `revocation` fields of credential definition are complex opaque types that contain data structures internal to Ursa.
/// They should not be parsed and are likely to change in future versions.
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_create_and_store_credential_def(command_handle: CommandHandle,
                                                          wallet_handle: WalletHandle,
                                                          issuer_did: *const c_char,
                                                          schema_json: *const c_char,
                                                          tag: *const c_char,
                                                          signature_type: *const c_char,
                                                          config_json: *const c_char,
                                                          cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                               cred_def_id: *const c_char,
                                                                               cred_def_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_create_and_store_credential_def: >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, tag: {:?}, \
    signature_type: {:?}, config_json: {:?}", wallet_handle, issuer_did, schema_json, tag, signature_type, config_json);

    check_useful_validatable_string!(issuer_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_json!(schema_json, ErrorCode::CommonInvalidParam4, Schema);
    check_useful_c_str!(tag, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_str!(signature_type, ErrorCode::CommonInvalidParam6);
    check_useful_opt_validatable_json!(config_json, ErrorCode::CommonInvalidParam7, CredentialDefinitionConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_issuer_create_and_store_credential_def: entities >>> wallet_handle: {:?}, issuer_did: {:?}, schema_json: {:?}, tag: {:?}, \
    signature_type: {:?}, config_json: {:?}", wallet_handle, issuer_did, schema_json, tag, signature_type, config_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::CreateAndStoreCredentialDefinition(
                    wallet_handle,
                    issuer_did,
                    schema_json,
                    tag,
                    signature_type,
                    config_json,
                    Box::new(move |result| {
                        let (err, cred_def_id, cred_def_json) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_issuer_create_and_store_credential_def: cred_def_id: {:?}, cred_def_json: {:?}", cred_def_id, cred_def_json);
                        let cred_def_id = ctypes::string_to_cstring(cred_def_id);
                        let cred_def_json = ctypes::string_to_cstring(cred_def_json);
                        cb(command_handle, err, cred_def_id.as_ptr(), cred_def_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_create_and_store_credential_def: <<< res: {:?}", res);

    res
}

/// Generate temporary credential definitional keys for an existing one (owned by the caller of the library).
///
/// Use `indy_issuer_rotate_credential_def_apply` function to set generated temporary keys as the main.
///
/// WARNING: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: an identifier of created credential definition stored in the wallet
/// config_json: (optional) type-specific configuration of credential definition as json:
/// - 'CL':
///     {
///         "support_revocation" - bool (optional, default false) whether to request non-revocation credential
///     }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// cred_def_json: public part of temporary created credential definition
/// {
///     id: string - identifier of credential definition
///     schemaId: string - identifier of stored in ledger schema
///     type: string - type of the credential definition. CL is the only supported type now.
///     tag: string - allows to distinct between credential definitions for the same issuer and schema
///     value: Dictionary with Credential Definition's data is depended on the signature type: {
///         primary: primary credential public key,
///         Optional<revocation>: revocation credential public key
///     }, - only this field differs from the original credential definition
///     ver: Version of the CredDef json
/// }
///
/// Note: `primary` and `revocation` fields of credential definition are complex opaque types that contain data structures internal to Ursa.
/// They should not be parsed and are likely to change in future versions.
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_rotate_credential_def_start(command_handle: CommandHandle,
                                                      wallet_handle: WalletHandle,
                                                      cred_def_id: *const c_char,
                                                      config_json: *const c_char,
                                                      cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                           cred_def_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_rotate_credential_def_start: >>> wallet_handle: {:?}, cred_def_id: {:?}, config_json: {:?}",
           wallet_handle, cred_def_id, config_json);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_opt_validatable_json!(config_json, ErrorCode::CommonInvalidParam4, CredentialDefinitionConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_issuer_rotate_credential_def_start: entities >>> wallet_handle: {:?}, cred_def_id: {:?}, config_json: {:?}",
           wallet_handle, cred_def_id, config_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::RotateCredentialDefinitionStart(
                    wallet_handle,
                    cred_def_id,
                    config_json,
                    Box::new(move |result| {
                        let (err, cred_def_json) = prepare_result_1!(result, String::new());
                        trace!("indy_issuer_rotate_credential_def_start:cred_def_json: {:?}", cred_def_json);
                        let cred_def_json = ctypes::string_to_cstring(cred_def_json);
                        cb(command_handle, err, cred_def_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_rotate_credential_def_start: <<< res: {:?}", res);

    res
}

///  Apply temporary keys as main for an existing Credential Definition (owned by the caller of the library).
///
/// WARNING: Rotating the credential definitional keys will result in making all credentials issued under the previous keys unverifiable.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: an identifier of created credential definition stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_rotate_credential_def_apply(command_handle: CommandHandle,
                                                      wallet_handle: WalletHandle,
                                                      cred_def_id: *const c_char,
                                                      cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_issuer_rotate_credential_def_apply: >>> wallet_handle: {:?}, cred_def_id: {:?}",
           wallet_handle, cred_def_id);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_issuer_rotate_credential_def_apply: entities >>> wallet_handle: {:?}, cred_def_id: {:?}",
           wallet_handle, cred_def_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::RotateCredentialDefinitionApply(
                    wallet_handle,
                    cred_def_id,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_issuer_rotate_credential_def_apply:");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_rotate_credential_def_apply: <<< res: {:?}", res);

    res
}

/// Create a new revocation registry for the given credential definition as tuple of entities
/// - Revocation registry definition that encapsulates credentials definition reference, revocation type specific configuration and
///   secrets used for credentials revocation
/// - Revocation registry state that stores the information about revoked entities in a non-disclosing way. The state can be
///   represented as ordered list of revocation registry entries were each entry represents the list of revocation or issuance operations.
///
/// Revocation registry definition entity contains private and public parts. Private part will be stored in the wallet. Public part
/// will be returned as json intended to be shared with all anoncreds workflow actors usually by publishing REVOC_REG_DEF transaction
/// to Indy distributed ledger.
///
/// Revocation registry state is stored on the wallet and also intended to be shared as the ordered list of REVOC_REG_ENTRY transactions.
/// This call initializes the state in the wallet and returns the initial entry.
///
/// Some revocation registry types (for example, 'CL_ACCUM') can require generation of binary blob called tails used to hide information about revoked credentials in public
/// revocation registry and intended to be distributed out of leger (REVOC_REG_DEF transaction will still contain uri and hash of tails).
/// This call requires access to pre-configured blob storage writer instance handle that will allow to write generated tails.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// issuer_did: a DID of the issuer
/// revoc_def_type: revocation registry type (optional, default value depends on credential definition type). Supported types are:
/// - 'CL_ACCUM': Type-3 pairing based accumulator implemented according to the algorithm in this paper:
///                   https://github.com/hyperledger/ursa/blob/master/libursa/docs/AnonCred.pdf
///               This type is default for 'CL' credential definition type.
/// tag: any string that allows to distinct between revocation registries for the same issuer and credential definition
/// cred_def_id: id of stored in ledger credential definition
/// config_json: type-specific configuration of revocation registry as json:
/// - 'CL_ACCUM': {
///     "issuance_type": (optional) type of issuance. Currently supported:
///         1) ISSUANCE_BY_DEFAULT: all indices are assumed to be issued and initial accumulator is calculated over all indices;
///            Revocation Registry is updated only during revocation.
///         2) ISSUANCE_ON_DEMAND: nothing is issued initially accumulator is 1 (used by default);
///     "max_cred_num": maximum number of credentials the new registry can process (optional, default 100000)
/// }
/// tails_writer_handle: handle of blob storage to store tails (returned by `indy_open_blob_storage_writer`).
/// cb: Callback that takes command result as parameter.
///
/// NOTE:
///     Recursive creation of folder for Default Tails Writer (correspondent to `tails_writer_handle`)
///     in the system-wide temporary directory may fail in some setup due to permissions: `IO error: Permission denied`.
///     In this case use `TMPDIR` environment variable to define temporary directory specific for an application.
///
/// #Returns
/// revoc_reg_id: identifier of created revocation registry definition
/// revoc_reg_def_json: public part of revocation registry definition
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
///             "publicKeys": <public_keys> - Registry's public key (opaque type that contains data structures internal to Ursa.
///                                                                  It should not be parsed and are likely to change in future versions).
///         },
///         "ver": string - version of revocation registry definition json.
///     }
/// revoc_reg_entry_json: revocation registry entry that defines initial state of revocation registry
/// {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> - an array of issued indices.
///         revoked: array<number> an array of revoked indices.
///     },
///     ver: string - version revocation registry entry json
/// }
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_create_and_store_revoc_reg(command_handle: CommandHandle,
                                                     wallet_handle: WalletHandle,
                                                     issuer_did: *const c_char,
                                                     revoc_def_type: *const c_char,
                                                     tag: *const c_char,
                                                     cred_def_id: *const c_char,
                                                     config_json: *const c_char,
                                                     tails_writer_handle: IndyHandle,
                                                     cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                          revoc_reg_id: *const c_char,
                                                                          revoc_reg_def_json: *const c_char,
                                                                          revoc_reg_entry_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_create_and_store_credential_def: >>> wallet_handle: {:?}, issuer_did: {:?}, revoc_def_type: {:?}, tag: {:?}, \
    cred_def_id: {:?}, config_json: {:?}, tails_writer_handle: {:?}", wallet_handle, issuer_did, revoc_def_type, tag, cred_def_id, config_json, tails_writer_handle);

    check_useful_validatable_string!(issuer_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_opt_c_str!(revoc_def_type, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(tag, ErrorCode::CommonInvalidParam5);
    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam6, CredentialDefinitionId);
    check_useful_validatable_json!(config_json, ErrorCode::CommonInvalidParam7, RevocationRegistryConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam9);

    trace!("indy_issuer_create_and_store_credential_def: entities >>> wallet_handle: {:?}, issuer_did: {:?}, revoc_def_type: {:?}, tag: {:?}, \
    cred_def_id: {:?}, config_json: {:?}, tails_writer_handle: {:?}", wallet_handle, issuer_did, revoc_def_type, tag, cred_def_id, config_json, tails_writer_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::CreateAndStoreRevocationRegistry(
                    wallet_handle,
                    issuer_did,
                    revoc_def_type,
                    tag,
                    cred_def_id,
                    config_json,
                    tails_writer_handle,
                    Box::new(move |result| {
                        let (err, revoc_reg_id, revoc_reg_def_json, revoc_reg_json) = prepare_result_3!(result, String::new(), String::new(), String::new());
                        trace!("indy_issuer_create_and_store_credential_def: revoc_reg_id: {:?}, revoc_reg_def_json: {:?}, revoc_reg_json: {:?}",
                               revoc_reg_id, revoc_reg_def_json, revoc_reg_json);
                        let revoc_reg_id = ctypes::string_to_cstring(revoc_reg_id);
                        let revoc_reg_def_json = ctypes::string_to_cstring(revoc_reg_def_json);
                        let revoc_reg_json = ctypes::string_to_cstring(revoc_reg_json);
                        cb(command_handle, err, revoc_reg_id.as_ptr(), revoc_reg_def_json.as_ptr(), revoc_reg_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_create_and_store_credential_def: <<< res: {:?}", res);

    res
}

/// Create credential offer that will be used by Prover for
/// credential request creation. Offer includes nonce and key correctness proof
/// for authentication between protocol steps and integrity checking.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// wallet_handle: wallet handle (created by open_wallet)
/// cred_def_id: id of credential definition stored in the wallet
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// credential offer json:
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         // Fields below can depend on Credential Definition type
///         "nonce": string,
///         "key_correctness_proof" : key correctness proof for credential definition correspondent to cred_def_id
///                                   (opaque type that contains data structures internal to Ursa.
///                                   It should not be parsed and are likely to change in future versions).
///     }
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_issuer_create_credential_offer(command_handle: CommandHandle,
                                                  wallet_handle: WalletHandle,
                                                  cred_def_id: *const c_char,
                                                  cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                       cred_offer_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_create_credential_offer: >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_issuer_create_credential_offer: entities >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::CreateCredentialOffer(
                    wallet_handle,
                    cred_def_id,
                    boxed_callback_string!("indy_issuer_create_credential_offer", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_create_credential_offer: <<< res: {:?}", res);

    res
}

/// Check Cred Request for the given Cred Offer and issue Credential for the given Cred Request.
///
/// Cred Request must match Cred Offer. The credential definition and revocation registry definition
/// referenced in Cred Offer and Cred Request must be already created and stored into the wallet.
///
/// Information for this credential revocation will be store in the wallet as part of revocation registry under
/// generated cred_revoc_id local for this wallet.
///
/// This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
/// Note that it is possible to accumulate deltas to reduce ledger load.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_offer_json: a cred offer created by indy_issuer_create_credential_offer
/// cred_req_json: a credential request created by indy_prover_create_credential_req
/// cred_values_json: a credential containing attribute values for each of requested attribute names.
///     Example:
///     {
///      "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
///      "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
///     }
///   If you want to use empty value for some credential field, you should set "raw" to "" and "encoded" should not be empty
/// rev_reg_id: id of revocation registry stored in the wallet
/// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`)
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// cred_json: Credential json containing signed credential values
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_def_id", Optional<string>, - identifier of revocation registry
///         "values": <see cred_values_json above>, - credential values.
///         // Fields below can depend on Cred Def type
///         "signature": <credential signature>,
///                      (opaque type that contains data structures internal to Ursa.
///                       It should not be parsed and are likely to change in future versions).
///         "signature_correctness_proof": credential signature correctness proof
///                      (opaque type that contains data structures internal to Ursa.
///                       It should not be parsed and are likely to change in future versions).
///         "rev_reg" - (Optional) revocation registry accumulator value on the issuing moment.
///                      (opaque type that contains data structures internal to Ursa.
///                       It should not be parsed and are likely to change in future versions).
///         "witness" - (Optional) revocation related data
///                      (opaque type that contains data structures internal to Ursa.
///                       It should not be parsed and are likely to change in future versions).
///     }
/// cred_revoc_id: local id for revocation info (Can be used for revocation of this credential)
/// revoc_reg_delta_json: Revocation registry delta json with a newly issued credential
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_issuer_create_credential(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            cred_offer_json: *const c_char,
                                            cred_req_json: *const c_char,
                                            cred_values_json: *const c_char,
                                            rev_reg_id: *const c_char,
                                            blob_storage_reader_handle: IndyHandle,
                                            cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                 cred_json: *const c_char,
                                                                 cred_revoc_id: *const c_char,
                                                                 revoc_reg_delta_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_create_credential: >>> wallet_handle: {:?}, cred_offer_json: {:?}, cred_req_json: {:?}, cred_values_json: {:?}, rev_reg_id: {:?}, \
    blob_storage_reader_handle: {:?}", wallet_handle, cred_offer_json, cred_req_json, cred_values_json, rev_reg_id, blob_storage_reader_handle);

    check_useful_validatable_json!(cred_offer_json, ErrorCode::CommonInvalidParam3, CredentialOffer);
    check_useful_validatable_json!(cred_req_json, ErrorCode::CommonInvalidParam4, CredentialRequest);
    check_useful_validatable_json!(cred_values_json, ErrorCode::CommonInvalidParam5, CredentialValues);
    check_useful_validatable_opt_string!(rev_reg_id, ErrorCode::CommonInvalidParam6, RevocationRegistryId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    let blob_storage_reader_handle = if blob_storage_reader_handle != -1 { Some(blob_storage_reader_handle) } else { None };

    trace!("indy_issuer_create_credential: entities >>> wallet_handle: {:?}, cred_offer_json: {:?}, cred_req_json: {:?}, cred_values_json: {:?}, rev_reg_id: {:?}, \
    blob_storage_reader_handle: {:?}", wallet_handle, cred_offer_json, secret!(&cred_req_json), secret!(&cred_values_json), secret!(&rev_reg_id), blob_storage_reader_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::CreateCredential(
                    wallet_handle,
                    cred_offer_json,
                    cred_req_json,
                    cred_values_json,
                    rev_reg_id,
                    blob_storage_reader_handle,
                    Box::new(move |result| {
                        let (err, cred_json, revoc_id, revoc_reg_delta_json) = prepare_result_3!(result, String::new(), None, None);
                        trace!("indy_issuer_create_credential: cred_json: {:?}, revoc_id: {:?}, revoc_reg_delta_json: {:?}",
                               secret!(cred_json.as_str()), secret!(&revoc_id), revoc_reg_delta_json);
                        let cred_json = ctypes::string_to_cstring(cred_json);
                        let revoc_id = revoc_id.map(ctypes::string_to_cstring);
                        let revoc_reg_delta_json = revoc_reg_delta_json.map(ctypes::string_to_cstring);
                        cb(command_handle, err, cred_json.as_ptr(),
                           revoc_id.as_ref().map(|id| id.as_ptr()).unwrap_or(ptr::null()),
                           revoc_reg_delta_json.as_ref().map(|delta| delta.as_ptr()).unwrap_or(ptr::null()))
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_create_credential: <<< res: {:?}", res);

    res
}

/// Revoke a credential identified by a cred_revoc_id (returned by indy_issuer_create_credential).
///
/// The corresponding credential definition and revocation registry must be already
/// created an stored into the wallet.
///
/// This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
/// Note that it is possible to accumulate deltas to reduce ledger load.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// blob_storage_reader_cfg_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`).
/// rev_reg_id: id of revocation registry stored in wallet
/// cred_revoc_id: local id for revocation info related to issued credential
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// revoc_reg_delta_json: Revocation registry delta json with a revoked credential
/// {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         revoked: array<number> an array of revoked indices.
///     },
///     ver: string - version revocation registry delta json
/// }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_issuer_revoke_credential(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            blob_storage_reader_cfg_handle: IndyHandle,
                                            rev_reg_id: *const c_char,
                                            cred_revoc_id: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                 revoc_reg_delta_json: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_revoke_credential: >>> wallet_handle: {:?}, blob_storage_reader_cfg_handle: {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
           wallet_handle, blob_storage_reader_cfg_handle, rev_reg_id, cred_revoc_id);

    check_useful_validatable_string!(rev_reg_id, ErrorCode::CommonInvalidParam4, RevocationRegistryId);
    check_useful_c_str!(cred_revoc_id, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_issuer_revoke_credential: entities >>> wallet_handle: {:?}, blob_storage_reader_cfg_handle: {:?}, rev_reg_id: {:?}, cred_revoc_id: {:?}",
           wallet_handle, blob_storage_reader_cfg_handle, rev_reg_id, secret!(cred_revoc_id.as_str()));

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::RevokeCredential(
                    wallet_handle,
                    blob_storage_reader_cfg_handle,
                    rev_reg_id,
                    cred_revoc_id,
                    boxed_callback_string!("indy_issuer_revoke_credential", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_revoke_credential: <<< res: {:?}", res);

    res
}

/*/// Recover a credential identified by a cred_revoc_id (returned by indy_issuer_create_credential).
///
/// The corresponding credential definition and revocation registry must be already
/// created an stored into the wallet.
///
/// This call returns revoc registry delta as json file intended to be shared as REVOC_REG_ENTRY transaction.
/// Note that it is possible to accumulate deltas to reduce ledger load.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// blob_storage_reader_cfg_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`).
/// rev_reg_id: id of revocation registry stored in wallet
/// cred_revoc_id: local id for revocation info related to issued credential
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// revoc_reg_delta_json: Revocation registry delta json with a recovered credential
/// {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> an array of issued indices.
///     },
///     ver: string - version revocation registry delta json
/// }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_issuer_recover_credential(command_handle: CommandHandle,
                                             wallet_handle: WalletHandle,
                                             blob_storage_reader_cfg_handle: IndyHandle,
                                             rev_reg_id: *const c_char,
                                             cred_revoc_id: *const c_char,
                                             cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                  revoc_reg_delta_json: *const c_char,
                                             )>) -> ErrorCode {
    check_useful_c_str!(rev_reg_id, ErrorCode::CommonInvalidParam4);
    check_useful_c_str!(cred_revoc_id, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::RecoverCredential(
                    wallet_handle,
                    blob_storage_reader_cfg_handle,
                    rev_reg_id,
                    cred_revoc_id,
                    Box::new(move |result| {
                        let (err, revoc_reg_update_json) = prepare_result_1!(result, String::new());
                        let revoc_reg_update_json = ctypes::string_to_cstring(revoc_reg_update_json);
                        cb(command_handle, err, revoc_reg_update_json.as_ptr())
                    })
                ))));

    prepare_result!(result)
}*/

/// Merge two revocation registry deltas (returned by indy_issuer_create_credential or indy_issuer_revoke_credential) to accumulate common delta.
/// Send common delta to ledger to reduce the load.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// rev_reg_delta_json: revocation registry delta.
/// {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> an array of issued indices.
///         revoked: array<number> an array of revoked indices.
///     },
///     ver: string - version revocation registry delta json
/// }
///
/// other_rev_reg_delta_json: revocation registry delta for which PrevAccum value is equal to value of accum field of rev_reg_delta_json parameter.
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// merged_rev_reg_delta: Merged revocation registry delta
/// {
///     value: {
///         prevAccum: string - previous accumulator value.
///         accum: string - current accumulator value.
///         issued: array<number> an array of issued indices.
///         revoked: array<number> an array of revoked indices.
///     },
///     ver: string - version revocation registry delta json
/// }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_issuer_merge_revocation_registry_deltas(command_handle: CommandHandle,
                                                           rev_reg_delta_json: *const c_char,
                                                           other_rev_reg_delta_json: *const c_char,
                                                           cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                                merged_rev_reg_delta: *const c_char)>) -> ErrorCode {
    trace!("indy_issuer_merge_revocation_registry_deltas: >>> rev_reg_delta_json: {:?}, other_rev_reg_delta_json: {:?}",
           rev_reg_delta_json, other_rev_reg_delta_json);

    check_useful_validatable_json!(rev_reg_delta_json, ErrorCode::CommonInvalidParam2, RevocationRegistryDelta);
    check_useful_validatable_json!(other_rev_reg_delta_json, ErrorCode::CommonInvalidParam3, RevocationRegistryDelta);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_issuer_merge_revocation_registry_deltas: entities >>> rev_reg_delta_json: {:?}, other_rev_reg_delta_json: {:?}",
           rev_reg_delta_json, other_rev_reg_delta_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Issuer(
                IssuerCommand::MergeRevocationRegistryDeltas(
                    rev_reg_delta_json,
                    other_rev_reg_delta_json,
                    boxed_callback_string!("indy_issuer_merge_revocation_registry_deltas", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_issuer_merge_revocation_registry_deltas: <<< res: {:?}", res);

    res
}

/// Creates a master secret with a given id and stores it in the wallet.
/// The id must be unique.
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// master_secret_id: (optional, if not present random one will be generated) new master id
///
/// #Returns
/// out_master_secret_id: Id of generated master secret
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_master_secret(command_handle: CommandHandle,
                                               wallet_handle: WalletHandle,
                                               master_secret_id: *const c_char,
                                               cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                    out_master_secret_id: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_master_secret: >>> wallet_handle: {:?}, master_secret_id: {:?}", wallet_handle, master_secret_id);

    check_useful_opt_c_str!(master_secret_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_create_master_secret: entities >>> wallet_handle: {:?}, master_secret_id: {:?}", wallet_handle, master_secret_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CreateMasterSecret(
                    wallet_handle,
                    master_secret_id,
                    boxed_callback_string!("indy_prover_create_master_secret", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_master_secret: <<< res: {:?}", res);

    res
}

/// Creates a credential request for the given credential offer.
///
/// The method creates a blinded master secret for a master secret identified by a provided name.
/// The master secret identified by the name must be already stored in the secure wallet (see prover_create_master_secret)
/// The blinded master secret is a part of the credential request.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// wallet_handle: wallet handle (created by open_wallet)
/// prover_did: a DID of the prover
/// cred_offer_json: credential offer as a json containing information about the issuer and a credential
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///          ...
///         Other fields that contains data structures internal to Ursa.
///         These fields should not be parsed and are likely to change in future versions.
///     }
/// cred_def_json: credential definition json related to <cred_def_id> in <cred_offer_json>
/// master_secret_id: the id of the master secret stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// cred_req_json: Credential request json for creation of credential by Issuer
///     {
///      "prover_did" : string,
///      "cred_def_id" : string,
///         // Fields below can depend on Cred Def type
///      "blinded_ms" : <blinded_master_secret>,
///                     (opaque type that contains data structures internal to Ursa.
///                      It should not be parsed and are likely to change in future versions).
///      "blinded_ms_correctness_proof" : <blinded_ms_correctness_proof>,
///                     (opaque type that contains data structures internal to Ursa.
///                      It should not be parsed and are likely to change in future versions).
///      "nonce": string
///    }
/// cred_req_metadata_json: Credential request metadata json for further processing of received form Issuer credential.
///     Credential request metadata contains data structures internal to Ursa.
///     Credential request metadata mustn't be shared with Issuer.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_credential_req(command_handle: CommandHandle,
                                                wallet_handle: WalletHandle,
                                                prover_did: *const c_char,
                                                cred_offer_json: *const c_char,
                                                cred_def_json: *const c_char,
                                                master_secret_id: *const c_char,
                                                cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                     cred_req_json: *const c_char,
                                                                     cred_req_metadata_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_credential_req: >>> wallet_handle: {:?}, prover_did: {:?}, cred_offer_json: {:?}, cred_def_json: {:?}, master_secret_id: {:?}",
           wallet_handle, prover_did, cred_offer_json, cred_def_json, master_secret_id);

    check_useful_validatable_string!(prover_did, ErrorCode::CommonInvalidParam3, DidValue);
    check_useful_validatable_json!(cred_offer_json, ErrorCode::CommonInvalidParam4, CredentialOffer);
    check_useful_validatable_json!(cred_def_json, ErrorCode::CommonInvalidParam5, CredentialDefinition);
    check_useful_c_str!(master_secret_id, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_prover_create_credential_req: entities >>> wallet_handle: {:?}, prover_did: {:?}, cred_offer_json: {:?}, cred_def_json: {:?}, master_secret_id: {:?}",
           wallet_handle, prover_did, cred_offer_json, cred_def_json, master_secret_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CreateCredentialRequest(
                    wallet_handle,
                    prover_did,
                    cred_offer_json,
                    cred_def_json,
                    master_secret_id,
                    Box::new(move |result| {
                        let (err, cred_req_json, cred_req_metadata_json) = prepare_result_2!(result, String::new(), String::new());
                        trace!("indy_prover_create_credential_req: cred_req_json: {:?}, cred_req_metadata_json: {:?}", cred_req_json, cred_req_metadata_json);
                        let cred_req_json = ctypes::string_to_cstring(cred_req_json);
                        let cred_req_metadata_json = ctypes::string_to_cstring(cred_req_metadata_json);
                        cb(command_handle, err, cred_req_json.as_ptr(), cred_req_metadata_json.as_ptr())
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_credential_req: <<< res: {:?}", res);

    res
}

/// Set credential attribute tagging policy.
/// Writes a non-secret record marking attributes to tag, and optionally
/// updates tags on existing credentials on the credential definition to match.
///
/// EXPERIMENTAL
///
/// The following tags are always present on write:
///     {
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///     }
///
/// The policy sets the following tags for each attribute it marks taggable, written to subsequent
/// credentials and (optionally) all existing credentials on the credential definition:
///     {
///         "attr::<attribute name>::marker": "1",
///         "attr::<attribute name>::value": <attribute raw value>,
///     }
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: credential definition id
/// tag_attrs_json: JSON array with names of attributes to tag by policy, or null for all
/// retroactive: boolean, whether to apply policy to existing credentials on credential definition identifier
/// cb: Callback that takes command result as parameter.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_set_credential_attr_tag_policy(command_handle: CommandHandle,
                                                         wallet_handle: WalletHandle,
                                                         cred_def_id: *const c_char,
                                                         tag_attrs_json: *const c_char,
                                                         retroactive: bool,
                                                         cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_set_credential_attr_tag_policy: >>> wallet_handle: {:?}, cred_def_id: {:?}, tag_attrs_json: {:?}, retroactive: {:?}", wallet_handle, cred_def_id, tag_attrs_json, retroactive);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_opt_json!(tag_attrs_json, ErrorCode::CommonInvalidParam4, CredentialAttrTagPolicy);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam6);

    trace!("indy_prover_set_credential_attr_tag_policy: entities >>> wallet_handle: {:?}, cred_def_id: {:?}, tag_attrs_json: {:?}, retroactive: {:?}",
           wallet_handle, cred_def_id, tag_attrs_json, retroactive);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SetCredentialAttrTagPolicy(
                    wallet_handle,
                    cred_def_id,
                    tag_attrs_json,
                    retroactive,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_set_credential_attr_tag_policy: ");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_set_credential_attr_tag_policy: <<< res: {:?}", res);

    res
}

/// Get credential attribute tagging policy by credential definition id.
///
/// EXPERIMENTAL
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_def_id: credential definition id
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// JSON array with all attributes that current policy marks taggable;
/// null for default policy (tag all credential attributes).
/// 
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_get_credential_attr_tag_policy(command_handle: CommandHandle,
                                                         wallet_handle: WalletHandle,
                                                         cred_def_id: *const c_char,
                                                         cb: Option<extern fn(command_handle_: CommandHandle,
                                                                              err: ErrorCode,
                                                                              catpol_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credential_attr_tag_policy: >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    check_useful_validatable_string!(cred_def_id, ErrorCode::CommonInvalidParam3, CredentialDefinitionId);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credential_attr_tag_policy: entities >>> wallet_handle: {:?}, cred_def_id: {:?}", wallet_handle, cred_def_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentialAttrTagPolicy(
                    wallet_handle,
                    cred_def_id,
                    boxed_callback_string!("indy_prover_get_credential_attr_tag_policy", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credential_attr_tag_policy: <<< res: {:?}", res);

    res
}

/// Check credential provided by Issuer for the given credential request,
/// updates the credential by a master secret and stores in a secure wallet.
///
/// To support efficient and flexible search the following tags will be created for stored credential:
///     {
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // for every attribute in <credential values> that credential attribute tagging policy marks taggable
///         "attr::<attribute name>::marker": "1",
///         "attr::<attribute name>::value": <attribute raw value>,
///     }
///
/// #Params
/// command_handle: command handle to map callback to user context.
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: (optional, default is a random one) identifier by which credential will be stored in the wallet
/// cred_req_metadata_json: a credential request metadata created by indy_prover_create_credential_req
/// cred_json: credential json received from issuer
///     {
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_def_id", Optional<string>, - identifier of revocation registry
///         "values": - credential values
///             {
///                 "attr1" : {"raw": "value1", "encoded": "value1_as_int" },
///                 "attr2" : {"raw": "value1", "encoded": "value1_as_int" }
///             }
///         // Fields below can depend on Cred Def type
///         Other fields that contains data structures internal to Ursa.
///         These fields should not be parsed and are likely to change in future versions.
///     }
/// cred_def_json: credential definition json related to <cred_def_id> in <cred_json>
/// rev_reg_def_json: revocation registry definition json related to <rev_reg_def_id> in <cred_json>
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// out_cred_id: identifier by which credential is stored in the wallet
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_store_credential(command_handle: CommandHandle,
                                           wallet_handle: WalletHandle,
                                           cred_id: *const c_char,
                                           cred_req_metadata_json: *const c_char,
                                           cred_json: *const c_char,
                                           cred_def_json: *const c_char,
                                           rev_reg_def_json: *const c_char,
                                           cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                out_cred_id: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_store_credential: >>> wallet_handle: {:?}, cred_id: {:?}, cred_req_metadata_json: {:?}, cred_json: {:?}, cred_def_json: {:?}, \
    cred_def_json: {:?}", wallet_handle, cred_id, cred_req_metadata_json, cred_json, cred_def_json, rev_reg_def_json);

    check_useful_opt_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_validatable_json!(cred_req_metadata_json, ErrorCode::CommonInvalidParam4, CredentialRequestMetadata);
    check_useful_validatable_json!(cred_json, ErrorCode::CommonInvalidParam5, Credential);
    check_useful_validatable_json!(cred_def_json, ErrorCode::CommonInvalidParam6, CredentialDefinition);
    check_useful_opt_validatable_json!(rev_reg_def_json, ErrorCode::CommonInvalidParam7, RevocationRegistryDefinition);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_prover_store_credential: entities >>> wallet_handle: {:?}, cred_id: {:?}, cred_req_metadata_json: {:?}, cred_json: {:?}, cred_def_json: {:?}, \
    rev_reg_def_json: {:?}", wallet_handle, cred_id, cred_req_metadata_json, cred_json, cred_def_json, rev_reg_def_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::StoreCredential(
                    wallet_handle,
                    cred_id,
                    cred_req_metadata_json,
                    cred_json,
                    cred_def_json,
                    rev_reg_def_json,
                    boxed_callback_string!("indy_prover_store_credential", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_store_credential: <<< res: {:?}", res);

    res
}

/// Gets human readable credential by the given id.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: Identifier by which requested credential is stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credential json:
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_get_credential(command_handle: CommandHandle,
                                         wallet_handle: WalletHandle,
                                         cred_id: *const c_char,
                                         cb: Option<extern fn(
                                             command_handle_: CommandHandle, err: ErrorCode,
                                             credential_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credential: >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    check_useful_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credential: entities >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredential(
                    wallet_handle,
                    cred_id,
                    boxed_callback_string!("indy_prover_get_credential", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credential: <<< res: {:?}", res);

    res
}

/// Deletes credential by given id.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// cred_id: Identifier by which requested credential is stored in the wallet
/// cb: Callback that takes command result as parameter.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_delete_credential(command_handle: CommandHandle,
                                            wallet_handle: WalletHandle,
                                            cred_id: *const c_char,
                                            cb: Option<extern fn(
                                                command_handle_: CommandHandle,
                                                err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_delete_credential: >>> wallet_handle: {:?}, cred_id: {:?}", wallet_handle, cred_id);

    check_useful_c_str!(cred_id, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::DeleteCredential(
                    wallet_handle,
                    cred_id,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_delete_credential: ");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_delete_credential: <<< res: {:?}", res);

    res
}

/// Gets human readable credentials according to the filter.
/// If filter is NULL, then all credentials are returned.
/// Credentials can be filtered by Issuer, credential_def and/or Schema.
///
/// NOTE: This method is deprecated because immediately returns all fetched credentials.
/// Use <indy_prover_search_credentials> to fetch records by small batches.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// filter_json: filter for credentials
///        {
///            "schema_id": string, (Optional)
///            "schema_issuer_did": string, (Optional)
///            "schema_name": string, (Optional)
///            "schema_version": string, (Optional)
///            "issuer_did": string, (Optional)
///            "cred_def_id": string, (Optional)
///        }
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials json
///     [{
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }]
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
#[deprecated(since = "1.6.1", note = "Please use indy_prover_search_credentials instead!")]
pub extern fn indy_prover_get_credentials(command_handle: CommandHandle,
                                          wallet_handle: WalletHandle,
                                          filter_json: *const c_char,
                                          cb: Option<extern fn(
                                              command_handle_: CommandHandle, err: ErrorCode,
                                              matched_credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credentials: >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

    check_useful_opt_c_str!(filter_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_get_credentials: entities >>> wallet_handle: {:?}, filter_json: {:?}", wallet_handle, filter_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentials(
                    wallet_handle,
                    filter_json,
                    boxed_callback_string!("indy_prover_get_credentials", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credentials: <<< res: {:?}", res);

    res
}

/// Search for credentials stored in wallet.
/// Credentials can be filtered by tags created during saving of credential.
///
/// Instead of immediately returning of fetched credentials
/// this call returns search_handle that can be used later
/// to fetch records by small batches (with indy_prover_fetch_credentials).
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// query_json: Wql query filter for credentials searching based on tags.
///     where query: indy-sdk/docs/design/011-wallet-query-language/README.md
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials)
/// total_count: Total count of records
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_search_credentials(command_handle: CommandHandle,
                                             wallet_handle: WalletHandle,
                                             query_json: *const c_char,
                                             cb: Option<extern fn(
                                                 command_handle_: CommandHandle, err: ErrorCode,
                                                 search_handle: SearchHandle,
                                                 total_count: usize)>) -> ErrorCode {
    trace!("indy_prover_search_credentials: >>> wallet_handle: {:?}, query_json: {:?}", wallet_handle, query_json);

    check_useful_opt_c_str!(query_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_search_credentials: entities >>> wallet_handle: {:?}, query_json: {:?}", wallet_handle, query_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SearchCredentials(
                    wallet_handle,
                    query_json,
                    Box::new(move |result| {
                        let (err, handle, total_count) = prepare_result_2!(result, INVALID_SEARCH_HANDLE, 0);
                        cb(command_handle, err, handle, total_count)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_search_credentials: <<< res: {:?}", res);

    res
}

/// Fetch next credentials for search.
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials)
/// count: Count of credentials to fetch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials_json: List of human readable credentials:
///     [{
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }]
/// NOTE: The list of length less than the requested count means credentials search iterator is completed.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_fetch_credentials(command_handle: CommandHandle,
                                             search_handle: SearchHandle,
                                             count: usize,
                                             cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                  credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_fetch_credentials: >>> search_handle: {:?}, count: {:?}", search_handle, count);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_prover_fetch_credentials: entities >>> search_handle: {:?}, count: {:?}", search_handle, count);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::FetchCredentials(
                    search_handle,
                    count,
                    boxed_callback_string!("indy_prover_fetch_credentials", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_fetch_credentials: <<< res: {:?}", res);

    res
}

/// Close credentials search (make search handle invalid)
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_close_credentials_search(command_handle: CommandHandle,
                                                    search_handle: SearchHandle,
                                                    cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_close_credentials_search: >>> search_handle: {:?}", search_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_close_credentials_search: entities >>> search_handle: {:?}", search_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CloseCredentialsSearch(
                    search_handle,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_close_credentials_search:");
                        cb(command_handle, err)
                    })
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_close_credentials_search: <<< res: {:?}", res);

    res
}

/// Gets human readable credentials matching the given proof request.
///
/// NOTE: This method is deprecated because immediately returns all fetched credentials.
/// Use <indy_prover_search_credentials_for_proof_req> to fetch records by small batches.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level),
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// cb: Callback that takes command result as parameter.
///
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions": Optional<filter_json>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": int predicate value
///         "restrictions": Optional<filter_json>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
///  filter_json:
///     {
///        "schema_id": string, (Optional)
///        "schema_issuer_did": string, (Optional)
///        "schema_name": string, (Optional)
///        "schema_version": string, (Optional)
///        "issuer_did": string, (Optional)
///        "cred_def_id": string, (Optional)
///     }
///
/// #Returns
/// credentials_json: json with credentials for the given proof request.
///     {
///         "attrs": {
///             "<attr_referent>": [{ cred_info: <credential_info>, interval: Optional<non_revoc_interval> }],
///             ...,
///         },
///         "predicates": {
///             "requested_predicates": [{ cred_info: <credential_info>, timestamp: Optional<integer> }, { cred_info: <credential_2_info>, timestamp: Optional<integer> }],
///             "requested_predicate_2_referent": [{ cred_info: <credential_2_info>, timestamp: Optional<integer> }]
///         }
///     }, where <credential_info> is
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[deprecated(since = "1.6.1", note = "Please use indy_prover_search_credentials_for_proof_req instead!")]
#[no_mangle]
pub extern fn indy_prover_get_credentials_for_proof_req(command_handle: CommandHandle,
                                                        wallet_handle: WalletHandle,
                                                        proof_request_json: *const c_char,
                                                        cb: Option<extern fn(
                                                            command_handle_: CommandHandle, err: ErrorCode,
                                                            credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_get_credentials_for_proof_req: >>> wallet_handle: {:?}, proof_request_json: {:?}", wallet_handle, proof_request_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_get_credentials_for_proof_req: entities >>> wallet_handle: {:?}, proof_request_json: {:?}",
           wallet_handle, proof_request_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::GetCredentialsForProofReq(
                    wallet_handle,
                    proof_request_json,
                    boxed_callback_string!("indy_prover_get_credentials_for_proof_req", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_get_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Search for credentials matching the given proof request.
///
/// Instead of immediately returning of fetched credentials
/// this call returns search_handle that can be used later
/// to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req).
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level)
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
///
/// where
/// attr_info: Describes requested attribute
///     {
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// extra_query_json:(Optional) List of extra queries that will be applied to correspondent attribute/predicate:
///     {
///         "<attr_referent>": <wql query>,
///         "<predicate_referent>": <wql query>,
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed keys that can be combine into complex queries.
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // the following keys can be used for every `attribute name` in credential.
///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
///
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// search_handle: Search handle that can be used later to fetch records by small batches (with indy_prover_fetch_credentials_for_proof_req)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_search_credentials_for_proof_req(command_handle: CommandHandle,
                                                           wallet_handle: WalletHandle,
                                                           proof_request_json: *const c_char,
                                                           extra_query_json: *const c_char,
                                                           cb: Option<extern fn(
                                                               command_handle_: CommandHandle, err: ErrorCode,
                                                               search_handle: SearchHandle)>) -> ErrorCode {
    trace!("indy_prover_search_credentials_for_proof_req: >>> wallet_handle: {:?}, proof_request_json: {:?}, extra_query_json: {:?}", wallet_handle, proof_request_json, extra_query_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_opt_json!(extra_query_json, ErrorCode::CommonInvalidParam4, ProofRequestExtraQuery);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_search_credentials_for_proof_req: entities >>> wallet_handle: {:?}, proof_request_json: {:?}, extra_query_json: {:?}",
           wallet_handle, proof_request_json, extra_query_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::SearchCredentialsForProofReq(
                    wallet_handle,
                    proof_request_json,
                    extra_query_json,
                    Box::new(move |result| {
                        let (err, search_handle) = prepare_result_1!(result, INVALID_SEARCH_HANDLE);
                        trace!("indy_prover_search_credentials_for_proof_req: search_handle: {:?}", search_handle);
                        cb(command_handle, err, search_handle)
                    }),
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_search_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Fetch next credentials for the requested item using proof request search
/// handle (created by indy_prover_search_credentials_for_proof_req).
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
/// item_referent: Referent of attribute/predicate in the proof request
/// count: Count of credentials to fetch
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// credentials_json: List of credentials for the given proof request.
///     [{
///         cred_info: <credential_info>,
///         interval: Optional<non_revoc_interval>
///     }]
/// where
/// credential_info:
///     {
///         "referent": string, - id of credential in the wallet
///         "attrs": {"key1":"raw_value1", "key2":"raw_value2"}, - credential attributes
///         "schema_id": string, - identifier of schema
///         "cred_def_id": string, - identifier of credential definition
///         "rev_reg_id": Optional<string>, - identifier of revocation registry definition
///         "cred_rev_id": Optional<string> - identifier of credential in the revocation registry definition
///     }
/// non_revoc_interval:
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// NOTE: The list of length less than the requested count means that search iterator
/// correspondent to the requested <item_referent> is completed.
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_fetch_credentials_for_proof_req(command_handle: CommandHandle,
                                                           search_handle: SearchHandle,
                                                           item_referent: *const c_char,
                                                           count: usize,
                                                           cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                                                credentials_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_fetch_credentials_for_proof_req: >>> search_handle: {:?}, count: {:?}", search_handle, count);

    check_useful_c_str!(item_referent, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_fetch_credentials_for_proof_req: entities >>> search_handle: {:?}, count: {:?}", search_handle, count);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::FetchCredentialForProofReq(
                    search_handle,
                    item_referent,
                    count,
                    boxed_callback_string!("indy_prover_fetch_credentials_for_proof_request", cb, command_handle)
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_fetch_credentials_for_proof_req: <<< res: {:?}", res);

    res
}

/// Close credentials search for proof request (make search handle invalid)
///
/// #Params
/// search_handle: Search handle (created by indy_prover_search_credentials_for_proof_req)
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub  extern fn indy_prover_close_credentials_search_for_proof_req(command_handle: CommandHandle,
                                                                  search_handle: SearchHandle,
                                                                  cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode)>) -> ErrorCode {
    trace!("indy_prover_close_credentials_search_for_proof_req: >>> search_handle: {:?}", search_handle);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_prover_close_credentials_search_for_proof_req: entities >>> search_handle: {:?}", search_handle);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(
            AnoncredsCommand::Prover(
                ProverCommand::CloseCredentialsSearchForProofReq(
                    search_handle,
                    Box::new(move |result| {
                        let err = prepare_result!(result);
                        trace!("indy_prover_close_credentials_search:");
                        cb(command_handle, err)
                    }),
                ))));

    let res = prepare_result!(result);

    trace!("indy_prover_close_credentials_search_for_proof_req: <<< res: {:?}", res);

    res
}

/// Creates a proof according to the given proof request
/// Either a corresponding credential with optionally revealed attributes or self-attested attribute must be provided
/// for each requested attribute (see indy_prover_get_credentials_for_pool_req).
/// A proof request may request multiple credentials from different schemas and different issuers.
/// All required schemas, public keys and revocation registries must be provided.
/// The proof request also contains nonce.
/// The proof contains either proof or self-attested attribute value for each requested attribute.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (applies to every attribute and predicate but can be overridden on attribute level)
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// requested_credentials_json: either a credential or self-attested attribute for each requested attribute
///     {
///         "self_attested_attributes": {
///             "self_attested_attribute_referent": string
///         },
///         "requested_attributes": {
///             "requested_attribute_referent_1": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }},
///             "requested_attribute_referent_2": {"cred_id": string, "timestamp": Optional<number>, revealed: <bool> }}
///         },
///         "requested_predicates": {
///             "requested_predicates_referent_1": {"cred_id": string, "timestamp": Optional<number> }},
///         }
///     }
/// master_secret_id: the id of the master secret stored in the wallet
/// schemas_json: all schemas participating in the proof request
///     {
///         <schema1_id>: <schema1>,
///         <schema2_id>: <schema2>,
///         <schema3_id>: <schema3>,
///     }
/// credential_defs_json: all credential definitions participating in the proof request
///     {
///         "cred_def1_id": <credential_def1>,
///         "cred_def2_id": <credential_def2>,
///         "cred_def3_id": <credential_def3>,
///     }
/// rev_states_json: all revocation states participating in the proof request
///     {
///         "rev_reg_def1_id or credential_1_id": {
///             "timestamp1": <rev_state1>,
///             "timestamp2": <rev_state2>,
///         },
///         "rev_reg_def2_id or credential_1_id"": {
///             "timestamp3": <rev_state3>
///         },
///         "rev_reg_def3_id or credential_1_id"": {
///             "timestamp4": <rev_state4>
///         },
///     }
/// Note: use credential_id instead rev_reg_id in case proving several credentials from the same revocation registry.
/// cb: Callback that takes command result as parameter.
///
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed keys that can be combine into complex queries.
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // the following keys can be used for every `attribute name` in credential.
///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
///
/// #Returns
/// Proof json
/// For each requested attribute either a proof (with optionally revealed attribute value) or
/// self-attested attribute value is provided.
/// Each proof is associated with a credential and corresponding schema_id, cred_def_id, rev_reg_id and timestamp.
/// There is also aggregated proof part common for all credential proofs.
///     {
///         "requested_proof": {
///             "revealed_attrs": {
///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string},
///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string},
///             },
///             "revealed_attr_groups": {
///                 "requested_attr5_id": {
///                     "sub_proof_index": number,
///                     "values": {
///                         "attribute_name": {
///                             "raw": string,
///                             "encoded": string
///                         }
///                     },
///                 }
///             },
///             "unrevealed_attrs": {
///                 "requested_attr3_id": {sub_proof_index: number}
///             },
///             "self_attested_attrs": {
///                 "requested_attr2_id": self_attested_value,
///             },
///             "predicates": {
///                 "requested_predicate_1_referent": {sub_proof_index: int},
///                 "requested_predicate_2_referent": {sub_proof_index: int},
///             }
///         }
///         "proof": {
///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
///             "aggregated_proof": <aggregated_proof>
///         } (opaque type that contains data structures internal to Ursa.
///           It should not be parsed and are likely to change in future versions).
///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
///     }
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_prover_create_proof(command_handle: CommandHandle,
                                       wallet_handle: WalletHandle,
                                       proof_req_json: *const c_char,
                                       requested_credentials_json: *const c_char,
                                       master_secret_id: *const c_char,
                                       schemas_json: *const c_char,
                                       credential_defs_json: *const c_char,
                                       rev_states_json: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                            proof_json: *const c_char)>) -> ErrorCode {
    trace!("indy_prover_create_proof: >>> wallet_handle: {:?}, proof_req_json: {:?}, requested_credentials_json: {:?}, master_secret_id: {:?}, \
    schemas_json: {:?}, credential_defs_json: {:?}, rev_states_json: {:?}",
           wallet_handle, proof_req_json, requested_credentials_json, master_secret_id, schemas_json, credential_defs_json, rev_states_json);

    check_useful_validatable_json!(proof_req_json, ErrorCode::CommonInvalidParam3, ProofRequest);
    check_useful_validatable_json!(requested_credentials_json, ErrorCode::CommonInvalidParam4, RequestedCredentials);
    check_useful_c_str!(master_secret_id, ErrorCode::CommonInvalidParam5);
    check_useful_json!(schemas_json, ErrorCode::CommonInvalidParam6, Schemas);
    check_useful_json!(credential_defs_json, ErrorCode::CommonInvalidParam7, CredentialDefinitions);
    check_useful_json!(rev_states_json, ErrorCode::CommonInvalidParam8, RevocationStates);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam9);

    trace!("indy_prover_create_proof: entities >>> wallet_handle: {:?}, proof_req_json: {:?}, requested_credentials_json: {:?}, master_secret_id: {:?}, \
    schemas_json: {:?}, credential_defs_json: {:?}, rev_states_json: {:?}",
           wallet_handle, proof_req_json, requested_credentials_json, master_secret_id, schemas_json, credential_defs_json, rev_states_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateProof(
            wallet_handle,
            proof_req_json,
            requested_credentials_json,
            master_secret_id,
            schemas_json,
            credential_defs_json,
            rev_states_json,
            boxed_callback_string!("indy_prover_create_proof", cb, command_handle)
        ))));

    let res = prepare_result!(result);

    trace!("indy_prover_create_proof: <<< res: {:?}", res);

    res
}

/// Verifies a proof (of multiple credential).
/// All required schemas, public keys and revocation registries must be provided.
///
/// IMPORTANT: You must use *_id's (`schema_id`, `cred_def_id`, `rev_reg_id`) listed in `proof[identifiers]`
/// as the keys for corresponding `schemas_json`, `credential_defs_json`, `rev_reg_defs_json`, `rev_regs_json` objects.
///
/// #Params
/// wallet_handle: wallet handle (created by open_wallet).
/// command_handle: command handle to map callback to user context.
/// proof_request_json: proof request json
///     {
///         "name": string,
///         "version": string,
///         "nonce": string, - a decimal number represented as a string (use `indy_generate_nonce` function to generate 80-bit number)
///         "requested_attributes": { // set of requested attributes
///              "<attr_referent>": <attr_info>, // see below
///              ...,
///         },
///         "requested_predicates": { // set of requested predicates
///              "<predicate_referent>": <predicate_info>, // see below
///              ...,
///          },
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval for each attribute
///                        // (can be overridden on attribute level)
///         "ver": Optional<str>  - proof request version:
///             - omit or "1.0" to use unqualified identifiers for restrictions
///             - "2.0" to use fully qualified identifiers for restrictions
///     }
/// proof_json: created for request proof json
///     {
///         "requested_proof": {
///             "revealed_attrs": {
///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string}, // NOTE: check that `encoded` value match to `raw` value on application level
///             },
///             "revealed_attr_groups": {
///                 "requested_attr5_id": {
///                     "sub_proof_index": number,
///                     "values": {
///                         "attribute_name": {
///                             "raw": string,
///                             "encoded": string
///                         }
///                     }, // NOTE: check that `encoded` value match to `raw` value on application level
///                 }
///             },
///             "unrevealed_attrs": {
///                 "requested_attr3_id": {sub_proof_index: number}
///             },
///             "self_attested_attrs": {
///                 "requested_attr2_id": self_attested_value,
///             },
///             "requested_predicates": {
///                 "requested_predicate_1_referent": {sub_proof_index: int},
///                 "requested_predicate_2_referent": {sub_proof_index: int},
///             }
///         }
///         "proof": {
///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
///             "aggregated_proof": <aggregated_proof>
///         }
///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
///     }
/// schemas_json: all schemas participating in the proof
///     {
///         <schema1_id>: <schema1>,
///         <schema2_id>: <schema2>,
///         <schema3_id>: <schema3>,
///     }
/// credential_defs_json: all credential definitions participating in the proof
///     {
///         "cred_def1_id": <credential_def1>,
///         "cred_def2_id": <credential_def2>,
///         "cred_def3_id": <credential_def3>,
///     }
/// rev_reg_defs_json: all revocation registry definitions participating in the proof
///     {
///         "rev_reg_def1_id": <rev_reg_def1>,
///         "rev_reg_def2_id": <rev_reg_def2>,
///         "rev_reg_def3_id": <rev_reg_def3>,
///     }
/// rev_regs_json: all revocation registries participating in the proof
///     {
///         "rev_reg_def1_id": {
///             "timestamp1": <rev_reg1>,
///             "timestamp2": <rev_reg2>,
///         },
///         "rev_reg_def2_id": {
///             "timestamp3": <rev_reg3>
///         },
///         "rev_reg_def3_id": {
///             "timestamp4": <rev_reg4>
///         },
///     }
/// where
/// attr_referent: Proof-request local identifier of requested attribute
/// attr_info: Describes requested attribute
///     {
///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
///         "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
///                                              // NOTE: should either be "name" or "names", not both and not none of them.
///                                              // Use "names" to specify several attributes that have to match a single credential.
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// predicate_referent: Proof-request local identifier of requested attribute predicate
/// predicate_info: Describes requested attribute predicate
///     {
///         "name": attribute name, (case insensitive and ignore spaces)
///         "p_type": predicate type (">=", ">", "<=", "<")
///         "p_value": predicate value
///         "restrictions": Optional<wql query>, // see below
///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
///                        // If specified prover must proof non-revocation
///                        // for date in this interval this attribute
///                        // (overrides proof level interval)
///     }
/// non_revoc_interval: Defines non-revocation interval
///     {
///         "from": Optional<int>, // timestamp of interval beginning
///         "to": Optional<int>, // timestamp of interval ending
///     }
/// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
///     The list of allowed keys that can be combine into complex queries.
///         "schema_id": <credential schema id>,
///         "schema_issuer_did": <credential schema issuer did>,
///         "schema_name": <credential schema name>,
///         "schema_version": <credential schema version>,
///         "issuer_did": <credential issuer did>,
///         "cred_def_id": <credential definition id>,
///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not present
///         // the following keys can be used for every `attribute name` in credential.
///         "attr::<attribute name>::marker": "1", - to filter based on existence of a specific attribute
///         "attr::<attribute name>::value": <attribute raw value>, - to filter based on value of a specific attribute
/// 
/// cb: Callback that takes command result as parameter.
///
/// #Returns
/// valid: true - if signature is valid, false - otherwise
///
/// #Errors
/// Anoncreds*
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_verifier_verify_proof(command_handle: CommandHandle,
                                         proof_request_json: *const c_char,
                                         proof_json: *const c_char,
                                         schemas_json: *const c_char,
                                         credential_defs_json: *const c_char,
                                         rev_reg_defs_json: *const c_char,
                                         rev_regs_json: *const c_char,
                                         cb: Option<extern fn(command_handle_: CommandHandle, err: ErrorCode,
                                                              valid: bool)>) -> ErrorCode {
    trace!("indy_verifier_verify_proof: >>> proof_request_json: {:?}, proof_json: {:?}, schemas_json: {:?}, credential_defs_json: {:?}, \
    rev_reg_defs_json: {:?}, rev_regs_json: {:?}", proof_request_json, proof_json, schemas_json, credential_defs_json, rev_reg_defs_json, rev_regs_json);

    check_useful_validatable_json!(proof_request_json, ErrorCode::CommonInvalidParam2, ProofRequest);
    check_useful_validatable_json!(proof_json, ErrorCode::CommonInvalidParam3, Proof);
    check_useful_json!(schemas_json, ErrorCode::CommonInvalidParam4, Schemas);
    check_useful_json!(credential_defs_json, ErrorCode::CommonInvalidParam5, CredentialDefinitions);
    check_useful_json!(rev_reg_defs_json, ErrorCode::CommonInvalidParam6, RevocationRegistryDefinitions);
    check_useful_json!(rev_regs_json, ErrorCode::CommonInvalidParam7, RevocationRegistries);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_verifier_verify_proof: entities >>> proof_request_json: {:?}, proof_json: {:?}, schemas_json: {:?}, credential_defs_json: {:?}, \
    rev_reg_defs_json: {:?}, rev_regs_json: {:?}", proof_request_json, proof_json, schemas_json, credential_defs_json, rev_reg_defs_json, rev_regs_json);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Verifier(VerifierCommand::VerifyProof(
            proof_request_json,
            proof_json,
            schemas_json,
            credential_defs_json,
            rev_reg_defs_json,
            rev_regs_json,
            Box::new(move |result| {
                let (err, valid) = prepare_result_1!(result, false);
                trace!("indy_verifier_verify_proof: valid: {:?}", valid);

                cb(command_handle, err, valid)
            })
        ))));

    let res = prepare_result!(result);

    trace!("indy_verifier_verify_proof: <<< res: {:?}", res);

    res
}

/// Create revocation state for a credential that corresponds to a particular time.
///
/// Note that revocation delta must cover the whole registry existence time.
/// You can use `from`: `0` and `to`: `needed_time` as parameters for building request to get correct revocation delta.
///
/// The resulting revocation state and provided timestamp can be saved and reused later with applying a new
/// revocation delta with `indy_update_revocation_state` function.
/// This new delta should be received with parameters: `from`: `timestamp` and `to`: `needed_time`.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`)
/// rev_reg_def_json: revocation registry definition json related to `rev_reg_id` in a credential
/// rev_reg_delta_json: revocation registry delta which covers the whole registry existence time
/// timestamp: time represented as a total number of seconds from Unix Epoch.
/// cred_rev_id: user credential revocation id in revocation registry (match to `cred_rev_id` in a credential)
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// revocation state json:
///     {
///         "rev_reg": <revocation registry>,
///         "witness": <witness>,  (opaque type that contains data structures internal to Ursa.
///                                 It should not be parsed and are likely to change in future versions).
///         "timestamp" : integer
///     }
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_create_revocation_state(command_handle: CommandHandle,
                                           blob_storage_reader_handle: IndyHandle,
                                           rev_reg_def_json: *const c_char,
                                           rev_reg_delta_json: *const c_char,
                                           timestamp: u64,
                                           cred_rev_id: *const c_char,
                                           cb: Option<extern fn(
                                               command_handle_: CommandHandle, err: ErrorCode,
                                               rev_state_json: *const c_char)>) -> ErrorCode {
    trace!("indy_create_revocation_state: >>> blob_storage_reader_handle: {:?}, rev_reg_def_json: {:?}, rev_reg_delta_json: {:?}, timestamp: {:?}, \
    cred_rev_id: {:?}", blob_storage_reader_handle, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id);

    check_useful_validatable_json!(rev_reg_def_json, ErrorCode::CommonInvalidParam3, RevocationRegistryDefinition);
    check_useful_validatable_json!(rev_reg_delta_json, ErrorCode::CommonInvalidParam4, RevocationRegistryDelta);
    check_useful_c_str!(cred_rev_id, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam7);

    trace!("indy_create_revocation_state: entities >>> blob_storage_reader_handle: {:?}, rev_reg_def_json: {:?}, rev_reg_delta_json: {:?}, timestamp: {:?}, \
    cred_rev_id: {:?}", blob_storage_reader_handle, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::CreateRevocationState(
            blob_storage_reader_handle,
            rev_reg_def_json,
            rev_reg_delta_json,
            timestamp,
            cred_rev_id,
            boxed_callback_string!("indy_create_revocation_state", cb, command_handle)
        ))));

    let res = prepare_result!(result);

    trace!("indy_create_revocation_state: <<< res: {:?}", res);

    res
}

/// Create a new revocation state for a credential based on a revocation state created before.
/// Note that provided revocation delta must cover the registry gap from based state creation until the specified time
/// (this new delta should be received with parameters: `from`: `state_timestamp` and `to`: `needed_time`).
///
/// This function reduces the calculation time.
///
/// The resulting revocation state and provided timestamp can be saved and reused later by applying a new revocation delta again.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// blob_storage_reader_handle: configuration of blob storage reader handle that will allow to read revocation tails (returned by `indy_open_blob_storage_reader`)
/// rev_state_json: revocation registry state json
/// rev_reg_def_json: revocation registry definition json related to `rev_reg_id` in a credential
/// rev_reg_delta_json: revocation registry definition delta which covers the gap form original `rev_state_json` creation till the requested timestamp
/// timestamp: time represented as a total number of seconds from Unix Epoch
/// cred_rev_id: user credential revocation id in revocation registry (match to `cred_rev_id` in a credential)
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// revocation state json:
///     {
///         "rev_reg": <revocation registry>,
///         "witness": <witness>,  (opaque type that contains data structures internal to Ursa.
///                                 It should not be parsed and are likely to change in future versions).
///         "timestamp" : integer
///     }
///
/// #Errors
/// Common*
/// Wallet*
/// Anoncreds*
#[no_mangle]
pub extern fn indy_update_revocation_state(command_handle: CommandHandle,
                                           blob_storage_reader_handle: IndyHandle,
                                           rev_state_json: *const c_char,
                                           rev_reg_def_json: *const c_char,
                                           rev_reg_delta_json: *const c_char,
                                           timestamp: u64,
                                           cred_rev_id: *const c_char,
                                           cb: Option<extern fn(
                                               command_handle_: CommandHandle, err: ErrorCode,
                                               updated_rev_state_json: *const c_char)>) -> ErrorCode {
    trace!("indy_update_revocation_state: >>> blob_storage_reader_handle: {:?}, rev_state_json: {:?}, rev_reg_def_json: {:?}, rev_reg_delta_json: {:?}, \
    timestamp: {:?}, cred_rev_id: {:?}", blob_storage_reader_handle, rev_state_json, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id);

    check_useful_validatable_json!(rev_state_json, ErrorCode::CommonInvalidParam3, RevocationState);
    check_useful_validatable_json!(rev_reg_def_json, ErrorCode::CommonInvalidParam4, RevocationRegistryDefinition);
    check_useful_validatable_json!(rev_reg_delta_json, ErrorCode::CommonInvalidParam5, RevocationRegistryDelta);
    check_useful_c_str!(cred_rev_id, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam8);

    trace!("indy_update_revocation_state: entities >>> blob_storage_reader_handle: {:?}, rev_state_json: {:?}, rev_reg_def_json: {:?}, rev_reg_delta_json: {:?}, \
    timestamp: {:?}, cred_rev_id: {:?}", blob_storage_reader_handle, rev_state_json, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Prover(ProverCommand::UpdateRevocationState(
            blob_storage_reader_handle,
            rev_state_json,
            rev_reg_def_json,
            rev_reg_delta_json,
            timestamp,
            cred_rev_id,
            boxed_callback_string!("indy_update_revocation_state", cb, command_handle)
        ))));

    let res = prepare_result!(result);

    trace!("indy_update_revocation_state: <<< res: {:?}", res);

    res
}


///  Generates 80-bit numbers that can be used as a nonce for proof request.
///
/// #Params
/// command_handle: command handle to map callback to user context
/// cb: Callback that takes command result as parameter
///
/// #Returns
/// nonce: generated number as a string
///
#[no_mangle]
pub extern fn indy_generate_nonce(command_handle: CommandHandle,
                                  cb: Option<extern fn(
                                      command_handle_: CommandHandle, err: ErrorCode,
                                      nonce: *const c_char)>) -> ErrorCode {
    trace!("indy_generate_nonce: >>> ");

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam2);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::Verifier(
            VerifierCommand::GenerateNonce(
                boxed_callback_string!("indy_generate_nonce", cb, command_handle)
            ))));

    let res = prepare_result!(result);

    trace!("indy_generate_nonce: <<< res: {:?}", res);

    res
}

/// Get unqualified form (short form without method) of a fully qualified entity like DID.
///
/// This function should be used to the proper casting of fully qualified entity to unqualified form in the following cases:
///     Issuer, which works with fully qualified identifiers, creates a Credential Offer for Prover, which doesn't support fully qualified identifiers.
///     Verifier prepares a Proof Request based on fully qualified identifiers or Prover, which doesn't support fully qualified identifiers.
///     another case when casting to unqualified form needed
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// entity: string - target entity to disqualify. Can be one of:
///             Did
///             SchemaId
///             CredentialDefinitionId
///             RevocationRegistryId
///             Schema
///             CredentialDefinition
///             RevocationRegistryDefinition
///             CredentialOffer
///             CredentialRequest
///             ProofRequest
///
/// #Returns
///   res: entity either in unqualified form or original if casting isn't possible
#[no_mangle]
pub  extern fn indy_to_unqualified(command_handle: CommandHandle,
                                   entity: *const c_char,
                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                        err: ErrorCode,
                                                        res: *const c_char)>) -> ErrorCode {
    trace!("indy_to_unqualified: >>> entity: {:?}", entity);

    check_useful_c_str!(entity, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_to_unqualified: entities >>> entity: {:?}", entity);

    let result = CommandExecutor::instance()
        .send(Command::Anoncreds(AnoncredsCommand::ToUnqualified(
            entity,
            Box::new(move |result| {
                let (err, res) = prepare_result_1!(result, String::new());
                trace!("indy_to_unqualified: did: {:?}", res);
                let res = ctypes::string_to_cstring(res);
                cb(command_handle, err, res.as_ptr())
            }),
        )));

    let res = prepare_result!(result);

    trace!("indy_to_unqualified: <<< res: {:?}", res);

    res
}

