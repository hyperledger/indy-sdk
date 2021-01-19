
use indy_api_types::{ErrorCode, IndyHandle, CommandHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::blob_storage::BlobStorageCommand;
use indy_api_types::errors::prelude::*;
use indy_utils::ctypes;

use libc::c_char;
use crate::services::metrics::MetricsService;
use std::rc::Rc;
use crate::utils::time::get_cur_time;
use crate::services::metrics::command_metrics::CommandMetric;

#[no_mangle]
pub extern fn indy_open_blob_storage_reader(command_handle: CommandHandle,
                                            type_: *const c_char,
                                            config_json: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 handle: IndyHandle)>) -> ErrorCode {
    trace!("indy_open_blob_storage_reader: >>> type_: {:?}, config_json: {:?}", type_, config_json);

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(config_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_open_blob_storage_reader: entities >>> type_: {:?}, config_json: {:?}", type_, config_json);

    let result = CommandExecutor::instance()
        .send(Command::BlobStorage(BlobStorageCommand::OpenReader(
            type_,
            config_json,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, handle) = prepare_result_1!(result, 0);
                trace!("indy_open_blob_storage_reader: handle: {:?}", handle);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, handle);
                metrics_service.cmd_callback(CommandMetric::BlobStorageCommandOpenReader,get_cur_time() - start_execution_ts);

                result
            }),
        )));

    let res = prepare_result!(result);

    trace!("indy_open_blob_storage_reader: <<< res: {:?}", res);

    res
}

#[no_mangle]
pub extern fn indy_open_blob_storage_writer(command_handle: CommandHandle,
                                            type_: *const c_char,
                                            config_json: *const c_char,
                                            cb: Option<extern fn(command_handle_: CommandHandle,
                                                                 err: ErrorCode,
                                                                 handle: IndyHandle)>) -> ErrorCode {
    trace!("indy_open_blob_storage_writer: >>> type_: {:?}, config_json: {:?}", type_, config_json);

    check_useful_c_str!(type_, ErrorCode::CommonInvalidParam2);
    check_useful_c_str!(config_json, ErrorCode::CommonInvalidParam3);
    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam4);

    trace!("indy_open_blob_storage_writer: entities >>> type_: {:?}, config_json: {:?}", type_, config_json);

    let result = CommandExecutor::instance()
        .send(Command::BlobStorage(BlobStorageCommand::OpenWriter(
            type_,
            config_json,
            Box::new(move |result, metrics_service: Rc<MetricsService>| {
                let (err, handle) = prepare_result_1!(result, 0);
                trace!("indy_open_blob_storage_writer: handle: {:?}", handle);
                let start_execution_ts = get_cur_time();
                let result = cb(command_handle, err, handle);
                metrics_service.cmd_callback(CommandMetric::BlobStorageCommandOpenWriter,get_cur_time() - start_execution_ts);

                result
            }),
        )));

    let res = prepare_result!(result);

    trace!("indy_open_blob_storage_writer: <<< res: {:?}", res);

    res
}
