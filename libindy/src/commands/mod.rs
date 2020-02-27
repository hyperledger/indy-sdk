extern crate ursa;
extern crate threadpool;

use std::env;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::commands::anoncreds::{AnoncredsCommand, AnoncredsCommandExecutor};
use crate::commands::blob_storage::{BlobStorageCommand, BlobStorageCommandExecutor};
use crate::commands::crypto::{CryptoCommand, CryptoCommandExecutor};
use crate::commands::did::{DidCommand, DidCommandExecutor};
use crate::commands::ledger::{LedgerCommand, LedgerCommandExecutor};
use crate::commands::non_secrets::{NonSecretsCommand, NonSecretsCommandExecutor};
use crate::commands::pairwise::{PairwiseCommand, PairwiseCommandExecutor};
use crate::commands::payments::{PaymentsCommand, PaymentsCommandExecutor};
use crate::commands::pool::{PoolCommand, PoolCommandExecutor};
use crate::commands::wallet::{WalletCommand, WalletCommandExecutor};
use crate::commands::cache::{CacheCommand, CacheCommandExecutor};
use crate::domain::IndyConfig;
use indy_api_types::errors::prelude::*;
use crate::services::anoncreds::AnoncredsService;
use crate::services::blob_storage::BlobStorageService;
use crate::services::ledger::LedgerService;
use crate::services::payments::PaymentsService;
use crate::services::pool::{PoolService, set_freshness_threshold};
use indy_wallet::WalletService;

use self::threadpool::ThreadPool;

pub mod anoncreds;
pub mod blob_storage;
pub mod crypto;
pub mod ledger;
pub mod pool;
pub mod did;
pub mod wallet;
pub mod pairwise;
pub mod non_secrets;
pub mod payments;
pub mod cache;

type BoxedCallbackStringStringSend = Box<dyn Fn(IndyResult<(String, String)>) + Send>;

pub enum Command {
    Exit,
    Anoncreds(AnoncredsCommand),
    BlobStorage(BlobStorageCommand),
    Crypto(CryptoCommand),
    Ledger(LedgerCommand),
    Pool(PoolCommand),
    Did(DidCommand),
    Wallet(WalletCommand),
    Pairwise(PairwiseCommand),
    NonSecrets(NonSecretsCommand),
    Payments(PaymentsCommand),
    Cache(CacheCommand),
}

lazy_static! {
    static ref THREADPOOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(4));
}

pub fn indy_set_runtime_config(config: IndyConfig) {
    if let Some(crypto_thread_pool_size) = config.crypto_thread_pool_size {
        THREADPOOL.lock().unwrap().set_num_threads(crypto_thread_pool_size);
    }
    match config.collect_backtrace {
        Some(true) => env::set_var("RUST_BACKTRACE", "1"),
        Some(false) => env::set_var("RUST_BACKTRACE", "0"),
        _ => {}
    }
    if let Some(threshold) = config.freshness_threshold {
        set_freshness_threshold(threshold);
    }
}

pub struct CommandExecutor {
    worker: Option<thread::JoinHandle<()>>,
    sender: Sender<Command>
}

// Global (lazy inited) instance of CommandExecutor
lazy_static! {
    static ref COMMAND_EXECUTOR: Mutex<CommandExecutor> = Mutex::new(CommandExecutor::new());
}

impl CommandExecutor {
    pub fn instance<'mutex>() -> MutexGuard<'mutex, CommandExecutor> {
        COMMAND_EXECUTOR.lock().unwrap()
    }

    fn new() -> CommandExecutor {
        let (sender, receiver) = channel();

        CommandExecutor {
            sender,
            worker: Some(thread::spawn(move || {
                info!(target: "command_executor", "Worker thread started");

                let anoncreds_service = Rc::new(AnoncredsService::new());
                let blob_storage_service = Rc::new(BlobStorageService::new());
                let ledger_service = Rc::new(LedgerService::new());
                let payments_service = Rc::new(PaymentsService::new());
                let pool_service = Rc::new(PoolService::new());
                let wallet_service = Rc::new(WalletService::new());

                let anoncreds_command_executor = AnoncredsCommandExecutor::new(anoncreds_service.clone(), blob_storage_service.clone(), pool_service.clone(), wallet_service.clone());
                let crypto_command_executor = CryptoCommandExecutor::new(wallet_service.clone());
                let ledger_command_executor = LedgerCommandExecutor::new(pool_service.clone(), wallet_service.clone(), ledger_service.clone());
                let pool_command_executor = PoolCommandExecutor::new(pool_service.clone());
                let did_command_executor = DidCommandExecutor::new(wallet_service.clone(), ledger_service.clone());
                let wallet_command_executor = WalletCommandExecutor::new(wallet_service.clone());
                let pairwise_command_executor = PairwiseCommandExecutor::new(wallet_service.clone());
                let blob_storage_command_executor = BlobStorageCommandExecutor::new(blob_storage_service.clone());
                let non_secret_command_executor = NonSecretsCommandExecutor::new(wallet_service.clone());
                let payments_command_executor = PaymentsCommandExecutor::new(payments_service.clone(), wallet_service.clone(),  ledger_service.clone());
                let cache_command_executor = CacheCommandExecutor::new(wallet_service.clone());

                loop {
                    match receiver.recv() {
                        Ok(Command::Anoncreds(cmd)) => {
                            debug!("AnoncredsCommand command received");
                            anoncreds_command_executor.execute(cmd);
                        }
                        Ok(Command::BlobStorage(cmd)) => {
                            debug!("BlobStorageCommand command received");
                            blob_storage_command_executor.execute(cmd);
                        }
                        Ok(Command::Crypto(cmd)) => {
                            debug!("CryptoCommand command received");
                            crypto_command_executor.execute(cmd);
                        }
                        Ok(Command::Ledger(cmd)) => {
                            debug!("LedgerCommand command received");
                            ledger_command_executor.execute(cmd);
                        }
                        Ok(Command::Pool(cmd)) => {
                            debug!("PoolCommand command received");
                            pool_command_executor.execute(cmd);
                        }
                        Ok(Command::Did(cmd)) => {
                            debug!("DidCommand command received");
                            did_command_executor.execute(cmd);
                        }
                        Ok(Command::Wallet(cmd)) => {
                            debug!("WalletCommand command received");
                            wallet_command_executor.execute(cmd);
                        }
                        Ok(Command::Pairwise(cmd)) => {
                            debug!("PairwiseCommand command received");
                            pairwise_command_executor.execute(cmd);
                        }
                        Ok(Command::NonSecrets(cmd)) => {
                            debug!("NonSecretCommand command received");
                            non_secret_command_executor.execute(cmd);
                        }
                        Ok(Command::Payments(cmd)) => {
                            debug!("PaymentsCommand command received");
                            payments_command_executor.execute(cmd);
                        }
                        Ok(Command::Cache(cmd)) => {
                            debug!("CacheCommand command received");
                            cache_command_executor.execute(cmd);
                        }
                        Ok(Command::Exit) => {
                            debug!("Exit command received");
                            break
                        }
                        Err(err) => {
                            error!("Failed to get command!");
                            panic!("Failed to get command! {:?}", err)
                        }
                    }
                }
            }))
        }
    }

    pub fn send(&self, cmd: Command) -> IndyResult<()> {
        self.sender
            .send(cmd)
            .map_err(|err| err_msg(IndyErrorKind::InvalidState, format!("Can't send msg to CommandExecutor: {}", err)))
    }
}

impl Drop for CommandExecutor {
    fn drop(&mut self) {
        info!(target: "command_executor", "Drop started");
        self.send(Command::Exit).unwrap();
        // Option worker type and this kludge is workaround for rust
        self.worker.take().unwrap().join().unwrap();
        info!(target: "command_executor", "Drop finished");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_executor_can_be_created() {
        let _command_executor = CommandExecutor::new();
        assert!(true, "No crashes on CommandExecutor::new");
    }

    #[test]
    fn command_executor_can_be_dropped() {
        fn drop_test() {
            let _command_executor = CommandExecutor::new();
        }

        drop_test();
        assert!(true, "No crashes on CommandExecutor::drop");
    }

    #[test]
    fn command_executor_can_get_instance() {
        let ref _command_executor: CommandExecutor = *CommandExecutor::instance();
        // Deadlock if another one instance will be requested (try to uncomment the next line)
        // let ref other_ce: CommandExecutor = *CommandExecutor::instance();
    }
}
