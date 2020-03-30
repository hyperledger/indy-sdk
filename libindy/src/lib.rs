#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#[macro_use]
extern crate futures;

extern crate failure;

#[macro_use]
extern crate log;

extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

extern crate rmp_serde;

#[macro_use]
extern crate lazy_static;

extern crate ursa;
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

extern crate zeroize;

extern crate regex;

extern crate byteorder;

extern crate indy_api_types;

#[macro_use]
extern crate indy_utils;

extern crate indy_wallet;

extern crate indy_vdr;

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
