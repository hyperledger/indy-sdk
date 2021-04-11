extern crate rand;

use self::rand::Rng;
use self::rand::distributions::Alphanumeric;
use std::iter;

pub fn get_rand_string(len: usize) -> String {
    iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}
