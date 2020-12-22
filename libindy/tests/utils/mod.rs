#![allow(dead_code, unused_macros)]

extern crate libc;
extern crate indyrs as indy;

use indy::{WalletHandle, PoolHandle, INVALID_WALLET_HANDLE, INVALID_POOL_HANDLE};

pub mod callback;

#[path = "../../indy-utils/src/environment.rs"]
pub mod environment;

pub mod pool;
pub mod crypto;
pub mod did;
pub mod wallet;
pub mod ledger;
pub mod anoncreds;
pub mod types;
pub mod pairwise;
pub mod constants;
pub mod blob_storage;
pub mod non_secrets;
pub mod results;
pub mod payments;
pub mod rand_utils;
pub mod logger;
pub mod cache;
pub mod metrics;

#[macro_use]
#[allow(unused_macros)]
#[path = "../../indy-utils/src/test.rs"]
pub mod test;

pub mod timeout;

#[path = "../../indy-utils/src/sequence.rs"]
pub mod sequence;

#[macro_use]
#[allow(unused_macros)]
#[path = "../../indy-utils/src/ctypes.rs"]
pub mod ctypes;

#[macro_use]
#[path = "../../src/utils/qualifier.rs"]
pub mod qualifier;

pub(crate) use indy::ErrorCode;

#[path = "../../indy-utils/src/inmem_wallet.rs"]
pub mod inmem_wallet;

#[path = "../../indy-utils/src/wql.rs"]
pub mod wql;

#[path = "../../src/domain/mod.rs"]
pub mod domain;

macro_rules! inject_indy_dependencies {
    () => {
        extern crate serde;

        #[macro_use]
        extern crate lazy_static;

        #[macro_use]
        extern crate derivative;

        #[macro_use]
        extern crate serde_derive;

        #[macro_use]
        extern crate serde_json;

        #[allow(unused_imports)]
        #[macro_use]
        extern crate log;

        extern crate byteorder;
        extern crate hex;
        extern crate ursa;
        extern crate uuid;
        extern crate openssl;
        extern crate rmp_serde;
        extern crate rust_base58;
        extern crate sodiumoxide;
        extern crate rand;
        extern crate regex;
        extern crate time;
        extern crate libc;
        extern crate dirs_next;
    }
}

fn setup() -> String {
    let name = crate::utils::rand_utils::get_rand_string(10);
    test::cleanup_storage(&name);
    logger::set_default_logger();
    name
}

fn tear_down(name: &str) {
    test::cleanup_storage(name);
}

pub struct Setup {
    pub name: String,
    pub wallet_config: String,
    pub wallet_handle: WalletHandle,
    pub pool_handle: PoolHandle,
    pub did: String,
    pub verkey: String
}

impl Setup {
    pub fn empty() -> Setup {
        let name = setup();
        Setup { name, wallet_config: String::new(), wallet_handle: INVALID_WALLET_HANDLE, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    }

    pub fn wallet() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    }

    pub fn plugged_wallet() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_plugged_wallet().unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    }

    pub fn pool() -> Setup {
        let name = setup();
        let pool_handle = pool::create_and_open_pool_ledger(&name).unwrap();
        Setup { name, wallet_config: String::new(), wallet_handle: INVALID_WALLET_HANDLE, pool_handle, did: String::new(), verkey: String::new() }
    }

    pub fn wallet_and_pool() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let pool_handle = pool::create_and_open_pool_ledger(&name).unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle, did: String::new(), verkey: String::new() }
    }

    pub fn trustee() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_and_store_my_did(setup.wallet_handle, Some(constants::TRUSTEE_SEED)).unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn trustee_fully_qualified() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_and_store_my_did_v1(setup.wallet_handle, Some(constants::TRUSTEE_SEED)).unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn steward() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_and_store_my_did(setup.wallet_handle, Some(constants::STEWARD_SEED)).unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn endorser() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_store_and_publish_did(setup.wallet_handle, setup.pool_handle, "ENDORSER", None).unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn new_identity() -> Setup {
        let mut setup = Setup::wallet_and_pool();
        let (did, verkey) = did::create_store_and_publish_did(setup.wallet_handle, setup.pool_handle, "TRUSTEE", None).unwrap();
        setup.did = did;
        setup.verkey = verkey;
        setup
    }

    pub fn did() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let (did, verkey) = did::create_and_store_my_did(wallet_handle, None).unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle: 0, did, verkey }
    }

    pub fn did_fully_qualified() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let (did, verkey) = did::create_and_store_my_did_v1(wallet_handle, None).unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle: 0, did, verkey }
    }

    pub fn key() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        let verkey = crypto::create_key(wallet_handle, None).unwrap();
        Setup { name, wallet_config, wallet_handle, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey }
    }

    pub fn payment() -> Setup {
        let name = setup();
        payments::mock_method::init();
        Setup { name, wallet_config: String::new(), wallet_handle: INVALID_WALLET_HANDLE, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    }

    pub fn payment_wallet() -> Setup {
        let name = setup();
        let (wallet_handle, wallet_config) = wallet::create_and_open_default_wallet(&name).unwrap();
        payments::mock_method::init();
        Setup { name, wallet_config, wallet_handle, pool_handle: INVALID_POOL_HANDLE, did: String::new(), verkey: String::new() }
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        if self.wallet_handle != INVALID_WALLET_HANDLE {
            wallet::close_and_delete_wallet(self.wallet_handle, &self.wallet_config).unwrap();
        }
        if self.pool_handle != INVALID_POOL_HANDLE {
            pool::close(self.pool_handle).unwrap();
        }
        tear_down(&self.name);
    }
}
