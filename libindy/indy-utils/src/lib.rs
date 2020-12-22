extern crate indy_api_types;

#[macro_use]
extern crate lazy_static;

extern crate log;

extern crate serde;

extern crate serde_derive;

#[macro_use]
extern crate serde_json;

extern crate zeroize;

extern crate dirs_next;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{ $val }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{ "_" }};
}

#[macro_use]
pub mod crypto;
pub mod ctypes;
pub mod environment;
pub mod inmem_wallet;
pub mod sequence;
#[macro_use]
#[allow(unused_macros)]
pub mod test;
pub mod wql;

pub(crate) use indy_api_types::ErrorCode;

use indy_api_types::{CommandHandle, PoolHandle, WalletHandle, SearchHandle};

pub fn next_wallet_handle() -> WalletHandle { WalletHandle(sequence::get_next_id()) }

pub fn next_pool_handle() -> PoolHandle {
    sequence::get_next_id()
}

pub fn next_command_handle() -> CommandHandle {
    sequence::get_next_id()
}

pub fn next_search_handle() -> SearchHandle {
    SearchHandle(sequence::get_next_id())
}
