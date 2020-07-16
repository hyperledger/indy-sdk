use utils::version_constants;
use libc::c_char;
use utils::cstring::CStringUtils;
use utils::libindy::{wallet, pool, ledger};
use utils::error;
use settings;
use std::ffi::CString;
use utils::threadpool::spawn;
use error::prelude::*;
use indy::{INVALID_WALLET_HANDLE, CommandHandle};
use utils::libindy::pool::init_pool;
use messages::agent_utils;

/// Initializes VCX with config settings
///
/// example configuration is in libvcx/sample_config/config.json
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config: config as json.
/// The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_init_with_config(command_handle: CommandHandle,
                                   config: *const c_char,
                                   cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_init_with_config >>>");

    check_useful_c_str!(config,VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init(command_handle: {}, config: {:?})",
           command_handle, config);

    if config == "ENABLE_TEST_MODE" {
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_defaults();
    } else {
        match settings::process_config_string(&config, true) {
            Err(e) => {
                error!("Invalid configuration specified: {}", e);
                return e.into();
            }
            Ok(_) => (),
        }
    };

    _finish_init(command_handle, cb)
}

/// Initializes VCX with config file
///
/// An example file is at libvcx/sample_config/config.json
/// The list of available options see here: https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md
/// NOTE: If a webhook url is present in the config, an agent is expected to have been provisioned
///
/// #Params
/// command_handle: command handle to map callback to user context.
///
/// config_path: path to a config file to populate config attributes
///
/// cb: Callback that provides error status of initialization
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_init(command_handle: CommandHandle,
                       config_path: *const c_char,
                       cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_init >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_init(command_handle: {}, config_path: {:?})",
           command_handle, config_path);


    if !config_path.is_null() {
        check_useful_c_str!(config_path,VcxErrorKind::InvalidOption);

        if config_path == "ENABLE_TEST_MODE" {
            settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
            settings::set_defaults();
        } else {
            match settings::process_config_file(&config_path) {
                Err(_) => {
                    return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot initialize with given config path.").into();
                }
                Ok(_) => {
                    match settings::validate_payment_method() {
                        Ok(_) => (),
                        Err(e) => return e.into()
                    }
                }
            };
        }
    } else {
        error!("Cannot initialize with given config path: config path is null.");
        return VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Cannot initialize with given config path: config path is null.").into();
    }

    _finish_init(command_handle, cb)
}

fn _finish_init(command_handle: CommandHandle, cb: extern fn(xcommand_handle: CommandHandle, err: u32)) -> u32 {
    ::utils::threadpool::init();

    settings::log_settings();

    if wallet::get_wallet_handle() != INVALID_WALLET_HANDLE {
        error!("Library was already initialized");
        return VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Library was already initialized").into();
    }
    // Wallet name was already validated
    let wallet_name = match settings::get_config_value(settings::CONFIG_WALLET_NAME) {
        Ok(x) => x,
        Err(_) => {
            trace!("Using default wallet: {}", settings::DEFAULT_WALLET_NAME.to_string());
            settings::set_config_value(settings::CONFIG_WALLET_NAME, settings::DEFAULT_WALLET_NAME);
            settings::DEFAULT_WALLET_NAME.to_string()
        }
    };

    let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
    let storage_config = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG).ok();
    let storage_creds = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CREDS).ok();

    trace!("libvcx version: {}{}", version_constants::VERSION, version_constants::REVISION);

    spawn(move || {
        if settings::get_config_value(settings::CONFIG_GENESIS_PATH).is_ok() {
            match init_pool() {
                Ok(()) => (),
                Err(e) => {
                    error!("Init Pool Error: {}", e);
                    cb(command_handle, e.into());
                    return Ok(());
                }
            }
        }

        match wallet::open_wallet(&wallet_name, wallet_type.as_ref().map(String::as_str),
                                  storage_config.as_ref().map(String::as_str), storage_creds.as_ref().map(String::as_str)) {
            Ok(_) => debug!("Init Wallet Successful"),
            Err(e) => {
                error!("Init Wallet Error: {}", e);
                cb(command_handle, e.into());
            }
        };

        match agent_utils::create_pub_agent() {
            Ok(_) => {
                debug!("Create public agent successful");
                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(e) => {
                error!("Error creating public agent: {}", e);
                cb(command_handle, e.into());
                return Ok(());
            }
        }

        match settings::get_config_value(settings::CONFIG_WEBHOOK_URL) {
            Ok(webhook_url) => match ::messages::agent_utils::update_agent_webhook(&webhook_url) {
                Ok(()) => {
                    info!("Agent webhook url updated on init, webhook_url={}", webhook_url);
                    cb(command_handle, error::SUCCESS.code_num);
                }
                Err(e) => {
                    error!("Error updating agent webhook on init (did you provision an agent?): {}", e);
                    cb(command_handle, e.into());
                }
            }
            Err(e) => {
                debug!("webhook_url was not updated in agency: {}", e);
                cb(command_handle, error::SUCCESS.code_num);
            }
        }
        Ok(())
    });
    error::SUCCESS.code_num
}

/// Initialize vcx with the minimal configuration (wallet, pool must already be set with
/// vcx_wallet_set_handle() and vcx_pool_set_handle()) and without any agency configuration
///
/// # Example:
/// vcx_init_minimal -> '{"institution_name":"faber","institution_did":"44x8p4HubxzUK1dwxcc5FU",\
//      "institution_verkey":"444MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE"}'
///
/// #Params
///
/// config: minimal configuration
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_init_minimal(config: *const c_char) -> u32 {
    check_useful_c_str!(config,VcxErrorKind::InvalidOption);

    trace!("vcx_init_minimal(config: {:?})", config);

    if config == "ENABLE_TEST_MODE" {
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_defaults();
    } else {
        match settings::process_config_string(&config, false) {
            Err(e) => {
                error!("Invalid configuration specified: {}", e);
                return e.into();
            }
            Ok(_) => (),
        }
    };

    if wallet::get_wallet_handle() == INVALID_WALLET_HANDLE || pool::get_pool_handle().is_err() {
        error!("Library cannot be initialized without wallet/pool");
        return error::INVALID_STATE.code_num;
    }

    ::utils::threadpool::init();

    settings::log_settings();

    trace!("libvcx version: {}{}", version_constants::VERSION, version_constants::REVISION);

    error::SUCCESS.code_num
}

lazy_static! {
    pub static ref VERSION_STRING: CString = CString::new(format!("{}{}", version_constants::VERSION, version_constants::REVISION)).unwrap();
}

#[no_mangle]
pub extern fn vcx_version() -> *const c_char {
    info!("vcx_version >>>");
    VERSION_STRING.as_ptr()
}

/// Reset libvcx to a pre-configured state, releasing/deleting any handles and freeing memory
///
/// libvcx will be inoperable and must be initialized again with vcx_init_with_config
///
/// #Params
/// delete: specify whether wallet/pool should be deleted
///
/// #Returns
/// Success
#[no_mangle]
pub extern fn vcx_shutdown(delete: bool) -> u32 {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);

    match wallet::close_wallet() {
        Ok(()) => {}
        Err(_) => {}
    };

    match pool::close() {
        Ok(()) => {}
        Err(_) => {}
    };

    ::schema::release_all();
    ::connection::release_all();
    ::issuer_credential::release_all();
    ::credential_def::release_all();
    ::proof::release_all();
    ::disclosed_proof::release_all();
    ::credential::release_all();

    if delete {
        let pool_name = settings::get_config_value(settings::CONFIG_POOL_NAME)
            .unwrap_or(settings::DEFAULT_POOL_NAME.to_string());

        let wallet_name = settings::get_config_value(settings::CONFIG_WALLET_NAME)
            .unwrap_or(settings::DEFAULT_WALLET_NAME.to_string());

        let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();

        match wallet::delete_wallet(&wallet_name, wallet_type.as_ref().map(String::as_str), None, None) {
            Ok(()) => (),
            Err(_) => (),
        };

        match pool::delete(&pool_name) {
            Ok(()) => (),
            Err(_) => (),
        };
    }

    settings::clear_config();
    trace!("vcx_shutdown(delete: {})", delete);
    error::SUCCESS.code_num
}

/// Get the message corresponding to an error code
///
/// #Params
/// error_code: code of error
///
/// #Returns
/// Error message
#[no_mangle]
pub extern fn vcx_error_c_message(error_code: u32) -> *const c_char {
    info!("vcx_error_c_message >>>");
    trace!("vcx_error_message(error_code: {})", error_code);
    error::error_c_message(&error_code).as_ptr()
}

/// Update setting to set new local institution information
///
/// #Params
/// name: institution name
/// logo_url: url containing institution logo
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_update_institution_info(name: *const c_char, logo_url: *const c_char) -> u32 {
    info!("vcx_update_institution_info >>>");

    check_useful_c_str!(name, VcxErrorKind::InvalidConfiguration);
    check_useful_c_str!(logo_url, VcxErrorKind::InvalidConfiguration);
    trace!("vcx_update_institution_info(name: {}, logo_url: {})", name, logo_url);

    settings::set_config_value(::settings::CONFIG_INSTITUTION_NAME, &name);
    settings::set_config_value(::settings::CONFIG_INSTITUTION_LOGO_URL, &logo_url);

    error::SUCCESS.code_num
}

/// Update agency webhook url setting
///
/// #Params
///
/// command_handle: command handle to map callback to user context.
///
/// notification_webhook_url: URL to which the notifications should be sent
///
/// cb: Callback that provides error code of the result
///
/// #Returns
/// Error code as u32
#[no_mangle]
pub extern fn vcx_update_webhook_url(command_handle: CommandHandle,
                                     notification_webhook_url: *const c_char,
                                     cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32)>) -> u32 {
    info!("vcx_update_webhook {:?} >>>", notification_webhook_url);

    check_useful_c_str!(notification_webhook_url, VcxErrorKind::InvalidOption);
    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_update_webhook(webhook_url: {})", notification_webhook_url);

    settings::set_config_value(::settings::CONFIG_WEBHOOK_URL, &notification_webhook_url);

    spawn(move || {
        match ::messages::agent_utils::update_agent_webhook(&notification_webhook_url[..]) {
            Ok(()) => {
                trace!("vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                        command_handle, error::SUCCESS.message);

                cb(command_handle, error::SUCCESS.code_num);
            }
            Err(err) => {
                warn!("vcx_update_webhook_url_cb(command_handle: {}, rc: {})",
                        command_handle, err);

                cb(command_handle, err.into());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Retrieve author agreement and acceptance mechanisms set on the Ledger
///
/// #params
///
/// command_handle: command handle to map callback to user context.
///
/// cb: Callback that provides array of matching messages retrieved
///
/// # Example author_agreement -> "{"text":"Default agreement", "version":"1.0.0", "aml": {"label1": "description"}}"
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_get_ledger_author_agreement(command_handle: CommandHandle,
                                              cb: Option<extern fn(xcommand_handle: CommandHandle, err: u32, author_agreement: *const c_char)>) -> u32 {
    info!("vcx_get_ledger_author_agreement >>>");

    check_useful_c_callback!(cb, VcxErrorKind::InvalidOption);

    trace!("vcx_get_ledger_author_agreement(command_handle: {})",
           command_handle);

    spawn(move || {
        match ledger::libindy_get_txn_author_agreement() {
            Ok(x) => {
                trace!("vcx_ledger_get_fees_cb(command_handle: {}, rc: {}, author_agreement: {})",
                       command_handle, error::SUCCESS.message, x);

                let msg = CStringUtils::string_to_cstring(x);
                cb(command_handle, error::SUCCESS.code_num, msg.as_ptr());
            }
            Err(e) => {
                error!("vcx_get_ledger_author_agreement(command_handle: {}, rc: {})",
                       command_handle, e);
                cb(command_handle, e.into(), ::std::ptr::null_mut());
            }
        };

        Ok(())
    });

    error::SUCCESS.code_num
}

/// Set some accepted agreement as active.
///
/// As result of successful call of this function appropriate metadata will be appended to each write request.
///
/// #Params
/// text and version - (optional) raw data about TAA from ledger.
///     These parameters should be passed together.
///     These parameters are required if hash parameter is ommited.
/// hash - (optional) hash on text and version. This parameter is required if text and version parameters are ommited.
/// acc_mech_type - mechanism how user has accepted the TAA
/// time_of_acceptance - UTC timestamp when user has accepted the TAA
///
/// #Returns
/// Error code as a u32
#[no_mangle]
pub extern fn vcx_set_active_txn_author_agreement_meta(text: *const c_char,
                                                       version: *const c_char,
                                                       hash: *const c_char,
                                                       acc_mech_type: *const c_char,
                                                       time_of_acceptance: u64) -> u32 {
    info!("vcx_set_active_txn_author_agreement_meta >>>");

    check_useful_opt_c_str!(text, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(version, VcxErrorKind::InvalidOption);
    check_useful_opt_c_str!(hash, VcxErrorKind::InvalidOption);
    check_useful_c_str!(acc_mech_type, VcxErrorKind::InvalidOption);

    trace!("vcx_set_active_txn_author_agreement_meta(text: {:?}, version: {:?}, hash: {:?}, acc_mech_type: {:?}, time_of_acceptance: {:?})",
           text, version, hash, acc_mech_type, time_of_acceptance);

    match ::utils::author_agreement::set_txn_author_agreement(text, version, hash, acc_mech_type, time_of_acceptance) {
        Ok(()) => error::SUCCESS.code_num,
        Err(err) => err.into()
    }
}

#[no_mangle]
pub extern fn vcx_mint_tokens(seed: *const c_char, fees: *const c_char) {
    info!("vcx_mint_tokens >>>");

    // TODO: CHEC
    let seed = if !seed.is_null() {
        match CStringUtils::c_str_to_string(seed) {
            Ok(opt_val) => opt_val.map(String::from),
            Err(_) => return ()
        }
    } else {
        None
    };

    let fees = if !fees.is_null() {
        match CStringUtils::c_str_to_string(fees) {
            Ok(opt_val) => opt_val.map(String::from),
            Err(_) => return ()
        }
    } else {
        None
    };
    trace!("vcx_mint_tokens(seed: {:?}, fees: {:?})", seed, fees);

    ::utils::libindy::payments::mint_tokens_and_set_fees(None, None, fees, seed).unwrap_or_default();
}

/// Get details for last occurred error.
///
/// This function should be called in two places to handle both cases of error occurrence:
///     1) synchronous  - in the same application thread
///     2) asynchronous - inside of function callback
///
/// NOTE: Error is stored until the next one occurs in the same execution thread or until asynchronous callback finished.
///       Returning pointer has the same lifetime.
///
/// #Params
/// * `error_json_p` - Reference that will contain error details (if any error has occurred before)
///  in the format:
/// {
///     "backtrace": Optional<str> - error backtrace.
///         Collecting of backtrace can be enabled by setting environment variable `RUST_BACKTRACE=1`
///     "message": str - human-readable error description
/// }
///
#[no_mangle]
pub extern fn vcx_get_current_error(error_json_p: *mut *const c_char) {
    trace!("vcx_get_current_error >>> error_json_p: {:?}", error_json_p);

    let error = get_current_error_c_json();
    unsafe { *error_json_p = error };

    trace!("vcx_get_current_error: <<<");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use utils::{
        libindy::{
            wallet::{import, tests::export_test_wallet},
            pool::get_pool_handle,
        }
    };
    use api::VcxStateType;
    use api::return_types_u32;
    use indy::WalletHandle;
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use utils::get_temp_dir_path;
    use utils::devsetup::*;
    #[cfg(feature = "pool_tests")]
    use indy_sys::INVALID_POOL_HANDLE;
    #[cfg(feature = "pool_tests")]
    use utils::libindy::wallet::get_wallet_handle;
    #[cfg(feature = "pool_tests")]
    use utils::libindy::pool::tests::delete_test_pool;
    use utils::timeout::TimeoutUtils;

    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    fn config() -> String {
        json!({"agency_did" : "72x8p4HubxzUK1dwxcc5FU",
               "remote_to_sdk_did" : "UJGjM6Cea2YVixjWwHN9wq",
               "sdk_to_remote_did" : "AB3JM851T4EQmhh8CdagSP",
               "sdk_to_remote_verkey" : "888MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "institution_name" : "evernym enterprise",
               "agency_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "remote_to_sdk_verkey" : "91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
               "genesis_path": get_temp_dir_path("pool1.txn").to_str().unwrap(),
               "payment_method": "null",
               "pool_config": json!({"timeout":60}).to_string()
           }).to_string()
    }

    fn _vcx_init_c_closure(path: &str) -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "agency");
        let rc = vcx_init(cb.command_handle,
                          CString::new(path.to_string()).unwrap().into_raw(),
                          Some(cb.get_callback()));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    fn _vcx_init_with_config_c_closure(config: &str) -> Result<(), u32> {
        let cb = return_types_u32::Return_U32::new().unwrap();

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "agency");
        let rc = vcx_init_with_config(cb.command_handle,
                                      CString::new(config.to_string()).unwrap().into_raw(),
                                      Some(cb.get_callback()));
        if rc != error::SUCCESS.code_num {
            return Err(rc);
        }
        cb.receive(TimeoutUtils::some_medium())
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_with_file() {
        let _setup = SetupWalletAndPool::init();

        let config = TempFile::create_with_data("test_init.json", &config());

        _vcx_init_c_closure(&config.path).unwrap();

        // Assert wallet and pool was initialized
        assert_ne!(get_pool_handle().unwrap(), 0);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_with_file_no_payment_method() {
        let _setup = SetupEmpty::init();

        let config = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let config = TempFile::create_with_data("test_init.json", &config);

        let rc = _vcx_init_c_closure(&config.path).unwrap_err();
        assert_eq!(rc, error::MISSING_PAYMENT_METHOD.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_with_config() {
        let _setup = SetupWalletAndPool::init();

        _vcx_init_with_config_c_closure(&config()).unwrap();

        // Assert pool was initialized
        assert_ne!(get_pool_handle().unwrap(), 0);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_when_open_pool_fails() {
        let _setup = SetupWallet::init();

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(::utils::constants::GENESIS_PATH, "{}");

        let err = _vcx_init_with_config_c_closure(&config()).unwrap_err();
        assert_eq!(err, error::POOL_LEDGER_CONNECT.code_num);

        assert_eq!(get_pool_handle().unwrap_err().kind(), VcxErrorKind::NoPoolOpen);
        assert_eq!(get_wallet_handle(), INVALID_WALLET_HANDLE);

        delete_test_pool();
    }

    #[test]
    fn test_init_can_be_called_with_no_pool_config() {
        let _setup = SetupWallet::init();

        let content = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        // assert that pool was never initialized
        assert!(get_pool_handle().is_err());
    }

    #[test]
    fn test_init_fails_with_no_wallet_key() {
        let _setup = SetupEmpty::init();

        let content = json!({
            "wallet_name": settings::DEFAULT_WALLET_NAME,
        }).to_string();

        let rc = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(rc, error::MISSING_WALLET_KEY.code_num);
    }

    #[test]
    fn test_config_with_no_wallet_uses_default() {
        let _setup = SetupEmpty::init();

        assert!(settings::get_config_value(settings::CONFIG_WALLET_NAME).is_err());

        let content = json!({
            "wallet_key": "key"
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap_err();

        // Assert default wallet name
        assert_eq!(settings::get_config_value(settings::CONFIG_WALLET_NAME).unwrap(), settings::DEFAULT_WALLET_NAME);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_with_default_values() {
        let _setup = SetupWalletAndPool::init();

        _vcx_init_with_config_c_closure("{}").unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_fails() {
        let _setup = SetupWalletAndPool::init();

        _vcx_init_with_config_c_closure("{}").unwrap();

        // Repeat call
        let rc = _vcx_init_with_config_c_closure("{}").unwrap_err();
        assert_eq!(rc, error::ALREADY_INITIALIZED.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_vcx_init_called_twice_passes_after_shutdown() {
        for _ in 0..2 {
            let _setup = SetupDefaults::init();

            wallet::create_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
            pool::tests::create_test_pool();

            _vcx_init_with_config_c_closure("{}").unwrap();

            //Assert config values were set correctly
            assert_eq!(settings::get_config_value("wallet_name").unwrap(), settings::DEFAULT_WALLET_NAME);

            //Verify shutdown was successful
            vcx_shutdown(true);
            assert_eq!(settings::get_config_value("wallet_name").unwrap_err().kind(), VcxErrorKind::InvalidConfiguration);
        }
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_with_open_wallet() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let config = TempFile::create_with_data("test_init.json", &config());

        let rc = _vcx_init_c_closure(&config.path).unwrap_err();
        assert_eq!(rc, error::ALREADY_INITIALIZED.code_num);
    }

    #[test]
    fn test_init_after_importing_wallet_success() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: &wallet_name,
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::DEFAULT_WALLET_KEY_DERIVATION,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": &wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        vcx_shutdown(true);
    }

    #[test]
    fn test_init_with_imported_wallet_fails_with_different_params() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_WALLET_KEY_DERIVATION: settings::DEFAULT_WALLET_KEY_DERIVATION,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        import(&import_config).unwrap();

        let content = json!({
            "wallet_name": "different_wallet_name",
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let err = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(err, error::WALLET_NOT_FOUND.code_num);

        wallet::delete_wallet(&wallet_name, None, None, None).unwrap();
    }

    #[test]
    fn test_import_after_init_fails() {
        let _setup = SetupDefaults::init();

        let (export_wallet_path, wallet_name) = export_test_wallet();

        let content = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        _vcx_init_with_config_c_closure(&content).unwrap();

        let import_config = json!({
            settings::CONFIG_WALLET_NAME: wallet_name.as_str(),
            settings::CONFIG_WALLET_KEY: settings::DEFAULT_WALLET_KEY,
            settings::CONFIG_EXPORTED_WALLET_PATH: export_wallet_path.path,
            settings::CONFIG_WALLET_BACKUP_KEY: settings::DEFAULT_WALLET_BACKUP_KEY,
        }).to_string();
        assert_eq!(import(&import_config).unwrap_err().kind(), VcxErrorKind::DuplicationWallet);

        vcx_shutdown(true);
    }

    #[test]
    fn test_init_bad_path() {
        let _setup = SetupEmpty::init();

        let rc = _vcx_init_c_closure("").unwrap_err();
        assert_eq!(rc, error::INVALID_OPTION.code_num);
    }

    #[test]
    fn test_init_no_config_path() {
        let _setup = SetupEmpty::init();

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(vcx_init(cb.command_handle,
                            ptr::null(),
                            Some(cb.get_callback())),
                   error::INVALID_CONFIGURATION.code_num);
    }

    #[test]
    fn test_shutdown_with_no_previous_config() {
        let _setup = SetupDefaults::init();

        vcx_shutdown(true);
        vcx_shutdown(false);
    }

    #[test]
    fn test_shutdown() {
        let _setup = SetupMocks::init();

        let data = r#"["name","male"]"#;
        let connection = ::connection::tests::build_test_connection();
        let credentialdef = ::credential_def::create_and_publish_credentialdef("SID".to_string(), "NAME".to_string(), "4fUDR9R7fjwELRvH9JT6HH".to_string(), "id".to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let issuer_credential = ::issuer_credential::issuer_credential_create(credentialdef, "1".to_string(), "8XFh8yBzrpJQmNyZzgoTqB".to_owned(), "credential_name".to_string(), "{\"attr\":\"value\"}".to_owned(), 1).unwrap();
        let proof = ::proof::create_proof("1".to_string(), "[]".to_string(), "[]".to_string(), r#"{"support_revocation":false}"#.to_string(), "Optional".to_owned()).unwrap();
        let schema = ::schema::create_and_publish_schema("5", "VsKV7grR1BUE29mG2Fm2kX".to_string(), "name".to_string(), "0.1".to_string(), data.to_string()).unwrap();
        let disclosed_proof = ::disclosed_proof::create_proof("id", ::utils::constants::PROOF_REQUEST_JSON).unwrap();
        let credential = ::credential::credential_create_with_offer("name", ::utils::constants::CREDENTIAL_OFFER_JSON).unwrap();

        vcx_shutdown(true);
        assert_eq!(::connection::release(connection).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(::issuer_credential::release(issuer_credential).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(::schema::release(schema).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(::proof::release(proof).unwrap_err().kind(), VcxErrorKind::InvalidProofHandle);
        assert_eq!(::credential_def::release(credentialdef).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(::credential::release(credential).unwrap_err().kind(), VcxErrorKind::InvalidCredentialHandle);
        assert_eq!(::disclosed_proof::release(disclosed_proof).unwrap_err().kind(), VcxErrorKind::InvalidDisclosedProofHandle);
        assert_eq!(wallet::get_wallet_handle(), INVALID_WALLET_HANDLE);
    }

    #[test]
    fn test_error_c_message() {
        let _setup = SetupMocks::init();

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(0)).unwrap().unwrap();
        assert_eq!(c_message, error::SUCCESS.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1001)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(100100)).unwrap().unwrap();
        assert_eq!(c_message, error::UNKNOWN_ERROR.message);

        let c_message = CStringUtils::c_str_to_string(vcx_error_c_message(1021)).unwrap().unwrap();
        assert_eq!(c_message, error::INVALID_ATTRIBUTES_STRUCTURE.message);
    }

    #[test]
    fn test_vcx_version() {
        let _setup = SetupDefaults::init();

        let return_version = CStringUtils::c_str_to_string(vcx_version()).unwrap().unwrap();
        assert!(return_version.len() > 5);
    }

    #[test]
    fn test_vcx_update_institution_info() {
        let _setup = SetupDefaults::init();

        let new_name = "new_name";
        let new_url = "http://www.evernym.com";
        assert_ne!(new_name, &settings::get_config_value(::settings::CONFIG_INSTITUTION_NAME).unwrap());
        assert_ne!(new_url, &settings::get_config_value(::settings::CONFIG_INSTITUTION_LOGO_URL).unwrap());

        assert_eq!(error::SUCCESS.code_num, vcx_update_institution_info(CString::new(new_name.to_string()).unwrap().into_raw(),
                                                                        CString::new(new_url.to_string()).unwrap().into_raw()));

        assert_eq!(new_name, &settings::get_config_value(::settings::CONFIG_INSTITUTION_NAME).unwrap());
        assert_eq!(new_url, &settings::get_config_value(::settings::CONFIG_INSTITUTION_LOGO_URL).unwrap());
    }

    #[test]
    fn test_vcx_update_institution_webhook() {
        let _setup = SetupDefaults::init();

        let webhook_url = "http://www.evernym.com";
        assert_ne!(webhook_url, &settings::get_config_value(::settings::CONFIG_WEBHOOK_URL).unwrap());

        let cb = return_types_u32::Return_U32::new().unwrap();
        assert_eq!(error::SUCCESS.code_num, vcx_update_webhook_url(cb.command_handle,
                                                                   CString::new(webhook_url.to_string()).unwrap().into_raw(),
                                                                   Some(cb.get_callback())));

        assert_eq!(webhook_url, &settings::get_config_value(::settings::CONFIG_WEBHOOK_URL).unwrap());
    }

    #[test]
    fn get_current_error_works_for_no_error() {
        let _setup = SetupDefaults::init();

        ::error::reset_current_error();

        let mut error_json_p: *const c_char = ptr::null();

        vcx_get_current_error(&mut error_json_p);
        assert_eq!(None, CStringUtils::c_str_to_string(error_json_p).unwrap());
    }

    #[test]
    fn get_current_error_works_for_sync_error() {
        let _setup = SetupDefaults::init();

        ::api::utils::vcx_provision_agent(ptr::null());

        let mut error_json_p: *const c_char = ptr::null();
        vcx_get_current_error(&mut error_json_p);
        assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
    }

    #[test]
    fn get_current_error_works_for_async_error() {
        let _setup = SetupDefaults::init();

        extern fn cb(_storage_handle: i32,
                     _err: u32,
                     _config: *const c_char) {
            let mut error_json_p: *const c_char = ptr::null();
            vcx_get_current_error(&mut error_json_p);
            assert!(CStringUtils::c_str_to_string(error_json_p).unwrap().is_some());
        }

        let config = CString::new("{}").unwrap();
        ::api::utils::vcx_agent_provision_async(0, config.as_ptr(), Some(cb));
        ::std::thread::sleep(::std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_vcx_set_active_txn_author_agreement_meta() {
        let _setup = SetupMocks::init();

        assert!(&settings::get_config_value(::settings::CONFIG_TXN_AUTHOR_AGREEMENT).is_err());

        let text = "text";
        let version = "1.0.0";
        let acc_mech_type = "type 1";
        let time_of_acceptance = 123456789;

        assert_eq!(error::SUCCESS.code_num, vcx_set_active_txn_author_agreement_meta(CString::new(text.to_string()).unwrap().into_raw(),
                                                                                     CString::new(version.to_string()).unwrap().into_raw(),
                                                                                     ::std::ptr::null(),
                                                                                     CString::new(acc_mech_type.to_string()).unwrap().into_raw(),
                                                                                     time_of_acceptance));

        let expected = json!({
            "text": text,
            "version": version,
            "acceptanceMechanismType": acc_mech_type,
            "timeOfAcceptance": time_of_acceptance,
        });

        let auth_agreement = settings::get_config_value(::settings::CONFIG_TXN_AUTHOR_AGREEMENT).unwrap();
        let auth_agreement = ::serde_json::from_str::<::serde_json::Value>(&auth_agreement).unwrap();

        assert_eq!(expected, auth_agreement);

        ::settings::set_defaults();
    }

    #[test]
    fn test_vcx_get_ledger_author_agreement() {
        let _setup = SetupMocks::init();

        let cb = return_types_u32::Return_U32_STR::new().unwrap();
        assert_eq!(vcx_get_ledger_author_agreement(cb.command_handle,
                                                   Some(cb.get_callback())), error::SUCCESS.code_num);
        let agreement = cb.receive(TimeoutUtils::some_short()).unwrap();
        assert_eq!(::utils::constants::DEFAULT_AUTHOR_AGREEMENT, agreement.unwrap());
    }

    #[cfg(feature = "pool_tests")]
    fn get_settings() -> String {
        json!({
            settings::CONFIG_AGENCY_DID:           settings::get_config_value(settings::CONFIG_AGENCY_DID).unwrap(),
            settings::CONFIG_AGENCY_VERKEY:        settings::get_config_value(settings::CONFIG_AGENCY_VERKEY).unwrap(),
            settings::CONFIG_AGENCY_ENDPOINT:      settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT).unwrap(),
            settings::CONFIG_REMOTE_TO_SDK_DID:    settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID).unwrap(),
            settings::CONFIG_REMOTE_TO_SDK_VERKEY: settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY).unwrap(),
            settings::CONFIG_SDK_TO_REMOTE_DID:    settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_DID).unwrap(),
            settings::CONFIG_SDK_TO_REMOTE_VERKEY: settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY).unwrap(),
            settings::CONFIG_INSTITUTION_NAME:     settings::get_config_value(settings::CONFIG_INSTITUTION_NAME).unwrap(),
            settings::CONFIG_INSTITUTION_DID:      settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap(),
            settings::CONFIG_INSTITUTION_LOGO_URL: settings::get_config_value(settings::CONFIG_INSTITUTION_LOGO_URL).unwrap(),
            settings::CONFIG_PAYMENT_METHOD:       settings::get_config_value(settings::CONFIG_PAYMENT_METHOD).unwrap()
        }).to_string()
    }

    fn _vcx_init_minimal_c_closure(content: &str) -> u32 {
        vcx_init_minimal(CString::new(content).unwrap().into_raw())
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_minimal() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let config = get_settings();

        settings::clear_config();

        // Store settings and handles
        let wallet_handle = ::utils::libindy::wallet::get_wallet_handle();
        let pool_handle = ::utils::libindy::pool::get_pool_handle().unwrap();
        assert_ne!(wallet_handle, INVALID_WALLET_HANDLE);
        assert_ne!(pool_handle, INVALID_POOL_HANDLE);

        // Reset handles to 0
        assert_eq!(::api::utils::vcx_pool_set_handle(INVALID_POOL_HANDLE), INVALID_POOL_HANDLE);
        assert_eq!(::api::wallet::vcx_wallet_set_handle(INVALID_WALLET_HANDLE), INVALID_WALLET_HANDLE);

        // Test for errors when handles not set
        assert_ne!(error::SUCCESS.code_num, _vcx_init_minimal_c_closure(&config));
        ::api::wallet::vcx_wallet_set_handle(wallet_handle);

        assert_ne!(error::SUCCESS.code_num, _vcx_init_minimal_c_closure(&config));
        ::api::utils::vcx_pool_set_handle(pool_handle);

        // NOTE: handles are set independently, test config with no wallet or pool
        assert_eq!(error::SUCCESS.code_num, _vcx_init_minimal_c_closure(&config));

        // test that wallet and pool are operational
        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        settings::set_defaults();
    }

    #[test]
    fn test_no_agency_config() {
        let _setup = SetupMocks::init();

        let config = json!({
            "institution_name": "faber",
            "institution_did": "44x8p4HubxzUK1dwxcc5FU",
            "institution_verkey": "444MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE"
        }).to_string();

        ::api::wallet::vcx_wallet_set_handle(WalletHandle(1));
        ::api::utils::vcx_pool_set_handle(1);

        let init_res = _vcx_init_minimal_c_closure(&config);
        assert_eq!(init_res, error::SUCCESS.code_num);

        let cred_handle = ::issuer_credential::from_string(&::api::issuer_credential::tests::issuer_credential_state_accepted()).unwrap();
        let connection_handle = ::connection::from_string(::utils::constants::DEFAULT_CONNECTION).unwrap();
        let my_pw_did = ::connection::get_pw_did(connection_handle).unwrap();
        let their_pw_did = ::connection::get_their_pw_did(connection_handle).unwrap();

        let (offer, _) = ::issuer_credential::generate_credential_offer_msg(cred_handle).unwrap();
        let mycred = ::credential::credential_create_with_offer("test1", &offer).unwrap();
        let request = ::credential::generate_credential_request_msg(mycred, &my_pw_did, &their_pw_did).unwrap();
        ::issuer_credential::update_state(cred_handle, Some(request)).unwrap();
        let cred = ::issuer_credential::generate_credential_msg(cred_handle, &my_pw_did).unwrap();
        ::credential::update_state(mycred, Some(cred)).unwrap();
        assert_eq!(::credential::get_state(mycred).unwrap(), VcxStateType::VcxStateAccepted as u32);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_minimal_with_invalid_agency_config() {
        let _setup = SetupLibraryWalletPool::init();

        let config = json!({
            "institution_name": "faber",
            "institution_did": "44x8p4HubxzUK1dwxcc5FU",
            "institution_verkey": "444MFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE"
        }).to_string();

        ::api::wallet::vcx_wallet_set_handle(get_wallet_handle());
        ::api::utils::vcx_pool_set_handle(get_pool_handle().unwrap());

        settings::clear_config();

        assert_eq!(_vcx_init_minimal_c_closure(&config), error::SUCCESS.code_num);

        let connection_handle = ::connection::create_connection("test_create_fails").unwrap();
        ::connection::connect(connection_handle, None).unwrap_err();

        settings::set_defaults();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_init_fails_with_not_found_pool_genesis_file() {
        let _setup = SetupWallet::init();

        let content = json!({
            "genesis_path": "invalid/txn/path",
            "wallet_name": settings::DEFAULT_WALLET_NAME,
            "wallet_key": settings::DEFAULT_WALLET_KEY,
            "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        }).to_string();

        let rc = _vcx_init_with_config_c_closure(&content).unwrap_err();
        assert_eq!(rc, error::INVALID_GENESIS_TXN_PATH.code_num);
    }
}
