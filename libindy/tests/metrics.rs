#[macro_use]
mod utils;

inject_indy_dependencies!();

extern crate indyrs as api;
extern crate indyrs as indy;
use crate::utils::constants::*;
use crate::utils::metrics;
use crate::utils::wallet;
use crate::utils::Setup;

mod collect {
    use super::*;
    use std::collections::HashMap;
    use serde_json::Value;
    use api::anoncreds;
    use indy_sys::WalletHandle;

    #[test]
    fn test_metrics_schema() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics)
            .expect("Top level object should be a dictionary");

        for metrics_set in metrics_map.values() {
            let metrics_set = metrics_set.as_array().expect("Metrics set should be an array");

            for metric in metrics_set.iter() {
                let metrics = metric.as_object().expect("Metrics should be an object");
                metrics.contains_key("value");
                metrics.contains_key("tags");
            }
        }
    }

    #[test]
    fn collect_metrics_contains_wallet_service_statistics() {
        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("wallet_count"));

        let wallet_count = metrics_map
            .get("wallet_count")
            .unwrap()
            .as_array()
            .unwrap();

        assert!(wallet_count.contains(&json!({"tags":{"label":"opened"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"opened_ids"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"pending_for_import"},"value":0})));
        assert!(wallet_count.contains(&json!({"tags":{"label":"pending_for_open"},"value":0})));
    }

    #[test]
    fn collect_metrics_contains_thread_pool_service_statistics() {
        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("threadpool_threads_count"));

        let threadpool_threads_count = metrics_map
            .get("threadpool_threads_count")
            .unwrap()
            .as_array()
            .unwrap();

        assert!(threadpool_threads_count.contains(&json!({"tags":{"label":"active"},"value":0})));
        assert!(threadpool_threads_count.contains(&json!({"tags":{"label":"queued"},"value":0})));
        assert!(threadpool_threads_count.contains(&json!({"tags":{"label":"panic"},"value":0})));
    }

    #[test]
    fn collect_metrics_includes_commands_count() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("commands_count"));

        let commands_count = metrics_map
            .get("commands_count")
            .unwrap()
            .as_array()
            .unwrap();

        assert!(commands_count.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed"} ,"value": 0})));
        assert!(commands_count.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "queued"} ,"value": 0})));
        assert!(commands_count.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed"} ,"value": 0})));
        assert!(commands_count.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "queued"} ,"value": 0})));
    }

    #[test]
    fn collect_metrics_includes_commands_duration_ms() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("commands_duration_ms"));

        let commands_duration_ms = metrics_map
            .get("commands_duration_ms")
            .unwrap()
            .as_array()
            .unwrap();

        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed"} ,"value": 0})));
        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "queued"} ,"value": 0})));
        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed"} ,"value": 0})));
        assert!(commands_duration_ms.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "queued"} ,"value": 0})));
    }

    #[test]
    fn collect_metrics_includes_commands_duration_ms_bucket() {
        let setup = Setup::empty();
        let config = config(&setup.name);
        wallet::create_wallet(&config, WALLET_CREDENTIALS).unwrap();

        let result_metrics = metrics::collect_metrics().unwrap();
        let metrics_map = serde_json::from_str::<HashMap<String, Value>>(&result_metrics).unwrap();

        assert!(metrics_map.contains_key("commands_duration_ms_bucket"));

        let commands_duration_ms_bucket = metrics_map
            .get("commands_duration_ms_bucket")
            .unwrap()
            .as_array()
            .unwrap();

        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "executed"} ,"value": 0})));
        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "pairwise_command_pairwise_exists", "stage": "queued"} ,"value": 0})));
        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "executed"} ,"value": 0})));
        assert!(commands_duration_ms_bucket.contains(&json!({"tags":{"command": "payments_command_build_set_txn_fees_req_ack", "stage": "queued"} ,"value": 0})));
    }

    fn config(name: &str) -> String {
        json!({ "id": name }).to_string()
    }

}
