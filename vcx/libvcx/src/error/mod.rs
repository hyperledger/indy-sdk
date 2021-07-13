use std::cell::RefCell;
use std::fmt;
use std::ffi::CString;
use std::ptr;

use failure::{Context, Backtrace, Fail};
use libc::c_char;

use utils::error;
use utils::cstring::CStringUtils;

pub mod prelude {
    pub use super::{err_msg, VcxError, VcxErrorExt, VcxErrorKind, VcxResult, VcxResultExt, get_current_error_c_json};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum VcxErrorKind {
    // Common
    #[fail(display = "Object is in invalid state for requested operation")]
    InvalidState,
    #[fail(display = "Invalid Configuration")]
    InvalidConfiguration,
    #[fail(display = "Obj was not found with handle")]
    InvalidHandle,
    #[fail(display = "Invalid JSON string")]
    InvalidJson,
    #[fail(display = "Invalid Option")]
    InvalidOption,
    #[fail(display = "Invalid MessagePack")]
    InvalidMessagePack,
    #[fail(display = "Object cache error")]
    ObjectCacheError,
    #[fail(display = "Object not ready for specified action")]
    NotReady,
    #[fail(display = "IO Error, possibly creating a backup wallet")]
    IOError,
    #[fail(display = "Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[fail(display = "Waiting for callback timed out")]
    TimeoutLibindy,
    #[fail(display = "Parameter passed to libindy was invalid")]
    InvalidLibindyParam,
    #[fail(display = "Library already initialized")]
    AlreadyInitialized,
    #[fail(display = "Action is not supported")]
    ActionNotSupported,

    // Connection
    #[fail(display = "Could not create connection")]
    CreateConnection,
    #[fail(display = "Invalid Connection Handle")]
    InvalidConnectionHandle,
    #[fail(display = "Invalid invite details structure")]
    InvalidInviteDetail,
    #[fail(display = "Invalid redirect details structure")]
    InvalidRedirectDetail,
    #[fail(display = "Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency.")]
    DeleteConnection,
    #[fail(display = "Error with Connection")]
    GeneralConnectionError,

    // Payment
    #[fail(display = "No payment information associated with object")]
    NoPaymentInformation,
    #[fail(display = "Insufficient amount of tokens to process request")]
    InsufficientTokenAmount,
    #[fail(display = "Invalid payment address")]
    InvalidPaymentAddress,

    // Credential Definition error
    #[fail(display = "Call to create Credential Definition failed")]
    CreateCredDef,
    #[fail(display = "Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,
    #[fail(display = "Invalid Credential Definition handle")]
    InvalidCredDefHandle,

    // Revocation
    #[fail(display = "Failed to create Revocation Registration Definition")]
    CreateRevRegDef,
    #[fail(display = "Invalid Revocation Details")]
    InvalidRevocationDetails,
    #[fail(display = "Unable to Update Revocation Delta On Ledger")]
    InvalidRevocationEntry,
    #[fail(display = "Invalid Credential Revocation timestamp")]
    InvalidRevocationTimestamp,
    #[fail(display = "No revocation definition found")]
    RevRegDefNotFound,

    // Credential
    #[fail(display = "Invalid credential handle")]
    InvalidCredentialHandle,
    #[fail(display = "could not create credential request")]
    CreateCredentialRequest,

    // Issuer Credential
    #[fail(display = "Invalid Credential Issuer Handle")]
    InvalidIssuerCredentialHandle,
    #[fail(display = "Invalid Credential Request")]
    InvalidCredentialRequest,
    #[fail(display = "Invalid credential json")]
    InvalidCredential,
    #[fail(display = "Attributes provided to Credential Offer are not correct, possibly malformed")]
    InvalidAttributesStructure,

    // Proof
    #[fail(display = "Invalid proof handle")]
    InvalidProofHandle,
    #[fail(display = "Obj was not found with handle")]
    InvalidDisclosedProofHandle,
    #[fail(display = "Proof had invalid format")]
    InvalidProof,
    #[fail(display = "Schema was invalid or corrupt")]
    InvalidSchema,
    #[fail(display = "The Proof received does not have valid credentials listed.")]
    InvalidProofCredentialData,
    #[fail(display = "Could not create proof")]
    CreateProof,
    #[fail(display = "Proof Request Passed into Libindy Call Was Invalid")]
    InvalidProofRequest,

    // Schema
    #[fail(display = "Could not create schema")]
    CreateSchema,
    #[fail(display = "Invalid Schema Handle")]
    InvalidSchemaHandle,
    #[fail(display = "No Schema for that schema sequence number")]
    InvalidSchemaSeqNo,
    #[fail(display = "Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name Combination")]
    DuplicationSchema,
    #[fail(display = "Unknown Rejection of Schema Creation, refer to libindy documentation")]
    UnknownSchemaRejection,

    // Pool
    #[fail(display = "Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[fail(display = "Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[fail(display = "Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[fail(display = "Invalid response from ledger for paid transaction")]
    InvalidLedgerResponse,
    #[fail(display = "No Pool open. Can't return handle.")]
    NoPoolOpen,
    #[fail(display = "Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[fail(display = "Error Creating a wallet")]
    WalletCreate,
    #[fail(display = "Missing wallet name in config")]
    MissingWalletName,
    #[fail(display = "Missing exported wallet path in config")]
    MissingExportedWalletPath,
    #[fail(display = "Missing exported backup key in config")]
    MissingBackupKey,
    #[fail(display = "Attempt to open wallet with invalid credentials")]
    WalletAccessFailed,
    #[fail(display = "Invalid Wallet or Search Handle")]
    InvalidWalletHandle,
    #[fail(display = "Indy wallet already exists")]
    DuplicationWallet,
    #[fail(display = "Wallet record not found")]
    WalletRecordNotFound,
    #[fail(display = "Record already exists in the wallet")]
    DuplicationWalletRecord,
    #[fail(display = "Wallet not found")]
    WalletNotFound,
    #[fail(display = "Indy wallet already open")]
    WalletAlreadyOpen,
    #[fail(display = "Configuration is missing wallet key")]
    MissingWalletKey,
    #[fail(display = "Attempted to add a Master Secret that already existed in wallet")]
    DuplicationMasterSecret,
    #[fail(display = "Attempted to add a DID to wallet when that DID already exists in wallet")]
    DuplicationDid,

    // Logger
    #[fail(display = "Logging Error")]
    LoggingError,

    // Validation
    #[fail(display = "Could not encode string to a big integer.")]
    EncodeError,
    #[fail(display = "Unknown Error")]
    UnknownError,
    #[fail(display = "Invalid DID")]
    InvalidDid,
    #[fail(display = "Invalid VERKEY")]
    InvalidVerkey,
    #[fail(display = "Invalid NONCE")]
    InvalidNonce,
    #[fail(display = "Invalid URL")]
    InvalidUrl,
    #[fail(display = "Configuration is missing the Payment Method parameter")]
    MissingPaymentMethod,
    #[fail(display = "Unable to serialize")]
    SerializationError,
    #[fail(display = "Value needs to be base58")]
    NotBase58,

    // A2A
    #[fail(display = "Invalid HTTP response.")]
    InvalidHttpResponse,
    #[fail(display = "No Endpoint set for Connection Object")]
    NoEndpoint,
    #[fail(display = "Error Retrieving messages from API")]
    InvalidMessages,

    #[fail(display = "Common error {}", 0)]
    Common(u32),
    #[fail(display = "Libndy error {}", 0)]
    LibndyError(u32),
    #[fail(display = "Unknown libindy error")]
    UnknownLibndyError,
    #[fail(display = "No Agent pairwise information")]
    NoAgentInformation,
}

#[derive(Debug)]
pub struct VcxError {
    inner: Context<VcxErrorKind>
}

impl Fail for VcxError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for VcxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for cause in Fail::iter_chain(&self.inner) {
            if first {
                first = false;
                writeln!(f, "Error: {}", cause)?;
            } else {
                writeln!(f, "  Caused by: {}", cause)?;
            }
        }

        Ok(())
    }
}

impl VcxError {
    pub fn from_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        VcxError { inner: Context::new(msg).context(kind) }
    }

    pub fn kind(&self) -> VcxErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        let kind = self.kind();
        VcxError { inner: self.inner.map(|_| msg).context(kind) }
    }

    pub fn map<D>(self, kind: VcxErrorKind, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        VcxError { inner: self.inner.map(|_| msg).context(kind) }
    }
}

pub fn err_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
    where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
    VcxError::from_msg(kind, msg)
}

impl From<VcxErrorKind> for VcxError {
    fn from(kind: VcxErrorKind) -> VcxError {
        VcxError::from_msg(kind, ::utils::error::error_message(&kind.clone().into()))
    }
}

impl From<Context<VcxErrorKind>> for VcxError {
    fn from(inner: Context<VcxErrorKind>) -> VcxError {
        VcxError { inner }
    }
}

impl From<VcxError> for u32 {
    fn from(code: VcxError) -> u32 {
        set_current_error(&code);
        code.kind().into()
    }
}

impl From<VcxErrorKind> for u32 {
    fn from(code: VcxErrorKind) -> u32 {
        match code {
            VcxErrorKind::InvalidState => error::INVALID_STATE.code_num,
            VcxErrorKind::InvalidConfiguration => error::INVALID_CONFIGURATION.code_num,
            VcxErrorKind::InvalidHandle => error::INVALID_OBJ_HANDLE.code_num,
            VcxErrorKind::InvalidJson => error::INVALID_JSON.code_num,
            VcxErrorKind::InvalidOption => error::INVALID_OPTION.code_num,
            VcxErrorKind::InvalidMessagePack => error::INVALID_MSGPACK.code_num,
            VcxErrorKind::ObjectCacheError => error::OBJECT_CACHE_ERROR.code_num,
            VcxErrorKind::NoPaymentInformation => error::NO_PAYMENT_INFORMATION.code_num,
            VcxErrorKind::NotReady => error::NOT_READY.code_num,
            VcxErrorKind::InvalidRevocationDetails => error::INVALID_REVOCATION_DETAILS.code_num,
            VcxErrorKind::GeneralConnectionError => error::CONNECTION_ERROR.code_num,
            VcxErrorKind::IOError => error::IOERROR.code_num,
            VcxErrorKind::LibindyInvalidStructure => error::LIBINDY_INVALID_STRUCTURE.code_num,
            VcxErrorKind::TimeoutLibindy => error::TIMEOUT_LIBINDY_ERROR.code_num,
            VcxErrorKind::InvalidLibindyParam => error::INVALID_LIBINDY_PARAM.code_num,
            VcxErrorKind::AlreadyInitialized => error::ALREADY_INITIALIZED.code_num,
            VcxErrorKind::CreateConnection => error::CREATE_CONNECTION_ERROR.code_num,
            VcxErrorKind::InvalidConnectionHandle => error::INVALID_CONNECTION_HANDLE.code_num,
            VcxErrorKind::InvalidInviteDetail => error::INVALID_INVITE_DETAILS.code_num,
            VcxErrorKind::InvalidRedirectDetail => error::INVALID_REDIRECT_DETAILS.code_num,
            VcxErrorKind::DeleteConnection => error::CANNOT_DELETE_CONNECTION.code_num,
            VcxErrorKind::CreateCredDef => error::CREATE_CREDENTIAL_DEF_ERR.code_num,
            VcxErrorKind::CredDefAlreadyCreated => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            VcxErrorKind::InvalidCredDefHandle => error::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            VcxErrorKind::InvalidRevocationEntry => error::INVALID_REV_ENTRY.code_num,
            VcxErrorKind::CreateRevRegDef => error::INVALID_REV_REG_DEF_CREATION.code_num,
            VcxErrorKind::InvalidCredentialHandle => error::INVALID_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::CreateCredentialRequest => error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            VcxErrorKind::InvalidIssuerCredentialHandle => error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::InvalidCredentialRequest => error::INVALID_CREDENTIAL_REQUEST.code_num,
            VcxErrorKind::InvalidCredential => error::INVALID_CREDENTIAL_JSON.code_num,
            VcxErrorKind::InsufficientTokenAmount => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            VcxErrorKind::InvalidProofHandle => error::INVALID_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidDisclosedProofHandle => error::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidProof => error::INVALID_PROOF.code_num,
            VcxErrorKind::InvalidSchema => error::INVALID_SCHEMA.code_num,
            VcxErrorKind::InvalidProofCredentialData => error::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            VcxErrorKind::CreateProof => error::CREATE_PROOF_ERROR.code_num,
            VcxErrorKind::InvalidRevocationTimestamp => error::INVALID_REVOCATION_TIMESTAMP.code_num,
            VcxErrorKind::CreateSchema => error::INVALID_SCHEMA_CREATION.code_num,
            VcxErrorKind::InvalidSchemaHandle => error::INVALID_SCHEMA_HANDLE.code_num,
            VcxErrorKind::InvalidSchemaSeqNo => error::INVALID_SCHEMA_SEQ_NO.code_num,
            VcxErrorKind::DuplicationSchema => error::DUPLICATE_SCHEMA.code_num,
            VcxErrorKind::UnknownSchemaRejection => error::UNKNOWN_SCHEMA_REJECTION.code_num,
            VcxErrorKind::WalletCreate => error::INVALID_WALLET_CREATION.code_num,
            VcxErrorKind::MissingWalletName => error::MISSING_WALLET_NAME.code_num,
            VcxErrorKind::WalletAccessFailed => error::WALLET_ACCESS_FAILED.code_num,
            VcxErrorKind::InvalidWalletHandle => error::INVALID_WALLET_HANDLE.code_num,
            VcxErrorKind::DuplicationWallet => error::WALLET_ALREADY_EXISTS.code_num,
            VcxErrorKind::WalletNotFound => error::WALLET_NOT_FOUND.code_num,
            VcxErrorKind::WalletRecordNotFound => error::WALLET_RECORD_NOT_FOUND.code_num,
            VcxErrorKind::PoolLedgerConnect => error::POOL_LEDGER_CONNECT.code_num,
            VcxErrorKind::InvalidGenesisTxnPath => error::INVALID_GENESIS_TXN_PATH.code_num,
            VcxErrorKind::CreatePoolConfig => error::CREATE_POOL_CONFIG.code_num,
            VcxErrorKind::DuplicationWalletRecord => error::DUPLICATE_WALLET_RECORD.code_num,
            VcxErrorKind::WalletAlreadyOpen => error::WALLET_ALREADY_OPEN.code_num,
            VcxErrorKind::DuplicationMasterSecret => error::DUPLICATE_MASTER_SECRET.code_num,
            VcxErrorKind::DuplicationDid => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            VcxErrorKind::InvalidLedgerResponse => error::INVALID_LEDGER_RESPONSE.code_num,
            VcxErrorKind::InvalidAttributesStructure => error::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            VcxErrorKind::InvalidPaymentAddress => error::INVALID_PAYMENT_ADDRESS.code_num,
            VcxErrorKind::NoEndpoint => error::NO_ENDPOINT.code_num,
            VcxErrorKind::InvalidProofRequest => error::INVALID_PROOF_REQUEST.code_num,
            VcxErrorKind::NoPoolOpen => error::NO_POOL_OPEN.code_num,
            VcxErrorKind::PostMessageFailed => error::POST_MSG_FAILURE.code_num,
            VcxErrorKind::LoggingError => error::LOGGING_ERROR.code_num,
            VcxErrorKind::EncodeError => error::BIG_NUMBER_ERROR.code_num,
            VcxErrorKind::UnknownError => error::UNKNOWN_ERROR.code_num,
            VcxErrorKind::InvalidDid => error::INVALID_DID.code_num,
            VcxErrorKind::InvalidVerkey => error::INVALID_VERKEY.code_num,
            VcxErrorKind::InvalidNonce => error::INVALID_NONCE.code_num,
            VcxErrorKind::InvalidUrl => error::INVALID_URL.code_num,
            VcxErrorKind::MissingWalletKey => error::MISSING_WALLET_KEY.code_num,
            VcxErrorKind::MissingPaymentMethod => error::MISSING_PAYMENT_METHOD.code_num,
            VcxErrorKind::SerializationError => error::SERIALIZATION_ERROR.code_num,
            VcxErrorKind::NotBase58 => error::NOT_BASE58.code_num,
            VcxErrorKind::InvalidHttpResponse => error::INVALID_HTTP_RESPONSE.code_num,
            VcxErrorKind::InvalidMessages => error::INVALID_MESSAGES.code_num,
            VcxErrorKind::MissingExportedWalletPath => error::MISSING_EXPORTED_WALLET_PATH.code_num,
            VcxErrorKind::MissingBackupKey => error::MISSING_BACKUP_KEY.code_num,
            VcxErrorKind::UnknownLibndyError => error::UNKNOWN_LIBINDY_ERROR.code_num,
            VcxErrorKind::ActionNotSupported => error::ACTION_NOT_SUPPORTED.code_num,
            VcxErrorKind::Common(num) => num,
            VcxErrorKind::LibndyError(num) => num,
            VcxErrorKind::NoAgentInformation => error::NO_AGENT_INFO.code_num,
            VcxErrorKind::RevRegDefNotFound => error::REV_REG_DEF_NOT_FOUND.code_num,
        }
    }
}

pub type VcxResult<T> = Result<T, VcxError>;

/// Extension methods for `Result`.
pub trait VcxResultExt<T, E> {
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VcxResultExt<T, E> for Result<T, E> where E: Fail
{
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait VcxErrorExt {
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxError where D: fmt::Display + Send + Sync + 'static;
}

impl<E> VcxErrorExt for E where E: Fail
{
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxError where D: fmt::Display + Send + Sync + 'static {
        self.context(format!("\n{}: {}", std::any::type_name::<E>(), msg)).context(kind).into()
    }
}

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn reset_current_error() {
    CURRENT_ERROR_C_JSON.with(|error| {
        error.replace(None);
    })
}

pub fn set_current_error(err: &VcxError) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "error": err.kind().to_string(),
            "message": err.to_string(),
            "cause": Fail::find_root_cause(err).to_string(),
            "backtrace": err.backtrace().map(|bt| bt.to_string())
        }).to_string();
        error.replace(Some(CStringUtils::string_to_cstring(error_json)));
    })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();
}

pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON.try_with(|err|
        err.borrow().as_ref().map(|err| value = err.as_ptr())
    )
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();

    value
}
