use indy_api_types::errors::prelude::*;
use rust_base58::{FromBase58, ToBase58};

pub(super) const DEFAULT_CRYPTO_TYPE: &str = "ed25519";

pub fn build_full_verkey(dest: &str, verkey: Option<&str>) -> IndyResult<String> {
    if let Some(verkey) = verkey {
        let verkey = if verkey.starts_with('~') {
            let mut result = dest.from_base58()?;
            let mut end = verkey[1..].from_base58()?;
            result.append(&mut end);
            result.to_base58()
        } else {
            verkey.to_owned()
        };

        Ok(verkey)
    } else {
        // Cryptonym
        Ok(dest.to_owned())
    }
}

// Remove crypto_type from verkey to be backward compatible
pub(super) fn clear_verkey(verkey: &str) -> IndyResult<&str> {
    let position = verkey.find(':');
    let key = match position {
        Some(p) => {
            if p + 1 < verkey.len() {
                let cryptoname: &str = verkey[p + 1..].as_ref();
                if cryptoname != DEFAULT_CRYPTO_TYPE {
                    return Err(err_msg(IndyErrorKind::UnknownCrypto, format!("Key contains unknown crypto: {}", cryptoname)));
                }
            };
            let v = if p > 0 {
                verkey[..p].as_ref()
            } else {
                ""
            };
            v
        }
        None => verkey
    };
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_verkey_empty() {
        assert_eq!(clear_verkey("").unwrap(), "")
    }

    #[test]
    fn clear_verkey_single_colon() {
        assert_eq!(clear_verkey(":").unwrap(), "")
    }

    #[test]
    fn clear_verkey_works() {
        assert_eq!(clear_verkey(&format!("foo:{}", DEFAULT_CRYPTO_TYPE)).unwrap(), "foo")
    }

    #[test]
    fn clear_verkey_unknown_crypto() {
        assert_eq!(clear_verkey("foo:crypto").unwrap_err().kind(), IndyErrorKind::UnknownCrypto);
    }
}