use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indy_api_types::wallet::*;
use crate::commands::{Command, CommandExecutor};
use indy_api_types::domain::wallet::{Config, Credentials, ExportConfig, KeyConfig};
use indy_api_types::errors::prelude::*;
use crate::services::crypto::CryptoService;
use indy_wallet::{KeyDerivationData, WalletService, Metadata};
use indy_utils::crypto::{chacha20poly1305_ietf, randombytes};
use indy_utils::crypto::chacha20poly1305_ietf::Key as MasterKey;
use indy_api_types::{WalletHandle, CallbackHandle};
use rand::Rng;
use rust_base58::ToBase58;

type DeriveKeyResult<T> = IndyResult<T>;

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
                       Box<dyn Fn(IndyResult<()>) + Send>),
    Create(Config, // config
           Credentials, // credentials
           Box<dyn Fn(IndyResult<()>) + Send>),
    CreateContinue(Config, // config
                   Credentials, // credentials
                   KeyDerivationData,
                   DeriveKeyResult<MasterKey>, // derive_key_result
                   CallbackHandle),
    Open(Config, // config
         Credentials, // credentials
         Box<dyn Fn(IndyResult<WalletHandle>) + Send>),
    OpenContinue(WalletHandle,
                 DeriveKeyResult<(MasterKey, Option<MasterKey>)>, // derive_key_result
    ),
    Close(WalletHandle,
          Box<dyn Fn(IndyResult<()>) + Send>),
    Delete(Config, // config
           Credentials, // credentials
           Box<dyn Fn(IndyResult<()>) + Send>),
    DeleteContinue(Config, // config
                   Credentials, // credentials
                   Metadata, // credentials
                   DeriveKeyResult<MasterKey>,
                   CallbackHandle),
    Export(WalletHandle,
           ExportConfig, // export config
           Box<dyn Fn(IndyResult<()>) + Send>),
    ExportContinue(WalletHandle,
                   ExportConfig, // export config
                   KeyDerivationData,
                   DeriveKeyResult<MasterKey>,
                   CallbackHandle),
    Import(Config, // config
           Credentials, // credentials
           ExportConfig, // import config
           Box<dyn Fn(IndyResult<()>) + Send>),
    ImportContinue(Config, // config
                   Credentials, // credentials
                   DeriveKeyResult<(MasterKey, MasterKey)>, // derive_key_result
                   WalletHandle,
                   CallbackHandle
    ),
    GenerateKey(Option<KeyConfig>, // config
                Box<dyn Fn(IndyResult<String>) + Send>),
    DeriveKey(KeyDerivationData,
              Box<dyn Fn(DeriveKeyResult<MasterKey>) + Send>),
}

macro_rules! get_cb {
    ($self_:ident, $e:expr) => (match $self_.pending_callbacks.borrow_mut().remove(&$e) {
        Some(val) => val,
        None => return error!("No pending command for id: {}", $e)
    });
}

pub struct WalletCommandExecutor {
    wallet_service: Rc<WalletService>,
    crypto_service: Rc<CryptoService>,
    open_callbacks: RefCell<HashMap<WalletHandle, Box<dyn Fn(IndyResult<WalletHandle>) + Send>>>,
    pending_callbacks: RefCell<HashMap<CallbackHandle, Box<dyn Fn(IndyResult<()>) + Send>>>
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
                self._create_continue(cb_id, &config, &credentials, key_data, key_result)
            }
            WalletCommand::Open(config, credentials, cb) => {
                debug!(target: "wallet_command_executor", "Open command received");
                self._open(&config, &credentials, cb);
            }
            WalletCommand::OpenContinue(wallet_handle, key_result) => {
                debug!(target: "wallet_command_executor", "OpenContinue command received");
                self._open_continue(wallet_handle, key_result)
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
                debug!(target: "wallet_command_executor", "DeleteContinue command received");
                self._delete_continue(cb_id, &config, &credentials, &metadata, key_result)
            }
            WalletCommand::Export(wallet_handle, export_config, cb) => {
                debug!(target: "wallet_command_executor", "Export command received");
                self._export(wallet_handle, &export_config, cb)
            }
            WalletCommand::ExportContinue(wallet_handle, export_config, key_data, key_result, cb_id) => {
                debug!(target: "wallet_command_executor", "ExportContinue command received");
                self._export_continue(cb_id, wallet_handle, &export_config, key_data, key_result)
            }
            WalletCommand::Import(config, credentials, import_config, cb) => {
                debug!(target: "wallet_command_executor", "Import command received");
                self._import(&config, &credentials, &import_config, cb);
            }
            WalletCommand::ImportContinue(config, credential, key_result, wallet_handle, cb_id) => {
                debug!(target: "wallet_command_executor", "ImportContinue command received");
                self._import_continue(cb_id, wallet_handle, &config, &credential, key_result);
            }
            WalletCommand::GenerateKey(config, cb) => {
                debug!(target: "wallet_command_executor", "DeriveKey command received");
                cb(self._generate_key(config.as_ref()));
            }
            WalletCommand::DeriveKey(key_data, cb) => {
                debug!(target: "wallet_command_executor", "DeriveKey command received");
                self._derive_key(key_data, cb);
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
                      free_search: WalletFreeSearch) -> IndyResult<()> {
        trace!("_register_type >>> type_: {:?}", type_);

        self
            .wallet_service
            .register_wallet_storage(
                type_, create, open, close, delete, add_record, update_record_value, update_record_tags,
                add_record_tags, delete_record_tags, delete_record, get_record, get_record_id, get_record_type,
                get_record_value, get_record_tags, free_record, get_storage_metadata, set_storage_metadata,
                free_storage_metadata, search_records, search_all_records,
                get_search_total_count, fetch_search_next_record, free_search)?;

        trace!("_register_type <<< res: ()");
        Ok(())
    }

    fn _create(&self,
               config: &Config,
               credentials: &Credentials,
               cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        trace!("_create >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let key_data = KeyDerivationData::from_passphrase_with_new_salt(&credentials.key, &credentials.key_derivation_method);

        let cb_id : CallbackHandle = indy_utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let config = config.clone();
        let credentials = credentials.clone();

        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
                key_data.clone(),
                Box::new(move |master_key_res| {
                    CommandExecutor::instance().send(
                        Command::Wallet(
                            WalletCommand::CreateContinue(
                                config.clone(),
                                credentials.clone(),
                                key_data.clone(),
                                master_key_res,
                                cb_id
                            ))).unwrap();
                }))
            )).unwrap();

        trace!("_create <<<");
    }

    fn _create_continue(&self,
                        cb_id: CallbackHandle,
                        config: &Config,
                        credentials: &Credentials,
                        key_data: KeyDerivationData,
                        key_result: DeriveKeyResult<MasterKey>) {
        let cb = get_cb!(self, cb_id );
        cb(key_result
            .and_then(|key| self.wallet_service.create_wallet(config, credentials, (&key_data,& key))))
    }

    fn _open(&self,
             config: &Config,
             credentials: &Credentials,
             cb: Box<dyn Fn(IndyResult<WalletHandle>) + Send>) {
        trace!("_open >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let (wallet_handle, key_derivation_data, rekey_data) = try_cb!(self.wallet_service.open_wallet_prepare(config, credentials), cb);

        self.open_callbacks.borrow_mut().insert(wallet_handle, cb);

        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
                key_derivation_data,
                Box::new(move |key_result| {
                    match (key_result, rekey_data.clone()) {
                        (Ok(key_result), Some(rekey_data)) => {
                            WalletCommandExecutor::_derive_rekey_and_continue(wallet_handle, key_result, rekey_data)
                        }
                        (key_result, _) => {
                            let key_result = key_result.map(|kr| (kr, None));
                            WalletCommandExecutor::_send_open_continue(wallet_handle, key_result)
                        }
                    }
                }),
            ))
        ).unwrap();

        let res = wallet_handle;

        trace!("_open <<< res: {:?}", res);
    }

    fn _derive_rekey_and_continue(wallet_handle: WalletHandle, key_result: MasterKey, rekey_data: KeyDerivationData) {
        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
                rekey_data,
                Box::new(move |rekey_result| {
                    let key_result = key_result.clone();
                    let key_result = rekey_result.map(move |rekey_result| (key_result, Some(rekey_result)));
                    WalletCommandExecutor::_send_open_continue(wallet_handle, key_result)
                }),
            ))
        ).unwrap();
    }

    fn _send_open_continue(wallet_handle: WalletHandle, key_result: DeriveKeyResult<(MasterKey, Option<MasterKey>)>) {
        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::OpenContinue(
                wallet_handle,
                key_result,
            ))
        ).unwrap();
    }

    fn _open_continue(&self,
                      wallet_handle: WalletHandle,
                      key_result: DeriveKeyResult<(MasterKey, Option<MasterKey>)>) {
        let cb = self.open_callbacks.borrow_mut().remove(&wallet_handle).unwrap();
        cb(key_result
            .and_then(|(key, rekey)| self.wallet_service.open_wallet_continue(wallet_handle, (&key, rekey.as_ref()))))
    }

    fn _close(&self,
              wallet_handle: WalletHandle) -> IndyResult<()> {
        trace!("_close >>> handle: {:?}", wallet_handle);

        self.wallet_service.close_wallet(wallet_handle)?;

        trace!("_close <<< res: ()");
        Ok(())
    }

    fn _delete(&self,
               config: &Config,
               credentials: &Credentials,
               cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        trace!("_delete >>> config: {:?}, credentials: {:?}", config, secret!(credentials));

        let (metadata, key_derivation_data) = try_cb!(self.wallet_service.delete_wallet_prepare(&config, &credentials), cb);

        let cb_id: CallbackHandle = indy_utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let config = config.clone();
        let credentials = credentials.clone();

        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
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
            ))
        ).unwrap();

        trace!("_delete <<<");
    }

    fn _delete_continue(&self,
                        cb_id: CallbackHandle,
                        config: &Config,
                        credentials: &Credentials,
                        metadata: &Metadata,
                        key_result: DeriveKeyResult<MasterKey>) {
        let cb = get_cb!(self, cb_id);
        cb(key_result
            .and_then(|key| self.wallet_service.delete_wallet_continue(config, credentials, metadata, &key)))
    }

    fn _export(&self,
               wallet_handle: WalletHandle,
               export_config: &ExportConfig,
               cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        trace!("_export >>> handle: {:?}, export_config: {:?}", wallet_handle, secret!(export_config));

        let key_data = KeyDerivationData::from_passphrase_with_new_salt(&export_config.key, &export_config.key_derivation_method);

        let cb_id = indy_utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let export_config = export_config.clone();

        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
                key_data.clone(),
                Box::new(move |master_key_res| {
                    CommandExecutor::instance().send(Command::Wallet(WalletCommand::ExportContinue(
                        wallet_handle,
                        export_config.clone(),
                        key_data.clone(),
                        master_key_res,
                        cb_id,
                    ))).unwrap();
                })
            ))
        ).unwrap();

        trace!("_export <<<");
    }

    fn _export_continue(&self,
                        cb_id: CallbackHandle,
                        wallet_handle: WalletHandle,
                        export_config: &ExportConfig,
                        key_data: KeyDerivationData,
                        key_result: DeriveKeyResult<MasterKey>) {
        let cb = get_cb!(self, cb_id);
        cb(key_result
            .and_then(|key| self.wallet_service.export_wallet(wallet_handle, export_config, 0, (&key_data,& key)))) // TODO - later add proper versioning
    }

    fn _import(&self,
               config: &Config,
               credentials: &Credentials,
               import_config: &ExportConfig,
               cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        trace!("_import >>> config: {:?}, credentials: {:?}, import_config: {:?}",
               config, secret!(credentials), secret!(import_config));

        let (wallet_handle, key_data, import_key_data) = try_cb!(self.wallet_service.import_wallet_prepare(&config, &credentials, &import_config), cb);

        let cb_id : CallbackHandle = indy_utils::sequence::get_next_id();
        self.pending_callbacks.borrow_mut().insert(cb_id, cb);

        let config = config.clone();
        let credentials = credentials.clone();

        CommandExecutor::instance().send(
            Command::Wallet(WalletCommand::DeriveKey(
                import_key_data,
                Box::new(move |import_key_result| {
                    let config = config.clone();
                    let credentials = credentials.clone();

                    CommandExecutor::instance().send(
                        Command::Wallet(WalletCommand::DeriveKey(
                            key_data.clone(),
                            Box::new(move |key_result| {
                                let import_key_result = import_key_result.clone();
                                CommandExecutor::instance().send(Command::Wallet(WalletCommand::ImportContinue(
                                    config.clone(),
                                    credentials.clone(),
                                    import_key_result.and_then(|import_key| key_result.map(|key| (import_key, key))),
                                    wallet_handle,
                                    cb_id
                                ))).unwrap();
                            }),
                        ))
                    ).unwrap();
                }),
            ))
        ).unwrap();

        trace!("_import <<<");
    }

    fn _import_continue(&self,
                        cb_id: CallbackHandle,
                        wallet_handle: WalletHandle,
                        config: &Config,
                        credential: &Credentials,
                        key_result: DeriveKeyResult<(MasterKey, MasterKey)>) {
        let cb = get_cb!(self, cb_id);
        cb(key_result
            .and_then(|key| self.wallet_service.import_wallet_continue(wallet_handle, &config, &credential, key)))
    }

    fn _generate_key(&self,
                     config: Option<&KeyConfig>) -> IndyResult<String> {
        trace!("_generate_key >>>config: {:?}", secret!(config));

        let seed = config.and_then(|config| config.seed.as_ref().map(String::as_str));

        let key = match self.crypto_service.convert_seed(seed)? {
            Some(seed) => randombytes::randombytes_deterministic(chacha20poly1305_ietf::KEYBYTES, &randombytes::Seed::from_slice(&seed[..])?),
            None => rand::thread_rng().gen::<[u8; chacha20poly1305_ietf::KEYBYTES]>().to_vec()
        };

        let res = key[..].to_base58();

        trace!("_generate_key <<< res: {:?}", res);
        Ok(res)
    }

    fn _derive_key(&self, key_data: KeyDerivationData, cb: Box<dyn Fn(DeriveKeyResult<MasterKey>) + Send>){
        crate::commands::THREADPOOL.lock().unwrap().execute(move || cb(key_data.calc_master_key()));
    }
}
