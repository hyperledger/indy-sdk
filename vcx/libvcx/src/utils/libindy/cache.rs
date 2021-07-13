use serde_json;

use utils::libindy::wallet::{add_record, get_record, update_record_value};

static CACHE_TYPE: &str = "cache";
static REV_REG_CACHE_PREFIX: &str = "rev_reg:";

///
/// Cache object for rev reg cache
///
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct RevRegCache {
    pub rev_state: Option<RevState>,
}

///
/// Rev reg delta object.
///
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RevState {
    pub timestamp: u64,
    pub value: String,
}

///
/// Returns the rev reg cache.
/// In case of error returns empty cache and silently ignores error.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id
///
pub fn get_rev_reg_cache(rev_reg_id: &str, cred_rev_id: &str) -> RevRegCache {
    let wallet_id = format!("{}{}:{}", REV_REG_CACHE_PREFIX, rev_reg_id, cred_rev_id);
    match get_record(CACHE_TYPE, &wallet_id, &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string()) {
        Ok(json) => {
            match serde_json::from_str(&json)
                .and_then(|x: serde_json::Value| {
                    serde_json::from_str(x.get("value").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or(""))
                })
                {
                Ok(cache) => cache,
                Err(err) => {
                    warn!("Unable to convert rev reg cache for rev_reg_id: {}, json: {}, error: {}", rev_reg_id, json, err);
                    RevRegCache::default()
                }
            }
        },
        Err(err) => {
            warn!("Unable to get rev_reg cache for rev_reg_id: {}, error: {}", rev_reg_id, err);
            RevRegCache::default()
        }
    }
}

///
/// Saves rev reg cache.
/// Errors are silently ignored.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
///
pub fn set_rev_reg_cache(rev_reg_id: &str, cred_rev_id: &str, cache: &RevRegCache) {
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}:{}", REV_REG_CACHE_PREFIX, rev_reg_id, cred_rev_id);
            let result = update_record_value(CACHE_TYPE, &wallet_id, &json)
                .or(add_record(CACHE_TYPE, &wallet_id, &json, None));
            if result.is_err() {
                warn!("Error when saving rev reg cache {:?}, error: {:?}", cache, result);
            }
        },
        Err(err) => {
            warn!("Unable to convert to JSON rev reg cache {:?}, error: {:?}", cache, err);
        }
    }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::SetupLibraryWallet;

    fn _rev_reg_id() -> &'static str {
        "test-rev-reg-id"
    }

    fn _cred_rev_id() -> &'static str {
        "test-cred-rev-id"
    }

    #[test]
    fn test_get_credential_cache_returns_default_when_not_exists_in_wallet() {
        let _setup = SetupLibraryWallet::init();

        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, RevRegCache::default());
    }

    #[test]
    fn test_get_credential_cache_returns_default_when_invalid_data_in_the_wallet() {
        let _setup = SetupLibraryWallet::init();

        add_record(CACHE_TYPE, _rev_reg_id(), "some invalid json", None).unwrap();

        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, RevRegCache::default());
    }

    #[test]
    fn test_credential_cache_set_than_get_works() {
        let _setup = SetupLibraryWallet::init();

        let data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: 1000,
                value: r#"{"key": "value1"}"#.to_string(),
            })
        };

        set_rev_reg_cache(_rev_reg_id(), _cred_rev_id(), &data);

        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());

        assert_eq!(result, data);
    }

    #[test]
    fn test_credential_cache_set_than_double_get_works() {
        let _setup = SetupLibraryWallet::init();

        let data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: 1000,
                value: r#"{"key": "value1"}"#.to_string(),
            })
        };

        set_rev_reg_cache(_rev_reg_id(), _cred_rev_id(), &data);

        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, data);

        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, data);
    }

    #[test]
    fn test_credential_cache_overwrite_works() {
        let _setup = SetupLibraryWallet::init();

        let data1 = RevRegCache {
            rev_state: Some(RevState {
                timestamp: 1000,
                value: r#"{"key": "value1"}"#.to_string(),
            })
        };

        let data2 = RevRegCache {
            rev_state: Some(RevState {
                timestamp: 2000,
                value: r#"{"key": "value2"}"#.to_string(),
            })
        };

        set_rev_reg_cache(_rev_reg_id(), _cred_rev_id(), &data1);
        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, data1);

        // overwrite
        set_rev_reg_cache(_rev_reg_id(), _cred_rev_id(), &data2);
        let result = get_rev_reg_cache(_rev_reg_id(), _cred_rev_id());
        assert_eq!(result, data2);
    }

}