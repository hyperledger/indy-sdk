
use indy_api_types::{ErrorCode, CommandHandle, WalletHandle, INVALID_WALLET_HANDLE};
use crate::commands::{Command, CommandExecutor};
use crate::commands::wallet::WalletCommand;
use indy_api_types::domain::wallet::{Config, Credentials, ExportConfig, KeyConfig};
use indy_api_types::wallet::*;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;
use indy_api_types::validation::Validatable;

use serde_json;
use libc::c_char;
use std::rc::Rc;
use crate::services::metrics::MetricsService;
use crate::utils::time::get_cur_time;
use crate::services::metrics::command_metrics::CommandMetric;


/// Register custom wallet storage implementation.
///
/// #Params
/// command_handle: Command handle to map callback to caller context.
/// type_: Storage type name.
/// create: WalletType create operation handler
/// open: WalletType open operation handler
/// close: Wallet close operation handler
/// delete: WalletType delete operation handler
/// add_record: WalletType add record operation handler
/// update_record_value: WalletType update record value operation handler
/// update_record_tags: WalletType update record tags operation handler
/// add_record_tags: WalletType add record tags operation handler
/// delete_record_tags: WalletType delete record tags operation handler
/// delete_record: WalletType delete record operation handler
/// get_record: WalletType get record operation handler
/// get_record_id: WalletType get record id operation handler
/// get_record_type: WalletType get record type operation handler
/// get_record_value: WalletType get record value operation handler
/// get_record_tags: WalletType get record tags operation handler
/// free_record: WalletType free record operation handler
/// search_records: WalletType search records operation handler
/// search_all_records: WalletType search all records operation handler
/// get_search_total_count: WalletType get search total count operation handler
/// fetch_search_next_record: WalletType fetch search next record operation handler
/// free_search: WalletType free search operation handler
/// free: Handler that allows to de-allocate strings allocated in caller code
///
/// #Returns
/// Error code
#[no_mangle]
pub extern fn indy_register_wallet_storage(command_handle: CommandHandle,
                                           type_: *const c_char,
                                           create: Option<WalletCreate>,
                                           open: Option<WalletOpen>,
                                           close: Option<WalletClose>,
                                           delete: Option<WalletDelete>,
                                           add_record: Option<WalletAddRecord>,
                                           update_record_value: Option<WalletUpdateRecordValue>,
                                           update_record_tags: Option<WalletUpdateRecordTags>,
                                           add_record_tags: Option<WalletAddRecordTags>,
                                           delete_record_tags: Option<WalletDeleteRecordTags>,
                                           delete_record: Option<WalletDeleteRecord>,
                                           get_record: Option<WalletGetRecord>,
                                           get_record_id: Option<WalletGetRecordId>,
                                           get_record_type: Option<WalletGetRecordType>,
                                           get_record_value: Option<WalletGetRecordValue>,
                                           get_record_tags: Option<WalletGetRecordTags>,
                                           free_record: Option<WalletFreeRecord>,
                                           get_storage_metadata: Option<WalletGetStorageMetadata>,
                                           set_storage_metadata: Option<WalletSetStorageMetadata>,
                                           free_storage_metadata: Option<WalletFreeStorageMetadata>,
                                           search_records: Option<WalletSearchRecords>,
                                           search_all_records: Option<WalletSearchAllRecords>,
                                           get_search_total_count: Option<WalletGetSearchTotalCount>,
                                           fetch_search_next_record: Option<WalletFetchSearchNextRecord>,
                                           free_search: Option<WalletFreeSearch>,
                                           cb: Option<extern fn(command_handle_: CommandHandle,
                                                                err: ErrorCode)>) -> ErrorCode {
    trace!("indy_register_wallet_type: >>> command_handle: {:?}, type_: {:?}, cb: {:?}",
           command_handle, type_, cb); // TODO: Log all params

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_callback!(create, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(open, ErrorCode::CommonInvalidParam4);
    check_useful_c_callback!(close, ErrorCode::CommonInvalidParam5);
    check_useful_c_callback!(delete, ErrorCode::CommonInvalidParam6);
    check_useful_c_callback!(add_record, ErrorCode::CommonInvalidParam7);
    check_useful_c_callback!(update_record_value, ErrorCode::CommonInvalidParam8);
    check_useful_c_callback!(update_record_tags, ErrorCode::CommonInvalidParam9);
    check_useful_c_callback!(add_record_tags, ErrorCode::CommonInvalidParam10);
    check_useful_c_callback!(delete_record_tags, ErrorCode::CommonInvalidParam11);
    check_useful_c_callback!(delete_record, ErrorCode::CommonInvalidParam12);
    check_useful_c_callback!(get_record, ErrorCode::CommonInvalidParam13);
    check_useful_c_callback!(get_record_id, ErrorCode::CommonInvalidParam14);
    check_useful_c_callback!(get_record_type, ErrorCode::CommonInvalidParam15);
    check_useful_c_callback!(get_record_value, ErrorCode::CommonInvalidParam16);
    check_useful_c_callback!(get_record_tags, ErrorCode::CommonInvalidParam17);
    check_useful_c_callback!(free_record, ErrorCode::CommonInvalidParam18);
    check_useful_c_callback!(get_storage_metadata, ErrorCode::CommonInvalidParam19);
    check_useful_c_callback!(set_storage_metadata, ErrorCode::CommonInvalidParam20);
    check_useful_c_callback!(free_storage_metadata, ErrorCode::CommonInvalidParam21);
    check_useful_c_callback!(search_records, ErrorCode::CommonInvalidParam22);
    check_useful_c_callback!(search_all_records, ErrorCode::CommonInvalidParam23);
    check_useful_c_callback!(get_search_total_count, ErrorCode::CommonInvalidParam24);
    check_useful_c_callback!(fetch_search_next_record, ErrorCode::CommonInvalidParam25);
    check_useful_c_callback!(free_search, ErrorCode::CommonInvalidParam26);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam27);

    trace!("indy_register_wallet_type: params type_: {:?}", type_);

    let result = CommandExecutor::instance()
        .send(Command::Wallet(
            WalletCommand::RegisterWalletType(
                type_,
                create,
                open,
                close,
                delete,
                add_record,
                update_record_value,
                update_record_tags,
                add_record_tags,
                delete_record_tags,
                delete_record,
                get_record,
                get_record_id,
                get_record_type,
                get_record_value,
                get_record_tags,
                free_record,
                get_storage_metadata,
                set_storage_metadata,
                free_storage_metadata,
                search_records,
                search_all_records,
                get_search_total_count,
                fetch_search_next_record,
                free_search,
                Box::new(move |result, metrics_service: Rc<MetricsService>| {
                    let err = prepare_result!(result);
                    trace!("indy_register_wallet_type: cb command_handle: {:?}, err: {:?}", command_handle, err);
                    let start_execution_ts = get_cur_time();
                    let result = cb(command_handle, err);
                    metrics_service.cmd_callback(CommandMetric::WalletCommandRegisterWalletType, get_cur_time() - start_execution_ts);

                    result
                })
            )));

    let res = prepare_result!(result);
    trace!("indy_register_wallet_type: <<< res: {:?}", res);
    res
}

/// Create a new secure wallet.
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                          ARGON2I_MOD - derive secured wallet master key (used by default)
///                          ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                          RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
///
/// #Returns
/// err: Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_create_wallet(command_handle: CommandHandle,
                                 config: *const c_char,
                                 credentials: *const c_char,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode)>) -> ErrorCode {
    trace!("indy_create_wallet: >>> command_handle: {:?}, config: {:?}, credentials: {:?}, cb: {:?}",
           command_handle, config, credentials, cb);

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_create_wallet: params config: {:?}, credentials: {:?}",
           config, secret!(&credentials));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Create(
            config,
            credentials,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_create_wallet: cb command_handle: {:?}, err: {:?}", command_handle, err);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::WalletCommandCreate,get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_create_wallet: <<< res: {:?}", res);
    res
}

/// Open the wallet.
///
/// Wallet must be previously created with indy_create_wallet method.
///
/// #Params
/// config: Wallet configuration json.
///   {
///       "id": string, Identifier of the wallet.
///             Configured storage uses this identifier to lookup exact wallet data placement.
///       "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                       'Default' storage type allows to store wallet data in the local file.
///                       Custom storage types can be registered with indy_register_wallet_storage call.
///       "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                         Can be optional if storage supports default configuration.
///                         For 'default' storage type configuration is:
///           {
///              "path": optional<string>, Path to the directory with wallet files.
///                      Defaults to $HOME/.indy_client/wallet.
///                      Wallet will be stored in the file {path}/{id}/sqlite.db
///           }
///
///   }
/// credentials: Wallet credentials json
///   {
///       "key": string, Key or passphrase used for wallet key derivation.
///                      Look to key_derivation_method param for information about supported key derivation methods.
///       "rekey": optional<string>, If present than wallet master key will be rotated to a new one.
///       "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                              Can be optional if storage supports default configuration.
///                              For 'default' storage type should be empty.
///       "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                          ARGON2I_MOD - derive secured wallet master key (used by default)
///                          ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                          RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///       "rekey_derivation_method": optional<string> Algorithm to use for wallet rekey derivation:
///                          ARGON2I_MOD - derive secured wallet master rekey (used by default)
///                          ARGON2I_INT - derive secured wallet master rekey (less secured but faster)
///                          RAW - raw wallet rekey master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///   }
///
/// #Returns
/// err: Error code
/// handle: Handle to opened wallet to use in methods that require wallet access.
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_open_wallet(command_handle: CommandHandle,
                               config: *const c_char,
                               credentials: *const c_char,
                               cb: Option<extern fn(command_handle_: CommandHandle,
                                                    err: ErrorCode,
                                                    wallet_handle: WalletHandle)>) -> ErrorCode {
    trace!("indy_open_wallet: >>> command_handle: {:?}, config: {:?}, credentials: {:?}, cb: {:?}",
           command_handle, config, credentials, cb);

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_open_wallet: params config: {:?}, credentials: {:?}",
           config, secret!(&credentials));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Open(
            config,
            credentials,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, handle) = prepare_result_1!(result, INVALID_WALLET_HANDLE);
                trace!("indy_open_wallet: cb command_handle: {:?} err: {:?}, handle: {:?}",
                       command_handle, err, handle);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, handle);
                metrics_service.cmd_callback(CommandMetric::WalletCommandOpen, get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_open_wallet: <<< res: {:?}", res);
    res
}

/// Exports opened wallet
///
/// #Params:
/// wallet_handle: wallet handle returned by indy_open_wallet
/// export_config: JSON containing settings for input operation.
///   {
///     "path": <string>, Path of the file that contains exported wallet content
///     "key": <string>, Key or passphrase used for wallet export key derivation.
///                     Look to key_derivation_method param for information about supported key derivation methods.
///     "key_derivation_method": optional<string> Algorithm to use for wallet export key derivation:
///                              ARGON2I_MOD - derive secured export key (used by default)
///                              ARGON2I_INT - derive secured export key (less secured but faster)
///                              RAW - raw export key provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
///   }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_export_wallet(command_handle: CommandHandle,
                                 wallet_handle: WalletHandle,
                                 export_config: *const c_char,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode)>) -> ErrorCode {
    trace!("indy_export_wallet: >>> wallet_handle: {:?}, export_config: {:?}", wallet_handle, export_config);

    check_useful_json!(export_config, ErrorCode::CommonInvalidParam3, ExportConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_export_wallet: params wallet_handle: {:?}, export_config: {:?}", wallet_handle, secret!(&export_config));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Export(
            wallet_handle,
            export_config,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_export_wallet: cb command_handle: {:?} err: {:?}", command_handle, err);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::WalletCommandExport, get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_export_wallet: <<< res: {:?}", res);
    res
}


/// Creates a new secure wallet and then imports its content
/// according to fields provided in import_config
/// This can be seen as an indy_create_wallet call with additional content import
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                             ARGON2I_MOD - derive secured wallet master key (used by default)
///                             ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                             RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
/// import_config: Import settings json.
/// {
///   "path": <string>, path of the file that contains exported wallet content
///   "key": <string>, key used for export of the wallet
/// }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_import_wallet(command_handle: CommandHandle,
                                 config: *const c_char,
                                 credentials: *const c_char,
                                 import_config: *const c_char,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode)>) -> ErrorCode {
    trace!("indy_import_wallet: >>> command_handle: {:?}, config: {:?}, credentials: {:?}, import_config: {:?}, cb: {:?}",
           command_handle, config, credentials, import_config, cb);

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_json!(import_config, ErrorCode::CommonInvalidParam4, ExportConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam5);

    trace!("indy_import_wallet: params config: {:?}, credentials: {:?}, import_config: {:?}",
           config, secret!(&credentials), secret!(&import_config));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Import(
            config,
            credentials,
            import_config,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_import_wallet: cb command_handle: {:?}, err: {:?}", command_handle, err);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::WalletCommandImport, get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_import_wallet: <<< res: {:?}", res);
    res
}


/// Closes opened wallet and frees allocated resources.
///
/// #Params
/// wallet_handle: wallet handle returned by indy_open_wallet.
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_close_wallet(command_handle: CommandHandle,
                                wallet_handle: WalletHandle,
                                cb: Option<extern fn(command_handle_: CommandHandle,
                                                     err: ErrorCode)>) -> ErrorCode {
    trace!("indy_close_wallet: >>> command_handle: {:?}, wallet_handle: {:?}, cb: {:?}",
           command_handle, wallet_handle, cb);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_close_wallet: params wallet_handle: {:?}", wallet_handle);

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Close(
            wallet_handle,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_close_wallet: cb command_handle: {:?}, err: {:?}", command_handle, err);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::WalletCommandClose, get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_close_wallet: <<< res: {:?}", res);
    res
}

/// Deletes created wallet.
///
/// #Params
/// config: Wallet configuration json.
/// {
///   "id": string, Identifier of the wallet.
///         Configured storage uses this identifier to lookup exact wallet data placement.
///   "storage_type": optional<string>, Type of the wallet storage. Defaults to 'default'.
///                  'Default' storage type allows to store wallet data in the local file.
///                  Custom storage types can be registered with indy_register_wallet_storage call.
///   "storage_config": optional<object>, Storage configuration json. Storage type defines set of supported keys.
///                     Can be optional if storage supports default configuration.
///                     For 'default' storage type configuration is:
///   {
///     "path": optional<string>, Path to the directory with wallet files.
///             Defaults to $HOME/.indy_client/wallet.
///             Wallet will be stored in the file {path}/{id}/sqlite.db
///   }
/// }
/// credentials: Wallet credentials json
/// {
///   "key": string, Key or passphrase used for wallet key derivation.
///                  Look to key_derivation_method param for information about supported key derivation methods.
///   "storage_credentials": optional<object> Credentials for wallet storage. Storage type defines set of supported keys.
///                          Can be optional if storage supports default configuration.
///                          For 'default' storage type should be empty.
///   "key_derivation_method": optional<string> Algorithm to use for wallet key derivation:
///                             ARGON2I_MOD - derive secured wallet master key (used by default)
///                             ARGON2I_INT - derive secured wallet master key (less secured but faster)
///                             RAW - raw wallet key master provided (skip derivation).
///                                RAW keys can be generated with indy_generate_wallet_key call
/// }
///
/// #Returns
/// Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_delete_wallet(command_handle: CommandHandle,
                                 config: *const c_char,
                                 credentials: *const c_char,
                                 cb: Option<extern fn(command_handle_: CommandHandle,
                                                      err: ErrorCode)>) -> ErrorCode {
    trace!("indy_delete_wallet: >>> command_handle: {:?}, config: {:?}, credentials: {:?}, cb: {:?}",
           command_handle, config, credentials, cb);

    check_useful_validatable_json!(config, ErrorCode::CommonInvalidParam2, Config);
    check_useful_json!(credentials, ErrorCode::CommonInvalidParam3, Credentials);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_delete_wallet: params config: {:?}, credentials: {:?}", config, secret!(&credentials));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::Delete(
            config,
            credentials,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let err = prepare_result!(result);
                trace!("indy_delete_wallet: cb command_handle: {:?}, err: {:?}", command_handle, err);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err);
                metrics_service.cmd_callback(CommandMetric::WalletCommandDelete, get_cur_time() - start_execution_ts);

                result
            })
        )));

    let res = prepare_result!(result);
    trace!("indy_delete_wallet: <<< res: {:?}", res);
    res
}

/// Generate wallet master key.
/// Returned key is compatible with "RAW" key derivation method.
/// It allows to avoid expensive key derivation for use cases when wallet keys can be stored in a secure enclave.
///
/// #Params
/// config: (optional) key configuration json.
/// {
///   "seed": string, (optional) Seed that allows deterministic key creation (if not set random one will be created).
///                              Can be UTF-8, base64 or hex string.
/// }
///
/// #Returns
/// err: Error code
///
/// #Errors
/// Common*
/// Wallet*
#[no_mangle]
pub extern fn indy_generate_wallet_key(command_handle: CommandHandle,
                                       config: *const c_char,
                                       cb: Option<extern fn(command_handle_: CommandHandle,
                                                            err: ErrorCode,
                                                            key: *const c_char)>) -> ErrorCode {
    trace!("indy_generate_wallet_key: >>> command_handle: {:?}, config: {:?}, cb: {:?}",
           command_handle, config, cb);

    check_useful_opt_json!(config, ErrorCode::CommonInvalidParam2, KeyConfig);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);

    trace!("indy_generate_wallet_key: params config: {:?}", secret!(config.as_ref()));

    let result = CommandExecutor::instance()
        .send(Command::Wallet(WalletCommand::GenerateKey(
            config,
            boxed_callback_string!("indy_generate_wallet_key", cb, command_handle, CommandMetric::WalletCommandGenerateKey)
        )));

    let res = prepare_result!(result);
    trace!("indy_generate_wallet_key: <<< res: {:?}", res);
    res
}
