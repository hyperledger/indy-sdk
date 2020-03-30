#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate futures;
extern crate indyrs as indy;
#[macro_use]
pub mod utils;

use utils::constants::{DID_1};
use utils::setup::{Setup, SetupConfig};
use indy::ErrorCode;
use indy::pool;
#[allow(unused_imports)]
use futures::Future;

#[cfg(test)]
mod open_pool {
    use super::*;
    use futures::future::Future;

    #[test]
    pub fn open_pool_works() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_config() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let config = Some(r#"{"timeout": 20}"#);

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, config).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_twice() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        let ec = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap_err();
        assert_eq!(ec.error_code, ErrorCode::PoolLedgerInvalidPoolHandle);

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_two_nodes() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 2,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_three_nodes() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 3,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_cached_txns() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });
        utils::pool::dump_correct_genesis_txns_to_cache(&setup.pool_name).unwrap();

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn open_pool_works_for_corrupted_cached_txns() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });
        utils::pool::dump_incorrect_genesis_txns_to_cache(&setup.pool_name).unwrap();

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }
}

#[cfg(test)]
mod close_pool {
    use super::*;
    use futures::future::Future;

    #[test]
    pub fn close_pool_works() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn close_pool_works_for_twice() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
        let ec = indy::pool::close_pool_ledger(pool_handle).wait().unwrap_err();
        assert_eq!(ec.error_code, ErrorCode::PoolLedgerInvalidPoolHandle);
    }

    #[test]
    pub fn close_pool_works_for_reopen_after_close() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();
        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();
    }

    #[test]
    pub fn close_pool_works_for_pending_request() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: false,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        let pool_handle = indy::pool::open_pool_ledger(&setup.pool_name, None).wait().unwrap();

        let get_nym_req = indy::ledger::build_get_nym_request(Some(DID_1), DID_1).wait().unwrap();
        
        let submit_fut = indy::ledger::submit_request(pool_handle, &get_nym_req);

        indy::pool::close_pool_ledger(pool_handle).wait().unwrap();

        let _response = submit_fut.wait().unwrap();
    }

}

#[cfg(test)]
mod test_pool_create_config {
    use super::*;

    use std::fs;
    use utils::file::TempFile;
    use utils::pool::{PoolList, test_pool_name, test_genesis_config};

    #[inline]
    pub fn assert_pool_exists(name: &str) {
        assert!(PoolList::new().pool_exists(name));
    }

    #[inline]
    pub fn assert_pool_not_exists(name: &str) {
        assert!(! PoolList::new().pool_exists(name));
    }

    /*
    Returns the file, otherwise the file would be deleted
    when it goes out of scope.rustc_lsan
    */
    fn invalid_temporary_genesis_config() -> (String, TempFile) {
        let file = TempFile::new(None).unwrap();
        fs::write(&file, b"Some nonsensical data").unwrap();
        let config = json!({"genesis_txn": file.as_ref()}).to_string();

        (config, file)
    }

    #[test]
    /* Create a valid config with custom genesis txn. */
    fn config_with_genesis_txn() {
        let name = test_pool_name();
        let (config, _file) = test_genesis_config();
        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();

        assert_eq!((), result.unwrap());
        assert_pool_exists(&name);
        pool::delete_pool_ledger(&name).wait().unwrap();
    }

    #[test]
    /* Config options missing genesis_txn */
    fn config_missing_genesis_txn() {
        let name = test_pool_name();
        let result = pool::create_pool_ledger_config(&name, Some("{}")).wait();

        assert_eq!(ErrorCode::CommonInvalidStructure, result.unwrap_err().error_code);
        assert_pool_not_exists(&name);
    }

    #[test]
    /* A path which doesn't exists results in error. */
    fn config_with_unknown_path_genesis_txn() {
        let name = test_pool_name();
        let config = json!({"genesis_txn": "/nonexist15794345"}).to_string();
        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();

        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err().error_code);
        assert_pool_not_exists(&name);
    }

    #[test]
    /* Error with an incorrectly formed gensis txn. */
    fn config_with_bad_genesis_txn() {
        let name = test_pool_name();
        let (config, _file) = invalid_temporary_genesis_config();

        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();

        assert_eq!(ErrorCode::CommonInvalidStructure, result.unwrap_err().error_code);
        assert_pool_not_exists(&name);
    }

    #[test]
    /* Must specify pool name when config is created. */
    fn config_with_empty_pool_name() {
        let name = test_pool_name();
        let (config, _file) = test_genesis_config();
        let result = pool::create_pool_ledger_config("", Some(&config)).wait();

        assert_eq!(ErrorCode::CommonInvalidParam2, result.unwrap_err().error_code);
        assert_pool_not_exists(&name);
    }

    #[test]
    /* Error when creating a pool that already exists */
    fn config_already_exists() {
        let name = test_pool_name();
        let (config, _file) = test_genesis_config();

        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();
        assert_eq!((), result.unwrap());

        let result = pool::create_pool_ledger_config(&name, Some(&config)).wait();
        assert_eq!(ErrorCode::PoolLedgerConfigAlreadyExistsError, result.unwrap_err().error_code);

        assert_pool_exists(&name);
        pool::delete_pool_ledger(&name).wait().unwrap();
    }

}

#[cfg(test)]
mod test_delete_config {
    use super::*;

    use futures::future::Future;

    use utils::pool::{PoolList, create_default_pool};

    const NON_EXISTENT_NAME: &str = "a_pool_name_which_does_not_exist";

    #[inline]
    pub fn assert_pool_exists(name: &str) {
        assert!(PoolList::new().pool_exists(name));
    }

    #[inline]
    pub fn assert_pool_not_exists(name: &str) {
        assert!(! PoolList::new().pool_exists(name));
    }

    #[test]
    /* Delete a pool_config. */
    fn delete_pool() {
        let pool_name = create_default_pool();
        assert_pool_exists(&pool_name);

        let result = pool::delete_pool_ledger(&pool_name).wait();
        assert_eq!((), result.unwrap());

        assert_pool_not_exists(&pool_name);
    }

    #[test]
    /* Error deleting non existent pool_config. */
    fn delete_pool_not_exist() {
        let result = pool::delete_pool_ledger(NON_EXISTENT_NAME).wait();
        assert_eq!(ErrorCode::CommonIOError, result.unwrap_err().error_code);
    }

    #[test]
    /* Error deleting an open pool_config. */
    fn delete_pool_open() {
        let pool_name = create_default_pool();
        let config = json!({
            "refresh_on_open": false,
            "auto_refresh_time": 0,
        }).to_string();

        pool::set_protocol_version(2).wait().unwrap();
        let pool_handle = pool::open_pool_ledger(&pool_name, Some(&config)).wait().unwrap();

        let result = pool::delete_pool_ledger(&pool_name).wait();
        assert_eq!(ErrorCode::CommonInvalidState, result.unwrap_err().error_code);
        assert_pool_exists(&pool_name);

        pool::close_pool_ledger(pool_handle).wait().unwrap();
        pool::delete_pool_ledger(&pool_name).wait().unwrap();
        pool::set_protocol_version(1).wait().unwrap();
    }

}

#[cfg(test)]
mod test_set_protocol_version {
    use super::*;

    use indy::ledger;
    use serde_json;

    const VALID_VERSIONS: [usize; 2] = [1, 2];

    fn assert_protocol_version_set(version: usize) {
        let did = "5UBVMdSADMjGzuJMQwJ6yyzYV1krTcKRp6EqRAz8tiDP";
        let request = ledger::build_get_nym_request(Some(did), did).wait().unwrap();
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(json!(version), *request.get("protocolVersion").unwrap());
    }

    #[test]
    /* Set all available protocol versions. */
    fn set_all_valid_versions() {
        for &version in VALID_VERSIONS.into_iter() {
            let result = pool::set_protocol_version(version).wait();
            assert_eq!((), result.unwrap());
            assert_protocol_version_set(version);
        }

        pool::set_protocol_version(1).wait().unwrap();
    }

    #[test]
    /* Error setting invalid protocol version. */
    fn set_invalid_versions() {
        let result = pool::set_protocol_version(0).wait();
        assert_eq!(ErrorCode::PoolIncompatibleProtocolVersion, result.unwrap_err().error_code);
        assert_protocol_version_set(1);

        let next_protocol_version = *VALID_VERSIONS.last().unwrap() + 1;
        let result = pool::set_protocol_version(next_protocol_version).wait();
        assert_eq!(ErrorCode::PoolIncompatibleProtocolVersion, result.unwrap_err().error_code);
        assert_protocol_version_set(1);
    }

}

#[cfg(test)]
/*
The sync pool::list is already tested by the create tests.
There aren't tests for failure because I'm not sure how it would fail.
*/
mod test_pool_list {
    use super::*;

    use utils::pool::{PoolList, create_default_pool};

    #[test]
    fn list_pool() {
        let name = create_default_pool();

        let pool_json = pool::list_pools().wait().unwrap();
        assert!(PoolList::from_json(&pool_json).pool_exists(&name));

        pool::delete_pool_ledger(&name).wait().unwrap();
    }

}

#[cfg(test)]
mod test_refresh_works {
    use super::*;

    #[test]
    pub fn refresh_pool_works() {
        let wallet = utils::wallet::Wallet::new();
        let setup = Setup::new(&wallet, SetupConfig {
            connect_to_pool: true,
            num_trustees: 0,
            num_nodes: 4,
            num_users: 0,
        });

        indy::pool::refresh_pool_ledger(setup.pool_handle.unwrap()).wait().unwrap();
    }

}
