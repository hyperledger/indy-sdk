#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
extern crate futures;

extern crate byteorder;
extern crate failure;

#[macro_use]
extern crate log;

extern crate serde;

extern crate variant_count;

#[macro_use]
extern crate num_derive;

extern crate num_traits;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

extern crate rmp_serde;

#[macro_use]
extern crate lazy_static;

extern crate ursa;
extern crate rlp;
extern crate time;
extern crate libc;
extern crate rand;
extern crate uuid;

#[macro_use]
extern crate derivative;
extern crate core;

extern crate hex;

extern crate log_derive;
extern crate rust_base58;

extern crate sha2;
extern crate sha3;

extern crate zeroize;

extern crate regex;

extern crate indy_api_types;

#[macro_use]
extern crate indy_utils;

extern crate indy_wallet;

// Note that to use macroses from util inside of other modules it must be loaded first!
#[macro_use]
mod utils;

pub mod api;
mod commands;
mod services;
mod domain;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn dummy() {
        assert!(true, "Dummy check!");
    }
}
