use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_cur_time() -> u128 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has gone backwards");
    since_epoch.as_millis()
}

