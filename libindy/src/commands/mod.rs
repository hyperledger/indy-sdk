extern crate threadpool;
extern crate ursa;

use std::env;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use std::thread;

use futures::StreamExt;
use futures::channel::mpsc::{unbounded, UnboundedSender};

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
use crate::commands::metrics::{MetricsCommand, MetricsCommandExecutor};
use crate::domain::IndyConfig;
use indy_api_types::errors::prelude::*;
use crate::services::anoncreds::AnoncredsService;
use crate::services::blob_storage::BlobStorageService;
use crate::services::crypto::CryptoService;
use crate::services::ledger::LedgerService;
use crate::services::payments::PaymentsService;
use crate::services::pool::{PoolService, set_freshness_threshold};
use crate::services::metrics::MetricsService;
use crate::services::metrics::command_index::CommandIndex;
use indy_wallet::WalletService;

use self::threadpool::ThreadPool;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::task::LocalSpawnExt;

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
pub mod metrics;

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
    Metrics(MetricsCommand),
}

pub struct InstrumentedCommand {
    pub enqueue_ts: u128,
    pub command: Command
}

impl InstrumentedCommand {
    pub fn new(command: Command) -> InstrumentedCommand {
        InstrumentedCommand {
            enqueue_ts: get_cur_time(),
            command
        }
    }
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

fn get_cur_time() -> u128 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has gone backwards");
    since_epoch.as_millis()
}

pub struct CommandExecutor {
    worker: Option<thread::JoinHandle<()>>,
    sender: UnboundedSender<InstrumentedCommand>
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
                let metrics_service = Rc::new(MetricsService::new());

                let anoncreds_command_executor = Rc::new(AnoncredsCommandExecutor::new(anoncreds_service.clone(), blob_storage_service.clone(), pool_service.clone(), wallet_service.clone(), crypto_service.clone()));
                let crypto_command_executor = Rc::new(CryptoCommandExecutor::new(wallet_service.clone(), crypto_service.clone()));
                let ledger_command_executor = Rc::new(LedgerCommandExecutor::new(pool_service.clone(), crypto_service.clone(), wallet_service.clone(), ledger_service.clone()));
                let pool_command_executor = Rc::new(PoolCommandExecutor::new(pool_service.clone()));
                let did_command_executor = Rc::new(DidCommandExecutor::new(wallet_service.clone(), crypto_service.clone(), ledger_service.clone(), pool_service.clone()));
                let wallet_command_executor = Rc::new(WalletCommandExecutor::new(wallet_service.clone(), crypto_service.clone()));
                let pairwise_command_executor = Rc::new(PairwiseCommandExecutor::new(wallet_service.clone()));
                let blob_storage_command_executor = Rc::new(BlobStorageCommandExecutor::new(blob_storage_service.clone()));
                let non_secret_command_executor = Rc::new(NonSecretsCommandExecutor::new(wallet_service.clone()));
                let payments_command_executor = Rc::new(PaymentsCommandExecutor::new(payments_service.clone(), wallet_service.clone(), crypto_service.clone(), ledger_service.clone()));
                let cache_command_executor = Rc::new(CacheCommandExecutor::new(crypto_service.clone(), ledger_service.clone(), pool_service.clone(), wallet_service.clone()));
                let metrics_command_executor = Rc::new(MetricsCommandExecutor::new(wallet_service.clone(), metrics_service.clone()));

                async fn _exec_cmd(instrumented_cmd: InstrumentedCommand,
                                   metrics_service: Rc<MetricsService>,
                                   anoncreds_command_executor: Rc<AnoncredsCommandExecutor>,
                                   crypto_command_executor: Rc<CryptoCommandExecutor>,
                                   ledger_command_executor: Rc<LedgerCommandExecutor>,
                                   pool_command_executor: Rc<PoolCommandExecutor>,
                                   did_command_executor: Rc<DidCommandExecutor>,
                                   wallet_command_executor: Rc<WalletCommandExecutor>,
                                   pairwise_command_executor: Rc<PairwiseCommandExecutor>,
                                   blob_storage_command_executor: Rc<BlobStorageCommandExecutor>,
                                   non_secret_command_executor: Rc<NonSecretsCommandExecutor>,
                                   payments_command_executor: Rc<PaymentsCommandExecutor>,
                                   cache_command_executor: Rc<CacheCommandExecutor>,
                                   metrics_command_executor: Rc<MetricsCommandExecutor>,
                ) {
                    let cmd_index: CommandIndex = (&instrumented_cmd.command).into();
                    let start_execution_ts = get_cur_time();
                    metrics_service.cmd_left_queue(cmd_index,
                                                   start_execution_ts - instrumented_cmd.enqueue_ts);

                    match instrumented_cmd.command {
                        Command::Anoncreds(cmd) => {
                            debug!("AnoncredsCommand command received");
                            anoncreds_command_executor.execute(cmd);
                        }
                        Command::BlobStorage(cmd) => {
                            debug!("BlobStorageCommand command received");
                            blob_storage_command_executor.execute(cmd);
                        }
                        Command::Crypto(cmd) => {
                            debug!("CryptoCommand command received");
                            crypto_command_executor.execute(cmd);
                        }
                        Command::Ledger(cmd) => {
                            debug!("LedgerCommand command received");
                            ledger_command_executor.execute(cmd).await;
                        }
                        Command::Pool(cmd) => {
                            debug!("PoolCommand command received");
                            pool_command_executor.execute(cmd).await;
                        }
                        Command::Did(cmd) => {
                            debug!("DidCommand command received");
                            did_command_executor.execute(cmd).await;
                        }
                        Command::Wallet(cmd) => {
                            debug!("WalletCommand command received");
                            wallet_command_executor.execute(cmd).await;
                        }
                        Command::Pairwise(cmd) => {
                            debug!("PairwiseCommand command received");
                            pairwise_command_executor.execute(cmd);
                        }
                        Command::NonSecrets(cmd) => {
                            debug!("NonSecretCommand command received");
                            non_secret_command_executor.execute(cmd);
                        }
                        Command::Payments(cmd) => {
                            debug!("PaymentsCommand command received");
                            payments_command_executor.execute(cmd).await;
                        }
                        Command::Cache(cmd) => {
                            debug!("CacheCommand command received");
                            cache_command_executor.execute(cmd).await;
                        }
                        Command::Metrics(cmd) => {
                            debug!("MetricsCommand command received");
                            metrics_command_executor.execute(cmd);
                        }
                        Command::Exit => {
                            debug!("Exit command received");
                        }
                    }

                    metrics_service.cmd_executed(cmd_index,
                                                 get_cur_time() - start_execution_ts);
                };

                let mut in_progress_tasks = futures::executor::LocalPool::new();
                let spawner = in_progress_tasks.spawner();
                loop {
                    trace!("CommandExecutor main loop >>");
                    let cmd = in_progress_tasks.run_until(receiver.next());
                    
                    let cmd = if let Some(cmd) = cmd {
                        cmd
                    } else {
                        warn!("No command to execute");
                        continue
                    };

                    if let Command::Exit = cmd.command {
                        break
                    }

                    spawner.spawn_local(_exec_cmd(cmd, metrics_service.clone(), anoncreds_command_executor.clone(), crypto_command_executor.clone(), ledger_command_executor.clone(), pool_command_executor.clone(), did_command_executor.clone(), wallet_command_executor.clone(), pairwise_command_executor.clone(), blob_storage_command_executor.clone(), non_secret_command_executor.clone(), payments_command_executor.clone(), cache_command_executor.clone(), metrics_command_executor.clone())).unwrap();
                    trace!("CommandExecutor main loop <<");
                }

                trace!("CommandExecutor main loop finished");
            }).unwrap())
        }
    }

    pub fn send(&mut self, cmd: Command) -> IndyResult<()> {
        self.sender
            .unbounded_send(InstrumentedCommand::new(cmd))
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