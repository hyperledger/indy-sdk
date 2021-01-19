use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::domain::ledger::request::ProtocolVersion;
use crate::domain::pool::{PoolConfig, PoolOpenConfig};
use indy_api_types::errors::prelude::*;
use crate::services::pool::PoolService;
use indy_api_types::{PoolHandle, CommandHandle};
use crate::services::metrics::MetricsService;

pub enum PoolCommand {
    Create(
        String, // name
        Option<PoolConfig>, // config
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    Delete(
        String, // name
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    Open(
        String, // name
        Option<PoolOpenConfig>, // config
        Box<dyn Fn(IndyResult<PoolHandle>, Rc<MetricsService>) + Send>),
    OpenAck(
        CommandHandle, // cmd id
        PoolHandle, // pool handle
        IndyResult<()>),
    List(Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>),
    Close(
        PoolHandle, // pool handle
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    CloseAck(CommandHandle,
             IndyResult<()>),
    Refresh(
        PoolHandle, // pool handle
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
    RefreshAck(CommandHandle,
               IndyResult<()>),
    SetProtocolVersion(
        usize, // protocol version
        Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>),
}

pub struct PoolCommandExecutor {
    pool_service: Rc<PoolService>,
    metrics_service: Rc<MetricsService>,
    close_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<()>, Rc<MetricsService>)>>>,
    refresh_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<()>, Rc<MetricsService>)>>>,
    open_callbacks: RefCell<HashMap<CommandHandle, Box<dyn Fn(IndyResult<PoolHandle>, Rc<MetricsService>)>>>,
}

impl PoolCommandExecutor {
    pub fn new(pool_service: Rc<PoolService>, metrics_service: Rc<MetricsService>) -> PoolCommandExecutor {
        PoolCommandExecutor {
            pool_service,
            metrics_service,
            close_callbacks: RefCell::new(HashMap::new()),
            refresh_callbacks: RefCell::new(HashMap::new()),
            open_callbacks: RefCell::new(HashMap::new()),
        }
    }

    pub fn execute(&self, command: PoolCommand) {
        match command {
            PoolCommand::Create(name, config, cb) => {
                debug!(target: "pool_command_executor", "Create command received");
                cb(self.create(&name, config), self.metrics_service.clone());
            }
            PoolCommand::Delete(name, cb) => {
                debug!(target: "pool_command_executor", "Delete command received");
                cb(self.delete(&name), self.metrics_service.clone());
            }
            PoolCommand::Open(name, config, cb) => {
                debug!(target: "pool_command_executor", "Open command received");
                self.open(&name, config, cb);
            }
            PoolCommand::OpenAck(handle, pool_id, result) => {
                info!("OpenAck handle {:?}, pool_id {:?}, result {:?}", handle, pool_id, result);
                match self.open_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => {
                                cb(result.and_then(|_| self.pool_service.add_open_pool(pool_id)), self.metrics_service.clone())
                            }
                            None => {
                                error!("Can't process PoolCommand::OpenAck for handle {:?} with result {:?} - appropriate callback not found!", handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
            PoolCommand::List(cb) => {
                debug!(target: "pool_command_executor", "List command received");
                cb(self.list(), self.metrics_service.clone());
            }
            PoolCommand::Close(handle, cb) => {
                debug!(target: "pool_command_executor", "Close command received");
                self.close(handle, cb);
            }
            PoolCommand::CloseAck(handle, result) => {
                debug!(target: "pool_command_executor", "CloseAck command received");
                match self.close_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => cb(result.map_err(IndyError::from), self.metrics_service.clone()),
                            None => {
                                error!("Can't process PoolCommand::CloseAck for handle {:?} with result {:?} - appropriate callback not found!", handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
            PoolCommand::Refresh(handle, cb) => {
                debug!(target: "pool_command_executor", "Refresh command received");
                self.refresh(handle, cb);
            }
            PoolCommand::RefreshAck(handle, result) => {
                debug!(target: "pool_command_executor", "RefreshAck command received");
                match self.refresh_callbacks.try_borrow_mut() {
                    Ok(mut cbs) => {
                        match cbs.remove(&handle) {
                            Some(cb) => cb(result, self.metrics_service.clone()),
                            None => {
                                error!("Can't process PoolCommand::RefreshAck for handle {:?} with result {:?} - appropriate callback not found!",
                                       handle, result);
                            }
                        }
                    }
                    Err(err) => { error!("{:?}", err); }
                }
            }
            PoolCommand::SetProtocolVersion(protocol_version, cb) => {
                debug!(target: "pool_command_executor", "SetProtocolVersion command received");
                cb(self.set_protocol_version(protocol_version), self.metrics_service.clone());
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

    fn open(&self, name: &str, config: Option<PoolOpenConfig>, cb: Box<dyn Fn(IndyResult<PoolHandle>, Rc<MetricsService>) + Send>) {
        debug!("open >>> name: {:?}, config: {:?}", name, config);

        let result = self.pool_service.open(name, config)
            .and_then(|handle| {
                match self.open_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, handle)),
                    Err(err) => Err(err.into()),
                }
            });
        match result {
            Err(err) => { cb(Err(err), self.metrics_service.clone()); }
            Ok((mut cbs, handle)) => { cbs.insert(handle, cb); /* TODO check if map contains same key */ }
        };

        debug!("open <<<");
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

    fn close(&self, pool_handle: PoolHandle, cb: Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>) {
        debug!("close >>> handle: {:?}", pool_handle);

        let result = self.pool_service.close(pool_handle)
            .and_then(|cmd_id| {
                match self.close_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, cmd_id)),
                    Err(err) => Err(err.into())
                }
            });
        match result {
            Err(err) => { cb(Err(err), self.metrics_service.clone()); }
            Ok((mut cbs, cmd_id)) => { cbs.insert(cmd_id, cb); /* TODO check if map contains same key */ }
        };

        debug!("close <<<");
    }

    fn refresh(&self, handle: PoolHandle, cb: Box<dyn Fn(IndyResult<()>, Rc<MetricsService>) + Send>) {
        debug!("refresh >>> handle: {:?}", handle);

        let result = self.pool_service.refresh(handle)
            .and_then(|handle| {
                match self.refresh_callbacks.try_borrow_mut() {
                    Ok(cbs) => Ok((cbs, handle)),
                    Err(err) => Err(err.into())
                }
            });
        match result {
            Err(err) => { cb(Err(err), self.metrics_service.clone()); }
            Ok((mut cbs, handle)) => { cbs.insert(handle, cb); /* TODO check if map contains same key */ }
        };

        debug!("refresh <<<");
    }

    fn set_protocol_version(&self, version: usize) -> IndyResult<()> {
        debug!("set_protocol_version >>> version: {:?}", version);

        if version != 1 && version != 2 {
            return Err(err_msg(IndyErrorKind::PoolIncompatibleProtocolVersion, format!("Unsupported Protocol version: {}", version)));
        }

        ProtocolVersion::set(version);

        debug!("set_protocol_version <<<");

        Ok(())
    }
}
