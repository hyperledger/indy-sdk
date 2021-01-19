use indy_api_types::{ErrorCode, CommandHandle};
use crate::commands::{Command, CommandExecutor};
use crate::commands::metrics::MetricsCommand;
use indy_utils::ctypes;
use libc::c_char;
use crate::services::metrics::MetricsService;
use std::rc::Rc;
use crate::utils::time::get_cur_time;
use crate::services::metrics::command_metrics::CommandMetric;

/// Collect metrics.
///
/// #Returns
/// Map in the JSON format. Where keys are names of metrics.
///
/// #Errors
/// Common*
#[no_mangle]
pub extern fn indy_collect_metrics(command_handle: CommandHandle,
                                   cb: Option<extern fn(command_handle_: CommandHandle,
                                                        err: ErrorCode,
                                                        metrics_json: *const c_char)>) -> ErrorCode {
    trace!("indy_collect_metrics: >>> command_handle: {:?}, cb: {:?}",
           command_handle, cb);

    check_useful_c_callback!(cb, ErrorCode::CommonInvalidParam3);
    let result = CommandExecutor::instance()
        .send(Command::Metrics(MetricsCommand::CollectMetrics(
            boxed_callback_string!("indy_collect_metrics", cb, command_handle, CommandMetric::MetricsCommandCollectMetrics)
        )));
    let res = prepare_result!(result);
    trace!("indy_collect_metrics: <<< res: {:?}", res);
    res
}
