use crate::indy_api_types::errors::prelude::*;
use libc::size_t;

use zeroize::Zeroize;

pub const SEEDBYTES: usize = 32; // randombytes_seedbytes

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Seed([u8; SEEDBYTES]);

impl Seed {
    pub fn from_slice(bytes: &[u8]) -> Result<Seed, IndyError> {
        if bytes.len() != SEEDBYTES {
            return Err(IndyError::from_msg(IndyErrorKind::InvalidStructure,
                                           format!("Invalid seed length, expected: {:}, provided: {}", SEEDBYTES, bytes.len())));
        }

        let mut seed = Seed([0; SEEDBYTES]);

        for (ni, &bsi) in seed.0.iter_mut().zip(bytes.iter()) {
            *ni = bsi
        }

        Ok(seed)
    }
}

pub fn randombytes_deterministic(size: usize, seed: &Seed) -> Vec<u8> {
    let mut out = vec![0u8; size];
    unsafe {
        randombytes_buf_deterministic(out.as_mut_ptr(),
                                      size,
                                      &seed.0)
    };
    out
}


extern {
    fn randombytes_buf_deterministic(out: *mut u8,
                                     size: size_t,
                                     seed: *const [u8; SEEDBYTES]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn randombytes_deterministic_works() {
        let seed = Seed::from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5]).unwrap();
        let res = randombytes_deterministic(32, &seed);
        let expected_bytes = vec![7, 183, 0, 143, 100, 203, 87, 27, 32, 132, 126, 172, 180, 123, 39, 26, 18, 243, 64, 60, 92, 43, 111, 227, 54, 129, 201, 185, 53, 73, 93, 93];
        assert_eq!(expected_bytes, res);
    }
}