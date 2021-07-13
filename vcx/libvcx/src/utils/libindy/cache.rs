use serde_json;

use utils::libindy::wallet::{add_record, get_record, update_record_value};
use error::{VcxErrorKind, VcxError, VcxResult}; 

static CACHE_TYPE: &str = "cache";
static REV_REG_CACHE_PREFIX: &str = "rev_reg:";
static REV_REG_IDS_CACHE_PREFIX: &str = "rev_reg_ids:";

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

// TODO: Maybe we need to persist more info
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct RevRegIdsCache {
    pub rev_reg_ids: Vec<String>
}

///
/// Returns the rev reg cache.
/// In case of error returns empty cache and silently ignores error.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id
///
pub fn get_rev_reg_cache(rev_reg_id: &str) -> RevRegCache {
    let wallet_id = format!("{}{}", REV_REG_CACHE_PREFIX, rev_reg_id);
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
pub fn set_rev_reg_cache(rev_reg_id: &str, cache: &RevRegCache) {
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}", REV_REG_CACHE_PREFIX, rev_reg_id);
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

fn set_rev_reg_ids_cache(cred_def_id: &str, cache: &str) -> VcxResult<()> {
    debug!("Setting rev_reg_ids for cred_def_id {}, cache {}", cred_def_id, cache);
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}", REV_REG_IDS_CACHE_PREFIX, cred_def_id);
            match update_record_value(CACHE_TYPE, &wallet_id, &json)
                .or(add_record(CACHE_TYPE, &wallet_id, &json, None)) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                }
        },
        Err(_) => {
            Err(VcxError::from(VcxErrorKind::SerializationError))
        }
    }
}

fn get_rev_reg_ids_cache(cred_def_id: &str) -> Option<RevRegIdsCache> {
    debug!("Getting rev_reg_delta_cache for cred_def_id {}", cred_def_id);
    let wallet_id = format!("{}{}", REV_REG_IDS_CACHE_PREFIX, cred_def_id);
    match get_record(CACHE_TYPE, &wallet_id, &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string()) {
        Ok(json) => {
            match serde_json::from_str(&json)
                .and_then(|x: serde_json::Value| 
                    serde_json::from_str(x.get("value").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or(""))) {
                Ok(cache) => cache,
                Err(err) => {
                    warn!("Unable to convert rev_reg_ids cache for cred_def_id: {}, json: {}, error: {}", cred_def_id, json, err);
                    None
                }
            }
        },
        Err(err) => {
            warn!("Unable to get rev_reg_ids cache for cred_def_id: {}, error: {}", cred_def_id, err);
            None
        }
    }
}

pub fn update_rev_reg_ids_cache(cred_def_id: &str, rev_reg_id: &str) -> VcxResult<()> {
    debug!("Setting rev_reg_ids cache for cred_def_id {}, rev_reg_id {}", cred_def_id, rev_reg_id);
	match get_rev_reg_ids_cache(cred_def_id) {
        Some(mut old_vec) => {
            old_vec.rev_reg_ids.push(String::from(rev_reg_id));
            match serde_json::to_string(&old_vec) {
                Ok(ser_new_vec) => set_rev_reg_ids_cache(cred_def_id, ser_new_vec.as_str()),
                Err(_) => Err(VcxError::from(VcxErrorKind::SerializationError))
            }
        },
        None => {
            match serde_json::to_string(&vec![rev_reg_id]) {
                Ok(ser_new_vec) => set_rev_reg_ids_cache(cred_def_id, ser_new_vec.as_str()),
                Err(_) => Err(VcxError::from(VcxErrorKind::SerializationError))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::SetupLibraryWallet;

    fn _rev_reg_id() -> &'static str {
        "test-id"
    }

    #[test]
    fn test_get_credential_cache_returns_default_when_not_exists_in_wallet() {
        let _setup = SetupLibraryWallet::init();

        let result = get_rev_reg_cache(_rev_reg_id());
        assert_eq!(result, RevRegCache::default());
    }

    #[test]
    fn test_get_credential_cache_returns_default_when_invalid_data_in_the_wallet() {
        let _setup = SetupLibraryWallet::init();

        add_record(CACHE_TYPE, _rev_reg_id(), "some invalid json", None).unwrap();

        let result = get_rev_reg_cache(_rev_reg_id());
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

        set_rev_reg_cache(_rev_reg_id(), &data);

        let result = get_rev_reg_cache(_rev_reg_id());

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

        set_rev_reg_cache(_rev_reg_id(), &data);

        let result = get_rev_reg_cache(_rev_reg_id());
        assert_eq!(result, data);

        let result = get_rev_reg_cache(_rev_reg_id());
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

        set_rev_reg_cache(_rev_reg_id(), &data1);
        let result = get_rev_reg_cache(_rev_reg_id());
        assert_eq!(result, data1);

        // overwrite
        set_rev_reg_cache(_rev_reg_id(), &data2);
        let result = get_rev_reg_cache(_rev_reg_id());
        assert_eq!(result, data2);
    }

}
