use crate::services::metrics::command_metrics::CommandMetric;
use convert_case::{Case, Casing};
use indy_api_types::errors::{IndyErrorKind, IndyResult, IndyResultExt};
use models::{MetricsValue, CommandCounters};
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::collections::HashMap;

pub mod command_metrics;
pub mod models;

const COMMANDS_COUNT: usize = MetricsService::commands_count();

pub struct MetricsService {
    queued_counters: RefCell<[CommandCounters; COMMANDS_COUNT]>,
    executed_counters: RefCell<[CommandCounters; COMMANDS_COUNT]>,
    callback_counters: RefCell<[CommandCounters; COMMANDS_COUNT]>,
}

impl MetricsService {
    pub fn new() -> Self {
        MetricsService {
            queued_counters: RefCell::new([CommandCounters::new(); COMMANDS_COUNT]),
            executed_counters: RefCell::new([CommandCounters::new(); COMMANDS_COUNT]),
            callback_counters: RefCell::new([CommandCounters::new(); COMMANDS_COUNT]),
        }
    }

    pub fn cmd_left_queue(&self, command_metric: CommandMetric, duration: u128) {
        self.queued_counters.borrow_mut()[command_metric as usize].add(duration);
    }

    pub fn cmd_executed(&self, command_metric: CommandMetric, duration: u128) {
        self.executed_counters.borrow_mut()[command_metric as usize].add(duration);
    }

    pub fn cmd_callback(&self, command_index: CommandMetric, duration: u128) {
        self.callback_counters.borrow_mut()[command_index as usize].add(duration);
    }

    pub fn cmd_name(index: usize) -> String {
        CommandMetric::from(index).to_string().to_case(Case::Snake)
    }

    const fn commands_count() -> usize {
        CommandMetric::VARIANT_COUNT
    }

    pub fn get_command_tags(
        command: String,
        stage: String,
    ) -> HashMap<String, String> {
        let mut tags = HashMap::<String, String>::new();
        tags.insert("command".to_owned(), command.to_owned());
        tags.insert("stage".to_owned(), stage.to_owned());
        tags
    }

    pub fn append_command_metrics(&self, metrics_map: &mut Map<String, Value>) -> IndyResult<()> {
        let mut commands_count = Vec::new();
        let mut commands_duration_ms = Vec::new();
        let mut commands_duration_ms_bucket = Vec::new();
        let mut commands_callback = Vec::new();

        for index in (0..MetricsService::commands_count()).rev() {
            let command_name = MetricsService::cmd_name(index);
            let tags_executed = MetricsService::get_command_tags(
                command_name.to_owned(),
                String::from("executed"),
            );
            let tags_queued = MetricsService::get_command_tags(
                command_name.to_owned(),
                String::from("queued"),
            );
            let tags_callback = MetricsService::get_command_tags(
                command_name.clone(),
                String::from("callback"),
            );

            commands_count.push(self.get_metric_json(self.executed_counters.borrow()[index].count as usize, tags_executed.clone())?);
            commands_count.push(self.get_metric_json(self.queued_counters.borrow()[index].count as usize, tags_queued.clone())?);

            commands_duration_ms.push(self.get_metric_json(self.executed_counters.borrow()[index].duration_ms_sum as usize, tags_executed.clone())?);
            commands_duration_ms.push(self.get_metric_json(self.queued_counters.borrow()[index].duration_ms_sum as usize,tags_queued.clone())?);
            commands_duration_ms.push(self.get_metric_json(self.callback_counters.borrow()[index].duration_ms_sum as usize,tags_callback.clone())?);

            commands_callback.push(self.get_metric_json(self.callback_counters.borrow()[index].count as usize, tags_callback.clone())?);

            for index_bucket in (0..self.executed_counters.borrow()[index].duration_ms_bucket.len()).rev() {
                let executed_bucket = self.executed_counters.borrow()[index].duration_ms_bucket[index_bucket];
                let queued_bucket = self.queued_counters.borrow()[index].duration_ms_bucket[index_bucket];
                let callback_bucket = self.callback_counters.borrow()[index].duration_ms_bucket[index_bucket];

                commands_duration_ms_bucket.push(self.get_metric_json(executed_bucket as usize, tags_executed.clone())?);
                commands_duration_ms_bucket.push(self.get_metric_json(queued_bucket as usize, tags_queued.clone())?);
                commands_duration_ms_bucket.push(self.get_metric_json(callback_bucket as usize, tags_callback.clone())?);
            }
        }

        metrics_map.insert(
            String::from("commands_count"),
            serde_json::to_value(commands_count)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );
        metrics_map.insert(
            String::from("commands_duration_ms"),
            serde_json::to_value(commands_duration_ms)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );
        metrics_map.insert(
            String::from("commands_duration_ms_bucket"),
            serde_json::to_value(commands_duration_ms_bucket)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );
        metrics_map.insert(
            String::from("commands_callback"),
            serde_json::to_value(commands_callback)
                .to_indy(IndyErrorKind::IOError, "Unable to convert json")?,
        );

        Ok(())
    }

    fn get_metric_json(&self, value: usize, tags: HashMap<String, String>) -> IndyResult<Value> {
        let res = serde_json::to_value(MetricsValue::new(
            value,
            tags,
        )).to_indy(IndyErrorKind::IOError, "Unable to convert json")?;

        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_counters_are_initialized() {
        let metrics_service = MetricsService::new();
        assert_eq!(metrics_service.queued_counters.borrow().len(), COMMANDS_COUNT);
        assert_eq!(metrics_service.executed_counters.borrow().len(), COMMANDS_COUNT);
        assert_eq!(metrics_service.callback_counters.borrow().len(), COMMANDS_COUNT);
    }

    #[test]
    fn test_cmd_left_queue_increments_relevant_queued_counters() {
        let metrics_service = MetricsService::new();
        let index = CommandMetric::IssuerCommandCreateSchema;
        let duration1 = 5u128;
        let duration2 = 2u128;

        metrics_service.cmd_left_queue(index, duration1);

        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].count, 1);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].duration_ms_sum, duration1);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                            metrics_service.queued_counters.borrow()[index as usize]
                            .duration_ms_bucket.len()-1
                       ],
                    1
        );

        metrics_service.cmd_left_queue(index, duration2);

        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].count, 1 + 1);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].duration_ms_sum,
                   duration1 + duration2);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize]
                        .duration_ms_bucket[
                            metrics_service.queued_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                    2
        );

        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                            metrics_service.executed_counters.borrow()[index as usize]
                            .duration_ms_bucket.len()-1
                       ],
                    0
        );

        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.callback_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   0
        );
    }

    #[test]
    fn test_cmd_executed_increments_relevant_executed_counters() {
        let metrics_service = MetricsService::new();
        let index = CommandMetric::IssuerCommandCreateSchema;
        let duration1 = 5u128;
        let duration2 = 2u128;

        metrics_service.cmd_executed(index, duration1);

        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].count, 1);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].duration_ms_sum, duration1);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.executed_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   1
        );

        metrics_service.cmd_executed(index, duration2);

        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].count, 1 + 1);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].duration_ms_sum,
                   duration1 + duration2);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.executed_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   2
        );

        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.queued_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   0
        );

        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.callback_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   0
        );
    }

    #[test]
    fn test_cmd_callback_increments_relevant_callback_counters() {
        let metrics_service = MetricsService::new();
        let index = CommandMetric::IssuerCommandCreateSchema;
        let duration1 = 5u128;
        let duration2 = 2u128;

        metrics_service.cmd_callback(index, duration1);

        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].count, 1);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].duration_ms_sum, duration1);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.callback_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   1
        );

        metrics_service.cmd_callback(index, duration2);

        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].count, 1 + 1);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize].duration_ms_sum,
                   duration1 + duration2);
        assert_eq!(metrics_service.callback_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.callback_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   2
        );

        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.executed_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.executed_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   0
        );

        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].count, 0);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize].duration_ms_sum, 0);
        assert_eq!(metrics_service.queued_counters.borrow()[index as usize]
                       .duration_ms_bucket[
                       metrics_service.queued_counters.borrow()[index as usize]
                           .duration_ms_bucket.len()-1
                       ],
                   0
        );
    }

    #[test]
    fn test_append_command_count_metrics() {
        let metrics_service = MetricsService::new();
        let mut metrics_map = serde_json::Map::new();

        metrics_service.append_command_metrics(&mut metrics_map).unwrap();

        assert!(metrics_map.contains_key("commands_count"));
        assert_eq!(
            metrics_map
                .get("commands_count")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            COMMANDS_COUNT * 2
        );

        let commands_count = metrics_map
            .get("commands_count")
            .unwrap()
            .as_array()
            .unwrap();

        let expected_commands_count = [
            generate_json("payments_command_build_set_txn_fees_req_ack", "executed", 0),
            generate_json("metrics_command_collect_metrics", "queued", 0),
            generate_json("cache_command_purge_cred_def_cache", "executed", 0),
            generate_json("non_secrets_command_fetch_search_next_records", "queued", 0),
        ];

        assert!(commands_count.contains(&expected_commands_count[0]));
        assert!(commands_count.contains(&expected_commands_count[1]));
        assert!(commands_count.contains(&expected_commands_count[2]));
        assert!(commands_count.contains(&expected_commands_count[3]));
    }

    #[test]
    fn test_append_command_duration_ms_metrics() {
        let metrics_service = MetricsService::new();
        let mut metrics_map = serde_json::Map::new();

        metrics_service.append_command_metrics(&mut metrics_map);

        assert!(metrics_map.contains_key("commands_duration_ms"));
        assert_eq!(
            metrics_map
                .get("commands_duration_ms")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            COMMANDS_COUNT * 3
        );

        let commands_duration_ms = metrics_map
            .get("commands_duration_ms")
            .unwrap()
            .as_array()
            .unwrap();

        let expected_commands_duration_ms = [
            generate_json("payments_command_build_set_txn_fees_req_ack", "executed", 0),
            generate_json("metrics_command_collect_metrics", "queued", 0),
            generate_json("cache_command_purge_cred_def_cache", "executed", 0),
            generate_json("non_secrets_command_fetch_search_next_records", "queued", 0),
        ];

        assert!(commands_duration_ms.contains(&expected_commands_duration_ms[0]));
        assert!(commands_duration_ms.contains(&expected_commands_duration_ms[1]));
        assert!(commands_duration_ms.contains(&expected_commands_duration_ms[2]));
        assert!(commands_duration_ms.contains(&expected_commands_duration_ms[3]));
    }

    #[test]
    fn test_append_command_duration_ms_bucket_metrics() {
        let metrics_service = MetricsService::new();
        let mut metrics_map = serde_json::Map::new();

        metrics_service.append_command_metrics(&mut metrics_map);

        assert!(metrics_map.contains_key("commands_duration_ms_bucket"));
        assert_eq!(
            metrics_map
                .get("commands_duration_ms_bucket")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            COMMANDS_COUNT * 48
        );

        let commands_duration_ms_bucket = metrics_map
            .get("commands_duration_ms_bucket")
            .unwrap()
            .as_array()
            .unwrap();

        let expected_commands_duration_ms_bucket = [
            generate_json("payments_command_build_set_txn_fees_req_ack", "executed", 0),
            generate_json("metrics_command_collect_metrics", "queued", 0),
            generate_json("cache_command_purge_cred_def_cache", "executed", 0),
            generate_json("non_secrets_command_fetch_search_next_records", "queued", 0),
        ];

        assert!(commands_duration_ms_bucket.contains(&expected_commands_duration_ms_bucket[0]));
        assert!(commands_duration_ms_bucket.contains(&expected_commands_duration_ms_bucket[1]));
        assert!(commands_duration_ms_bucket.contains(&expected_commands_duration_ms_bucket[2]));
        assert!(commands_duration_ms_bucket.contains(&expected_commands_duration_ms_bucket[3]));
    }

    #[test]
    fn test_append_command_callback_metrics() {
        let metrics_service = MetricsService::new();
        let mut metrics_map = serde_json::Map::new();

        metrics_service.append_command_metrics(&mut metrics_map);

        assert!(metrics_map.contains_key("commands_callback"));
        assert_eq!(
            metrics_map
                .get("commands_callback")
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            COMMANDS_COUNT
        );

        let commands_callback = metrics_map
            .get("commands_callback")
            .unwrap()
            .as_array()
            .unwrap();
        let expected_commands_callback = [
            generate_json("payments_command_build_set_txn_fees_req_ack", "callback", 0),
            generate_json("metrics_command_collect_metrics", "callback", 0),
            generate_json("cache_command_purge_cred_def_cache", "callback", 0),
            generate_json("non_secrets_command_fetch_search_next_records", "callback", 0),
        ];

        assert!(commands_callback.contains(&expected_commands_callback[0]));
        assert!(commands_callback.contains(&expected_commands_callback[1]));
        assert!(commands_callback.contains(&expected_commands_callback[2]));
        assert!(commands_callback.contains(&expected_commands_callback[3]));
    }

    fn generate_json(command: &str, stage: &str, value: usize) -> Value {
        json!({"tags":{"command": command, "stage": stage} ,"value": value})
    }
}