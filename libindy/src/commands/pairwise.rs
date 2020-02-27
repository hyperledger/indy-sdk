use indy_api_types::domain::crypto::did::{Did, TheirDid};
use crate::domain::pairwise::{Pairwise, PairwiseInfo};
use indy_api_types::errors::prelude::*;
use indy_wallet::{RecordOptions, WalletService};
use std::collections::HashMap;
use std::rc::Rc;
use std::str;
use indy_api_types::WalletHandle;
use indy_api_types::domain::crypto::did::DidValue;


pub enum PairwiseCommand {
    PairwiseExists(
        WalletHandle,
        DidValue, // their_did
        Box<dyn Fn(IndyResult<bool>) + Send>),
    CreatePairwise(
        WalletHandle,
        DidValue, // their_did
        DidValue, // my_did
        Option<String>, // metadata
        Box<dyn Fn(IndyResult<()>) + Send>),
    ListPairwise(
        WalletHandle,
        Box<dyn Fn(IndyResult<String>) + Send>),
    GetPairwise(
        WalletHandle,
        DidValue, // their_did
        Box<dyn Fn(IndyResult<String>) + Send>),
    SetPairwiseMetadata(
        WalletHandle,
        DidValue, // their_did
        Option<String>, // metadata
        Box<dyn Fn(IndyResult<()>) + Send>)
}

pub struct PairwiseCommandExecutor {
    wallet_service: Rc<WalletService>
}

impl PairwiseCommandExecutor {
    pub fn new(wallet_service: Rc<WalletService>) -> PairwiseCommandExecutor {
        PairwiseCommandExecutor {
            wallet_service
        }
    }

    pub fn execute(&self, command: PairwiseCommand) {
        match command {
            PairwiseCommand::PairwiseExists(wallet_handle, their_did, cb) => {
                debug!(target: "pairwise_command_executor", "PairwiseExists command received");
                cb(self.pairwise_exists(wallet_handle, &their_did));
            }
            PairwiseCommand::CreatePairwise(wallet_handle, their_did, my_did, metadata, cb) => {
                debug!(target: "pairwise_command_executor", "CreatePairwise command received");
                cb(self.create_pairwise(wallet_handle, &their_did, &my_did, metadata.as_ref().map(String::as_str)));
            }
            PairwiseCommand::ListPairwise(wallet_handle, cb) => {
                debug!(target: "pairwise_command_executor", "ListPairwise command received");
                cb(self.list_pairwise(wallet_handle));
            }
            PairwiseCommand::GetPairwise(wallet_handle, their_did, cb) => {
                debug!(target: "pairwise_command_executor", "GetPairwise command received");
                cb(self.get_pairwise(wallet_handle, &their_did));
            }
            PairwiseCommand::SetPairwiseMetadata(wallet_handle, their_did, metadata, cb) => {
                debug!(target: "pairwise_command_executor", "SetPairwiseMetadata command received");
                cb(self.set_pairwise_metadata(wallet_handle, &their_did, metadata.as_ref().map(String::as_str)));
            }
        };
    }

    fn pairwise_exists(&self,
                       wallet_handle: WalletHandle,
                       their_did: &DidValue) -> IndyResult<bool> {
        debug!("pairwise_exists >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

        let res = self.wallet_service.record_exists::<Pairwise>(wallet_handle, &their_did.0)?;

        debug!("pairwise_exists <<< res: {:?}", res);

        Ok(res)
    }

    fn create_pairwise(&self,
                       wallet_handle: WalletHandle,
                       their_did: &DidValue,
                       my_did: &DidValue,
                       metadata: Option<&str>) -> IndyResult<()> {
        debug!("create_pairwise >>> wallet_handle: {:?}, their_did: {:?}, my_did: {:?}, metadata: {:?}", wallet_handle, their_did, my_did, metadata);

        self.wallet_service.get_indy_record::<Did>(wallet_handle, &my_did.0, &RecordOptions::id())?;
        self.wallet_service.get_indy_record::<TheirDid>(wallet_handle, &their_did.0, &RecordOptions::id())?;

        let pairwise = Pairwise {
            my_did: my_did.clone(),
            their_did: their_did.clone(),
            metadata: metadata.map(str::to_string)
        };

        self.wallet_service.add_indy_object(wallet_handle, &their_did.0, &pairwise, &HashMap::new())?;

        debug!("create_pairwise <<<");

        Ok(())
    }

    fn list_pairwise(&self,
                     wallet_handle: WalletHandle) -> IndyResult<String> {
        debug!("list_pairwise >>> wallet_handle: {:?}", wallet_handle);

        let mut pairwise_search =
            self.wallet_service.search_indy_records::<Pairwise>(wallet_handle, "{}", &RecordOptions::id_value())?;

        let mut list_pairwise: Vec<String> = Vec::new();

        while let Some(pairwise_record) = pairwise_search.fetch_next_record()? {
            let pairwise_id = pairwise_record.get_id();

            let pairwise_value = pairwise_record.get_value()
                .ok_or_else(||err_msg(IndyErrorKind::InvalidStructure, format!("Pairwise not found for id: {}", pairwise_id)))?.to_string();

            list_pairwise.push(pairwise_value);
        }

        let res = serde_json::to_string(&list_pairwise)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize pairwise list")?;

        debug!("list_pairwise <<< res: {:?}", res);

        Ok(res)
    }

    fn get_pairwise(&self,
                    wallet_handle: WalletHandle,
                    their_did: &DidValue) -> IndyResult<String> {
        debug!("get_pairwise >>> wallet_handle: {:?}, their_did: {:?}", wallet_handle, their_did);

        let pairwise_info =
            PairwiseInfo::from(
                self.wallet_service.get_indy_object::<Pairwise>(wallet_handle, &their_did.0, &RecordOptions::id_value())?);

        let res = serde_json::to_string(&pairwise_info)
            .to_indy(IndyErrorKind::InvalidState, "Can't serialize PairwiseInfo")?;

        debug!("get_pairwise <<< res: {:?}", res);

        Ok(res)
    }


    fn set_pairwise_metadata(&self,
                             wallet_handle: WalletHandle,
                             their_did: &DidValue,
                             metadata: Option<&str>) -> IndyResult<()> {
        debug!("set_pairwise_metadata >>> wallet_handle: {:?}, their_did: {:?}, metadata: {:?}", wallet_handle, their_did, metadata);

        let mut pairwise: Pairwise =
            self.wallet_service.get_indy_object(wallet_handle, &their_did.0, &RecordOptions::id_value())?;

        pairwise.metadata = metadata.map(str::to_string);

        self.wallet_service.update_indy_object(wallet_handle, &their_did.0, &pairwise)?;

        debug!("set_pairwise_metadata <<<");

        Ok(())
    }
}
