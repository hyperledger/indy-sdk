pub mod issuer;
pub mod prover;
pub mod verifier;
mod tails;

use crate::commands::anoncreds::issuer::{IssuerCommand, IssuerCommandExecutor};
use crate::commands::anoncreds::prover::{ProverCommand, ProverCommandExecutor};
use crate::commands::anoncreds::verifier::{VerifierCommand, VerifierCommandExecutor};

use crate::services::anoncreds::AnoncredsService;
use crate::services::blob_storage::BlobStorageService;
use crate::services::pool::PoolService;
use indy_wallet::WalletService;
use crate::services::anoncreds::helpers::to_unqualified;

use indy_api_types::errors::prelude::*;

use std::rc::Rc;

pub enum AnoncredsCommand {
    Issuer(IssuerCommand),
    Prover(ProverCommand),
    Verifier(VerifierCommand),
    ToUnqualified(
        String, // entity
        Box<dyn Fn(IndyResult<String>) + Send>)
}

pub struct AnoncredsCommandExecutor {
    issuer_command_cxecutor: IssuerCommandExecutor,
    prover_command_cxecutor: ProverCommandExecutor,
    verifier_command_cxecutor: VerifierCommandExecutor
}

impl AnoncredsCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>,
               blob_storage_service: Rc<BlobStorageService>,
               pool_service: Rc<PoolService>,
               wallet_service: Rc<WalletService>) -> AnoncredsCommandExecutor {
        AnoncredsCommandExecutor {
            issuer_command_cxecutor: IssuerCommandExecutor::new(
                anoncreds_service.clone(), pool_service.clone(),
                blob_storage_service.clone(), wallet_service.clone()),
            prover_command_cxecutor: ProverCommandExecutor::new(
                anoncreds_service.clone(), wallet_service.clone(), blob_storage_service.clone()),
            verifier_command_cxecutor: VerifierCommandExecutor::new(
                anoncreds_service.clone()),
        }
    }

    pub fn execute(&self, command: AnoncredsCommand) {
        match command {
            AnoncredsCommand::Issuer(cmd) => {
                debug!(target: "anoncreds_command_executor", "Issuer command received");
                self.issuer_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::Prover(cmd) => {
                debug!(target: "anoncreds_command_executor", "Prover command received");
                self.prover_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::Verifier(cmd) => {
                debug!(target: "anoncreds_command_executor", "Verifier command received");
                self.verifier_command_cxecutor.execute(cmd);
            }
            AnoncredsCommand::ToUnqualified(entity, cb) => {
                debug!("ToUnqualified command received");
                cb(to_unqualified(&entity));
            }
        };
    }
}
