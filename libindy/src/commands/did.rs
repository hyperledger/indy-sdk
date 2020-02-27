use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde_json;

use crate::commands::{Command, CommandExecutor, BoxedCallbackStringStringSend};
use crate::commands::ledger::LedgerCommand;
use indy_api_types::domain::crypto::did::{Did, DidValue, DidMetadata, DidWithMeta, MyDidInfo, TemporaryDid, TheirDid, TheirDidInfo, DidMethod};
use indy_api_types::domain::crypto::key::{KeyInfo, Key};
use crate::domain::ledger::attrib::{AttribData, Endpoint, GetAttrReplyResult};
use crate::domain::ledger::nym::{GetNymReplyResult, GetNymResultDataV0};
use crate::domain::ledger::response::Reply;
use crate::domain::pairwise::Pairwise;
use indy_api_types::errors::prelude::*;
use crate::services::ledger::LedgerService;
use indy_wallet::{RecordOptions, SearchOptions, WalletService};
use indy_api_types::{WalletHandle, PoolHandle, CommandHandle};
use indy_utils::next_command_handle;
use rust_base58::{FromBase58, ToBase58};
use named_type::NamedType;
use indy_utils::crypto::{validate_did, validate_key, create_key};
use indy_utils::crypto::verkey_builder::build_full_verkey;

pub enum DidCommand {
    CreateAndStoreMyDid(
        WalletHandle,
        MyDidInfo, // my did info
        BoxedCallbackStringStringSend),
    ReplaceKeysStart(
        WalletHandle,
        KeyInfo, // key info
        DidValue, // did
        Box<dyn Fn(IndyResult<String>) + Send>),
    ReplaceKeysApply(
        WalletHandle,
        DidValue, // my did
        Box<dyn Fn(IndyResult<()>) + Send>),
    StoreTheirDid(
        WalletHandle,
        TheirDidInfo, // their did info json
        Box<dyn Fn(IndyResult<()>) + Send>),
    GetMyDidWithMeta(
        WalletHandle,
        DidValue, // my did
        Box<dyn Fn(IndyResult<String>) + Send>),
    ListMyDidsWithMeta(
        WalletHandle,
        Box<dyn Fn(IndyResult<String>) + Send>),
    KeyForDid(
        PoolHandle, // pool handle
        WalletHandle,
        DidValue, // did (my or their)
        Box<dyn Fn(IndyResult<String/*key*/>) + Send>),
    KeyForLocalDid(
        WalletHandle,
        DidValue, // did (my or their)
        Box<dyn Fn(IndyResult<String/*key*/>) + Send>),
    SetEndpointForDid(
        WalletHandle,
        DidValue, // did
        Endpoint, // endpoint address and optional verkey
        Box<dyn Fn(IndyResult<()>) + Send>),
    GetEndpointForDid(
        WalletHandle,
        PoolHandle, // pool handle
        DidValue, // did
        Box<dyn Fn(IndyResult<(String, Option<String>)>) + Send>),
    SetDidMetadata(
        WalletHandle,
        DidValue, // did
        String, // metadata
        Box<dyn Fn(IndyResult<()>) + Send>),
    GetDidMetadata(
        WalletHandle,
        DidValue, // did
        Box<dyn Fn(IndyResult<String>) + Send>),
    AbbreviateVerkey(
        DidValue, // did
        String, // verkey
        Box<dyn Fn(IndyResult<String>) + Send>),
    // Internal commands
    GetNymAck(
        WalletHandle,
        DidValue, // did
        IndyResult<String>, // GetNym Result
        CommandHandle, // deferred cmd id
    ),
    // Internal commands
    GetAttribAck(
        WalletHandle,
        IndyResult<String>, // GetAttrib Result
        CommandHandle, // deferred cmd id
    ),
    QualifyDid(
        WalletHandle,
        DidValue, // did
        DidMethod, // method
        Box<dyn Fn(IndyResult<String /*full qualified did*/>) + Send>,
    ),
}

macro_rules! ensure_their_did {
    ($self_:ident, $wallet_handle:ident, $pool_handle:ident, $their_did:ident, $deferred_cmd:expr, $cb:ident) => (
            match $self_._wallet_get_their_did($wallet_handle, &$their_did) {
                Ok(val) => val,
                // No their their_did present in the wallet. Defer this command until it is fetched from ledger.
            Err(ref err) if err.kind() == IndyErrorKind::WalletItemNotFound  => return $self_._fetch_their_did_from_ledger($wallet_handle, $pool_handle, &$their_did, $deferred_cmd),
                Err(err) => return $cb(Err(err)),
            }
        );
}

pub struct DidCommandExecutor {
    wallet_service: Rc<WalletService>,
    ledger_service: Rc<LedgerService>,
    deferred_commands: RefCell<HashMap<CommandHandle, DidCommand>>,
}

impl DidCommandExecutor {
    pub fn new(wallet_service: Rc<WalletService>,
               ledger_service: Rc<LedgerService>) -> DidCommandExecutor {
        DidCommandExecutor {
            wallet_service,
            ledger_service,
            deferred_commands: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: DidCommand) {
        match command {
            DidCommand::CreateAndStoreMyDid(wallet_handle, my_did_info, cb) => {
                debug!("CreateAndStoreMyDid command received");
                cb(self.create_and_store_my_did(wallet_handle, &my_did_info));
            }
            DidCommand::ReplaceKeysStart(wallet_handle, key_info, did, cb) => {
                debug!("ReplaceKeysStart command received");
                cb(self.replace_keys_start(wallet_handle, &key_info, &did));
            }
            DidCommand::ReplaceKeysApply(wallet_handle, did, cb) => {
                debug!("ReplaceKeysApply command received");
                cb(self.replace_keys_apply(wallet_handle, &did));
            }
            DidCommand::StoreTheirDid(wallet_handle, their_did_info, cb) => {
                debug!("StoreTheirDid command received");
                cb(self.store_their_did(wallet_handle, &their_did_info));
            }
            DidCommand::GetMyDidWithMeta(wallet_handle, my_did, cb) => {
                debug!("GetMyDidWithMeta command received");
                cb(self.get_my_did_with_meta(wallet_handle, &my_did))
            }
            DidCommand::ListMyDidsWithMeta(wallet_handle, cb) => {
                debug!("ListMyDidsWithMeta command received");
                cb(self.list_my_dids_with_meta(wallet_handle));
            }
            DidCommand::KeyForDid(pool_handle, wallet_handle, did, cb) => {
                debug!("KeyForDid command received");
                self.key_for_did(pool_handle, wallet_handle, did, cb);
            }
            DidCommand::KeyForLocalDid(wallet_handle, did, cb) => {
                debug!("KeyForLocalDid command received");
                cb(self.key_for_local_did(wallet_handle, &did));
            }
            DidCommand::SetEndpointForDid(wallet_handle, did, endpoint, cb) => {
                debug!("SetEndpointForDid command received");
                cb(self.set_endpoint_for_did(wallet_handle, &did, &endpoint));
            }
            DidCommand::GetEndpointForDid(wallet_handle, pool_handle, did, cb) => {
                debug!("GetEndpointForDid command received");
                self.get_endpoint_for_did(wallet_handle, pool_handle, did, cb);
            }
            DidCommand::SetDidMetadata(wallet_handle, did, metadata, cb) => {
                debug!("SetDidMetadata command received");
                cb(self.set_did_metadata(wallet_handle, &did, metadata));
            }
            DidCommand::GetDidMetadata(wallet_handle, did, cb) => {
                debug!("GetDidMetadata command received");
                cb(self.get_did_metadata(wallet_handle, &did));
            }
            DidCommand::AbbreviateVerkey(did, verkey, cb) => {
                debug!("AbbreviateVerkey command received");
                cb(self.abbreviate_verkey(&did, verkey));
            }
            DidCommand::GetNymAck(wallet_handle, did, result, deferred_cmd_id) => {
                debug!("GetNymAck command received");
                self.get_nym_ack(wallet_handle, did, result, deferred_cmd_id);
            }
            DidCommand::GetAttribAck(wallet_handle, result, deferred_cmd_id) => {
                debug!("GetAttribAck command received");
                self.get_attrib_ack(wallet_handle, result, deferred_cmd_id);
            }
            DidCommand::QualifyDid(wallet_handle, did, method, cb) => {
                info!("QualifyDid command received");
                cb(self.qualify_did(wallet_handle, &did, &method));
            }
        };
    }

    fn create_and_store_my_did(&self,
                               wallet_handle: WalletHandle,
                               my_did_info: &MyDidInfo) -> IndyResult<(String, String)> {
        debug!("create_and_store_my_did >>> wallet_handle: {:?}, my_did_info_json: {:?}", wallet_handle, secret!(my_did_info));

        let (did, key) = create_did(&my_did_info)?;

        if let Ok(current_did) = self._wallet_get_my_did(wallet_handle, &did.did) {
            if did.verkey == current_did.verkey {
                return Ok((did.did.0, did.verkey));
            } else {
                return Err(err_msg(IndyErrorKind::DIDAlreadyExists,
                                   format!("DID \"{}\" already exists but with different Verkey. You should specify Seed used for initial generation", did.did.0)));
            }
        }

        self.wallet_service.add_indy_object(wallet_handle, &did.did.0, &did, &HashMap::new())?;
        let _ = self.wallet_service.add_indy_object(wallet_handle, &key.verkey, &key, &HashMap::new()).ok();

        let res = (did.did.0, did.verkey);

        debug!("create_and_store_my_did <<< res: {:?}", res);

        Ok(res)
    }

    fn replace_keys_start(&self,
                          wallet_handle: WalletHandle,
                          key_info: &KeyInfo,
                          my_did: &DidValue) -> IndyResult<String> {
        debug!("replace_keys_start >>> wallet_handle: {:?}, key_info_json: {:?}, my_did: {:?}", wallet_handle, secret!(key_info), my_did);

        validate_did(my_did)?;

        let my_did = self._wallet_get_my_did(wallet_handle, my_did)?;

        let temporary_key = create_key(&key_info)?;
        let my_temporary_did = TemporaryDid { did: my_did.did, verkey: temporary_key.verkey.clone() };

        self.wallet_service.add_indy_object(wallet_handle, &temporary_key.verkey, &temporary_key, &HashMap::new())?;
        self.wallet_service.add_indy_object(wallet_handle, &my_temporary_did.did.0, &my_temporary_did, &HashMap::new())?;

        let res = my_temporary_did.verkey;

        debug!("replace_keys_start <<< res: {:?}", res);

        Ok(res)
    }

    fn replace_keys_apply(&self,
                          wallet_handle: WalletHandle,
                          my_did: &DidValue) -> IndyResult<()> {
        debug!("replace_keys_apply >>> wallet_handle: {:?}, my_did: {:?}", wallet_handle, my_did);

        validate_did(my_did)?;

        let my_did = self._wallet_get_my_did(wallet_handle, my_did)?;
        let my_temporary_did: TemporaryDid =
            self.wallet_service.get_indy_object(wallet_handle, &my_did.did.0, &RecordOptions::id_value())?;

        let my_did = Did::from(my_temporary_did);

        self.wallet_service.update_indy_object(wallet_handle, &my_did.did.0, &my_did)?;
        self.wallet_service.delete_indy_record::<TemporaryDid>(wallet_handle, &my_did.did.0)?;

        debug!("replace_keys_apply <<<");

        Ok(())
    }

    fn store_their_did(&self,
                       wallet_handle: WalletHandle,
                       their_did_info: &TheirDidInfo) -> IndyResult<()> {
        debug!("store_their_did >>> wallet_handle: {:?}, their_did_info: {:?}", wallet_handle, their_did_info);

        let their_did = create_their_did(their_did_info)?;

        self.wallet_service.upsert_indy_object(wallet_handle, &their_did.did.0, &their_did)?;

        debug!("store_their_did <<<");

        Ok(())
    }

    fn get_my_did_with_meta(&self, wallet_handle: WalletHandle, my_did: &DidValue) -> IndyResult<String> {
        debug!("get_my_did_with_meta >>> wallet_handle: {:?}, my_did: {:?}", wallet_handle, my_did);

        let did = self.wallet_service.get_indy_object::<Did>(wallet_handle, &my_did.0, &RecordOptions::id_value())?;
        let metadata = self.wallet_service.get_indy_opt_object::<DidMetadata>(wallet_handle, &did.did.0, &RecordOptions::id_value())?;
        let temp_verkey = self.wallet_service.get_indy_opt_object::<TemporaryDid>(wallet_handle, &did.did.0, &RecordOptions::id_value())?;

        let did_with_meta = DidWithMeta {
            did: did.did,
            verkey: did.verkey,
            temp_verkey: temp_verkey.map(|tv| tv.verkey),
            metadata: metadata.map(|m| m.value),
        };

        let res = serde_json::to_string(&did_with_meta)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize DID")?;

        debug!("get_my_did_with_meta <<< res: {:?}", res);

        Ok(res)
    }

    fn list_my_dids_with_meta(&self, wallet_handle: WalletHandle) -> IndyResult<String> {
        debug!("list_my_dids_with_meta >>> wallet_handle: {:?}", wallet_handle);

        let mut did_search =
            self.wallet_service.search_indy_records::<Did>(wallet_handle, "{}", &SearchOptions::id_value())?;

        let mut dids: Vec<DidWithMeta> = Vec::new();

        while let Some(did_record) = did_search.fetch_next_record()? {
            let did_id = did_record.get_id();

            let did: Did = did_record.get_value()
                .ok_or_else(|| err_msg(IndyErrorKind::InvalidState, "No value for DID record"))
                .and_then(|tags_json| serde_json::from_str(&tags_json)
                    .to_indy(IndyErrorKind::InvalidState, format!("Cannot deserialize Did: {:?}", did_id)))?;

            let metadata = self.wallet_service.get_indy_opt_object::<DidMetadata>(wallet_handle, &did.did.0, &RecordOptions::id_value())?;
            let temp_verkey = self.wallet_service.get_indy_opt_object::<TemporaryDid>(wallet_handle, &did.did.0, &RecordOptions::id_value())?;

            let did_with_meta = DidWithMeta {
                did: did.did,
                verkey: did.verkey,
                temp_verkey: temp_verkey.map(|tv| tv.verkey),
                metadata: metadata.map(|m| m.value),
            };

            dids.push(did_with_meta);
        }

        let res = serde_json::to_string(&dids)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize DIDs list")?;

        debug!("list_my_dids_with_meta <<< res: {:?}", res);

        Ok(res)
    }

    fn key_for_did(&self,
                   pool_handle: PoolHandle,
                   wallet_handle: WalletHandle,
                   did: DidValue,
                   cb: Box<dyn Fn(IndyResult<String>) + Send>) {
        debug!("key_for_did >>> pool_handle: {:?}, wallet_handle: {:?}, did: {:?}", pool_handle, wallet_handle, did);

        try_cb!(validate_did(&did), cb);

        // Look to my did
        match self._wallet_get_my_did(wallet_handle, &did) {
            Ok(my_did) => return cb(Ok(my_did.verkey)),
            Err(ref err) if err.kind() == IndyErrorKind::WalletItemNotFound => {}
            Err(err) => return cb(Err(err))
        };

        // look to their did
        let their_did = ensure_their_did!(self,
                                          wallet_handle,
                                          pool_handle,
                                          did,
                                          DidCommand::KeyForDid(
                                              pool_handle,
                                              wallet_handle,
                                              did.clone(),
                                              cb),
                                           cb);

        let res = their_did.verkey;

        debug!("key_for_did <<< res: {:?}", res);

        cb(Ok(res))
    }

    fn key_for_local_did(&self,
                         wallet_handle: WalletHandle,
                         did: &DidValue) -> IndyResult<String> {
        info!("key_for_local_did >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

        validate_did(&did)?;

        // Look to my did
        match self._wallet_get_my_did(wallet_handle, did) {
            Ok(my_did) => return Ok(my_did.verkey),
            Err(err) => match err.kind() {
                IndyErrorKind::WalletItemNotFound => {}
                _ => return Err(err)
            }
        };

        // look to their did
        let their_did = self._wallet_get_their_did(wallet_handle, did)?;

        let res = their_did.verkey;

        info!("key_for_local_did <<< res: {:?}", res);

        Ok(res)
    }

    fn set_endpoint_for_did(&self,
                            wallet_handle: WalletHandle,
                            did: &DidValue,
                            endpoint: &Endpoint) -> IndyResult<()> {
        debug!("set_endpoint_for_did >>> wallet_handle: {:?}, did: {:?}, endpoint: {:?}", wallet_handle, did, endpoint);

        validate_did(did)?;

        if endpoint.verkey.is_some() {
            let transport_key = endpoint.verkey.as_ref().unwrap();
            validate_key(transport_key)?;
        }

        self.wallet_service.upsert_indy_object(wallet_handle, &did.0, endpoint)?;

        debug!("set_endpoint_for_did <<<");
        Ok(())
    }

    fn get_endpoint_for_did(&self,
                            wallet_handle: WalletHandle,
                            pool_handle: PoolHandle,
                            did: DidValue,
                            cb: Box<dyn Fn(IndyResult<(String, Option<String>)>) + Send>) {
        debug!("get_endpoint_for_did >>> wallet_handle: {:?}, pool_handle: {:?}, did: {:?}", wallet_handle, pool_handle, did);

        try_cb!(validate_did(&did), cb);

        let endpoint =
            self.wallet_service.get_indy_object::<Endpoint>(wallet_handle, &did.0, &RecordOptions::id_value());

        match endpoint {
            Ok(endpoint) => cb(Ok((endpoint.ha, endpoint.verkey))),
            Err(ref err) if err.kind() == IndyErrorKind::WalletItemNotFound => self._fetch_attrib_from_ledger(wallet_handle,
                                                                                                              pool_handle,
                                                                                                              &did,
                                                                                                              DidCommand::GetEndpointForDid(
                                                                                                                  wallet_handle,
                                                                                                                  pool_handle,
                                                                                                                  did.clone(),
                                                                                                                  cb)),
            Err(err) => cb(Err(err)),
        };
    }

    fn set_did_metadata(&self,
                        wallet_handle: WalletHandle,
                        did: &DidValue,
                        metadata: String) -> IndyResult<()> {
        debug!("set_did_metadata >>> wallet_handle: {:?}, did: {:?}, metadata: {:?}", wallet_handle, did, metadata);

        validate_did(did)?;

        let metadata = DidMetadata { value: metadata };

        self.wallet_service.upsert_indy_object(wallet_handle, &did.0, &metadata)?;

        debug!("set_did_metadata >>>");

        Ok(())
    }

    fn get_did_metadata(&self,
                        wallet_handle: WalletHandle,
                        did: &DidValue) -> IndyResult<String> {
        debug!("get_did_metadata >>> wallet_handle: {:?}, did: {:?}", wallet_handle, did);

        validate_did(did)?;

        let metadata = self.wallet_service.get_indy_object::<DidMetadata>(wallet_handle, &did.0, &RecordOptions::id_value())?;

        let res = metadata.value;

        debug!("get_did_metadata <<< res: {:?}", res);

        Ok(res)
    }

    fn abbreviate_verkey(&self,
                         did: &DidValue,
                         verkey: String) -> IndyResult<String> {
        info!("abbreviate_verkey >>> did: {:?}, verkey: {:?}", did, verkey);

        validate_did(&did)?;
        validate_key(&verkey)?;

        if !did.is_abbreviatable() {
            return Ok(verkey);
        }

        let did = &did.to_unqualified().0.from_base58()?;
        let dverkey = &verkey.from_base58()?;

        let (first_part, second_part) = dverkey.split_at(16);

        let res = if first_part.eq(did.as_slice()) {
            format!("~{}", second_part.to_base58())
        } else {
            verkey
        };

        debug!("abbreviate_verkey <<< res: {:?}", res);

        Ok(res)
    }

    fn qualify_did(&self,
                   wallet_handle: WalletHandle,
                   did: &DidValue,
                   method: &DidMethod) -> IndyResult<String> {
        info!("qualify_did >>> wallet_handle: {:?}, curr_did: {:?}, method: {:?}", wallet_handle, did, method);

        validate_did(did)?;

        let mut curr_did: Did = self.wallet_service.get_indy_object::<Did>(wallet_handle, &did.0, &RecordOptions::id_value())?;

        curr_did.did = DidValue::new(&did.to_short().0, Some(&method.0));

        self.wallet_service.delete_indy_record::<Did>(wallet_handle, &did.0)?;
        self.wallet_service.add_indy_object(wallet_handle, &curr_did.did.0, &curr_did, &HashMap::new())?;

        // move temporary Did
        if let Ok(mut temp_did) = self.wallet_service.get_indy_object::<TemporaryDid>(wallet_handle, &did.0, &RecordOptions::id_value()) {
            temp_did.did = curr_did.did.clone();
            self.wallet_service.delete_indy_record::<TemporaryDid>(wallet_handle, &did.0)?;
            self.wallet_service.add_indy_object(wallet_handle, &curr_did.did.0, &temp_did, &HashMap::new())?;
        }

        // move metadata
        self.update_dependent_entity_reference::<DidMetadata>(wallet_handle, &did.0, &curr_did.did.0)?;

        // move endpoint
        self.update_dependent_entity_reference::<Endpoint>(wallet_handle, &did.0, &curr_did.did.0)?;

        // move all pairwise
        let mut pairwise_search =
            self.wallet_service.search_indy_records::<Pairwise>(wallet_handle, "{}", &RecordOptions::id_value())?;

        while let Some(pairwise_record) = pairwise_search.fetch_next_record()? {
            let mut pairwise: Pairwise = pairwise_record.get_value()
                .ok_or_else(|| err_msg(IndyErrorKind::InvalidState, "No value for Pairwise record"))
                .and_then(|pairwise_json| serde_json::from_str(&pairwise_json)
                    .map_err(|err| IndyError::from_msg(IndyErrorKind::InvalidState, format!("Cannot deserialize Pairwise: {:?}", err))))?;

            if pairwise.my_did.eq(did) {
                pairwise.my_did = curr_did.did.clone();
                self.wallet_service.update_indy_object(wallet_handle, &pairwise.their_did.0, &pairwise)?;
            }
        }

        debug!("qualify_did <<< res: {:?}", curr_did.did);

        Ok(curr_did.did.0)
    }

    fn update_dependent_entity_reference<T>(&self, wallet_handle: WalletHandle, id: &str, new_id: &str) -> IndyResult<()>
        where T: ::serde::Serialize + ::serde::de::DeserializeOwned + NamedType {
        if let Ok(record) = self.wallet_service.get_indy_record_value::<T>(wallet_handle, id, "{}") {
            self.wallet_service.delete_indy_record::<T>(wallet_handle, id)?;
            self.wallet_service.add_indy_record::<T>(wallet_handle, new_id, &record, &HashMap::new())?;
        }
        Ok(())
    }

    fn get_nym_ack(&self,
                   wallet_handle: WalletHandle,
                   did: DidValue,
                   get_nym_reply_result: IndyResult<String>,
                   deferred_cmd_id: CommandHandle) {
        let res = self._get_nym_ack(wallet_handle, did, get_nym_reply_result);
        self._execute_deferred_command(deferred_cmd_id, res.err());
    }

    fn _get_nym_ack(&self, wallet_handle: WalletHandle, did: DidValue, get_nym_reply_result: IndyResult<String>) -> IndyResult<()> {
        trace!("_get_nym_ack >>> wallet_handle: {:?}, get_nym_reply_result: {:?}", wallet_handle, get_nym_reply_result);

        let get_nym_reply = get_nym_reply_result?;

        let get_nym_response: Reply<GetNymReplyResult> = serde_json::from_str(&get_nym_reply)
            .to_indy(IndyErrorKind::InvalidState, "Invalid GetNymReplyResult json")?;

        let their_did_info = match get_nym_response.result() {
            GetNymReplyResult::GetNymReplyResultV0(res) => {
                if let Some(data) = &res.data {
                    let gen_nym_result_data: GetNymResultDataV0 = serde_json::from_str(data)
                        .to_indy(IndyErrorKind::InvalidState, "Invalid GetNymResultData json")?;

                    TheirDidInfo::new(gen_nym_result_data.dest.qualify(did.get_method()), gen_nym_result_data.verkey)
                } else {
                    return Err(err_msg(IndyErrorKind::WalletItemNotFound, "Their DID isn't found on the ledger")); //TODO FIXME use separate error
                }
            }
            GetNymReplyResult::GetNymReplyResultV1(res) => TheirDidInfo::new(res.txn.data.did.qualify(did.get_method()), res.txn.data.verkey)
        };

        let their_did = create_their_did(&their_did_info)?;

        self.wallet_service.add_indy_object(wallet_handle, &their_did.did.0, &their_did, &HashMap::new())?;

        trace!("_get_nym_ack <<<");

        Ok(())
    }

    fn get_attrib_ack(&self,
                      wallet_handle: WalletHandle,
                      get_attrib_reply_result: IndyResult<String>,
                      deferred_cmd_id: CommandHandle) {
        let res = self._get_attrib_ack(wallet_handle, get_attrib_reply_result);
        self._execute_deferred_command(deferred_cmd_id, res.err());
    }

    fn _get_attrib_ack(&self, wallet_handle: WalletHandle, get_attrib_reply_result: IndyResult<String>) -> IndyResult<()> {
        trace!("_get_attrib_ack >>> wallet_handle: {:?}, get_attrib_reply_result: {:?}", wallet_handle, get_attrib_reply_result);

        let get_attrib_reply = get_attrib_reply_result?;

        let get_attrib_reply: Reply<GetAttrReplyResult> = serde_json::from_str(&get_attrib_reply)
            .to_indy(IndyErrorKind::InvalidState, "Invalid GetAttrReplyResult json")?;

        let (raw, did) = match get_attrib_reply.result() {
            GetAttrReplyResult::GetAttrReplyResultV0(res) => (res.data, res.dest),
            GetAttrReplyResult::GetAttrReplyResultV1(res) => (res.txn.data.raw, res.txn.data.did)
        };

        let attrib_data: AttribData = serde_json::from_str(&raw)
            .to_indy(IndyErrorKind::InvalidState, "Invalid GetAttReply json")?;

        let endpoint = Endpoint::new(attrib_data.endpoint.ha, attrib_data.endpoint.verkey);

        self.wallet_service.add_indy_object(wallet_handle, &did.0, &endpoint, &HashMap::new())?;

        trace!("_get_attrib_ack <<<");

        Ok(())
    }

    fn _defer_command(&self, cmd: DidCommand) -> CommandHandle {
        let deferred_cmd_id = next_command_handle();
        self.deferred_commands.borrow_mut().insert(deferred_cmd_id, cmd);
        deferred_cmd_id
    }

    fn _execute_deferred_command(&self, deferred_cmd_id: CommandHandle, err: Option<IndyError>) {
        if let Some(cmd) = self.deferred_commands.borrow_mut().remove(&deferred_cmd_id) {
            if let Some(err) = err {
                self._call_error_cb(cmd, err);
            } else {
                self.execute(cmd);
            }
        } else {
            error!("No deferred command for id: {:?}", deferred_cmd_id)
        }
    }

    fn _call_error_cb(&self, command: DidCommand, err: IndyError) {
        match command {
            DidCommand::CreateAndStoreMyDid(_, _, cb) => {
                cb(Err(err));
            }
            DidCommand::ReplaceKeysStart(_, _, _, cb) => {
                cb(Err(err));
            }
            DidCommand::ReplaceKeysApply(_, _, cb) => {
                cb(Err(err));
            }
            DidCommand::StoreTheirDid(_, _, cb) => {
                cb(Err(err));
            }
            DidCommand::KeyForDid(_, _, _, cb) => {
                cb(Err(err));
            }
            DidCommand::GetEndpointForDid(_, _, _, cb) => {
                cb(Err(err));
            }
            _ => {}
        }
    }

    fn _fetch_their_did_from_ledger(&self,
                                    wallet_handle: WalletHandle, pool_handle: PoolHandle,
                                    did: &DidValue, deferred_cmd: DidCommand) {
        // Defer this command until their did is fetched from ledger.
        let deferred_cmd_id = self._defer_command(deferred_cmd);

        // TODO we need passing of my_did as identifier
        // TODO: FIXME: Remove this unwrap by sending GetNymAck with the error.
        let get_nym_request = self.ledger_service.build_get_nym_request(None, did).unwrap();
        let did = did.clone();

        CommandExecutor::instance()
            .send(Command::Ledger(LedgerCommand::SubmitRequest(
                pool_handle,
                get_nym_request,
                Box::new(move |result| {
                    CommandExecutor::instance()
                        .send(Command::Did(DidCommand::GetNymAck(
                            wallet_handle,
                            did.clone(),
                            result,
                            deferred_cmd_id,
                        ))).unwrap();
                }),
            ))).unwrap();
    }

    fn _fetch_attrib_from_ledger(&self,
                                 wallet_handle: WalletHandle, pool_handle: PoolHandle,
                                 did: &DidValue, deferred_cmd: DidCommand) {
        // Defer this command until their did is fetched from ledger.
        let deferred_cmd_id = self._defer_command(deferred_cmd);

        // TODO we need passing of my_did as identifier
        // TODO: FIXME: Remove this unwrap by sending GetAttribAck with the error.
        let get_attrib_request = self.ledger_service.build_get_attrib_request(None, did, Some("endpoint"), None, None).unwrap();

        CommandExecutor::instance()
            .send(Command::Ledger(LedgerCommand::SubmitRequest(
                pool_handle,
                get_attrib_request,
                Box::new(move |result| {
                    CommandExecutor::instance()
                        .send(Command::Did(DidCommand::GetAttribAck(
                            wallet_handle,
                            result,
                            deferred_cmd_id,
                        ))).unwrap();
                }),
            ))).unwrap();
    }

    fn _wallet_get_my_did(&self, wallet_handle: WalletHandle, my_did: &DidValue) -> IndyResult<Did> {
        self.wallet_service.get_indy_object(wallet_handle, &my_did.0, &RecordOptions::id_value())
    }

    fn _wallet_get_their_did(&self, wallet_handle: WalletHandle, their_did: &DidValue) -> IndyResult<TheirDid> {
        self.wallet_service.get_indy_object(wallet_handle, &their_did.0, &RecordOptions::id_value())
    }
}

pub fn create_did(my_did_info: &MyDidInfo) -> IndyResult<(Did, Key)> {
    trace!("create_my_did >>> my_did_info: {:?}", secret!(my_did_info));

    let key_info = KeyInfo { seed: my_did_info.seed.clone(), crypto_type: my_did_info.crypto_type.clone() };
    let key = create_key(&key_info)?;

    let did = match my_did_info.did {
        Some(ref did) => did.clone(),
        _ if my_did_info.cid == Some(true) => {
            DidValue::new(&key.verkey, my_did_info.method_name.as_ref().map(|method| method.0.as_str()))
        }
        _ => {
            let verkey = key.verkey.as_str().from_base58()?.as_slice()[0..16].to_vec().to_base58();
            DidValue::new(&verkey, my_did_info.method_name.as_ref().map(|method| method.0.as_str()))
        }
    };

    let res = (Did::new(did, key.verkey.clone()), key);

    trace!("create_my_did <<< res: {:?}", res);

    Ok(res)
}

pub fn create_their_did(their_did_info: &TheirDidInfo) -> IndyResult<TheirDid> {
    trace!("create_their_did >>> their_did_info: {:?}", their_did_info);

    // Check did is correct Base58
    let _ = validate_did(&their_did_info.did)?;

    let verkey = build_full_verkey(&their_did_info.did.to_unqualified().0,
                                   their_did_info.verkey.as_ref().map(String::as_str))?;

    validate_key(&verkey)?;

    let did = TheirDid { did: their_did_info.did.clone(), verkey };

    trace!("create_their_did <<< did: {:?}", did);

    Ok(did)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_my_did_works() {
        let did_info = MyDidInfo { did: None, cid: None, seed: None, crypto_type: None, method_name: None };
        let _ = create_did(&did_info).unwrap();
    }

    #[test]
    fn create_my_did_works_for_passed_did() {
        let did = DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string());
        let did_info = MyDidInfo { did: Some(did.clone()), cid: None, seed: None, crypto_type: None, method_name: None };

        let (my_did, _) = create_did(&did_info).unwrap();
        assert_eq!(did, my_did.did);
    }

    #[test]
    fn create_my_did_works_as_cid() {
        let did_info = MyDidInfo { did: None, cid: Some(true), seed: None, crypto_type: None, method_name: None };

        let (my_did, my_vk) = create_did(&did_info).unwrap();
        assert_eq!(my_did.did.0, my_vk.verkey);
    }

    #[test]
    fn create_my_did_works_for_seed() {
        let seed = String::from("00000000000000000000000000000My1");

        let did_info = MyDidInfo { did: None, cid: None, seed: Some(seed), crypto_type: None, method_name: None };

        let (my_did, _my_vk) = create_did(&did_info).unwrap();
        assert_eq!(my_did.did, DidValue::new("VsKV7grR1BUE29mG2Fm2kX", None));
        assert_eq!(my_did.verkey, String::from("GjZWsBLgZCR18aL468JAT7w9CZRiBnpxUPPgyQxh4voa"));
    }

    #[test]
    fn create_my_did_works_for_network_name() {
        let seed = String::from("00000000000000000000000000000My1");
        let method_name = DidMethod(String::from("test"));

        let did_info = MyDidInfo { did: None, cid: None, seed: Some(seed), crypto_type: None, method_name: Some(method_name.clone()) };

        let (my_did, _) = create_did(&did_info).unwrap();
        assert_eq!(my_did.did, DidValue::new("VsKV7grR1BUE29mG2Fm2kX", Some(&method_name.0)));
    }

    #[test]
    fn create_their_did_works_without_verkey() {
        let did = DidValue("CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW".to_string());

        let their_did_info = TheirDidInfo::new(did.clone(), None);
        let their_did = create_their_did(&their_did_info).unwrap();

        assert_eq!(did, their_did.did);
        assert_eq!(did.0, their_did.verkey);
    }

    #[test]
    fn create_their_did_works_for_full_verkey() {
        let did = DidValue("8wZcEriaNLNKtteJvx7f8i".to_string());
        let verkey = "5L2HBnzbu6Auh2pkDRbFt5f4prvgE2LzknkuYLsKkacp";

        let their_did_info = TheirDidInfo::new(did.clone(), Some(verkey.to_string()));
        let their_did = create_their_did(&their_did_info).unwrap();

        assert_eq!(did, their_did.did);
        assert_eq!(verkey, their_did.verkey);
    }

    #[test]
    fn create_their_did_works_for_abbreviated_verkey() {
        let did = DidValue("8wZcEriaNLNKtteJvx7f8i".to_string());
        let their_did_info = TheirDidInfo::new(did.clone(), Some("~NcYxiDXkpYi6ov5FcYDi1e".to_string()));
        let their_did = create_their_did(&their_did_info).unwrap();

        assert_eq!(did, their_did.did);
        assert_eq!("5L2HBnzbu6Auh2pkDRbFt5f4prvgE2LzknkuYLsKkacp", their_did.verkey);
    }
}