use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use indy_api_types::domain::wallet::Tags;
use crate::domain::anoncreds::schema::SchemaId;
use crate::domain::anoncreds::credential_definition::CredentialDefinitionId;
use indy_api_types::errors::prelude::*;
use indy_wallet::{WalletService, WalletRecord};
use indy_api_types::{WalletHandle, PoolHandle, CommandHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::ledger::LedgerCommand;
use crate::domain::cache::{GetCacheOptions, PurgeOptions};
use crate::domain::crypto::did::DidValue;

use indy_utils::next_command_handle;
use crate::services::metrics::MetricsService;

const CRED_DEF_CACHE: &str = "cred_def_cache";
const SCHEMA_CACHE: &str = "schema_cache";

pub enum CacheCommand {
    GetSchema(PoolHandle,
              WalletHandle,
              DidValue, // submitter_did
              SchemaId, // id
              GetCacheOptions, // options
              Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    GetSchemaContinue(
        WalletHandle,
        IndyResult<(String, String)>, // ledger_response
        GetCacheOptions,              // options
        CommandHandle,                          // cb_id
    ),
    GetCredDef(PoolHandle,
               WalletHandle,
               DidValue, // submitter_did
               CredentialDefinitionId, // id
               GetCacheOptions, // options
               Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    GetCredDefContinue(
        WalletHandle,
        IndyResult<(String, String)>, // ledger_response
        GetCacheOptions,              // options
        CommandHandle,                          // cb_id
    ),
    PurgeSchemaCache(WalletHandle,
                     PurgeOptions, // options
                     Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    PurgeCredDefCache(WalletHandle,
                      PurgeOptions, // options
                      Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
}

pub struct CacheCommandExecutor {
    wallet_service: Rc<WalletService>,
    metrics_service: Rc<MetricsService>,

    pending_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<String>, Rc<MetricsService>)>>>,
}

macro_rules! check_cache {
    ($cache: ident, $options: ident, $cb: ident, $metrics:expr) => {
    if let Some(cache) = $cache {
            let min_fresh = $options.min_fresh.unwrap_or(-1);
            if min_fresh >= 0 {
                let ts = match CacheCommandExecutor::get_seconds_since_epoch() {
                    Ok(ts) => ts,
                    Err(err) => {
                        return $cb(Err(err), $metrics)
                    }
                };
                if ts - min_fresh <= cache.get_tags().unwrap_or(&Tags::new()).get("timestamp").unwrap_or(&"-1".to_string()).parse().unwrap_or(-1) {
                    return $cb(Ok(cache.get_value().unwrap_or("").to_string()), $metrics)
                }
            } else {
                return $cb(Ok(cache.get_value().unwrap_or("").to_string()), $metrics)
            }
        }
    };
}

impl CacheCommandExecutor {
    pub fn new(wallet_service: Rc<WalletService>, metrics_service: Rc<MetricsService>) -> CacheCommandExecutor {
        CacheCommandExecutor {
            wallet_service,
            metrics_service,
            pending_callbacks: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: CacheCommand) {
        match command {
            CacheCommand::GetSchema(pool_handle, wallet_handle, submitter_did, id, options, cb) => {
                debug!(target: "non_secrets_command_executor", "GetSchema command received");
                self.get_schema(pool_handle, wallet_handle, &submitter_did, &id, options, cb);
            }
            CacheCommand::GetSchemaContinue(wallet_handle, ledger_response, options, cb_id) => {
                debug!(target: "non_secrets_command_executor", "GetSchemaContinue command received");
                self._get_schema_continue(wallet_handle, ledger_response, options, cb_id);
            }
            CacheCommand::GetCredDef(pool_handle, wallet_handle, submitter_did, id, options, cb) => {
                debug!(target: "non_secrets_command_executor", "GetCredDef command received");
                self.get_cred_def(pool_handle, wallet_handle, &submitter_did, &id, options, cb);
            }
            CacheCommand::GetCredDefContinue(wallet_handle, ledger_response, options, cb_id) => {
                debug!(target: "non_secrets_command_executor", "GetCredDefContinue command received");
                self._get_cred_def_continue(wallet_handle, ledger_response, options, cb_id);
            }
            CacheCommand::PurgeSchemaCache(wallet_handle, options, cb) => {
                debug!(target: "non_secrets_command_executor", "PurgeSchemaCache command received");
                cb(self.purge_schema_cache(wallet_handle, options), self.metrics_service.clone());
            }
            CacheCommand::PurgeCredDefCache(wallet_handle, options, cb) => {
                debug!(target: "non_secrets_command_executor", "PurgeCredDefCache command received");
                cb(self.purge_cred_def_cache(wallet_handle, options), self.metrics_service.clone());
            }
        }
    }

    fn get_schema(&self,
                  pool_handle: PoolHandle,
                  wallet_handle: WalletHandle,
                  submitter_did: &DidValue,
                  id: &SchemaId,
                  options: GetCacheOptions,
                  cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("get_schema >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, id: {:?}, options: {:?}",
               pool_handle, wallet_handle, submitter_did, id, options);

        let cache = self.get_record_from_cache(wallet_handle, &id.0, &options, SCHEMA_CACHE);
        let cache = try_cb!(cache, cb, self.metrics_service.clone());

        check_cache!(cache, options, cb, self.metrics_service.clone());

        if options.no_update.unwrap_or(false) {
            return cb(Err(IndyError::from(IndyErrorKind::LedgerItemNotFound)), self.metrics_service.clone());
        }

        let cb_id = next_command_handle();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        CommandExecutor::instance().send(
            Command::Ledger(
                LedgerCommand::GetSchema(
                    pool_handle,
                    Some(submitter_did.clone()),
                    id.clone(),
                    Box::new(move |ledger_response, _| {
                        CommandExecutor::instance().send(
                            Command::Cache(
                                CacheCommand::GetSchemaContinue(
                                    wallet_handle,
                                    ledger_response,
                                    options.clone(),
                                    cb_id,
                                )
                            )
                        ).unwrap();
                    })
                )
            )
        ).unwrap();
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

    fn _get_schema_continue(&self,
                            wallet_handle: WalletHandle,
                            ledger_response: IndyResult<(String, String)>,
                            options: GetCacheOptions, cb_id: CommandHandle) {
        let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");

        let (schema_id, schema_json) = try_cb!(ledger_response, cb, self.metrics_service.clone());

        match self._delete_and_add_record(wallet_handle, options, &schema_id, &schema_json, SCHEMA_CACHE) {
            Ok(_) => cb(Ok(schema_json), self.metrics_service.clone()),
            Err(err) => cb(Err(IndyError::from_msg(IndyErrorKind::InvalidState, format!("get_schema_continue failed: {:?}", err))), self.metrics_service.clone())
        }
    }

    fn get_cred_def(&self,
                    pool_handle: PoolHandle,
                    wallet_handle: WalletHandle,
                    submitter_did: &DidValue,
                    id: &CredentialDefinitionId,
                    options: GetCacheOptions,
                    cb: Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>) {
        trace!("get_cred_def >>> pool_handle: {:?}, wallet_handle: {:?}, submitter_did: {:?}, id: {:?}, options: {:?}",
               pool_handle, wallet_handle, submitter_did, id, options);

        let cache = self.get_record_from_cache(wallet_handle, &id.0, &options, CRED_DEF_CACHE);
        let cache = try_cb!(cache, cb, self.metrics_service.clone());

        check_cache!(cache, options, cb, self.metrics_service.clone());

        if options.no_update.unwrap_or(false) {
            return cb(Err(IndyError::from(IndyErrorKind::LedgerItemNotFound)), self.metrics_service.clone());
        }

        let cb_id = next_command_handle();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        CommandExecutor::instance().send(
            Command::Ledger(
                LedgerCommand::GetCredDef(
                    pool_handle,
                    Some(submitter_did.clone()),
                    id.clone(),
                    Box::new(move |ledger_response, _| {
                        CommandExecutor::instance().send(
                            Command::Cache(
                                CacheCommand::GetCredDefContinue(
                                    wallet_handle,
                                    ledger_response,
                                    options.clone(),
                                    cb_id,
                                )
                            )
                        ).unwrap();
                    })
                )
            )
        ).unwrap();
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

    fn _get_cred_def_continue(&self, wallet_handle: WalletHandle, ledger_response: IndyResult<(String, String)>, options: GetCacheOptions, cb_id: CommandHandle) {
        let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");

        let (cred_def_id, cred_def_json) = try_cb!(ledger_response, cb, self.metrics_service.clone());

        match self._delete_and_add_record(wallet_handle, options, &cred_def_id, &cred_def_json, CRED_DEF_CACHE) {
            Ok(_) => cb(Ok(cred_def_json), self.metrics_service.clone()),
            Err(err) => cb(Err(IndyError::from_msg(IndyErrorKind::InvalidState, format!("get_cred_def_continue failed: {:?}", err))), self.metrics_service.clone())
        }
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
