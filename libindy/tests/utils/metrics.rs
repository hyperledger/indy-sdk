extern crate futures;

use indy::IndyError;
use indy::metrics;

use crate::indy::future::Future;

pub fn collect_metrics() -> Result<String, IndyError> {
    metrics::collect_metrics().wait()
}