use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indy_api_types::domain::wallet::Tags;
use indy_api_types::errors::prelude::*;
use indy_wallet::{RecordOptions, SearchOptions, WalletRecord, WalletSearch, WalletService};
use indy_utils::next_search_handle;
use indy_api_types::{WalletHandle, SearchHandle};
use crate::services::metrics::MetricsService;


pub enum NonSecretsCommand {
    AddRecord(WalletHandle,
              String, // type
              String, // id
              String, // value
              Option<Tags>, //tags
              Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    UpdateRecordValue(WalletHandle,
                      String, // type
                      String, // id
                      String, // value
                      Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    UpdateRecordTags(WalletHandle,
                     String, // type
                     String, // id
                     Tags, //tags
                     Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    AddRecordTags(WalletHandle,
                  String, // type
                  String, // id
                  Tags, //tags
                  Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    DeleteRecordTags(WalletHandle,
                     String, // type
                     String, // id
                     String, //tag names json
                     Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    DeleteRecord(WalletHandle,
                 String, // type
                 String, // id
                 Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    GetRecord(WalletHandle,
              String, // type
              String, // id
              String, // options json
              Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    OpenSearch(WalletHandle,
               String, // type
               String, // query json
               String, // options json
               Box<dyn Fn(IndyResult<SearchHandle>, Rc<MetricsService>) + Send>),
    FetchSearchNextRecords(WalletHandle,
                           SearchHandle, // wallet search handle
                           usize, // count
                           Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    CloseSearch(SearchHandle, // wallet search handle
                Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
}

pub struct NonSecretsCommandExecutor {
    wallet_service: Rc<WalletService>,
    metrics_service: Rc<MetricsService>,
    searches: RefCell<HashMap<SearchHandle, Box<WalletSearch>>>,
}

impl NonSecretsCommandExecutor {
    pub fn new(wallet_service: Rc<WalletService>, metrics_service: Rc<MetricsService>) -> NonSecretsCommandExecutor {
        NonSecretsCommandExecutor {
            wallet_service,
            metrics_service,
            searches: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: NonSecretsCommand) {
        match command {
            NonSecretsCommand::AddRecord(handle, type_, id, value, tags, cb) => {
                debug!(target: "non_secrets_command_executor", "AddRecord command received");
                cb(self.add_record(handle, &type_, &id, &value, tags.as_ref()), self.metrics_service.clone());
            }
            NonSecretsCommand::UpdateRecordValue(handle, type_, id, value, cb) => {
                debug!(target: "non_secrets_command_executor", "UpdateRecordValue command received");
                cb(self.update_record_value(handle, &type_, &id, &value), self.metrics_service.clone());
            }
            NonSecretsCommand::UpdateRecordTags(handle, type_, id, tags, cb) => {
                debug!(target: "non_secrets_command_executor", "UpdateRecordTags command received");
                cb(self.update_record_tags(handle, &type_, &id, &tags), self.metrics_service.clone());
            }
            NonSecretsCommand::AddRecordTags(handle, type_, id, tags, cb) => {
                debug!(target: "non_secrets_command_executor", "AddRecordTags command received");
                cb(self.add_record_tags(handle, &type_, &id, &tags), self.metrics_service.clone());
            }
            NonSecretsCommand::DeleteRecordTags(handle, type_, id, tags_names_json, cb) => {
                debug!(target: "non_secrets_command_executor", "DeleteRecordTags command received");
                cb(self.delete_record_tags(handle, &type_, &id, &tags_names_json), self.metrics_service.clone());
            }
            NonSecretsCommand::DeleteRecord(handle, type_, id, cb) => {
                debug!(target: "non_secrets_command_executor", "DeleteRecord command received");
                cb(self.delete_record(handle, &type_, &id), self.metrics_service.clone());
            }
            NonSecretsCommand::GetRecord(handle, type_, id, options_json, cb) => {
                debug!(target: "non_secrets_command_executor", "GetRecord command received");
                cb(self.get_record(handle, &type_, &id, &options_json), self.metrics_service.clone());
            }
            NonSecretsCommand::OpenSearch(handle, type_, query_json, options_json, cb) => {
                debug!(target: "non_secrets_command_executor", "OpenSearch command received");
                cb(self.open_search(handle, &type_, &query_json, &options_json), self.metrics_service.clone());
            }
            NonSecretsCommand::FetchSearchNextRecords(wallet_handle, wallet_search_handle, count, cb) => {
                debug!(target: "non_secrets_command_executor", "SearchNextRecords command received");
                cb(self.fetch_search_next_records(wallet_handle, wallet_search_handle, count), self.metrics_service.clone());
            }
            NonSecretsCommand::CloseSearch(wallet_search_handle, cb) => {
                debug!(target: "non_secrets_command_executor", "CloseSearch command received");
                cb(self.close_search(wallet_search_handle), self.metrics_service.clone());
            }
        };
    }

    fn add_record(&self,
                  wallet_handle: WalletHandle,
                  type_: &str,
                  id: &str,
                  value: &str,
                  tags: Option<&Tags>) -> IndyResult<()> {
        trace!("add_record >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, value: {:?}, tags: {:?}", wallet_handle, type_, id, value, tags);

        self._check_type(type_)?;

        self.wallet_service.add_record(wallet_handle, type_, id, value, tags.unwrap_or(&Tags::new()))?;

        trace!("add_record <<< res: ()");

        Ok(())
    }

    fn update_record_value(&self,
                           wallet_handle: WalletHandle,
                           type_: &str,
                           id: &str,
                           value: &str) -> IndyResult<()> {
        trace!("update_record_value >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, value: {:?}", wallet_handle, type_, id, value);

        self._check_type(type_)?;

        self.wallet_service.update_record_value(wallet_handle, type_, id, value)?;

        trace!("update_record_value <<< res: ()");

        Ok(())
    }

    fn update_record_tags(&self,
                          wallet_handle: WalletHandle,
                          type_: &str,
                          id: &str,
                          tags: &Tags) -> IndyResult<()> {
        trace!("update_record_tags >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, tags: {:?}", wallet_handle, type_, id, tags);

        self._check_type(type_)?;

        self.wallet_service.update_record_tags(wallet_handle, type_, id, &tags)?;

        trace!("update_record_tags <<< res: ()");

        Ok(())
    }

    fn add_record_tags(&self,
                       wallet_handle: WalletHandle,
                       type_: &str,
                       id: &str,
                       tags: &Tags) -> IndyResult<()> {
        trace!("add_record_tags >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, tags: {:?}", wallet_handle, type_, id, tags);

        self._check_type(type_)?;

        self.wallet_service.add_record_tags(wallet_handle, type_, id, &tags)?;

        trace!("add_record_tags <<< res: ()");

        Ok(())
    }

    fn delete_record_tags(&self,
                          wallet_handle: WalletHandle,
                          type_: &str,
                          id: &str,
                          tag_names_json: &str) -> IndyResult<()> {
        trace!("delete_record_tags >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, tag_names_json: {:?}", wallet_handle, type_, id, tag_names_json);

        self._check_type(type_)?;

        let tag_names: Vec<&str> = serde_json::from_str(tag_names_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Cannot deserialize tag names")?;

        self.wallet_service.delete_record_tags(wallet_handle, type_, id, &tag_names)?;

        trace!("delete_record_tags <<< res: ()");

        Ok(())
    }

    fn delete_record(&self,
                     wallet_handle: WalletHandle,
                     type_: &str,
                     id: &str) -> IndyResult<()> {
        trace!("delete_record >>> wallet_handle: {:?}, type_: {:?}, id: {:?}", wallet_handle, type_, id);

        self._check_type(type_)?;

        self.wallet_service.delete_record(wallet_handle, type_, id)?;

        trace!("delete_record <<< res: ()");

        Ok(())
    }

    fn get_record(&self,
                  wallet_handle: WalletHandle,
                  type_: &str,
                  id: &str,
                  options_json: &str) -> IndyResult<String> {
        trace!("get_record >>> wallet_handle: {:?}, type_: {:?}, id: {:?}, options_json: {:?}", wallet_handle, type_, id, options_json);

        self._check_type(type_)?;

        serde_json::from_str::<RecordOptions>(options_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Cannot deserialize options")?;

        let record = self.wallet_service.get_record(wallet_handle, type_, id, &options_json)?;

        let res = serde_json::to_string(&record)
            .to_indy(IndyErrorKind::InvalidStructure, "Cannot serialize WalletRecord")?;

        trace!("get_record <<< res: {:?}", res);

        Ok(res)
    }

    fn open_search(&self,
                   wallet_handle: WalletHandle,
                   type_: &str,
                   query_json: &str,
                   options_json: &str) -> IndyResult<SearchHandle> {
        trace!("open_search >>> wallet_handle: {:?}, type_: {:?}, query_json: {:?}, options_json: {:?}", wallet_handle, type_, query_json, options_json);

        self._check_type(type_)?;

        serde_json::from_str::<SearchOptions>(options_json)
            .to_indy(IndyErrorKind::InvalidStructure, "Cannot deserialize options")?;

        let search = self.wallet_service.search_records(wallet_handle, type_, query_json, &options_json)?;

        let search_handle = next_search_handle();

        self.searches.borrow_mut().insert(search_handle, Box::new(search));

        trace!("open_search <<< res: {:?}", search_handle);

        Ok(search_handle)
    }

    fn fetch_search_next_records(&self,
                                 wallet_handle: WalletHandle,
                                 wallet_search_handle: SearchHandle,
                                 count: usize) -> IndyResult<String> {
        trace!("fetch_search_next_records >>> wallet_handle: {:?}, wallet_search_handle: {:?}, count: {:?}", wallet_handle, wallet_search_handle, count);

        let mut searches = self.searches.borrow_mut();
        let search = searches.get_mut(&wallet_search_handle)
            .ok_or_else(||err_msg(IndyErrorKind::InvalidWalletHandle, format!("Unknown WalletSearch handle: {:?}", wallet_search_handle)))?;

        let mut records: Vec<WalletRecord> = Vec::new();
        for _ in 0..count {
            match search.fetch_next_record()? {
                Some(record) => records.push(record),
                None => break
            }
        }

        let search_result = SearchRecords {
            total_count: search.get_total_count()?,
            records: if records.is_empty() { None } else { Some(records) }
        };

        let res = serde_json::to_string(&search_result)
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize SearchRecords")?;

        trace!("fetch_search_next_records <<< res: {:?}", res);
        Ok(res)
    }

    fn close_search(&self,
                    wallet_search_handle: SearchHandle) -> IndyResult<()> {
        trace!("close_search >>> wallet_search_handle: {:?}", wallet_search_handle);

        match self.searches.borrow_mut().remove(&wallet_search_handle) {
            Some(_) => Ok(()),
            None => Err(err_msg(IndyErrorKind::InvalidWalletHandle, format!("Wallet Search Handle is invalid: {:?}", wallet_search_handle)))
        }?;

        trace!("close_search <<< res: ()");

        Ok(())
    }

    fn _check_type(&self, type_: &str) -> IndyResult<()> {
        if type_.starts_with(WalletService::PREFIX) {
            return Err(err_msg(IndyErrorKind::WalletAccessFailed, format!("Record of type \"{}\" is not available for fetching", type_)));
        }
        Ok(())
    }
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchRecords {
    pub total_count: Option<usize>,
    pub records: Option<Vec<WalletRecord>>
}
