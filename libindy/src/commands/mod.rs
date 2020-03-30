extern crate threadpool;
extern crate ursa;

use std::env;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use std::thread;

use futures::StreamExt;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::executor::block_on;

use crate::commands::anoncreds::{AnoncredsCommand, AnoncredsCommandExecutor};
use crate::commands::blob_storage::{BlobStorageCommand, BlobStorageCommandExecutor};
use crate::commands::cache::{CacheCommand, CacheCommandExecutor};
use crate::commands::crypto::{CryptoCommand, CryptoCommandExecutor};
use crate::commands::did::{DidCommand, DidCommandExecutor};
use crate::commands::ledger::{LedgerCommand, LedgerCommandExecutor};
use crate::commands::non_secrets::{NonSecretsCommand, NonSecretsCommandExecutor};
use crate::commands::pairwise::{PairwiseCommand, PairwiseCommandExecutor};
use crate::commands::payments::{PaymentsCommand, PaymentsCommandExecutor};
use crate::commands::pool::{PoolCommand, PoolCommandExecutor};
use crate::commands::wallet::{WalletCommand, WalletCommandExecutor};
use crate::domain::IndyConfig;
use indy_api_types::errors::prelude::*;
use crate::services::anoncreds::AnoncredsService;
use crate::services::blob_storage::BlobStorageService;
use crate::services::crypto::CryptoService;
use crate::services::ledger::LedgerService;
use crate::services::payments::PaymentsService;
use crate::services::pool::{PoolService, set_freshness_threshold};
use indy_wallet::WalletService;

use self::threadpool::ThreadPool;
use futures::stream::FuturesUnordered;

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
    sender: UnboundedSender<Command>
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
        let (sender, mut receiver) = unbounded();
        let thread = thread::Builder::new().name("libindy CommandExecutor".to_owned());

        CommandExecutor {
            sender,
            worker: Some(thread.spawn(move || {
                info!(target: "command_executor", "Worker thread started");

                let anoncreds_service = Rc::new(AnoncredsService::new());
                let blob_storage_service = Rc::new(BlobStorageService::new());
                let crypto_service = Rc::new(CryptoService::new());
                let ledger_service = Rc::new(LedgerService::new());
                let payments_service = Rc::new(PaymentsService::new());
                let pool_service = Rc::new(PoolService::new());
                let wallet_service = Rc::new(WalletService::new());

                let anoncreds_command_executor = AnoncredsCommandExecutor::new(anoncreds_service.clone(), blob_storage_service.clone(), pool_service.clone(), wallet_service.clone(), crypto_service.clone());
                let crypto_command_executor = CryptoCommandExecutor::new(wallet_service.clone(), crypto_service.clone());
                let ledger_command_executor = LedgerCommandExecutor::new(pool_service.clone(), crypto_service.clone(), wallet_service.clone(), ledger_service.clone());
                let pool_command_executor = PoolCommandExecutor::new(pool_service.clone(), ledger_service.clone());
                let did_command_executor = DidCommandExecutor::new(wallet_service.clone(), crypto_service.clone(), ledger_service.clone(), pool_service.clone());
                let wallet_command_executor = WalletCommandExecutor::new(wallet_service.clone(), crypto_service.clone());
                let pairwise_command_executor = PairwiseCommandExecutor::new(wallet_service.clone());
                let blob_storage_command_executor = BlobStorageCommandExecutor::new(blob_storage_service.clone());
                let non_secret_command_executor = NonSecretsCommandExecutor::new(wallet_service.clone());
                let payments_command_executor = PaymentsCommandExecutor::new(payments_service.clone(), wallet_service.clone(), crypto_service.clone(), ledger_service.clone());
                let cache_command_executor = CacheCommandExecutor::new(crypto_service.clone(), ledger_service.clone(), pool_service.clone(), wallet_service.clone());

                async fn _exec_cmd(cmd: Option<Command>,
                                   anoncreds_command_executor: &AnoncredsCommandExecutor,
                                   crypto_command_executor: &CryptoCommandExecutor,
                                   ledger_command_executor: &LedgerCommandExecutor,
                                   pool_command_executor: &PoolCommandExecutor,
                                   did_command_executor: &DidCommandExecutor,
                                   wallet_command_executor: &WalletCommandExecutor,
                                   pairwise_command_executor: &PairwiseCommandExecutor,
                                   blob_storage_command_executor: &BlobStorageCommandExecutor,
                                   non_secret_command_executor: &NonSecretsCommandExecutor,
                                   payments_command_executor: &PaymentsCommandExecutor,
                                   cache_command_executor: &CacheCommandExecutor,
                ) -> bool {
                    match cmd {
                        Some(Command::Anoncreds(cmd)) => {
                            debug!("AnoncredsCommand command received");
                            anoncreds_command_executor.execute(cmd).await;
                        }
                        Some(Command::BlobStorage(cmd)) => {
                            debug!("BlobStorageCommand command received");
                            blob_storage_command_executor.execute(cmd);
                        }
                        Some(Command::Crypto(cmd)) => {
                            debug!("CryptoCommand command received");
                            crypto_command_executor.execute(cmd);
                        }
                        Some(Command::Ledger(cmd)) => {
                            debug!("LedgerCommand command received");
                            ledger_command_executor.execute(cmd).await;
                        }
                        Some(Command::Pool(cmd)) => {
                            debug!("PoolCommand command received");
                            pool_command_executor.execute(cmd).await;
                        }
                        Some(Command::Did(cmd)) => {
                            debug!("DidCommand command received");
                            did_command_executor.execute(cmd).await;
                        }
                        Some(Command::Wallet(cmd)) => {
                            debug!("WalletCommand command received");
                            wallet_command_executor.execute(cmd).await;
                        }
                        Some(Command::Pairwise(cmd)) => {
                            debug!("PairwiseCommand command received");
                            pairwise_command_executor.execute(cmd);
                        }
                        Some(Command::NonSecrets(cmd)) => {
                            debug!("NonSecretCommand command received");
                            non_secret_command_executor.execute(cmd);
                        }
                        Some(Command::Payments(cmd)) => {
                            debug!("PaymentsCommand command received");
                            payments_command_executor.execute(cmd).await;
                        }
                        Some(Command::Cache(cmd)) => {
                            debug!("CacheCommand command received");
                            cache_command_executor.execute(cmd).await;
                        }
                        Some(Command::Exit) => {
                            debug!("Exit command received");
                            return true
                        }
                        None => {
                            warn!("No command to execute");
                        }
                    }

                    false
                };

                let mut in_progress_tasks = FuturesUnordered::new();
                loop {
                    trace!("CommandExecutor main loop >>");
                    let mut break_main_loop = false;
                    block_on(async {
                        trace!("CommandExecutor async block");
                        select! {
                            cmd = receiver.next() => {
                                trace!("CommandExecutor::select new command");
                                let in_progress_task = _exec_cmd(cmd, &anoncreds_command_executor, &crypto_command_executor, &ledger_command_executor, &pool_command_executor, &did_command_executor, &wallet_command_executor, &pairwise_command_executor, &blob_storage_command_executor, &non_secret_command_executor, &payments_command_executor, &cache_command_executor);
                                in_progress_tasks.push(in_progress_task);
                            }
                            should_complete_main_loop = in_progress_tasks.next() => {
                                trace!("CommandExecutor::select in progress task, break loop {:?}", should_complete_main_loop);
                                break_main_loop = should_complete_main_loop.unwrap_or(false);
                            }
                            complete => {
                                trace!("CommandExecutor::select complete");
                                break_main_loop = true;
                            }
                        }
                    });
                    if break_main_loop {
                        trace!("CommandExecutor main loop break");
                        break
                    }
                    trace!("CommandExecutor main loop <<");
                }
                trace!("CommandExecutor main loop finished");
            }).unwrap())
        }
    }

    pub fn send(&mut self, cmd: Command) -> IndyResult<()> {
        self.sender
            .unbounded_send(cmd)
            .map_err(|err| err_msg(IndyErrorKind::InvalidState, format!("Can't send msg to CommandExecutor: {}", err)))
    }
}

impl Drop for CommandExecutor {
    fn drop(&mut self) {
        info!(target: "command_executor", "Drop started");
        self.send(Command::Exit).unwrap();
        self.sender.disconnect();
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
