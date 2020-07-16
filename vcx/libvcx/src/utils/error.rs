use std::collections::HashMap;
use std::fmt;
use std::ffi::CString;


// **** DEFINE NEW ERRORS HERE ****
// STEP 1: create new public static instance of Error, assign it a new unused number and
// give it a human readable error message
// STEP 2: Add Error to the static MAP (used for getting messages to wrappers)
// STEP 3: create a test making sure that your message can be retrieved

pub static SUCCESS: Error = Error{code_num:0, message:"Success"};
pub static INDY_WALLET_RECORD_NOT_FOUND: Error = Error{code_num:212, message:"Error from Indy: Wallet Item not found"};
pub static INDY_DUPLICATE_WALLET_RECORD: Error = Error{code_num:213, message:"Error from Indy: Duplicate Wallet Record"};
pub static UNKNOWN_ERROR: Error = Error{code_num:1001, message:"Unknown Error"};
pub static CONNECTION_ERROR: Error = Error{code_num:1002, message:"Error with Connection"};
pub static INVALID_CONNECTION_HANDLE: Error = Error{code_num:1003, message:"Invalid Connection Handle"};
pub static INVALID_CONFIGURATION: Error = Error{code_num:1004, message:"Invalid Configuration"};
pub static NOT_READY: Error = Error{code_num:1005, message:"Object not ready for specified action"};
pub static NO_ENDPOINT: Error = Error{code_num:1006, message:"No Endpoint set for Connection Object"};
pub static INVALID_OPTION: Error = Error{code_num:1007, message:"Invalid Option"};
pub static INVALID_DID: Error = Error{code_num:1008, message:"Invalid DID"};
pub static INVALID_VERKEY: Error = Error{code_num:1009, message:"Invalid VERKEY"};
pub static POST_MSG_FAILURE: Error = Error{code_num:1010, message:"Message failed in post"};
pub static INVALID_NONCE: Error = Error{code_num:1011, message:"Invalid NONCE"};
pub static INVALID_KEY_DELEGATE: Error = Error{code_num:1012, message:"Invalid DELEGATE"};
pub static INVALID_URL: Error = Error{code_num:1013, message:"Invalid URL"};
pub static NOT_BASE58: Error = Error{code_num:1014, message:"Value needs to be base58"};
pub static INVALID_ISSUER_CREDENTIAL_HANDLE: Error = Error{code_num:1015, message:"Invalid Credential Issuer Handle"};
pub static INVALID_JSON: Error = Error{code_num:1016, message:"Invalid JSON string"};
pub static INVALID_PROOF_HANDLE: Error = Error{code_num:1017, message:"Invalid Proof Handle"};
pub static INVALID_CREDENTIAL_REQUEST: Error = Error{code_num:1018, message:"Invalid Credential Request"};
pub static INVALID_MSGPACK: Error = Error{code_num:1019, message:"Invalid MessagePack"};
pub static INVALID_MESSAGES: Error = Error{code_num:1020, message:"Error Retrieving messages from API"};
pub static INVALID_ATTRIBUTES_STRUCTURE: Error = Error{code_num:1021, message: "Attributes provided to Credential Offer are not correct, possibly malformed"};
pub static BIG_NUMBER_ERROR: Error = Error{code_num: 1022, message: "Could not encode string to a big integer."};
pub static INVALID_PROOF: Error = Error{code_num: 1023, message: "Proof had invalid format"};
pub static INVALID_GENESIS_TXN_PATH: Error = Error{code_num: 1024, message: "Must have valid genesis txn file path"};
pub static POOL_LEDGER_CONNECT: Error = Error{code_num: 1025, message: "Connection to Pool Ledger."};
pub static CREATE_POOL_CONFIG: Error = Error{code_num: 1026, message: "Formatting for Pool Config are incorrect."};
pub static INVALID_PROOF_CREDENTIAL_DATA: Error = Error{code_num: 1027, message: "The Proof received does not have valid credentials listed."};
pub static INDY_SUBMIT_REQUEST_ERR: Error = Error{code_num: 1028, message: "Call to indy submit request failed"};
pub static BUILD_CREDENTIAL_DEF_REQ_ERR: Error = Error{code_num: 1029, message: "Call to indy credential def request failed"};
pub static NO_POOL_OPEN: Error = Error{code_num: 1030, message: "No Pool open. Can't return handle."};
pub static INVALID_SCHEMA: Error = Error{code_num: 1031, message: "Schema was invalid or corrupt"};
pub static FAILED_PROOF_COMPLIANCE: Error = Error{code_num: 1032, message: "Proof is not compliant to proof request"};
pub static INVALID_HTTP_RESPONSE: Error = Error{code_num: 1033, message: "Invalid HTTP response."};
pub static CREATE_CREDENTIAL_DEF_ERR: Error = Error{code_num: 1034, message: "Call to create Credential Definition failed"};
pub static UNKNOWN_LIBINDY_ERROR: Error = Error{code_num: 1035, message: "Unknown libindy error"};
pub static INVALID_CREDENTIAL_DEF_JSON: Error = Error{code_num: 1036, message: "Credential Def not in valid json"};
pub static INVALID_CREDENTIAL_DEF_HANDLE: Error = Error{code_num: 1037, message: "Invalid Credential Definition handle"};
pub static TIMEOUT_LIBINDY_ERROR: Error = Error{code_num: 1038, message: "Waiting for callback timed out"};
pub static CREDENTIAL_DEF_ALREADY_CREATED: Error = Error{code_num: 1039, message: "Can't create, Credential Def already exists in wallet"};
pub static INVALID_SCHEMA_SEQ_NO: Error = Error{code_num: 1040, message: "No Schema for that schema sequence number"};
pub static INVALID_SCHEMA_CREATION: Error = Error{code_num: 1041, message: "Could not create schema"};
pub static INVALID_SCHEMA_HANDLE: Error = Error{code_num: 1042, message: "Invalid Schema Handle"};
pub static INVALID_MASTER_SECRET: Error = Error{code_num: 1043, message: "Invalid master secret"};
pub static ALREADY_INITIALIZED: Error = Error{code_num: 1044, message: "Library already initialized"};
pub static INVALID_INVITE_DETAILS: Error = Error{code_num: 1045, message: "Invalid invite details structure"};
pub static INVALID_SELF_ATTESTED_VAL: Error = Error{code_num: 1046, message: "Self Attested Value invalid"};
pub static INVALID_PREDICATE: Error = Error{code_num: 1047, message: "Predicate in proof is invalid"};
pub static INVALID_OBJ_HANDLE: Error = Error{code_num: 1048, message: "Obj was not found with handle"};
pub static INVALID_DISCLOSED_PROOF_HANDLE: Error = Error{code_num: 1049, message: "Obj was not found with handle"};
pub static SERIALIZATION_ERROR: Error = Error{code_num: 1050, message: "Unable to serialize"};
pub static WALLET_ALREADY_EXISTS: Error = Error{code_num: 1051, message: "Indy wallet already exists"};
pub static WALLET_ALREADY_OPEN: Error = Error{code_num: 1052, message: "Indy wallet already open"};
pub static INVALID_CREDENTIAL_HANDLE: Error = Error{code_num: 1053, message: "Invalid credential handle"};
pub static INVALID_CREDENTIAL_JSON: Error = Error{code_num: 1054, message: "Invalid credential json"};
pub static CREATE_CREDENTIAL_REQUEST_ERROR: Error = Error{code_num: 1055, message: "could not create credential request"};
pub static CREATE_PROOF_ERROR: Error = Error{code_num: 1056, message: "could not create proof"};
pub static INVALID_WALLET_HANDLE: Error = Error{code_num: 1057, message: "Invalid Wallet or Search Handle"};
pub static INVALID_WALLET_CREATION: Error = Error{code_num: 1058, message: "Error Creating a wallet"};
pub static INVALID_POOL_NAME: Error = Error{code_num: 1059, message: "Pool Name in config was invalid"};
pub static CANNOT_DELETE_CONNECTION: Error = Error{code_num: 1060, message: "Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency."};
pub static CREATE_CONNECTION_ERROR: Error = Error{code_num:1061, message: "Could not create connection"};
pub static INVALID_WALLET_SETUP: Error = Error{code_num:1062, message: "Invalid wallet keys...have you provisioned correctly?"};
pub static COMMON_ERROR: Error = Error{ code_num: 1063, message: "Common Error"};
pub static INSUFFICIENT_TOKEN_AMOUNT: Error = Error{code_num: 1064, message: "Insufficient amount of tokens to process request"};
pub static UNKNOWN_TXN_TYPE: Error = Error{code_num: 1065, message: "Unknown ledger transaction type"};
pub static INVALID_PAYMENT_ADDRESS: Error = Error{code_num: 1066, message: "Invalid payment address"};
pub static INVALID_LIBINDY_PARAM: Error = Error{code_num: 1067, message: "Parameter passed to libindy was invalid"};
pub static INVALID_PAYMENT: Error = Error{code_num: 1068, message: "Invalid Payment Details"};
pub static MISSING_WALLET_KEY: Error = Error{ code_num: 1069, message: "Configuration is missing wallet key"};
pub static OBJECT_CACHE_ERROR: Error = Error{ code_num: 1070, message: "Object cache error"};
pub static NO_PAYMENT_INFORMATION: Error = Error { code_num: 1071, message: "No payment information associated with object"};
pub static DUPLICATE_WALLET_RECORD: Error = Error{ code_num: 1072, message: "Record already exists in the wallet"};
pub static WALLET_RECORD_NOT_FOUND: Error = Error{ code_num: 1073, message: "Wallet record not found"};
pub static IOERROR: Error = Error { code_num: 1074, message: "IO Error, possibly creating a backup wallet"};
pub static WALLET_ACCESS_FAILED: Error = Error { code_num: 1075, message: "Attempt to open wallet with invalid credentials"};
pub static MISSING_WALLET_NAME: Error = Error { code_num: 1076, message: "Missing wallet name in config"};
pub static MISSING_EXPORTED_WALLET_PATH: Error = Error { code_num: 1077, message: "Missing exported wallet path in config"};
pub static MISSING_BACKUP_KEY: Error = Error { code_num: 1078, message: "Missing exported backup key in config"};
pub static WALLET_NOT_FOUND: Error = Error { code_num: 1079, message: "Wallet Not Found"};
pub static LIBINDY_INVALID_STRUCTURE: Error = Error { code_num: 1080, message: "Object (json, config, key, credential and etc...) passed to libindy has invalid structure"};
pub static INVALID_STATE: Error = Error { code_num: 1081, message: "Object is in invalid state for requested operation"};
pub static INVALID_LEDGER_RESPONSE: Error = Error {code_num: 1082, message: "Invalid response from ledger for paid transaction"};
pub static DID_ALREADY_EXISTS_IN_WALLET: Error = Error { code_num: 1083, message: "Attempted to add a DID to wallet when that DID already exists in wallet" };
pub static DUPLICATE_MASTER_SECRET: Error = Error { code_num: 1084, message: "Attempted to add a Master Secret that already existed in wallet"};
pub static THREAD_ERROR: Error = Error{ code_num: 1085, message: "Unable to create thread"};
pub static INVALID_PROOF_REQUEST: Error = Error{ code_num: 1086, message: "Proof Request Passed into Libindy Call Was Invalid"};
pub static MISSING_PAYMENT_METHOD: Error = Error{ code_num: 1087, message: "Configuration is missing the Payment Method parameter"};
pub static DUPLICATE_SCHEMA: Error = Error{ code_num: 1088, message: "Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name Combination"};
pub static UKNOWN_LIBINDY_TRANSACTION_REJECTION: Error = Error{ code_num: 1089, message: "Unknown Libindy Rejection"};
pub static LOGGING_ERROR: Error = Error{ code_num: 1090, message: "Logging Error" };
pub static INVALID_REVOCATION_DETAILS: Error = Error{ code_num: 1091, message: "Invalid Revocation Details"};
pub static INVALID_REV_ENTRY: Error = Error{ code_num: 1092, message: "Unable to Update Revocation Delta On Ledger"};
pub static INVALID_REVOCATION_TIMESTAMP: Error = Error{ code_num: 1093, message: "Invalid Credential Revocation timestamp"};
pub static UNKNOWN_SCHEMA_REJECTION: Error = Error{ code_num: 1094, message: "Unknown Rejection of Schema Creation, refer to libindy documentation"};
pub static INVALID_REV_REG_DEF_CREATION: Error = Error{ code_num: 1095, message: "Failed to create Revocation Registration Definition"};
/* EC 1096 - 1099 are reserved for proprietary forks of libVCX */
pub static INVALID_ATTACHMENT_ENCODING: Error = Error { code_num: 1100, message: "Failed to decode attachment"};
pub static UNKNOWN_ATTACHMENT_ENCODING: Error = Error { code_num: 1101, message: "This type of attachment can not be used"};
pub static UNKNOWN_MIME_TYPE: Error = Error { code_num: 1102, message: "Unknown mime type"};
pub static ACTION_NOT_SUPPORTED: Error = Error { code_num: 1103, message: "Action is not supported"};
pub static INVALID_REDIRECT_DETAILS: Error = Error{code_num: 1104, message: "Invalid redirect details structure"};
/* EC 1105 is reserved for proprietary forks of libVCX */
pub static NO_AGENT_INFO: Error = Error{code_num: 1106, message: "Agent pairwise information not found"};
pub static REV_DELTA_NOT_FOUND: Error = Error{code_num: 1107, message: "No revocation delta found in storage for this revocation registry. Were any credentials locally revoked?"};

lazy_static! {
    static ref ERROR_C_MESSAGES: HashMap<u32, CString> = {
       let mut m = HashMap::new();
        insert_c_message(&mut m, &SUCCESS);
        insert_c_message(&mut m, &UNKNOWN_ERROR);
        insert_c_message(&mut m, &CONNECTION_ERROR);
        insert_c_message(&mut m, &INVALID_CONNECTION_HANDLE);
        insert_c_message(&mut m, &INVALID_CONFIGURATION);
        insert_c_message(&mut m, &NOT_READY);
        insert_c_message(&mut m, &NO_ENDPOINT);
        insert_c_message(&mut m, &INVALID_OPTION);
        insert_c_message(&mut m, &INVALID_DID);
        insert_c_message(&mut m, &INVALID_VERKEY);
        insert_c_message(&mut m, &POST_MSG_FAILURE);
        insert_c_message(&mut m, &INVALID_NONCE);
        insert_c_message(&mut m, &INVALID_KEY_DELEGATE);
        insert_c_message(&mut m, &INVALID_URL);
        insert_c_message(&mut m, &NOT_BASE58);
        insert_c_message(&mut m, &INVALID_ISSUER_CREDENTIAL_HANDLE);
        insert_c_message(&mut m, &INVALID_JSON);
        insert_c_message(&mut m, &INVALID_MESSAGES);
        insert_c_message(&mut m, &INVALID_MSGPACK);
        insert_c_message(&mut m, &INVALID_ATTRIBUTES_STRUCTURE);
        insert_c_message(&mut m, &INVALID_PROOF_HANDLE);
        insert_c_message(&mut m, &INVALID_CREDENTIAL_REQUEST);
        insert_c_message(&mut m, &BIG_NUMBER_ERROR);
        insert_c_message(&mut m, &INVALID_PROOF);
        insert_c_message(&mut m, &INVALID_GENESIS_TXN_PATH);
        insert_c_message(&mut m, &CREATE_POOL_CONFIG);
        insert_c_message(&mut m, &INVALID_PROOF_CREDENTIAL_DATA);
        insert_c_message(&mut m, &POOL_LEDGER_CONNECT);
        insert_c_message(&mut m, &INDY_SUBMIT_REQUEST_ERR);
        insert_c_message(&mut m, &BUILD_CREDENTIAL_DEF_REQ_ERR);
        insert_c_message(&mut m, &NO_POOL_OPEN);
        insert_c_message(&mut m, &INVALID_SCHEMA);
        insert_c_message(&mut m, &FAILED_PROOF_COMPLIANCE);
        insert_c_message(&mut m, &INVALID_HTTP_RESPONSE);
        insert_c_message(&mut m, &CREATE_CREDENTIAL_DEF_ERR);
        insert_c_message(&mut m, &UNKNOWN_LIBINDY_ERROR);
        insert_c_message(&mut m, &TIMEOUT_LIBINDY_ERROR);
        insert_c_message(&mut m, &INVALID_CREDENTIAL_DEF_JSON);
        insert_c_message(&mut m, &INVALID_CREDENTIAL_DEF_HANDLE);
        insert_c_message(&mut m, &CREDENTIAL_DEF_ALREADY_CREATED);
        insert_c_message(&mut m, &INVALID_SCHEMA_SEQ_NO);
        insert_c_message(&mut m, &INVALID_SCHEMA_CREATION);
        insert_c_message(&mut m, &INVALID_SCHEMA_HANDLE);
        insert_c_message(&mut m, &ALREADY_INITIALIZED);
        insert_c_message(&mut m, &INVALID_INVITE_DETAILS);
        insert_c_message(&mut m, &INVALID_MASTER_SECRET);
        insert_c_message(&mut m, &INVALID_OBJ_HANDLE);
        insert_c_message(&mut m, &INVALID_DISCLOSED_PROOF_HANDLE);
        insert_c_message(&mut m, &SERIALIZATION_ERROR);
        insert_c_message(&mut m, &WALLET_ALREADY_EXISTS);
        insert_c_message(&mut m, &WALLET_ALREADY_OPEN);
        insert_c_message(&mut m, &INVALID_CREDENTIAL_HANDLE);
        insert_c_message(&mut m, &INVALID_CREDENTIAL_JSON);
        insert_c_message(&mut m, &CREATE_CREDENTIAL_REQUEST_ERROR);
        insert_c_message(&mut m, &CREATE_PROOF_ERROR);
        insert_c_message(&mut m, &INVALID_WALLET_HANDLE);
        insert_c_message(&mut m, &INVALID_WALLET_CREATION);
        insert_c_message(&mut m, &INVALID_POOL_NAME);
        insert_c_message(&mut m, &CANNOT_DELETE_CONNECTION);
        insert_c_message(&mut m, &CREATE_CONNECTION_ERROR);
        insert_c_message(&mut m, &INVALID_WALLET_SETUP);
        insert_c_message(&mut m, &COMMON_ERROR);
        insert_c_message(&mut m, &INSUFFICIENT_TOKEN_AMOUNT);
        insert_c_message(&mut m, &UNKNOWN_TXN_TYPE);
        insert_c_message(&mut m, &INVALID_PAYMENT);
        insert_c_message(&mut m, &INVALID_PAYMENT_ADDRESS);
        insert_c_message(&mut m, &INVALID_LIBINDY_PARAM);
        insert_c_message(&mut m, &MISSING_WALLET_KEY);
        insert_c_message(&mut m, &DUPLICATE_WALLET_RECORD);
        insert_c_message(&mut m, &WALLET_RECORD_NOT_FOUND);
        insert_c_message(&mut m, &IOERROR);
        insert_c_message(&mut m, &WALLET_ACCESS_FAILED);
        insert_c_message(&mut m, &OBJECT_CACHE_ERROR);
        insert_c_message(&mut m, &NO_PAYMENT_INFORMATION);
        insert_c_message(&mut m, &INDY_DUPLICATE_WALLET_RECORD);
        insert_c_message(&mut m, &INDY_WALLET_RECORD_NOT_FOUND);
        insert_c_message(&mut m, &MISSING_WALLET_NAME);
        insert_c_message(&mut m, &MISSING_EXPORTED_WALLET_PATH);
        insert_c_message(&mut m, &MISSING_BACKUP_KEY);
        insert_c_message(&mut m, &WALLET_NOT_FOUND);
        insert_c_message(&mut m, &LIBINDY_INVALID_STRUCTURE);
        insert_c_message(&mut m, &INVALID_STATE);
        insert_c_message(&mut m, &DID_ALREADY_EXISTS_IN_WALLET);
        insert_c_message(&mut m, &DUPLICATE_MASTER_SECRET);
        insert_c_message(&mut m, &INVALID_LEDGER_RESPONSE);
        insert_c_message(&mut m, &THREAD_ERROR);
        insert_c_message(&mut m, &INVALID_PROOF_REQUEST);
        insert_c_message(&mut m, &INVALID_REVOCATION_DETAILS);
        insert_c_message(&mut m, &INVALID_REV_REG_DEF_CREATION);
        insert_c_message(&mut m, &INVALID_REVOCATION_TIMESTAMP);
        insert_c_message(&mut m, &INVALID_REV_ENTRY);
        insert_c_message(&mut m, &DUPLICATE_SCHEMA);
        insert_c_message(&mut m, &UNKNOWN_SCHEMA_REJECTION);
        insert_c_message(&mut m, &UKNOWN_LIBINDY_TRANSACTION_REJECTION);
        insert_c_message(&mut m, &MISSING_PAYMENT_METHOD);
        insert_c_message(&mut m, &LOGGING_ERROR);
        insert_c_message(&mut m, &INVALID_ATTACHMENT_ENCODING);
        insert_c_message(&mut m, &UNKNOWN_ATTACHMENT_ENCODING);
        insert_c_message(&mut m, &UNKNOWN_MIME_TYPE);
        insert_c_message(&mut m, &ACTION_NOT_SUPPORTED);
        insert_c_message(&mut m, &INVALID_REDIRECT_DETAILS);
        insert_c_message(&mut m, &NO_AGENT_INFO);

        m
    };
}

// ******* END *******

// Helper function for static defining of error messages. Does limited checking that it can.
fn insert_c_message(map: &mut HashMap<u32, CString>, error: &Error) {
    if map.contains_key(&error.code_num) {
       panic!("Error Code number was repeated which is not allowed! (likely a copy/paste error)")
    }
    map.insert(error.code_num, CString::new(error.message).unwrap());

}

#[derive(Clone, Copy)]
pub struct Error {
    pub code_num: u32,
    pub message: &'static str
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = error_message(&self.code_num);
        write!(f, "{}: (Error Num:{})", msg, &self.code_num)
    }
}

pub fn error_c_message(code_num:&u32) -> &CString {
    match ERROR_C_MESSAGES.get(code_num) {
        Some(msg) => &msg,
        None => error_c_message(&UNKNOWN_ERROR.code_num),
    }
}

pub fn error_message(code_num:&u32) -> String {
    match ERROR_C_MESSAGES.get(code_num) {
        Some(msg) => msg.to_str().unwrap().to_string(),
        None => error_message(&UNKNOWN_ERROR.code_num),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_has_error(){
        let e = &UNKNOWN_ERROR;
        assert_eq!(e.code_num, 1001);
    }

    #[test]
    fn test_display_error(){
        let msg = format!("{}",UNKNOWN_ERROR);
        assert_eq!(msg, "Unknown Error: (Error Num:1001)")
    }

    #[test]
    fn test_error_message(){
        let msg = error_message(&1);
        assert_eq!(msg, "Unknown Error");

        let msg = error_message(&1002);
        assert_eq!(msg, "Error with Connection");
    }

    #[test]
    fn test_unknown_error(){
        assert_eq!(error_message(&UNKNOWN_ERROR.code_num), UNKNOWN_ERROR.message);
    }

    #[test]
    fn test_connection_error(){
        assert_eq!(error_message(&CONNECTION_ERROR.code_num), CONNECTION_ERROR.message);
    }

    #[test]
    fn test_success_error(){
        assert_eq!(error_message(&SUCCESS.code_num), SUCCESS.message);
    }

    #[test]
    fn test_no_endpoint_error(){
        assert_eq!(error_message(&NO_ENDPOINT.code_num), NO_ENDPOINT.message);
    }

    #[test]
    fn test_invalid_option_error(){
        assert_eq!(error_message(&INVALID_OPTION.code_num), INVALID_OPTION.message);
    }

    #[test]
    fn test_error_retrieving_messages(){
        assert_eq!(error_message(&INVALID_MESSAGES.code_num), INVALID_MESSAGES.message);
    }

    #[test]
    fn test_malformed_attributes_for_credential_offer(){
        assert_eq!(error_message(&INVALID_ATTRIBUTES_STRUCTURE.code_num), INVALID_ATTRIBUTES_STRUCTURE.message);
    }

    #[test]
    fn test_invalid_proof_handle_error(){
        assert_eq!(error_message(&INVALID_PROOF_HANDLE.code_num), INVALID_PROOF_HANDLE.message);
    }

    #[test]
    fn test_credential_request_incorrect_json_format_error(){
        assert_eq!(error_message(&INVALID_CREDENTIAL_REQUEST.code_num), INVALID_CREDENTIAL_REQUEST.message);
    }

    #[test]
    fn test_error_invalid_proof() {
        assert_eq!(error_message(&INVALID_PROOF.code_num), INVALID_PROOF.message);
    }
    #[test]
    fn test_error_genesis() {
        assert_eq!(error_message(&INVALID_GENESIS_TXN_PATH.code_num), INVALID_GENESIS_TXN_PATH.message);
    }
    #[test]
    fn test_error_config() {
        assert_eq!(error_message(&POOL_LEDGER_CONNECT.code_num), POOL_LEDGER_CONNECT.message);
    }
    #[test]
    fn test_error_pool_config() {
        assert_eq!(error_message(&CREATE_POOL_CONFIG.code_num), CREATE_POOL_CONFIG.message);
    }
    #[test]
    fn test_error_big_number() {
        assert_eq!(error_message(&BIG_NUMBER_ERROR.code_num), BIG_NUMBER_ERROR.message);
        assert_eq!(error_message(&INVALID_PROOF_CREDENTIAL_DATA.code_num), INVALID_PROOF_CREDENTIAL_DATA.message);
        assert_eq!(error_message(&INDY_SUBMIT_REQUEST_ERR.code_num), INDY_SUBMIT_REQUEST_ERR.message);
        assert_eq!(error_message(&BUILD_CREDENTIAL_DEF_REQ_ERR.code_num), BUILD_CREDENTIAL_DEF_REQ_ERR.message);
        assert_eq!(error_message(&NO_POOL_OPEN.code_num), NO_POOL_OPEN.message);
    }

    #[test]
    fn test_proof_incorrect_json_format_error(){
        assert_eq!(error_message(&INVALID_PROOF.code_num), INVALID_PROOF.message);
    }

    #[test]
    fn test_error_credential_data() {
        assert_eq!(error_message(&INVALID_PROOF_CREDENTIAL_DATA.code_num), INVALID_PROOF_CREDENTIAL_DATA.message);
    }
    #[test]
    fn test_failed_proof_compliance() {
        assert_eq!(error_message(&FAILED_PROOF_COMPLIANCE.code_num), FAILED_PROOF_COMPLIANCE.message);
    }

    #[test]
    fn test_credential_def_err() {
        assert_eq!(error_message(&CREATE_CREDENTIAL_DEF_ERR.code_num), CREATE_CREDENTIAL_DEF_ERR.message);
    }

    #[test]
    fn test_unknown_libindy_error() {
        assert_eq!(error_message(&UNKNOWN_LIBINDY_ERROR.code_num), UNKNOWN_LIBINDY_ERROR.message);
    }

    #[test]
    fn test_timeout_libindy_error() {
        assert_eq!(error_message(&TIMEOUT_LIBINDY_ERROR.code_num), TIMEOUT_LIBINDY_ERROR.message);
    }

    #[test]
    fn test_invalid_credential_def_json() {
        assert_eq!(error_message(&INVALID_CREDENTIAL_DEF_JSON.code_num), INVALID_CREDENTIAL_DEF_JSON.message);
    }

    #[test]
    fn test_credential_def_handle_err() {
        assert_eq!(error_message(&INVALID_CREDENTIAL_DEF_HANDLE.code_num), INVALID_CREDENTIAL_DEF_HANDLE.message);
    }

    #[test]
    fn test_credential_def_already_on_ledger_err() {
        assert_eq!(error_message(&CREDENTIAL_DEF_ALREADY_CREATED.code_num), CREDENTIAL_DEF_ALREADY_CREATED.message);
    }

    #[test]
    fn test_schema_err() {
        assert_eq!(error_message(&INVALID_SCHEMA.code_num), INVALID_SCHEMA.message);
        assert_eq!(error_message(&INVALID_SCHEMA_SEQ_NO.code_num), INVALID_SCHEMA_SEQ_NO.message);
        assert_eq!(error_message(&INVALID_SCHEMA_CREATION.code_num), INVALID_SCHEMA_CREATION.message);
        assert_eq!(error_message(&INVALID_SCHEMA_HANDLE.code_num), INVALID_SCHEMA_HANDLE.message);
    }

    #[test]
    fn test_already_initialized() {
        assert_eq!(error_message(&ALREADY_INITIALIZED.code_num), ALREADY_INITIALIZED.message);
    }

    #[test]
    fn test_invalid_invite_details() {
        assert_eq!(error_message(&INVALID_INVITE_DETAILS.code_num), INVALID_INVITE_DETAILS.message);
    }

    #[test]
    fn test_invalid_redirect_details() {
        assert_eq!(error_message(&INVALID_REDIRECT_DETAILS.code_num), INVALID_REDIRECT_DETAILS.message);
    }

    #[test]
    fn test_invalid_master_secret() {
        assert_eq!(error_message(&INVALID_MASTER_SECRET.code_num), INVALID_MASTER_SECRET.message);
    }
}
