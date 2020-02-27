#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[cfg(feature = "casting_errors")]
extern crate zmq;

extern crate rmp_serde;

extern crate zeroize;

extern crate named_type;

#[macro_use]
extern crate named_type_derive;

extern crate regex;

#[macro_use]
extern crate derivative;

pub type IndyHandle = i32;

#[repr(transparent)]
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct WalletHandle(pub i32);

pub const INVALID_WALLET_HANDLE: WalletHandle = WalletHandle(0);

pub type CallbackHandle = i32;

pub type PoolHandle = i32;

pub const INVALID_POOL_HANDLE: PoolHandle = 0;

pub type CommandHandle = i32;

pub const INVALID_COMMAND_HANDLE: CommandHandle = 0;

pub type StorageHandle = i32;

#[repr(transparent)]
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct SearchHandle(pub i32);

pub const INVALID_SEARCH_HANDLE: SearchHandle = SearchHandle(0);

/*
pub type SearchHandle = i32;
pub const INVALID_SEARCH_HANDLE : SearchHandle = 0;
*/

#[macro_use]
pub mod qualifier;

pub mod domain;

pub mod errors;

pub use errors::IndyError;

pub mod validation;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(i32)]
pub enum ErrorCode
{
    Success = 0,

    // Common errors

    // Caller passed invalid value as param 1 (null, invalid json and etc..)
    CommonInvalidParam1 = 100,

    // Caller passed invalid value as param 2 (null, invalid json and etc..)
    CommonInvalidParam2 = 101,

    // Caller passed invalid value as param 3 (null, invalid json and etc..)
    CommonInvalidParam3 = 102,

    // Caller passed invalid value as param 4 (null, invalid json and etc..)
    CommonInvalidParam4 = 103,

    // Caller passed invalid value as param 5 (null, invalid json and etc..)
    CommonInvalidParam5 = 104,

    // Caller passed invalid value as param 6 (null, invalid json and etc..)
    CommonInvalidParam6 = 105,

    // Caller passed invalid value as param 7 (null, invalid json and etc..)
    CommonInvalidParam7 = 106,

    // Caller passed invalid value as param 8 (null, invalid json and etc..)
    CommonInvalidParam8 = 107,

    // Caller passed invalid value as param 9 (null, invalid json and etc..)
    CommonInvalidParam9 = 108,

    // Caller passed invalid value as param 10 (null, invalid json and etc..)
    CommonInvalidParam10 = 109,

    // Caller passed invalid value as param 11 (null, invalid json and etc..)
    CommonInvalidParam11 = 110,

    // Caller passed invalid value as param 12 (null, invalid json and etc..)
    CommonInvalidParam12 = 111,

    // Invalid library state was detected in runtime. It signals library bug
    CommonInvalidState = 112,

    // Object (json, config, key, credential and etc...) passed by library caller has invalid structure
    CommonInvalidStructure = 113,

    // IO Error
    CommonIOError = 114,

    // Caller passed invalid value as param 13 (null, invalid json and etc..)
    CommonInvalidParam13 = 115,

    // Caller passed invalid value as param 14 (null, invalid json and etc..)
    CommonInvalidParam14 = 116,

    // Caller passed invalid value as param 15 (null, invalid json and etc..)
    CommonInvalidParam15 = 117,

    // Caller passed invalid value as param 16 (null, invalid json and etc..)
    CommonInvalidParam16 = 118,

    // Caller passed invalid value as param 17 (null, invalid json and etc..)
    CommonInvalidParam17 = 119,

    // Caller passed invalid value as param 18 (null, invalid json and etc..)
    CommonInvalidParam18 = 120,

    // Caller passed invalid value as param 19 (null, invalid json and etc..)
    CommonInvalidParam19 = 121,

    // Caller passed invalid value as param 20 (null, invalid json and etc..)
    CommonInvalidParam20 = 122,

    // Caller passed invalid value as param 21 (null, invalid json and etc..)
    CommonInvalidParam21 = 123,

    // Caller passed invalid value as param 22 (null, invalid json and etc..)
    CommonInvalidParam22 = 124,

    // Caller passed invalid value as param 23 (null, invalid json and etc..)
    CommonInvalidParam23 = 125,

    // Caller passed invalid value as param 24 (null, invalid json and etc..)
    CommonInvalidParam24 = 126,

    // Caller passed invalid value as param 25 (null, invalid json and etc..)
    CommonInvalidParam25 = 127,

    // Caller passed invalid value as param 26 (null, invalid json and etc..)
    CommonInvalidParam26 = 128,

    // Caller passed invalid value as param 27 (null, invalid json and etc..)
    CommonInvalidParam27 = 129,

    // Wallet errors
    // Caller passed invalid wallet handle
    WalletInvalidHandle = 200,

    // Unknown type of wallet was passed on create_wallet
    WalletUnknownTypeError = 201,

    // Attempt to register already existing wallet type
    WalletTypeAlreadyRegisteredError = 202,

    // Attempt to create wallet with name used for another exists wallet
    WalletAlreadyExistsError = 203,

    // Requested entity id isn't present in wallet
    WalletNotFoundError = 204,

    // Trying to use wallet with pool that has different name
    WalletIncompatiblePoolError = 205,

    // Trying to open wallet that was opened already
    WalletAlreadyOpenedError = 206,

    // Attempt to open encrypted wallet with invalid credentials
    WalletAccessFailed = 207,

    // Input provided to wallet operations is considered not valid
    WalletInputError = 208,

    // Decoding of wallet data during input/output failed
    WalletDecodingError = 209,

    // Storage error occurred during wallet operation
    WalletStorageError = 210,

    // Error during encryption-related operations
    WalletEncryptionError = 211,

    // Requested wallet item not found
    WalletItemNotFound = 212,

    // Returned if wallet's add_record operation is used with record name that already exists
    WalletItemAlreadyExists = 213,

    // Returned if provided wallet query is invalid
    WalletQueryError = 214,

    // Ledger errors
    // Trying to open pool ledger that wasn't created before
    PoolLedgerNotCreatedError = 300,

    // Caller passed invalid pool ledger handle
    PoolLedgerInvalidPoolHandle = 301,

    // Pool ledger terminated
    PoolLedgerTerminated = 302,

    // No consensus during ledger operation
    LedgerNoConsensusError = 303,

    // Attempt to parse invalid transaction response
    LedgerInvalidTransaction = 304,

    // Attempt to send transaction without the necessary privileges
    LedgerSecurityError = 305,

    // Attempt to create pool ledger config with name used for another existing pool
    PoolLedgerConfigAlreadyExistsError = 306,

    // Timeout for action
    PoolLedgerTimeout = 307,

    // Attempt to open Pool for witch Genesis Transactions are not compatible with set Protocol version.
    // Call pool.indy_set_protocol_version to set correct Protocol version.
    PoolIncompatibleProtocolVersion = 308,

    // Item not found on ledger.
    LedgerNotFound = 309,

    // Revocation registry is full and creation of new registry is necessary
    AnoncredsRevocationRegistryFullError = 400,

    AnoncredsInvalidUserRevocId = 401,

    // Attempt to generate master secret with duplicated name
    AnoncredsMasterSecretDuplicateNameError = 404,

    AnoncredsProofRejected = 405,

    AnoncredsCredentialRevoked = 406,

    // Attempt to create credential definition with duplicated id
    AnoncredsCredDefAlreadyExistsError = 407,

    // Crypto errors
    // Unknown format of DID entity keys
    UnknownCryptoTypeError = 500,

    // Attempt to create duplicate did
    DidAlreadyExistsError = 600,

    // Unknown payment method was given
    PaymentUnknownMethodError = 700,

    //No method were scraped from inputs/outputs or more than one were scraped
    PaymentIncompatibleMethodsError = 701,

    // Insufficient funds on inputs
    PaymentInsufficientFundsError = 702,

    // No such source on a ledger
    PaymentSourceDoesNotExistError = 703,

    // Operation is not supported for payment method
    PaymentOperationNotSupportedError = 704,

    // Extra funds on inputs
    PaymentExtraFundsError = 705,

    // The transaction is not allowed to a requester
    TransactionNotAllowedError = 706,

}

pub mod wallet {
    use super::*;
    use libc::c_char;

    /// Create the wallet storage (For example, database creation)
    ///
    /// #Params
    /// name: wallet storage name (the same as wallet name)
    /// config: wallet storage config (For example, database config)
    /// credentials_json: wallet storage credentials (For example, database credentials)
    /// metadata: wallet metadata (For example encrypted keys).
    pub type WalletCreate = extern fn(name: *const c_char,
                                      config: *const c_char,
                                      credentials_json: *const c_char,
                                      metadata: *const c_char) -> ErrorCode;

    /// Open the wallet storage (For example, opening database connection)
    ///
    /// #Params
    /// name: wallet storage name (the same as wallet name)
    /// config: wallet storage config (For example, database config)
    /// credentials_json: wallet storage credentials (For example, database credentials)
    /// storage_handle_p: pointer to store opened storage handle
    pub type WalletOpen = extern fn(name: *const c_char,
                                    config: *const c_char,
                                    credentials_json: *const c_char,
                                    storage_handle_p: *mut IndyHandle) -> ErrorCode;

    /// Close the opened walled storage (For example, closing database connection)
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    pub type WalletClose = extern fn(storage_handle: StorageHandle) -> ErrorCode;

    /// Delete the wallet storage (For example, database deletion)
    ///
    /// #Params
    /// name: wallet storage name (the same as wallet name)
    /// config: wallet storage config (For example, database config)
    /// credentials_json: wallet storage credentials (For example, database credentials)
    pub type WalletDelete = extern fn(name: *const c_char,
                                      config: *const c_char,
                                      credentials_json: *const c_char) -> ErrorCode;

    /// Create a new record in the wallet storage
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// value: the value of record (pointer to buffer)
    /// value_len: the value of record (buffer size)
    /// tags_json: the record tags used for search and storing meta information as json:
    ///   {
    ///     "tagName1": "tag value 1", // string value
    ///     "tagName2": 123, // numeric value
    ///   }
    ///   Note that null means no tags
    pub type WalletAddRecord = extern fn(storage_handle: StorageHandle,
                                         type_: *const c_char,
                                         id: *const c_char,
                                         value: *const u8,
                                         value_len: usize,
                                         tags_json: *const c_char) -> ErrorCode;

    /// Update a record value
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// value: the value of record (pointer to buffer)
    /// value_len: the value of record (buffer size)
    pub type WalletUpdateRecordValue = extern fn(storage_handle: StorageHandle,
                                                 type_: *const c_char,
                                                 id: *const c_char,
                                                 value: *const u8,
                                                 value_len: usize, ) -> ErrorCode;

    /// Update a record tags
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tags_json: the new record tags used for search and storing meta information as json:
    ///   {
    ///     "tagName1": "tag value 1", // string value
    ///     "tagName2": 123, // numeric value
    ///   }
    ///   Note that null means no tags
    pub type WalletUpdateRecordTags = extern fn(storage_handle: StorageHandle,
                                                type_: *const c_char,
                                                id: *const c_char,
                                                tags_json: *const c_char) -> ErrorCode;

    /// Add new tags to the record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tags_json: the additional record tags as json:
    ///   {
    ///     "tagName1": "tag value 1", // string value
    ///     "tagName2": 123, // numeric value,
    ///     ...
    ///   }
    ///   Note that null means no tags
    ///   Note if some from provided tags already assigned to the record than
    ///     corresponding tags values will be replaced
    pub type WalletAddRecordTags = extern fn(storage_handle: StorageHandle,
                                             type_: *const c_char,
                                             id: *const c_char,
                                             tags_json: *const c_char) -> ErrorCode;

    /// Delete tags from the record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// tag_names_json: the list of tag names to remove from the record as json array:
    ///   ["tagName1", "tagName2", ...]
    ///   Note that null means no tag names
    pub type WalletDeleteRecordTags = extern fn(storage_handle: StorageHandle,
                                                type_: *const c_char,
                                                id: *const c_char,
                                                tag_names_json: *const c_char) -> ErrorCode;

    /// Delete an existing record in the wallet storage
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: record type
    /// id: the id of record
    pub type WalletDeleteRecord = extern fn(storage_handle: StorageHandle,
                                            type_: *const c_char,
                                            id: *const c_char) -> ErrorCode;

    /// Get an wallet storage record by id
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// id: the id of record
    /// options_json: //TODO: FIXME: Think about replacing by bitmask
    ///  {
    ///    retrieveType: (optional, false by default) Retrieve record type,
    ///    retrieveValue: (optional, true by default) Retrieve record value,
    ///    retrieveTags: (optional, true by default) Retrieve record tags
    ///  }
    /// record_handle_p: pointer to store retrieved record handle
    pub type WalletGetRecord = extern fn(storage_handle: StorageHandle,
                                         type_: *const c_char,
                                         id: *const c_char,
                                         options_json: *const c_char,
                                         record_handle_p: *mut IndyHandle) -> ErrorCode;

    /// Get an id for retrieved wallet storage record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// record_handle: retrieved record handle (See get_record handler)
    ///
    /// returns: record id
    ///          Note that pointer lifetime the same as retrieved record lifetime
    ///            (until record_free called)
    pub type WalletGetRecordId = extern fn(storage_handle: StorageHandle,
                                           record_handle: IndyHandle,
                                           record_id_p: *mut *const c_char) -> ErrorCode;

    /// Get an type for retrieved wallet storage record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// record_handle: retrieved record handle (See get_record handler)
    ///
    /// returns: record type
    ///          Note that pointer lifetime the same as retrieved record lifetime
    ///            (until record_free called)
    pub type WalletGetRecordType = extern fn(storage_handle: StorageHandle,
                                             record_handle: IndyHandle,
                                             record_type_p: *mut *const c_char) -> ErrorCode;

    /// Get an value for retrieved wallet storage record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// record_handle: retrieved record handle (See get_record handler)
    ///
    /// returns: record value
    ///          Note that pointer lifetime the same as retrieved record lifetime
    ///            (until record_free called)
    ///          Note that null be returned if no value retrieved
    pub type WalletGetRecordValue = extern fn(storage_handle: StorageHandle,
                                              record_handle: IndyHandle,
                                              record_value_p: *mut *const u8,
                                              record_value_len_p: *mut usize) -> ErrorCode;

    /// Get an tags for retrieved wallet record
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// record_handle: retrieved record handle (See get_record handler)
    ///
    /// returns: record tags as json
    ///          Note that pointer lifetime the same as retrieved record lifetime
    ///            (until record_free called)
    ///          Note that null be returned if no tags retrieved
    pub type WalletGetRecordTags = extern fn(storage_handle: StorageHandle,
                                             record_handle: IndyHandle,
                                             record_tags_p: *mut *const c_char) -> ErrorCode;

    /// Free retrieved wallet record (make retrieved record handle invalid)
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open_wallet_storage)
    /// record_handle: retrieved record handle (See wallet_storage_get_wallet_record)
    pub type WalletFreeRecord = extern fn(storage_handle: StorageHandle,
                                          record_handle: IndyHandle) -> ErrorCode;

    /// Get storage metadata
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    ///
    /// returns: metadata as base64 value
    ///          Note that pointer lifetime is static
    pub type WalletGetStorageMetadata = extern fn(storage_handle: StorageHandle,
                                                  metadata_p: *mut *const c_char,
                                                  metadata_handle: *mut IndyHandle) -> ErrorCode;

    /// Set storage metadata
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// metadata_p: base64 value of metadata
    ///
    ///   Note if storage already have metadata record it will be overwritten.
    pub type WalletSetStorageMetadata = extern fn(storage_handle: StorageHandle,
                                                  metadata_p: *const c_char) -> ErrorCode;

    /// Free retrieved storage metadata record (make retrieved storage metadata handle invalid)
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open_wallet_storage)
    /// metadata_handle: retrieved record handle (See wallet_storage_get_storage_metadata)
    pub type WalletFreeStorageMetadata = extern fn(storage_handle: StorageHandle,
                                                   metadata_handle: IndyHandle) -> ErrorCode;

    /// Search for wallet storage records
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// type_: allows to separate different record types collections
    /// query_json: MongoDB style query to wallet record tags:
    ///  {
    ///    "tagName": "tagValue",
    ///    $or: {
    ///      "tagName2": { $regex: 'pattern' },
    ///      "tagName3": { $gte: 123 },
    ///    },
    ///  }
    /// options_json: //TODO: FIXME: Think about replacing by bitmask
    ///  {
    ///    retrieveRecords: (optional, true by default) If false only "counts" will be calculated,
    ///    retrieveTotalCount: (optional, false by default) Calculate total count,
    ///    retrieveType: (optional, false by default) Retrieve record type,
    ///    retrieveValue: (optional, true by default) Retrieve record value,
    ///    retrieveTags: (optional, true by default) Retrieve record tags,
    ///  }
    /// search_handle_p: pointer to store wallet search handle
    pub type WalletSearchRecords = extern fn(storage_handle: StorageHandle,
                                             type_: *const c_char,
                                             query_json: *const c_char,
                                             options_json: *const c_char,
                                             search_handle_p: *mut i32) -> ErrorCode;

    /// Search for all wallet storage records
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// search_handle_p: pointer to store wallet search handle
    pub type WalletSearchAllRecords = extern fn(storage_handle: StorageHandle,
                                                search_handle_p: *mut i32) -> ErrorCode;

    /// Get total count of records that corresponds to wallet storage search query
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// search_handle: wallet search handle (See search_records handler)
    ///
    /// returns: total count of records that corresponds to wallet storage search query
    ///          Note -1 will be returned if retrieveTotalCount set to false for search_records
    pub type WalletGetSearchTotalCount = extern fn(storage_handle: StorageHandle,
                                                   search_handle: i32,
                                                   total_count_p: *mut usize) -> ErrorCode;

    /// Get the next wallet storage record handle retrieved by this wallet search.
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// search_handle: wallet search handle (See search_records handler)
    ///
    /// returns: record handle (the same as for get_record handler)
    ///          Note if no more records WalletNoRecords error will be returned
    pub type WalletFetchSearchNextRecord = extern fn(storage_handle: StorageHandle,
                                                     search_handle: i32,
                                                     record_handle_p: *mut IndyHandle) -> ErrorCode;

    /// Free wallet search (make search handle invalid)
    ///
    /// #Params
    /// storage_handle: opened storage handle (See open handler)
    /// search_handle: wallet search handle (See search_records handler)
    pub type WalletFreeSearch = extern fn(storage_handle: StorageHandle,
                                          search_handle: i32) -> ErrorCode;
}