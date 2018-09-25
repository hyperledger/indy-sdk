extern crate libc;
extern crate serde_json;
extern crate indy_crypto;

use errors::indy::IndyError;
use errors::wallet::WalletError;
use commands::{Command, CommandExecutor};
use services::wallet::{WalletService, KeyDerivationData};
use services::crypto::CryptoService;
use api::wallet::*;
use utils::crypto::{base58, randombytes, chacha20poly1305_ietf};
use utils::crypto::chacha20poly1305_ietf::Key as MasterKey;
use domain::wallet::{KeyConfig, Config, Credentials, ExportConfig, Metadata};

use std::rc::Rc;
use std::cell::RefCell;
use std::result;
use std::collections::HashMap;

type Result<T> = result::Result<T, IndyError>;

pub enum WalletCommand {
    RegisterWalletType(String, // type_
                       WalletCreate, // create
                       WalletOpen, // open
                       WalletClose, // close
                       WalletDelete, // delete
                       WalletAddRecord, // add record
                       WalletUpdateRecordValue, // update record value
                       WalletUpdateRecordTags, // update record value
                       WalletAddRecordTags, // add record tags
                       WalletDeleteRecordTags, // delete record tags
                       WalletDeleteRecord, // delete record
                       WalletGetRecord, // get record
                       WalletGetRecordId, // get record id
                       WalletGetRecordType, // get record id
                       WalletGetRecordValue, // get record value
                       WalletGetRecordTags, // get record tags
                       WalletFreeRecord, // free record
                       WalletGetStorageMetadata, // get storage metadata
                       WalletSetStorageMetadata, // set storage metadata
                       WalletFreeStorageMetadata, // free storage metadata
                       WalletSearchRecords, // search records
                       WalletSearchAllRecords, // search all records
                       WalletGetSearchTotalCount, // get search total count
                       WalletFetchSearchNextRecord, // fetch search next record
                       WalletFreeSearch, // free search
                       Box<Fn(Result<()>) + Send>),
    Create(Config, // config
           Credentials, // credentials
           Box<Fn(Result<()>) + Send>),
    CreateContinue(Config, // config
                   Credentials, // credentials
                   KeyDerivationData,
                   result::Result<MasterKey, ::errors::common::CommonError>,
                   i32),
    Open(Config, // config
         Credentials, // credentials
         Box<Fn(Result<i32>) + Send>),
    OpenContinue(i32, // wallet handle
                 result::Result<(MasterKey, Option<MasterKey>),
                     ::errors::common::CommonError>, // derive_key_result
    ),
    Close(i32, // handle
          Box<Fn(Result<()>) + Send>),
    Delete(Config, // config
           Credentials, // credentials
           Box<Fn(Result<()>) + Send>),
    DeleteContinue(Config, // config
                   Credentials, // credentials
                   Metadata, // credentials
                   result::Result<chacha20poly1305_ietf::Key, ::errors::common::CommonError>,
                   i32),
    Export(i32, // wallet_handle
           ExportConfig, // export config
           Box<Fn(Result<()>) + Send>),
    ExportContinue(i32, // wallet_handle
                   ExportConfig, // export config
                   KeyDerivationData,
                   result::Result<MasterKey, ::errors::common::CommonError>,
                   i32),
    Import(Config, // config
           Credentials, // credentials
           ExportConfig, // import config
           Box<Fn(Result<()>) + Send>),
    ImportContinue(Config, // config
                   Credentials, // credentials
                   result::Result<(MasterKey, MasterKey),
                       ::errors::common::CommonError>, // derive_key_result
                   i32, // handle
    ),
    GenerateKey(Option<KeyConfig>, // config
                Box<Fn(Result<String>) + Send>),
}

pub struct WalletCommandExecutor {
    wallet_service: Rc<WalletService>,
    crypto_service: Rc<CryptoService>,
    open_callbacks: RefCell<HashMap<i32, Box<Fn(Result<i32>) + Send>>>,
    pending_callbacks: RefCell<HashMap<i32, Box<Fn(Result<()>) + Send>>>
}

impl WalletCommandExecutor {
    pub fn new(wallet_service: Rc<WalletService>, crypto_service: Rc<CryptoService>) -> WalletCommandExecutor {
        WalletCommandExecutor {
            wallet_service,
            crypto_service,
            open_callbacks: RefCell::new(HashMap::new()),
            pending_callbacks: RefCell::new(HashMap::new())
        }
    }

    pub fn execute(&self, command: WalletCommand) {
        match command {
            WalletCommand::RegisterWalletType(type_, create, open, close, delete, add_record,
                                              update_record_value, update_record_tags, add_record_tags,
                                              delete_record_tags, delete_record, get_record, get_record_id, get_record_type,
                                              get_record_value, get_record_tags, free_record, get_storage_metadata, set_storage_metadata,
                                              free_storage_metadata, search_records, search_all_records, get_search_total_count,
                                              fetch_search_next_record, free_search, cb) => {
                debug!(target: "wallet_command_executor", "RegisterWalletType command received");
                cb(self._register_type(&type_, create, open, close, delete, add_record,
                                       update_record_value, update_record_tags, add_record_tags,
                                       delete_record_tags, delete_record, get_record, get_record_id, get_record_type,
                                       get_record_value, get_record_tags, free_record, get_storage_metadata, set_storage_metadata,
                                       free_storage_metadata, search_records, search_all_records, get_search_total_count,
                                       fetch_search_next_record, free_search));
            }
            WalletCommand::Create(config, credentials, cb) => {
                debug!(target: "wallet_command_executor", "Create command received");
                self._create(&config, &credentials, cb)
            }
            WalletCommand::CreateContinue(config, credentials, key_data, key_result, cb_id) => {
                debug!(target: "wallet_command_executor", "CreateContinue command received");
                let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");
                cb(key_result.map_err(WalletError::from).and_then(|key| {
                    self.wallet_service.create_wallet(&config, (&credentials, key_data, key))
                }).map_err(IndyError::from))
            }
            WalletCommand::Open(config, credentials, cb) => {
                debug!(target: "wallet_command_executor", "Open command received");
                self._open(&config, &credentials, cb);
            }
            WalletCommand::OpenContinue(wallet_handle, key_result) => {
                let cb = self.open_callbacks.borrow_mut().remove(&wallet_handle).unwrap();
                cb(self.wallet_service.open_wallet_continue(wallet_handle, key_result).map_err(IndyError::from))
            }
            WalletCommand::Close(handle, cb) => {
                debug!(target: "wallet_command_executor", "Close command received");
                cb(self._close(handle));
            }
            WalletCommand::Delete(config, credentials, cb) => {
                debug!(target: "wallet_command_executor", "Delete command received");
                self._delete(&config, &credentials, cb)
            }
            WalletCommand::DeleteContinue(config, credentials, metadata, key_result, cb_id) => {
                let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).unwrap();
                cb(self.wallet_service.delete_wallet_continue(&config, &credentials, &metadata, key_result).map_err(IndyError::from))
            }
            WalletCommand::Export(wallet_handle, export_config, cb) => {
                debug!(target: "wallet_command_executor", "Export command received");
                self._export(wallet_handle, &export_config, cb)
            }
            WalletCommand::ExportContinue(wallet_handle, export_config, key_data, key_result, cb_id) => {
                debug!(target: "wallet_command_executor", "ExportContinue command received");
                let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME INVALID STATE");
                cb(key_result.map_err(WalletError::from).and_then(|key|
                    // TODO - later add proper versioning
                    self.wallet_service.export_wallet(wallet_handle, &export_config, 0, (key_data, key))
                ).map_err(IndyError::from))
            }
            WalletCommand::Import(config, credentials, import_config, cb) => {
                debug!(target: "wallet_command_executor", "Import command received");
                self._import(&config, &credentials, &import_config, cb);
            }
            WalletCommand::ImportContinue(config, credential, key_result, cb_id) => {
                let cb = self.pending_callbacks.borrow_mut().remove(&cb_id).expect("FIXME ERR TRACE");
                cb(self.wallet_service.import_wallet_continue(&config, &credential, cb_id, key_result).map_err(IndyError::from))
            }
            WalletCommand::GenerateKey(config, cb) => {
                debug!(target: "wallet_command_executor", "DeriveKey command received");
                cb(self._generate_key(config.as_ref()));
            }
        };
    }

    fn _register_type(&self,
                      type_: &str,
                      create: WalletCreate,
                      open: WalletOpen,
                      close: WalletClose,
                      delete: WalletDelete,
                      add_record: WalletAddRecord,
                      update_record_value: WalletUpdateRecordValue,
                      update_record_tags: WalletUpdateRecordTags,
                      add_record_tags: WalletAddRecordTags,
                      delete_record_tags: WalletDeleteRecordTags,
                      delete_record: WalletDeleteRecord,
                      get_record: WalletGetRecord,
                      get_record_id: WalletGetRecordId,
                      get_record_type: WalletGetRecordType,
                      get_record_value: WalletGetRecordValue,
                      get_record_tags: WalletGetRecordTags,
                      free_record: WalletFreeRecord,
                      get_storage_metadata: WalletGetStorageMetadata,
                      set_storage_metadata: WalletSetStorageMetadata,
                      free_storage_metadata: WalletFreeStorageMetadata,
                      search_records: WalletSearchRecords,
                      search_all_records: WalletSearchAllRecords,
                      get_search_total_count: WalletGetSearchTotalCount,
                      fetch_search_next_record: WalletFetchSearchNextRecord,
                      free_search: WalletFreeSearch) -> Result<()> {
        trace!("_register_type >>> type_: {:?}", type_);

        let res = self
            .wallet_service
            .register_wallet_storage(
                type_, create, open, close, delete, add_record, update_record_value, update_record_tags,
                add_record_tags, delete_record_tags, delete_record, get_record, get_record_id, get_record_type,
                get_record_value, get_record_tags, free_record, get_storage_metadata, set_storage_metadata,
                free_storage_metadata, search_records, search_all_records,
                get_search_total_count, fetch_search_next_record, free_search)?;

        trace!("_register_type <<< res: {:?}", res);
        Ok(res)
    }

    fn _create(&self,
               config: &Config,
               credentials: &Credentials,
               cb: Box<Fn(Result<()>) + Send>) {
        trace!("_create >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let key_data = KeyDerivationData::from_passphrase_with_new_salt(&credentials.key, &credentials.key_derivation_method);

        let cb_id = ::utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let config = config.clone();
        let credentials = credentials.clone();

        CommandExecutor::instance().send(Command::DeriveKey(
            key_data.clone(),
            Box::new(move |master_key_res| {
                CommandExecutor::instance().send(
                    Command::Wallet(
                        WalletCommand::CreateContinue(
                            config.clone(),
                            credentials.clone(),
                            key_data.clone(),
                            master_key_res,
                            cb_id,
                        ))).unwrap();
            }))).unwrap();

        trace!("_create <<<");
    }

    fn _open(&self,
             config: &Config,
             credentials: &Credentials,
             cb: Box<Fn(Result<i32>) + Send>) {
        trace!("_open >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let (wallet_handle, key_derivation_data, rekey_data) =
            match self.wallet_service.open_wallet_prepare(config, credentials) {
                Ok((wallet_handle, key_derivation_data, rekey_data)) => (wallet_handle, key_derivation_data, rekey_data),
                Err(err) => return cb(Err(IndyError::from(err)))
            };

        self.open_callbacks.borrow_mut().insert(wallet_handle, cb);

        CommandExecutor::instance().send(Command::DeriveKey(
            key_derivation_data,
            Box::new(move |key_result| {
                let key_result = key_result.clone();
                match (key_result, rekey_data.clone()) {
                    (Ok(key_result), Some(rekey_data)) => {
                        CommandExecutor::instance().send(
                            Command::DeriveKey(
                                rekey_data,
                                Box::new(move |rekey_result| {
                                    let key_result = key_result.clone();
                                    CommandExecutor::instance().send(
                                        Command::Wallet(WalletCommand::OpenContinue(
                                            wallet_handle,
                                            rekey_result.map(move |rekey_result| (key_result.clone(), Some(rekey_result))),
                                        ))
                                    ).unwrap();
                                }),
                            )
                        ).unwrap();
                    }
                    (key_result, _) => {
                        CommandExecutor::instance().send(
                            Command::Wallet(WalletCommand::OpenContinue(
                                wallet_handle,
                                key_result.map(|kr| (kr, None)),
                            ))
                        ).unwrap()
                    }
                }
            }),
        )).unwrap();

        let res = wallet_handle;

        trace!("_open <<< res: {:?}", res);
    }

    fn _close(&self,
              handle: i32) -> Result<()> {
        trace!("_close >>> handle: {:?}", handle);

        let res = self.wallet_service.close_wallet(handle)?;

        trace!("_close <<< res: {:?}", res);
        Ok(res)
    }

    fn _delete(&self,
               config: &Config,
               credentials: &Credentials,
               cb: Box<Fn(Result<()>) + Send>) {
        trace!("_delete >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let (metadata, key_derivation_data) = match  self.wallet_service.delete_wallet_prepare(&config, &credentials) {
            Ok((metadata, key_derivation_data)) => ((metadata, key_derivation_data)),
            Err(err) => return cb(Err(IndyError::from(err)))
        };

        let cb_id = ::utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let config = config.clone();
        let credentials = credentials.clone();

        CommandExecutor::instance().send(Command::DeriveKey(
            key_derivation_data,
            Box::new(move |key_result| {
                let key_result = key_result.clone();
                CommandExecutor::instance().send(
                    Command::Wallet(WalletCommand::DeleteContinue(
                        config.clone(),
                        credentials.clone(),
                        metadata.clone(),
                        key_result,
                        cb_id)
                    )).unwrap()
            }),
        )).unwrap();

        trace!("_delete <<<");
    }

    fn _export(&self,
               wallet_handle: i32,
               export_config: &ExportConfig,
               cb: Box<Fn(Result<()>) + Send>) {
        trace!("_export >>> handle: {:?}, export_config: {:?}", wallet_handle, secret!(export_config));

        let key_data = KeyDerivationData::from_passphrase_with_new_salt(&export_config.key, &export_config.key_derivation_method);
        let cb_id = ::utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let export_config = export_config.clone();

        CommandExecutor::instance().send(Command::DeriveKey(
            key_data.clone(),
            Box::new(move |master_key_res| {
                CommandExecutor::instance().send(Command::Wallet(WalletCommand::ExportContinue(
                    wallet_handle,
                    export_config.clone(),
                    key_data.clone(),
                    master_key_res,
                    cb_id,
                ))).unwrap();
            }))).unwrap();

        trace!("_export <<<");
    }

    fn _import(&self,
               config: &Config,
               credentials: &Credentials,
               import_config: &ExportConfig,
               cb: Box<Fn(Result<()>) + Send>) {
        trace!("_import >>> config: {:?}, credentials: {:?}, import_config: {:?}",
               config, secret!(credentials), secret!(import_config));

        let config = config.clone();
        let credentials = credentials.clone();

        let (wallet_handle, key_data, import_key_derivation_data) = match  self.wallet_service.import_wallet_prepare(&config, &credentials, &import_config) {
            Ok((wallet_handle, key_data, import_key_derivation_data)) => ((wallet_handle, key_data, import_key_derivation_data)),
            Err(err) => return cb(Err(IndyError::from(err)))
        };

        self.pending_callbacks.borrow_mut().insert(wallet_handle, cb);

        CommandExecutor::instance().send(Command::DeriveKey(
            import_key_derivation_data,
            Box::new(move |import_key_result| {
                let config = config.clone();
                let credentials = credentials.clone();

                CommandExecutor::instance().send(Command::DeriveKey(
                    key_data.clone(),
                    Box::new(move |key_result| {
                        let import_key_result = import_key_result.clone();
                        CommandExecutor::instance().send(Command::Wallet(WalletCommand::ImportContinue(
                            config.clone(),
                            credentials.clone(),
                            import_key_result.and_then(|import_key| key_result.map(|key| (import_key, key))),
                            wallet_handle,
                        ))).unwrap();
                    }),
                )).unwrap();
            }),
        )).expect("FIXME INVALID STATE");

        trace!("_import <<<");
    }

    fn _generate_key(&self,
                     config: Option<&KeyConfig>) -> Result<String> {
        trace!("_generate_key >>>config: {:?}", secret!(config));

        let seed = config.and_then(|config| config.seed.as_ref().map(String::as_str));

        let key = match self.crypto_service.convert_seed(seed)? {
            Some(seed) => randombytes::randombytes_deterministic(chacha20poly1305_ietf::KEYBYTES, &randombytes::Seed::from_slice(&seed[..])?),
            None => randombytes::randombytes(chacha20poly1305_ietf::KEYBYTES)
        };

        let res = base58::encode(&key[..]);

        trace!("_generate_key <<< res: {:?}", res);
        Ok(res)
    }
}
