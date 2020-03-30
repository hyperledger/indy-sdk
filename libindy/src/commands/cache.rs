use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use indy_api_types::domain::wallet::Tags;
use indy_vdr::ledger::identifiers::schema::SchemaId;
use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;
use indy_api_types::errors::prelude::*;
use indy_wallet::{WalletService, WalletRecord};
use indy_api_types::{WalletHandle, PoolHandle};
use crate::domain::cache::{GetCacheOptions, PurgeOptions};
use crate::services::crypto::CryptoService;
use crate::services::ledger::LedgerService;
use crate::services::pool::PoolService;
use indy_vdr::common::did::DidValue;
use indy_vdr::utils::qualifier::Qualifiable;

const CRED_DEF_CACHE: &str = "cred_def_cache";
const SCHEMA_CACHE: &str = "schema_cache";

pub enum CacheCommand {
    GetSchema(PoolHandle,
              WalletHandle,
              DidValue, // submitter_did
              SchemaId, // id
              GetCacheOptions, // options
              Box<dyn Fn(IndyResult<String>) + Send>),
    GetCredDef(PoolHandle,
               WalletHandle,
               DidValue, // submitter_did
               CredentialDefinitionId, // id
               GetCacheOptions, // options
               Box<dyn Fn(IndyResult<String>) + Send>),
    PurgeSchemaCache(WalletHandle,
                     PurgeOptions, // options
                     Box<dyn Fn(IndyResult<()>) + Send>),
    PurgeCredDefCache(WalletHandle,
                      PurgeOptions, // options
                      Box<dyn Fn(IndyResult<()>) + Send>),
}

pub struct CacheCommandExecutor {
    crypto_service: Rc<CryptoService>,
    ledger_service: Rc<LedgerService>,
    pool_service: Rc<PoolService>,
    wallet_service: Rc<WalletService>,
}

macro_rules! check_cache {
    ($cache: ident, $options: ident) => {
    if let Some(cache) = $cache {
            let min_fresh = $options.min_fresh.unwrap_or(-1);
            if min_fresh >= 0 {
                let ts = match CacheCommandExecutor::get_seconds_since_epoch() {
                    Ok(ts) => ts,
                    Err(err) => {
                        return Err(err)
                    }
                };
                if ts - min_fresh <= cache.get_tags().unwrap_or(&Tags::new()).get("timestamp").unwrap_or(&"-1".to_string()).parse().unwrap_or(-1) {
                    return Ok(cache.get_value().unwrap_or("").to_string())
                }
            } else {
                return Ok(cache.get_value().unwrap_or("").to_string())
            }
        }
    };
}

impl CacheCommandExecutor {
    pub fn new(crypto_service: Rc<CryptoService>, ledger_service: Rc<LedgerService>, pool_service: Rc<PoolService>, wallet_service: Rc<WalletService>) -> CacheCommandExecutor {
        CacheCommandExecutor {
            crypto_service,
            ledger_service,
            pool_service,
            wallet_service,
        }
    }

    pub async fn execute(&self, command: CacheCommand) {
        match command {
            CacheCommand::GetSchema(pool_handle, wallet_handle, submitter_did, id, options, cb) => {
                debug!(target: "non_secrets_command_executor", "GetSchema command received");
                cb(self.get_schema(pool_handle, wallet_handle, &submitter_did, &id, options).await);
            }
            CacheCommand::GetCredDef(pool_handle, wallet_handle, submitter_did, id, options, cb) => {
                debug!(target: "non_secrets_command_executor", "GetCredDef command received");
                cb(self.get_cred_def(pool_handle, wallet_handle, &submitter_did, &id, options).await);
            }
            CacheCommand::PurgeSchemaCache(wallet_handle, options, cb) => {
                debug!(target: "non_secrets_command_executor", "PurgeSchemaCache command received");
                cb(self.purge_schema_cache(wallet_handle, options));
            }
            CacheCommand::PurgeCredDefCache(wallet_handle, options, cb) => {
                debug!(target: "non_secrets_command_executor", "PurgeCredDefCache command received");
                cb(self.purge_cred_def_cache(wallet_handle, options));
            }
        }
    }

    async fn get_schema(&self,
                  pool_handle: PoolHandle,
                  wallet_handle: WalletHandle,
                  submitter_did: &DidValue,
                  id: &SchemaId,
                  options: GetCacheOptions) -> IndyResult<String> {
        trace!("get_schema >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, id: {:?}, options: {:?}",
               pool_handle, wallet_handle, submitter_did, id, options);

        let cache = self.get_record_from_cache(wallet_handle, &id.0, &options, SCHEMA_CACHE)?;

        check_cache!(cache, options);

        if options.no_update.unwrap_or(false) {
            return Err(IndyError::from(IndyErrorKind::LedgerItemNotFound));
        }

        self.crypto_service.validate_opt_did(Some(submitter_did))?;

        let ledger_response = {
            let prepared_request = self.ledger_service
                .request_builder()?
                .build_get_schema_request(Some(submitter_did), id)?;

            let pool_response = self.pool_service.send_request(pool_handle, prepared_request).await?;

            self.ledger_service.parse_get_schema_response(&pool_response, id.get_method())
        };

        let (schema_id, schema_json) = ledger_response?;

        self._delete_and_add_record(wallet_handle, options, &schema_id, &schema_json, SCHEMA_CACHE)
            .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("get_schema_continue failed: {:?}", err)))?;

        Ok(schema_json)
    }

    fn _delete_and_add_record(&self,
                              wallet_handle: WalletHandle,
                              options: GetCacheOptions,
                              schema_id: &str,
                              schema_json: &str,
                              which_cache: &str) -> IndyResult<()>
    {
        if !options.no_store.unwrap_or(false) {
            let mut tags = Tags::new();
            let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(ts) => ts.as_secs() as i32,
                Err(err) => {
                    warn!("Cannot get time: {:?}", err);
                    0
                }
            };
            tags.insert("timestamp".to_string(), ts.to_string());
            let _ignore = self.wallet_service.delete_record(wallet_handle, which_cache, &schema_id);
            self.wallet_service.add_record(wallet_handle, which_cache, &schema_id, &schema_json, &tags)?
        }
        Ok(())
    }

    async fn get_cred_def<'a>(&'a self,
                    pool_handle: PoolHandle,
                    wallet_handle: WalletHandle,
                    submitter_did: &'a DidValue,
                    id: &'a CredentialDefinitionId,
                    options: GetCacheOptions) -> IndyResult<String> {
        trace!("get_cred_def >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, id: {:?}, options: {:?}",
               pool_handle, wallet_handle, submitter_did, id, options);

        let cache = self.get_record_from_cache(wallet_handle, &id.0, &options, CRED_DEF_CACHE)?;

        check_cache!(cache, options);

        if options.no_update.unwrap_or(false) {
            return Err(IndyError::from(IndyErrorKind::LedgerItemNotFound));
        }

        let (cred_def_id, cred_def_json) = self.ledger_get_cred_def_and_parse(pool_handle, Some(submitter_did), id).await?;

        match self._delete_and_add_record(wallet_handle, options, &cred_def_id, &cred_def_json, CRED_DEF_CACHE) {
            Ok(_) => Ok(cred_def_json),
            Err(err) => Err(IndyError::from_msg(IndyErrorKind::InvalidState, format!("get_cred_def_continue failed: {:?}", err)))
        }
    }

    async fn ledger_get_cred_def_and_parse<'a>(&'a self, pool_handle: i32, submitter_did: Option<&'a DidValue>, id: &'a CredentialDefinitionId) -> IndyResult<(String, String)> {
        self.crypto_service.validate_opt_did(submitter_did)?;

        let prepared_request = self.ledger_service
            .request_builder()?
            .build_get_cred_def_request(submitter_did, id)?;

        let pool_response = self.pool_service.send_request(pool_handle, prepared_request).await?;

        self.ledger_service.parse_get_cred_def_response(&pool_response, id.get_method())
    }

    fn get_record_from_cache(&self, wallet_handle: WalletHandle, id: &str, options: &GetCacheOptions, which_cache: &str) -> Result<Option<WalletRecord>, IndyError> {
        if !options.no_cache.unwrap_or(false) {
            let options_json = json!({
                "retrieveType": false,
                "retrieveValue": true,
                "retrieveTags": true,
            }).to_string();
            match self.wallet_service.get_record(wallet_handle, which_cache, &id, &options_json) {
                Ok(record) => Ok(Some(record)),
                Err(err) => if err.kind() == IndyErrorKind::WalletItemNotFound { Ok(None) } else { Err(err) }
            }
        } else { Ok(None) }
    }

    fn get_seconds_since_epoch() -> Result<i32, IndyError> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(ts) => Ok(ts.as_secs() as i32),
            Err(err) => {
                error!("Cannot get time: {:?}", err);
                Err(IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot get time: {:?}", err)))
            }
        }
    }

    fn build_query_json(max_age: i32) -> Result<String, IndyError> {
        if max_age >= 0 {
            let ts = CacheCommandExecutor::get_seconds_since_epoch()?;
            Ok(json!({"timestamp": {"$lt": ts - max_age}}).to_string())
        } else {
            Ok("{}".to_string())
        }
    }

    fn purge_schema_cache(&self,
                          wallet_handle: WalletHandle,
                          options: PurgeOptions) -> IndyResult<()> {
        trace!("purge_schema_cache >>> wallet_handle: {:?}, options: {:?}", wallet_handle, options);

        let max_age = options.max_age.unwrap_or(-1);
        let query_json = CacheCommandExecutor::build_query_json(max_age)?;

        let options_json = json!({
            "retrieveType": false,
            "retrieveValue": false,
            "retrieveTags": false,
        }).to_string();

        let mut search = self.wallet_service.search_records(
            wallet_handle,
            SCHEMA_CACHE,
            &query_json,
            &options_json,
        )?;

        while let Some(record) = search.fetch_next_record()? {
            self.wallet_service.delete_record(wallet_handle, SCHEMA_CACHE, record.get_id())?;
        }

        trace!("purge_schema_cache <<< res: ()");

        Ok(())
    }

    fn purge_cred_def_cache(&self,
                            wallet_handle: WalletHandle,
                            options: PurgeOptions) -> IndyResult<()> {
        trace!("purge_cred_def_cache >>> wallet_handle: {:?}, options: {:?}", wallet_handle, options);

        let max_age = options.max_age.unwrap_or(-1);
        let query_json = CacheCommandExecutor::build_query_json(max_age)?;

        let options_json = json!({
            "retrieveType": false,
            "retrieveValue": false,
            "retrieveTags": false,
        }).to_string();

        let mut search = self.wallet_service.search_records(
            wallet_handle,
            CRED_DEF_CACHE,
            &query_json,
            &options_json,
        )?;

        while let Some(record) = search.fetch_next_record()? {
            self.wallet_service.delete_record(wallet_handle, CRED_DEF_CACHE, record.get_id())?;
        }

        trace!("purge_cred_def_cache <<< res: ()");

        Ok(())
    }
}
