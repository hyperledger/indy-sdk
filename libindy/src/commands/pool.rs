use std::rc::Rc;

use crate::domain::pool::{PoolConfig, PoolOpenConfig};
use indy_api_types::errors::prelude::*;
use crate::services::pool::PoolService;
use indy_api_types::PoolHandle;
use crate::services::ledger::LedgerService;

pub enum PoolCommand {
    Create(
        String, // name
        Option<PoolConfig>, // config
        Box<dyn Fn(IndyResult<()>) + Send>),
    Delete(
        String, // name
        Box<dyn Fn(IndyResult<()>) + Send>),
    Open(
        String, // name
        Option<PoolOpenConfig>, // config
        Box<dyn Fn(IndyResult<PoolHandle>) + Send>),
    List(Box<dyn Fn(IndyResult<String>) + Send>),
    Close(
        PoolHandle, // pool handle
        Box<dyn Fn(IndyResult<()>) + Send>),
    Refresh(
        PoolHandle, // pool handle
        Box<dyn Fn(IndyResult<()>) + Send>),
    SetProtocolVersion(
        usize, // protocol version
        Box<dyn Fn(IndyResult<()>) + Send>),
}

pub struct PoolCommandExecutor {
    pool_service: Rc<PoolService>,
    ledger_service: Rc<LedgerService>,
}

impl PoolCommandExecutor {
    pub fn new(pool_service: Rc<PoolService>, ledger_service: Rc<LedgerService>) -> PoolCommandExecutor {
        PoolCommandExecutor {
            pool_service,
            ledger_service,
        }
    }

    pub async fn execute(&self, command: PoolCommand) {
        match command {
            PoolCommand::Create(name, config, cb) => {
                debug!(target: "pool_command_executor", "Create command received");
                cb(self.create(&name, config));
            }
            PoolCommand::Delete(name, cb) => {
                debug!(target: "pool_command_executor", "Delete command received");
                cb(self.delete(&name));
            }
            PoolCommand::Open(name, config, cb) => {
                debug!(target: "pool_command_executor", "Open command received");
                cb(self.open(name, config).await);
            }
            PoolCommand::List(cb) => {
                debug!(target: "pool_command_executor", "List command received");
                cb(self.list());
            }
            PoolCommand::Close(handle, cb) => {
                debug!(target: "pool_command_executor", "Close command received");
                self.close(handle, cb).await;
            }
            PoolCommand::Refresh(handle, cb) => {
                debug!(target: "pool_command_executor", "Refresh command received");
                self.refresh(handle, cb).await;
            }
            PoolCommand::SetProtocolVersion(protocol_version, cb) => {
                debug!(target: "pool_command_executor", "SetProtocolVersion command received");
                cb(self.set_protocol_version(protocol_version));
            }
        };
    }

    fn create(&self, name: &str, config: Option<PoolConfig>) -> IndyResult<()> {
        debug!("create >>> name: {:?}, config: {:?}", name, config);

        self.pool_service.create(name, config)?;

        debug!("create << res: ()");

        Ok(())
    }

    fn delete(&self, name: &str) -> IndyResult<()> {
        debug!("delete >>> name: {:?}", name);

        self.pool_service.delete(name)?;

        debug!("delete << res: ()");

        Ok(())
    }

    async fn open(&self, name: String, config: Option<PoolOpenConfig>) -> IndyResult<PoolHandle> {
        debug!("open >>> name: {:?}, config: {:?}", name, config);

        let protocol_version = self.ledger_service.get_protocol_version()?;

        let result = self.pool_service.open(name, config, protocol_version).await?;

        debug!("open <<<");
        Ok(result)
    }

    fn list(&self) -> IndyResult<String> {
        debug!("list >>> ");

        let res = self.pool_service
            .list()
            .and_then(|pools| ::serde_json::to_string(&pools)
                .to_indy(IndyErrorKind::InvalidState, "Can't serialize pools list"))?;

        debug!("list << res: {:?}", res);
        Ok(res)
    }

    async fn close(&self, pool_handle: PoolHandle, cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        debug!("close >>> handle: {:?}", pool_handle);

        let result = self.pool_service.close(pool_handle).await;

        cb(result);

        debug!("close <<<");
    }

    async fn refresh(&self, handle: PoolHandle, cb: Box<dyn Fn(IndyResult<()>) + Send>) {
        debug!("refresh >>> handle: {:?}", handle);

        let result = self.pool_service.refresh(handle).await;

        cb(result);

        debug!("refresh <<<");
    }

    fn set_protocol_version(&self, version: usize) -> IndyResult<()> {
        debug!("set_protocol_version >>> version: {:?}", version);

        let result = self.ledger_service.set_protocol_version(version)?;

        debug!("set_protocol_version <<<");

        Ok(result)
    }
}
