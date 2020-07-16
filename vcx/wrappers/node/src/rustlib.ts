import * as ref from 'ref'
import * as StructType from 'ref-struct'

import { VCXRuntime } from './vcx'

export const VcxStatus = StructType({
  handle: 'int',
  msg: 'string',
  status: 'int'
})

interface IUintTypes {
  [key: string]: string
}
const UINTS_TYPES: IUintTypes = { x86: 'uint32', x64: 'uint64' }
const ARCHITECTURE: string = process.env.LIBVCX_FFI_ARCHITECTURE || 'x86'
const FFI_UINT: string = UINTS_TYPES[ARCHITECTURE]

// FFI Type Strings
export const FFI_ERROR_CODE = 'int'
export const FFI_BOOL = 'bool'
export const FFI_CONNECTION_HANDLE = 'uint32'
export const FFI_UNSIGNED_INT = 'uint32'
export const FFI_UNSIGNED_LONG = 'uint64'
export const FFI_UNSIGNED_INT_PTR = FFI_UINT
export const FFI_STRING = 'string'
export const FFI_CONFIG_PATH = FFI_STRING
export const FFI_STRING_DATA = 'string'
export const FFI_SOURCE_ID = 'string'
export const FFI_CONNECTION_DATA = 'string'
export const FFI_VOID = ref.types.void
export const FFI_CONNECTION_HANDLE_PTR = ref.refType(FFI_CONNECTION_HANDLE)
export const FFI_CALLBACK_PTR = 'pointer'
export const FFI_COMMAND_HANDLE = 'uint32'
export const FFI_CREDENTIAL_HANDLE = 'uint32'
export const FFI_PROOF_HANDLE = 'uint32'
export const FFI_CREDENTIALDEF_HANDLE = 'uint32'
export const FFI_SCHEMA_HANDLE = 'uint32'
export const FFI_SCHEMA_NUMBER = 'uint32'
export const FFI_PAYMENT_HANDLE = 'uint32'
export const FFI_PRICE = 'uint32'
export const FFI_LOG_FN = 'pointer'
export const FFI_POINTER = 'pointer'
export const FFI_VOID_POINTER = 'void *'

// Evernym extensions
export const FFI_INDY_NUMBER = 'int32'

// Rust Lib Native Types
export type rust_did = string
export type rust_error_code = number
export type rust_command_handle = number
export type rust_object_handle = number
export type rust_pool_handle = rust_object_handle
export type rust_wallet_handle = rust_object_handle
export type rust_listener_handle = rust_object_handle
export type rust_connection_handle = rust_object_handle

export interface IFFIEntryPoint {
  vcx_init: (commandId: number, configPath: string, cb: any) => number,
  vcx_init_with_config: (commandId: number, config: string, cb: any) => number,
  vcx_init_minimal: (config: string) => number,
  vcx_shutdown: (deleteIndyInfo: boolean) => number,
  vcx_error_c_message: (errorCode: number) => string,
  vcx_mint_tokens: (seed: string | undefined | null, fees: string | undefined | null) => void,
  vcx_version: () => string,
  vcx_messages_download: (commandId: number, status: string, uids: string, pairwiseDids: string, cb: any) => number,
  vcx_messages_update_status: (commandId: number, status: string, msgIds: string, cb: any) => number,
  vcx_get_ledger_author_agreement: (commandId: number, cb: any) => number,
  vcx_set_active_txn_author_agreement_meta: (text: string | undefined | null, version: string | undefined | null,
                                             hash: string | undefined | null, accMechType: string,
                                             timeOfAcceptance: number) => number,

  // wallet
  vcx_wallet_get_token_info: (commandId: number, payment: number | undefined | null, cb: any) => number,
  vcx_wallet_create_payment_address: (commandId: number, seed: string | null, cb: any) => number,
  vcx_wallet_sign_with_address: (commandID: number, address: string, message: number, messageLen: number, cb: any) =>
    number,
  vcx_wallet_verify_with_address: (commandID: number, address: string, message: number, messageLen: number,
  								                         signature: number, signatureLen: number, cb: any) => number,
  vcx_wallet_send_tokens: (commandId: number, payment: number, tokens: string, recipient: string, cb: any) => number,
  vcx_wallet_add_record: (commandId: number, type: string, id: string, value: string, tags: string, cb: any) => number,
  vcx_wallet_update_record_value: (commandId: number, type: string, id: string, value: string, cb: any) => number,
  vcx_wallet_update_record_tags: (commandId: number, type: string, id: string, tags: string, cb: any) => number,
  vcx_wallet_add_record_tags: (commandId: number, type: string, id: string, tags: string, cb: any) => number,
  vcx_wallet_delete_record_tags: (commandId: number, type: string, id: string, tagsList: string, cb: any) => number,
  vcx_wallet_delete_record: (commandId: number, type: string, id: string, cb: any) => number,
  vcx_wallet_get_record: (commandId: number, type: string, id: string, options: string, cb: any) => number,
  vcx_wallet_open_search: (commandId: number, type: string, query: string, options: string, cb: any) => number,
  vcx_wallet_close_search: (commandId: number, handle: number, cb: any) => number,
  vcx_wallet_search_next_records: (commandId: number, handle: number, count: number, cb: any) => number,
  vcx_wallet_set_handle: (handle: number) => void,
  vcx_ledger_get_fees: (commandId: number, cb: any) => number,
  vcx_agent_provision_async: (commandId: number, config: string, cb: any) => number,
  vcx_agent_update_info: (commandId: number, config: string, cb: any) => number,
  vcx_wallet_import: (commandId: number, config: string, cb: any) => number,
  vcx_wallet_export: (commandId: number, importPath: string, backupKey: string, cb: any) => number,
  vcx_wallet_validate_payment_address: (commandId: number, paymentAddress: string, cb: any) => number,
  vcx_update_institution_info: (institutionName: string, institutionLogoUrl: string) => number,
  vcx_update_webhook_url: (commandId: number, webhookUrl: string, cb: any) => number,
  vcx_pool_set_handle: (handle: number) => void,
  vcx_endorse_transaction: (commandId: number, transaction: string, cb: any) => number,

  // connection
  vcx_connection_delete_connection: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_connect: (commandId: number, handle: number, data: string, cb: any) => number,
  vcx_connection_create: (commandId: number, data: string, cb: any) => number,
  vcx_connection_create_with_invite: (commandId: number, data: string, invite: string, cb: any) => number,
  vcx_connection_create_with_connection_request_message: (commandId: number, data: string, request: string, cb: any) => number,
  vcx_connection_create_with_obtained_connection_request: (commandId: number, data: string, cb: any) => number,
  vcx_connection_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_connection_release: (handle: number) => number,
  vcx_connection_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_update_state_with_message: (commandId: number, handle: number, message: string, cb: any) => number,
  vcx_connection_get_state: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_invite_details: (commandId: number, handle: number, abbreviated: boolean, cb: any) => number,
  vcx_connection_public_invite_details: (commandId: number, cb: any) => number,
  vcx_connection_send_message: (commandId: number, handle: number, msg: string, sendMsgOptions: string, cb: any) =>
    number,
  vcx_connection_sign_data: (commandId: number, handle: number, data: number, dataLength: number, cb: any) => number
  vcx_connection_verify_signature: (commandId: number, handle: number, data: number, dataLength: number,
                                    signature: number, signatureLength: number, cb: any) => number,
  vcx_connection_send_ping: (commandId: number, handle: number, comment: string | null | undefined, cb: any) => number,
  vcx_connection_send_discovery_features: (commandId: number, handle: number, query: string | null | undefined,
                                           comment: string | null | undefined, cb: any) => number,
  vcx_connection_get_pw_did: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_get_their_pw_did: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_info: (commandId: number, handle: number, cb: any) => number,

  // issuer
  vcx_issuer_credential_release: (handle: number) => number,
  vcx_issuer_credential_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_issuer_credential_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_issuer_credential_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_issuer_credential_update_state_with_message: (commandId: number, handle: number, message: string, cb: any) =>
    number,
  vcx_issuer_credential_get_state: (commandId: number, handle: number, cb: any) => number,
  vcx_issuer_create_credential: (commandId: number, sourceId: string, credDefHandle: number, issuerDid: string | null,
                                 attr: string, credentialName: string, price: string, cb: any) => number,
  vcx_issuer_revoke_credential: (commandId: number, handle: number, cb: any) => number,
  vcx_issuer_send_credential: (commandId: number, credentialHandle: number, connectionHandle: number, cb: any) =>
    number,
  vcx_issuer_get_credential_msg: (commandId: number, credentialHandle: number, myPwDid: string, cb: any) =>
   number,
  vcx_issuer_send_credential_offer: (commandId: number, credentialHandle: number, connectionHandle: number, cb: any) =>
    number,
  vcx_issuer_get_credential_offer_msg: (commandId: number, credentialHandle: number, cb: any) => number
  vcx_issuer_credential_get_payment_txn: (commandId: number, handle: number, cb: any) => number,

  // proof
  vcx_proof_create: (commandId: number, sourceId: string, attrs: string, predicates: string,
                     revocationInterval: string, name: string, cb: any) => number,
  vcx_proof_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_get_proof: (commandId: number, proofHandle: number, connectionHandle: number, cb: any) => number,
  vcx_proof_release: (handle: number) => number,
  vcx_proof_send_request: (commandId: number, proofHandle: number, connectionHandle: number, cb: any) => number,
  vcx_proof_get_request_msg: (commandId: number, proofHandle: number, cb: any) => number,
  vcx_proof_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_proof_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_proof_update_state_with_message: (commandId: number, handle: number, message: string, cb: any) => number,
  vcx_proof_get_state: (commandId: number, handle: number, cb: any) => number,

  // disclosed proof
  vcx_disclosed_proof_create_with_request: (commandId: number, sourceId: string, req: string, cb: any) => number,
  vcx_disclosed_proof_create_with_msgid: (commandId: number, sourceId: string, connectionHandle: number,
                                          msgId: string, cb: any) => number,
  vcx_disclosed_proof_release: (handle: number) => number,
  vcx_disclosed_proof_send_proof: (commandId: number, proofHandle: number, connectionHandle: number, cb: any) => number,
  vcx_disclosed_proof_reject_proof: (commandId: number, proofHandle: number, connectionHandle: number,
                                     cb: any) => number,
  vcx_disclosed_proof_get_proof_msg: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_get_reject_msg: (commandId: number, handle: number, cb: any) => number,
  vcx_connection_redirect: (commandId: number, connectionHandle: number, redirectConnectionHandle: number, cb: any) => number,
  vcx_connection_get_redirect_details: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_disclosed_proof_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_update_state_with_message: (commandId: number, handle: number, message: string,
                                                  cb: any) => number,
  vcx_disclosed_proof_get_state: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_get_requests: (commandId: number, connectionHandle: number, cb: any) => number,
  vcx_disclosed_proof_retrieve_credentials: (commandId: number, handle: number, cb: any) => number,
  vcx_disclosed_proof_generate_proof: (commandId: number, handle: number, selectedCreds: string,
                                       selfAttestedAttrs: string, cb: any) => number,
  vcx_disclosed_proof_decline_presentation_request: (commandId: number, handle: number, connectionHandle: number,
                                                     reason: string | undefined | null,
                                                     proposal: string | undefined | null, cb: any) => number,

  // credential
  vcx_credential_create_with_offer: (commandId: number, sourceId: string, offer: string, cb: any) => number,
  vcx_credential_create_with_msgid: (commandId: number, sourceId: string, connectionHandle: number,
                                     msgId: string, cb: any) => number,
  vcx_credential_release: (handle: number) => number,
  vcx_credential_send_request: (commandId: number, handle: number, connectionHandle: number,
                                payment: number, cb: any) => number,
  vcx_credential_get_request_msg: (commandId: number, handle: number, myPwDid: string,
                                   theirPwDid: string | undefined | null, payment: number, cb: any) => number,
  vcx_credential_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_credential_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_credential_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_credential_update_state_with_message: (commandId: number, handle: number, message: string, cb: any) => number,
  vcx_credential_get_state: (commandId: number, handle: number, cb: any) => number,
  vcx_credential_get_offers: (commandId: number, connectionHandle: number, cb: any) => number,
  vcx_credential_get_payment_info: (commandId: number, handle: number, cb: any) => number,
  vcx_credential_get_payment_txn: (commandId: number, handle: number, cb: any) => number,

  // logger
  vcx_set_default_logger: (level: string) => number,
  vcx_set_logger: (context: any, enabled: any, logFn: any, flush: any) => number,

  // mock
  vcx_set_next_agency_response: (messageIndex: number) => void,

  // credentialdef
  vcx_credentialdef_create: (commandId: number, sourceId: string, credentialDefName: string, schemaId: string,
                             issuerDid: string | null, tag: string, config: string, payment: number, cb: any) => number
  vcx_credentialdef_prepare_for_endorser: (commandId: number, sourceId: string, credentialDefName: string,
                                           schemaId: string, issuerDid: string | null, tag: string, config: string,
                                           endorser: string, cb: any) => number
  vcx_credentialdef_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_credentialdef_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_credentialdef_release: (handle: number) => number,
  vcx_credentialdef_get_cred_def_id: (commandId: number, handle: number, cb: any) => string,
  vcx_credentialdef_get_payment_txn: (commandId: number, handle: number, cb: any) => number,
  vcx_credentialdef_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_credentialdef_get_state: (commandId: number, handle: number, cb: any) => number,

  // schema
  vcx_schema_get_attributes: (commandId: number, sourceId: string, schemaId: string, cb: any) => number,
  vcx_schema_create: (commandId: number, sourceId: string, schemaName: string, version: string, schemaData: string,
                      paymentHandle: number, cb: any) => number,
  vcx_schema_prepare_for_endorser: (commandId: number, sourceId: string, schemaName: string, version: string,
                                    schemaData: string, endorser: string, cb: any) => number,
  vcx_schema_get_schema_id: (commandId: number, handle: number, cb: any) => number,
  vcx_schema_deserialize: (commandId: number, data: string, cb: any) => number,
  vcx_schema_serialize: (commandId: number, handle: number, cb: any) => number,
  vcx_schema_release: (handle: number) => number,
  vcx_schema_get_payment_txn: (commandId: number, handle: number, cb: any) => number,
  vcx_schema_update_state: (commandId: number, handle: number, cb: any) => number,
  vcx_schema_get_state: (commandId: number, handle: number, cb: any) => number,
}

// tslint:disable object-literal-sort-keys
export const FFIConfiguration: { [ Key in keyof IFFIEntryPoint ]: any } = {

  vcx_init: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONFIG_PATH, FFI_CALLBACK_PTR]],
  vcx_init_with_config: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONFIG_PATH, FFI_CALLBACK_PTR]],
  vcx_init_minimal: [FFI_ERROR_CODE, [FFI_STRING]],
  vcx_shutdown: [FFI_ERROR_CODE, [FFI_BOOL]],
  vcx_error_c_message: [FFI_STRING, [FFI_ERROR_CODE]],
  vcx_version: [FFI_STRING, []],
  vcx_agent_provision_async: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_agent_update_info: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_update_institution_info: [FFI_ERROR_CODE, [FFI_STRING_DATA, FFI_STRING_DATA]],
  vcx_update_webhook_url: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_mint_tokens: [FFI_VOID, [FFI_STRING_DATA, FFI_STRING_DATA]],
  vcx_messages_download: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_messages_update_status: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_get_ledger_author_agreement: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_set_active_txn_author_agreement_meta: [FFI_ERROR_CODE, [FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_STRING_DATA, FFI_UNSIGNED_LONG]],
  vcx_endorse_transaction: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],

  // wallet
  vcx_wallet_get_token_info: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PAYMENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_wallet_create_payment_address: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_sign_with_address: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_UNSIGNED_INT_PTR,
    FFI_UNSIGNED_INT, FFI_CALLBACK_PTR]],
  vcx_wallet_verify_with_address: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_UNSIGNED_INT_PTR,
    FFI_UNSIGNED_INT, FFI_UNSIGNED_INT_PTR, FFI_UNSIGNED_INT, FFI_CALLBACK_PTR]],
  vcx_wallet_send_tokens: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PAYMENT_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_wallet_set_handle: [FFI_INDY_NUMBER, [FFI_INDY_NUMBER]],
  vcx_pool_set_handle: [FFI_INDY_NUMBER, [FFI_INDY_NUMBER]],
  vcx_ledger_get_fees: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_wallet_add_record: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING,
    FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_update_record_value: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING,
    FFI_CALLBACK_PTR]],
  vcx_wallet_update_record_tags: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING,
    FFI_CALLBACK_PTR]],
  vcx_wallet_add_record_tags: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING,
    FFI_CALLBACK_PTR]],
  vcx_wallet_delete_record_tags: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING,
    FFI_CALLBACK_PTR]],
  vcx_wallet_delete_record: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_get_record: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_open_search: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_close_search: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_wallet_search_next_records: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE,
    FFI_COMMAND_HANDLE, FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_wallet_import: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_export: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_STRING, FFI_CALLBACK_PTR]],
  vcx_wallet_validate_payment_address: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING, FFI_CALLBACK_PTR]],

  // connection
  vcx_connection_delete_connection: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_connect: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CONNECTION_DATA,
    FFI_CALLBACK_PTR]],
  vcx_connection_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_create_with_invite: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_connection_create_with_connection_request_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_create_with_obtained_connection_request: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_connection_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_release: [FFI_ERROR_CODE, [FFI_CONNECTION_HANDLE]],
  vcx_connection_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_update_state_with_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_invite_details: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_BOOL,
    FFI_CALLBACK_PTR]],
  vcx_connection_public_invite_details: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_send_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_sign_data: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_UNSIGNED_INT_PTR,
    FFI_UNSIGNED_INT, FFI_CALLBACK_PTR]],
  vcx_connection_verify_signature: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_UNSIGNED_INT_PTR,
    FFI_UNSIGNED_INT, FFI_UNSIGNED_INT_PTR, FFI_UNSIGNED_INT, FFI_CALLBACK_PTR]],
  vcx_connection_send_ping: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_send_discovery_features: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_connection_get_pw_did: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_get_their_pw_did: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_redirect: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_connection_get_redirect_details: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_connection_info: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],

  // issuer
  vcx_issuer_credential_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_issuer_credential_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_issuer_credential_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_issuer_credential_update_state_with_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_issuer_credential_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_issuer_create_credential: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID,
    FFI_CREDENTIALDEF_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_issuer_revoke_credential: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_issuer_send_credential: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_issuer_get_credential_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_issuer_send_credential_offer: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_issuer_get_credential_offer_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_issuer_credential_release: [FFI_ERROR_CODE, [FFI_CREDENTIAL_HANDLE]],
  vcx_issuer_credential_get_payment_txn: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],

  // proof
  vcx_proof_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_proof_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_get_proof: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],// tslint:disable-line
  vcx_proof_release: [FFI_ERROR_CODE, [FFI_PROOF_HANDLE]],
  vcx_proof_send_request: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_proof_get_request_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_proof_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_proof_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_proof_update_state_with_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_proof_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],

  // disclosed proof
  vcx_disclosed_proof_create_with_request: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_create_with_msgid: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_CONNECTION_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_release: [FFI_ERROR_CODE, [FFI_PROOF_HANDLE]],
  vcx_disclosed_proof_send_proof: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_reject_proof: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_get_proof_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_get_reject_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_update_state_with_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_get_requests: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_retrieve_credentials: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_generate_proof: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_disclosed_proof_decline_presentation_request: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_PROOF_HANDLE,
    FFI_CONNECTION_HANDLE, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR]],

  // credential
  vcx_credential_create_with_offer: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA,
    FFI_CALLBACK_PTR]],
  vcx_credential_create_with_msgid: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_CONNECTION_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_credential_release: [FFI_ERROR_CODE, [FFI_CREDENTIAL_HANDLE]],
  vcx_credential_send_request: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CONNECTION_HANDLE,
    FFI_PAYMENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_get_request_msg: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_PAYMENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_credential_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_update_state_with_message: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,
    FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_credential_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_get_offers: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CONNECTION_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credential_get_payment_info: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],
  vcx_credential_get_payment_txn: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],

  // credentialDef
  vcx_credentialdef_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_PAYMENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credentialdef_prepare_for_endorser: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_credentialdef_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_credentialdef_release: [FFI_ERROR_CODE, [FFI_CREDENTIALDEF_HANDLE]],
  vcx_credentialdef_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIALDEF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credentialdef_get_cred_def_id: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIALDEF_HANDLE, FFI_CALLBACK_PTR]],
  vcx_credentialdef_get_payment_txn: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],
  vcx_credentialdef_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],
  vcx_credentialdef_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],

  // logger
  vcx_set_default_logger: [FFI_ERROR_CODE, [FFI_STRING]],
  vcx_set_logger: [FFI_ERROR_CODE, [FFI_VOID_POINTER, FFI_CALLBACK_PTR, FFI_CALLBACK_PTR, FFI_CALLBACK_PTR]],

  // mock
  vcx_set_next_agency_response: [FFI_VOID, [FFI_UNSIGNED_INT]],

  // schema
  vcx_schema_get_attributes: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_schema_create: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_PAYMENT_HANDLE, FFI_CALLBACK_PTR]],
  vcx_schema_prepare_for_endorser: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SOURCE_ID, FFI_STRING_DATA,
    FFI_STRING_DATA, FFI_STRING_DATA, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_schema_get_schema_id: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SCHEMA_HANDLE, FFI_CALLBACK_PTR]],
  vcx_schema_deserialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_STRING_DATA, FFI_CALLBACK_PTR]],
  vcx_schema_release: [FFI_ERROR_CODE, [FFI_SCHEMA_HANDLE]],
  vcx_schema_serialize: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_SCHEMA_HANDLE, FFI_CALLBACK_PTR]],
  vcx_schema_get_payment_txn: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],
  vcx_schema_update_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]],
  vcx_schema_get_state: [FFI_ERROR_CODE, [FFI_COMMAND_HANDLE, FFI_CREDENTIAL_HANDLE,FFI_CALLBACK_PTR]]

}

let _rustAPI: IFFIEntryPoint
export const initRustAPI = (path?: string) => _rustAPI = new VCXRuntime({ basepath: path }).ffi
export const rustAPI = () => _rustAPI
