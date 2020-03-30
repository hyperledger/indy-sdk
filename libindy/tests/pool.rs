#[macro_use]
mod utils;

inject_indy_dependencies!();

extern crate indyrs as indy;
extern crate indyrs as api;

use self::indy::ErrorCode;

use crate::utils::{environment, pool};
#[cfg(not(feature="only_high_cases"))]
use crate::utils::constants::*;
use crate::utils::Setup;
use crate::utils::types::ResponseType;

mod high_cases {
    use super::*;

    mod create {
        use super::*;
        use std::fs;

        #[test]
        fn create_pool_ledger_config_works() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());

            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();
        }

        #[test]
        fn create_pool_ledger_config_works_for_specific_config() {
            let setup = Setup::empty();

            let txn_file_path = environment::tmp_file_path("specific_filename.txn");
            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, Some(txn_file_path.as_path()));
            let pool_config = pool::pool_config_json(txn_file_path.as_path());

            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let _ = fs::remove_file(txn_file_path);
        }
    }

    mod open {
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            pool::open_pool_ledger(&setup.name, None).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_config() {
            let setup = Setup::empty();

            let config = r#"{"timeout": 20}"#;

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            pool::open_pool_ledger(&setup.name, Some(config)).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_two_nodes() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, Some(2), None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            pool::open_pool_ledger(&setup.name, None).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_three_nodes() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, Some(3), None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            pool::open_pool_ledger(&setup.name, None).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        pub fn open_pool_ledger_works_for_cached_txns() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();
            pool::dump_correct_genesis_txns_to_cache(&setup.name).unwrap();

            pool::open_pool_ledger(&setup.name, None).unwrap();
        }
    }

    mod refresh {
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_refresh_pool_ledger_works() {
            let setup = Setup::pool();
            pool::refresh(setup.pool_handle).unwrap();
        }
    }

    mod close {
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_close_pool_ledger_works() {
            Setup::pool();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_close_pool_ledger_works_for_reopen_after_close() {
            let mut setup = Setup::pool();

            pool::close(setup.pool_handle).unwrap();
            setup.pool_handle = pool::open_pool_ledger(&setup.name, None).unwrap();
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn indy_delete_pool_ledger_config_works() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            pool::delete(&setup.name).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_delete_pool_ledger_config_works_for_opened() {
            let setup = Setup::pool();

            let res = pool::delete(&setup.name);
            assert_code!(ErrorCode::CommonInvalidState, res);
        }
    }

    mod set_protocol_version {
        use super::*;

        #[test]
        fn indy_set_protocol_version_works() {
            pool::set_protocol_version(1).unwrap();
            pool::set_protocol_version(2).unwrap();
        }
    }
}

#[cfg(not(feature = "only_high_cases"))]
mod medium_cases {
    use super::*;
    use crate::utils::ledger;

    mod create {
        use super::*;

        #[test]
        fn create_pool_ledger_config_works_for_empty_name() {
            Setup::empty();

            let pool_name = "";
            let res = pool::create_pool_ledger_config(pool_name, None);
            assert_code!(ErrorCode::CommonInvalidParam2, res);
        }

        #[test]
        fn create_pool_ledger_config_works_for_empty_genesis_txns() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, Some(0), None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            let res = pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str()));
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn create_pool_ledger_config_works_for_invalid_config_json() {
            let setup = Setup::empty();

            let res = pool::create_pool_ledger_config(&setup.name, Some(r#"{}"#));
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        fn create_pool_ledger_config_works_for_invalid_genesis_txn_path() {
            let setup = Setup::empty();

            let config = r#"{"genesis_txn": "path"}"#.to_string();

            let res = pool::create_pool_ledger_config(&setup.name, Some(config.as_str()));
            assert_code!(ErrorCode::CommonIOError, res);
        }

        #[test]
        fn create_pool_ledger_config_works_for_twice() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());

            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();
            let res = pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str()));
            assert_code!(ErrorCode::PoolLedgerConfigAlreadyExistsError, res);
        }

        #[test]
        fn create_pool_ledger_config_works_for_empty_lines_in_genesis_txn_file() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_empty_lines(&setup.name, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();
        }
    }

    mod open {
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_twice() {
            let setup = Setup::empty();

            pool::create_and_open_pool_ledger(&setup.name).unwrap();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::PoolLedgerInvalidPoolHandle, res);
        }

        #[test]
        pub fn open_pool_ledger_works_for_corrupted_cached_txns() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();
            pool::dump_incorrect_genesis_txns_to_cache(&setup.name).unwrap();

            pool::open_pool_ledger(&setup.name, None).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_invalid_name() {
            let setup = Setup::empty();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::PoolLedgerNotCreatedError, res);
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_after_error() {
            let setup = Setup::empty();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::PoolLedgerNotCreatedError, res);

            let pool_handle = pool::create_and_open_pool_ledger(&setup.name).unwrap();

            pool::close(pool_handle).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_invalid_nodes_file() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool_with_invalid_nodes(&setup.name, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let res = pool::open_pool_ledger(&setup.name, Some(pool_config.as_str()));
            assert_code!(ErrorCode::CommonInvalidStructure, res); // TODO: changed error
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_wrong_alias() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool_with_wrong_alias(&setup.name, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::CommonInvalidStructure, res); // TODO: changed error
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_invalid_config() {
            let setup = Setup::empty();

            let config = r#"{"timeout": "true"}"#;

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let res = pool::open_pool_ledger(&setup.name, Some(config));
            assert_code!(ErrorCode::CommonInvalidStructure, res);
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_incompatible_protocol_version() {
            let setup = Setup::empty();

            pool::set_protocol_version(1).unwrap();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::PoolIncompatibleProtocolVersion, res);

            pool::set_protocol_version(PROTOCOL_VERSION).unwrap();

        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_wrong_ips() {
            let setup = Setup::empty();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool_with_wrong_ips(&setup.name, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let res = pool::open_pool_ledger(&setup.name, None);
            assert_code!(ErrorCode::PoolLedgerTimeout, res);
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn open_pool_ledger_works_for_config_read_nodes_count() {
            let setup = Setup::empty();

            let config = json!({"read_nodes_count": 3}).to_string();

            let txn_file_path = pool::create_genesis_txn_file_for_test_pool(&setup.name, None, None);
            let pool_config = pool::pool_config_json(txn_file_path.as_path());
            pool::create_pool_ledger_config(&setup.name, Some(pool_config.as_str())).unwrap();

            let pool_handle = pool::open_pool_ledger(&setup.name, Some(&config)).unwrap();

            let request = ledger::build_get_nym_request(None, DID_TRUSTEE).unwrap();
            let _ = ledger::submit_request(pool_handle, &request).unwrap();

            pool::close(pool_handle).unwrap();
        }
    }

    mod close {
        use super::*;

        extern crate futures;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_close_pool_ledger_works_for_twice() {
            let setup = Setup::empty();

            let pool_handle = pool::create_and_open_pool_ledger(&setup.name).unwrap();

            pool::close(pool_handle).unwrap();
            let res = pool::close(pool_handle);
            assert_code!(ErrorCode::PoolLedgerInvalidPoolHandle, res);
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_close_pool_ledger_works_for_pending_request() {
            use crate::indy::future::Future;

            let setup = Setup::empty();

            let pool_handle = pool::create_and_open_pool_ledger(&setup.name).unwrap();

            let get_nym_req = ledger::build_get_nym_request(Some(DID_MY1), DID_MY1).unwrap();

            let submit_fut = indy::ledger::submit_request(pool_handle, &get_nym_req);

            pool::close(pool_handle).unwrap();

            let response = submit_fut.wait().unwrap();
            pool::check_response_type(&response, ResponseType::REPLY);

            /* Now any request to API can failed, if pool::close works incorrect in case of pending requests.
               For example try to delete the pool. */
            pool::delete(&setup.name).unwrap();
        }

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_close_pool_ledger_works_for_invalid_handle() {
            Setup::empty();

            let res = pool::close(0);
            assert_code!(ErrorCode::PoolLedgerInvalidPoolHandle, res);
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn indy_delete_pool_ledger_config_works_for_closed() {
            let setup = Setup::empty();

            let pool_handle = pool::create_and_open_pool_ledger(&setup.name).unwrap();
            pool::close(pool_handle).unwrap();
            pool::delete(&setup.name).unwrap();
        }

        #[test]
        fn indy_delete_pool_ledger_config_works_for_not_created() {
            let setup = Setup::empty();

            let res = pool::delete(&setup.name);
            assert_code!(ErrorCode::CommonIOError, res);
        }

        #[test]
        fn indy_delete_pool_ledger_config_works_for_twice() {
            let setup = Setup::empty();

            let pool_handle = pool::create_and_open_pool_ledger(&setup.name).unwrap();
            pool::close(pool_handle).unwrap();
            pool::delete(&setup.name).unwrap();
            let res = pool::delete(&setup.name);
            assert_code!(ErrorCode::CommonIOError, res);
        }
    }

    mod refresh {
        use super::*;

        #[test]
        #[cfg(feature = "local_nodes_pool")]
        fn indy_refresh_pool_ledger_works_for_invalid_handle() {
            Setup::empty();

            let res = pool::refresh(0);
            assert_code!(ErrorCode::PoolLedgerInvalidPoolHandle, res);
        }
    }

    mod set_protocol_version {
        use super::*;

        #[test]
        fn indy_set_protocol_version_works_for_unsupported() {
            let res = pool::set_protocol_version(0);
            assert_code!(ErrorCode::PoolIncompatibleProtocolVersion, res);
        }
    }
}
