use crate::services::blob_storage::BlobStorageService;
use std::rc::Rc;

use indy_api_types::errors::prelude::*;
use crate::services::metrics::MetricsService;

pub enum BlobStorageCommand {
    OpenReader(
        String, // type
        String, // config
        Box<dyn Fn(IndyResult<i32 /* handle */>, Rc<MetricsService>) + Send>),
    OpenWriter(
        String, // writer type
        String, // writer config JSON
        Box<dyn Fn(IndyResult<i32 /* handle */>, Rc<MetricsService>) + Send>),
}

pub struct BlobStorageCommandExecutor {
    blob_storage_service: Rc<BlobStorageService>,
    metrics_service: Rc<MetricsService>,
}

impl BlobStorageCommandExecutor {
    pub fn new(blob_storage_service: Rc<BlobStorageService>, metrics_service: Rc<MetricsService>) -> BlobStorageCommandExecutor {
        BlobStorageCommandExecutor {
            blob_storage_service,
            metrics_service,
        }
    }

    pub fn execute(&self, command: BlobStorageCommand) {
        match command {
            BlobStorageCommand::OpenReader(type_, config, cb) => {
                debug!("OpenReader command received");
                cb(self.open_reader(&type_, &config), self.metrics_service.clone());
            }
            BlobStorageCommand::OpenWriter(writer_type, writer_config, cb) => {
                debug!("OpenWriter command received");
                cb(self.open_writer(&writer_type, &writer_config), self.metrics_service.clone());
            }
        }
    }

    fn open_reader(&self, type_: &str, config: &str) -> IndyResult<i32> {
        debug!("open_reader >>> type_: {:?}, config: {:?}", type_, config);

        let res = self.blob_storage_service.open_reader(type_, config).map_err(IndyError::from);

        debug!("open_reader << res: {:?}", res);

        res
    }

    fn open_writer(&self, type_: &str, config: &str) -> IndyResult<i32> {
        debug!("open_writer >>> type_: {:?}, config: {:?}", type_, config);

        let res = self.blob_storage_service.open_writer(type_, config).map_err(IndyError::from);

        debug!("open_writer << res: {:?}", res);

        res
    }
}
