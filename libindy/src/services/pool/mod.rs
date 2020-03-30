extern crate hex;
extern crate rand;
extern crate rmp_serde;
extern crate time;

use std::{fs, io};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
use std::ffi::{CString, CStr};

use serde_json;
use serde::de::DeserializeOwned;

use crate::api::ledger::{CustomFree, CustomTransactionParser};
use crate::domain::pool::{
    PoolConfig,
    PoolOpenConfig,
};
use crate::services::ledger::parsers::response::{
    Message,
    Reply,
    ResponseMetadata,
};

use indy_api_types::errors::*;
use crate::utils::environment;
use indy_api_types::{PoolHandle, ErrorCode};
use indy_utils::next_pool_handle;

use indy_vdr::pool::{LocalPool, PoolBuilder, PoolTransactions, helpers::perform_refresh, Pool, RequestResult};
use indy_vdr::pool::helpers::format_full_reply;
use indy_vdr::pool::handlers::{handle_full_request, handle_consensus_request};
use indy_vdr::ledger::PreparedRequest;
use indy_vdr::config::{PoolConfig as VdrPoolOpenConfig, ProtocolVersion};
use indy_vdr::ledger::constants::{POOL_RESTART, GET_VALIDATOR_INFO};
use indy_vdr::state_proof::types::ParsedSP;
use indy_vdr::state_proof::constants::REQUESTS_FOR_STATE_PROOFS;

mod pool_transactions;

lazy_static! {
    static ref REGISTERED_SP_PARSERS: Mutex<HashMap<String, (CustomTransactionParser, CustomFree)>> = Mutex::new(HashMap::new());
}

pub struct PoolService {
    open_pools: RefCell<HashMap<PoolHandle, PoolDescriptor>>,
}

struct PoolDescriptor {
    name: String,
    pool: LocalPool,
}

impl PoolService {
    pub fn new() -> PoolService {
        PoolService {
            open_pools: RefCell::new(HashMap::new()),
        }
    }

    pub fn create(&self, name: &str, config: Option<PoolConfig>) -> IndyResult<()> {
        trace!("PoolService::create {} with config {:?}", name, config);

        let mut path = environment::pool_path(name);
        let pool_config = config.unwrap_or_else(|| PoolConfig::default_for_name(name));

        if path.as_path().exists() {
            return Err(err_msg(IndyErrorKind::PoolConfigAlreadyExists, format!("Pool ledger config file with name \"{}\" already exists", name)));
        }

        // check that we can build MerkeleTree from genesis transaction file
        PoolTransactions::from_file(&pool_config.genesis_txn)?;

        fs::create_dir_all(path.as_path())
            .to_indy(IndyErrorKind::IOError, "Can't create pool config directory")?;

        path.push(name);
        path.set_extension("txn");

        {
            // fs::copy also copies attributes of the file
            // and copying permissions can be problem for some cases

            let mut gt_fin = fs::File::open(&pool_config.genesis_txn)
                .to_indy(IndyErrorKind::IOError,
                         format!("Can't open genesis txn file {:?}", &pool_config.genesis_txn))?;

            let mut gt_fout = fs::File::create(path.as_path())
                .to_indy(IndyErrorKind::IOError,
                         format!("Can't create genesis txn file {:?}", path.as_path()))?;

            io::copy(&mut gt_fin, &mut gt_fout)
                .to_indy(IndyErrorKind::IOError,
                         format!("Can't copy genesis txn file from {:?} to {:?}",
                                 &pool_config.genesis_txn, path.as_path()))?;
        }

        path.pop();
        path.push("config");
        path.set_extension("json");

        let mut f: fs::File = fs::File::create(path.as_path())
            .to_indy(IndyErrorKind::IOError, "Can't create pool config file")?;

        f
            .write_all({
                serde_json::to_string(&pool_config)
                    .to_indy(IndyErrorKind::InvalidState, "Can't serialize pool config")?
                    .as_bytes()
            })
            .to_indy(IndyErrorKind::IOError, "Can't write to pool config file")?;

        f
            .flush()
            .to_indy(IndyErrorKind::IOError, "Can't write to pool config file")?;

        // TODO probably create another one file pool.json with pool description,
        // but now there is no info to save (except name witch equal to directory)
        Ok(())
    }

    pub fn delete(&self, name: &str) -> IndyResult<()> {
        if self.open_pools.try_borrow()?.values()
            .find(|pool| pool.name.eq(name)).is_some() {
            return Err(err_msg(IndyErrorKind::InvalidState, "Can't delete pool config - pool is open now"));
        }

        let path = environment::pool_path(name);

        fs::remove_dir_all(path)
            .to_indy(IndyErrorKind::IOError, "Can't delete pool config directory")
    }

    pub async fn open(&self, name: String, config: Option<PoolOpenConfig>, protocol_version: ProtocolVersion) -> IndyResult<PoolHandle> {
        if self.open_pools.try_borrow()?.values()
            .find(|pool| pool.name.eq(&name)).is_some() {
            return Err(err_msg(IndyErrorKind::InvalidPoolHandle, "Pool with the same name is already opened"));
        }

        let mut config = VdrPoolOpenConfig::from(config.unwrap_or_default());
        config.protocol_version = protocol_version;

        let transactions = pool_transactions::create(&name)?;

        let mut pool = self._build_pool(&config, transactions)?;

        pool = self._refresh_pool(&pool, &name).await?;

        let pool_handle: PoolHandle = next_pool_handle();
        let pool_descriptor = PoolDescriptor { name, pool };

        self.open_pools.try_borrow_mut()?.insert(pool_handle, pool_descriptor);

        Ok(pool_handle)
    }

    pub async fn send_tx(&self, handle: PoolHandle, msg: &str) -> IndyResult<String> {
        let prepared_request: PreparedRequest = PreparedRequest::from_request_json(msg)?;

        if prepared_request.txn_type == POOL_RESTART.to_string() || prepared_request.txn_type == GET_VALIDATOR_INFO.to_string() {
            return self._send_action(handle, prepared_request, None, None).await;
        }

        self.send_request(handle, prepared_request).await
    }

    pub async fn send_request(&self, handle: PoolHandle, prepared_request: PreparedRequest) -> IndyResult<String> {
        let (request_result, _timing) = {
            let borrowed_pools = self.open_pools.try_borrow()?;

            let pool = borrowed_pools.get(&handle)
                .ok_or(err_msg(IndyErrorKind::InvalidPoolHandle, format!("No pool with requested handle {:?}", handle)))?;

            let request = pool.pool
                .create_request(prepared_request.req_id.to_string(),
                                prepared_request.req_json.to_string()).await?;

            let custom_state_proofs_parser: Option<Box<dyn Fn(&str, &str) -> Option<Vec<ParsedSP>>>> =
                if REGISTERED_SP_PARSERS.lock().unwrap().contains_key(&prepared_request.txn_type) {
                    Some(Box::new(Self::custom_state_proofs_parser))
                } else { None };

            handle_consensus_request(request,
                                     prepared_request.sp_key,
                                     prepared_request.sp_timestamps,
                                     prepared_request.is_read_request,
                                     custom_state_proofs_parser,
            )
        }.await?;

        _get_response(request_result)
    }

    pub async fn send_action(&self, handle: PoolHandle, msg: &str, node_aliases: Option<Vec<String>>, timeout: Option<i64>) -> IndyResult<String> {
        let prepared_request: PreparedRequest = PreparedRequest::from_request_json(msg)?;
        self._send_action(handle, prepared_request, node_aliases, timeout).await
    }

    async fn _send_action(&self, handle: PoolHandle, prepared_request: PreparedRequest, node_aliases: Option<Vec<String>>, timeout: Option<i64>) -> IndyResult<String> {
        let (request_result, _timing) = {
            let borrowed_pools = self.open_pools.try_borrow()?;

            let pool = borrowed_pools.get(&handle)
                .ok_or(err_msg(IndyErrorKind::InvalidPoolHandle, format!("No pool with requested handle {:?}", handle)))?;

            if let Some(ref nodes_) = node_aliases.as_ref() {
                let node_names = pool.pool.get_node_aliases();
                if nodes_.iter().any(|node| !node_names.contains(node)) {
                    return Err(IndyError::from(IndyErrorKind::InvalidStructure));
                }
            }

            let request = pool.pool
                .create_request(prepared_request.req_id.to_string(),
                                prepared_request.req_json.to_string()).await?;

            handle_full_request(request,
                                node_aliases,
                                timeout)
        }.await?;

        let request_result = request_result.map_result(format_full_reply)?;

        _get_response(request_result)
    }

    pub fn register_sp_parser(txn_type: &str,
                              parser: CustomTransactionParser, free: CustomFree) -> IndyResult<()> {
        if REQUESTS_FOR_STATE_PROOFS.contains(&txn_type) {
            return Err(err_msg(IndyErrorKind::InvalidStructure,
                               format!("Try to override StateProof parser for default TXN_TYPE {}", txn_type)));
        }

        REGISTERED_SP_PARSERS.lock()
            .map(|mut map| {
                map.insert(txn_type.to_owned(), (parser, free));
            })
            .unwrap(); // FIXME: Can we avoid unwrap?

        Ok(())
    }

    pub fn get_sp_parser(txn_type: &str) -> Option<(CustomTransactionParser, CustomFree)> {
        let parsers = REGISTERED_SP_PARSERS.lock().unwrap(); // FIXME: Can we avoid unwrap here?
        parsers.get(txn_type).map(Clone::clone)
    }

    pub async fn close(&self, handle: PoolHandle) -> IndyResult<()> {
        let mut pools = self.open_pools.try_borrow_mut()?;

        match pools.remove(&handle) {
            Some(_pool) => Ok(()),
            None => Err(err_msg(IndyErrorKind::InvalidPoolHandle, format!("No pool with requested handle {}", handle)))
        }
    }

    pub async fn refresh(&self, handle: PoolHandle) -> IndyResult<()> {
        let mut borrowed_pools = self.open_pools.try_borrow_mut()?;

        let mut pool = borrowed_pools.get_mut(&handle)
            .ok_or(err_msg(IndyErrorKind::InvalidPoolHandle, format!("No pool with requested handle {:?}", handle)))?;

        pool.pool = self._refresh_pool(&pool.pool, &pool.name).await?;

        Ok(())
    }

    pub fn list(&self) -> IndyResult<Vec<serde_json::Value>> {
        let mut pool = Vec::new();
        let pool_home_path = environment::pool_home_path();

        if let Ok(entries) = fs::read_dir(pool_home_path) {
            for entry in entries {
                let dir_entry = if let Ok(dir_entry) = entry { dir_entry } else { continue; };
                if let Some(pool_name) = dir_entry.path().file_name().and_then(|os_str| os_str.to_str()) {
                    let json = json!({"pool":pool_name.to_owned()});
                    pool.push(json);
                }
            }
        }

        Ok(pool)
    }

    async fn _refresh_pool(&self, pool: &LocalPool, pool_name: &str) -> IndyResult<LocalPool> {
        let config = pool.get_config().to_owned();

        let transactions = match perform_refresh(pool).await {
            Ok((new_transactions, _timing)) => {
                let mut transactions = pool.get_transactions()?;

                // Drop new transactions to file
                if let Some(new_transaction_) = new_transactions {
                    transactions.extend_from_slice(&new_transaction_);

                    pool_transactions::dump_new_txns(pool_name,
                                                     PoolTransactions::from_transactions_json(&new_transaction_)?.iter())?;
                }

                transactions
            }
            Err(_) => {
                // Drop cache and try again
                pool_transactions::drop_cache(pool_name)?;

                let transactions = pool_transactions::create(pool_name)?;
                let pool = self._build_pool(&config, transactions)?;

                let (new_transactions, _timing) = perform_refresh(&pool).await?;

                let mut transactions = pool.get_transactions()?;

                // Add new transactions to genesis and drop to file
                if let Some(new_transaction_) = new_transactions.as_ref() {
                    transactions.extend_from_slice(&new_transaction_);

                    pool_transactions::dump_new_txns(pool_name,
                                                     PoolTransactions::from_transactions_json(&transactions)?.iter())?;
                }

                transactions
            }
        };

        let transactions = PoolTransactions::from_transactions_json(transactions)?;
        self._build_pool(&config, transactions)
    }

    fn _build_pool(&self, config: &VdrPoolOpenConfig, transactions: PoolTransactions) -> IndyResult<LocalPool> {
        PoolBuilder::from_config(config.to_owned())
            .transactions(transactions)?
            .into_local()
            .map_err(IndyError::from)
    }

    fn custom_state_proofs_parser(txn_type: &str, reply_from_node: &str) -> Option<Vec<ParsedSP>> {
        trace!("PoolService::custom_state_proofs_parser: txn_type {:?}, reply_from_node {:?}",
               txn_type, reply_from_node);

        match Self::get_sp_parser(txn_type) {
            Some((parser, free)) => {
                let reply_from_node = CString::new(reply_from_node).unwrap();

                let mut parsed_c_str = ::std::ptr::null();

                let err = parser(reply_from_node.as_ptr(), &mut parsed_c_str);
                if err != ErrorCode::Success {
                    debug!("PoolService::custom_state_proofs_parser: <<< parser: plugin return err {:?}", err);
                    return None;
                }

                let c_str = if parsed_c_str.is_null() {
                    return None;
                } else {
                    Some(unsafe { CStr::from_ptr(parsed_c_str) })
                };

                let parsed_sps = c_str
                    .and_then(|c_str| c_str.to_str().ok())
                    .and_then(|c_str|
                        serde_json::from_str::<Vec<ParsedSP>>(c_str)
                            .map_err(|err|
                                debug!("PoolService::custom_state_proofs_parser: <<< can't parse plugin response {}", err))
                            .ok());

                let err = free(parsed_c_str);
                if err != ErrorCode::Success {
                    debug!("PoolService::custom_state_proofs_parser: <<< free: plugin return err {:?}", err);
                    return None;
                }

                trace!("PoolService::custom_state_proofs_parser: plugin res {:?}", parsed_sps);

                parsed_sps
            }
            None => None
        }
    }
}

lazy_static! {
    static ref THRESHOLD: Mutex<u64> = Mutex::new(600);
}

pub fn set_freshness_threshold(threshold: u64) {
    let mut th = THRESHOLD.lock().unwrap();
    *th = ::std::cmp::max(threshold, 300);
}

fn get_freshness_threshold() -> u64 {
    THRESHOLD.lock().unwrap().clone()
}


pub fn parse_response_metadata(response: &str) -> IndyResult<ResponseMetadata> {
    trace!("indy::services::pool::parse_response_metadata << response: {}", response);
    let message: Message<serde_json::Value> = serde_json::from_str(response)
        .to_indy(IndyErrorKind::InvalidTransaction, "Cannot deserialize transaction Response")?;

    let response_object: Reply<serde_json::Value> = _handle_response_message_type(message)?;
    let response_result = response_object.result();

    let response_metadata = match response_result["ver"].as_str() {
        None => _parse_transaction_metadata_v0(&response_result),
        Some("1") => _parse_transaction_metadata_v1(&response_result),
        ver => return Err(err_msg(IndyErrorKind::InvalidTransaction, format!("Unsupported transaction response version: {:?}", ver)))
    };

    trace!("indy::services::pool::parse_response_metadata >> response_metadata: {:?}", response_metadata);

    Ok(response_metadata)
}

fn _handle_response_message_type<T>(message: Message<T>) -> IndyResult<Reply<T>> where T: DeserializeOwned + ::std::fmt::Debug {
    trace!("handle_response_message_type >>> message {:?}", message);

    match message {
        Message::Reject(response) | Message::ReqNACK(response) =>
            Err(err_msg(IndyErrorKind::InvalidTransaction, format!("Transaction has been failed: {:?}", response.reason))),
        Message::Reply(reply) =>
            Ok(reply)
    }
}

fn _parse_transaction_metadata_v0(message: &serde_json::Value) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["seqNo"].as_u64(),
        txn_time: message["txnTime"].as_u64(),
        last_txn_time: message["state_proof"]["multi_signature"]["value"]["timestamp"].as_u64(),
        last_seq_no: None,
    }
}

fn _parse_transaction_metadata_v1(message: &serde_json::Value) -> ResponseMetadata {
    ResponseMetadata {
        seq_no: message["txnMetadata"]["seqNo"].as_u64(),
        txn_time: message["txnMetadata"]["txnTime"].as_u64(),
        last_txn_time: message["multiSignature"]["signedState"]["stateMetadata"]["timestamp"].as_u64(),
        last_seq_no: None,
    }
}

fn _get_response(request_result: RequestResult<String>) -> IndyResult<String> {
    match request_result {
        RequestResult::Reply(message) => Ok(message),
        RequestResult::Failed(err) => Ok(err.extra().unwrap_or_default())
    }
}

impl From<PoolOpenConfig> for VdrPoolOpenConfig {
    fn from(pool_config: PoolOpenConfig) -> Self {
        VdrPoolOpenConfig {
            protocol_version: ProtocolVersion::default(),
            freshness_threshold: get_freshness_threshold(),
            ack_timeout: pool_config.timeout,
            reply_timeout: pool_config.extended_timeout,
            conn_request_limit: pool_config.conn_limit,
            conn_active_timeout: pool_config.conn_active_timeout,
            request_read_nodes: pool_config.number_read_nodes as usize,
        }
    }
}

/*#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn fake_pool_handle_for_poolsm() -> (indy_api_types::PoolHandle, oneshot::Receiver<IndyResult<indy_api_types::PoolHandle>>) {
        let pool_handle = indy_utils::next_pool_handle();
        let (sender, receiver) = oneshot::channel();
        super::POOL_HANDLE_SENDERS.lock().unwrap().insert(pool_handle, sender);
        (pool_handle, receiver)
    }

    pub fn fake_cmd_id() -> (indy_api_types::CommandHandle, oneshot::Receiver<IndyResult<String>>) {
        let cmd_id = indy_utils::next_command_handle();
        let (sender, receiver) = oneshot::channel();
        super::SUBMIT_SENDERS.lock().unwrap().insert(cmd_id, sender);
        (cmd_id, receiver)
    }

    pub fn fake_pool_handle_for_close_cmd() -> (indy_api_types::CommandHandle, oneshot::Receiver<IndyResult<()>>) {
        let pool_handle = indy_utils::next_command_handle();
        let (sender, receiver) = oneshot::channel();
        super::CLOSE_SENDERS.lock().unwrap().insert(pool_handle, sender);
        (pool_handle, receiver)
    }
}

#[cfg(test)]
pub mod tests {
    use std::thread;

    use futures::executor::block_on;

    use crate::domain::ledger::request::ProtocolVersion;
    use crate::services::pool::types::*;
    use crate::utils::test;

    use super::*;

    const TEST_PROTOCOL_VERSION: usize = 2;

    fn _set_protocol_version(version: usize) {
        ProtocolVersion::set(version);
    }

    mod pool_service {
        use std::path;

        use libc::c_char;

        use indy_api_types::{ErrorCode, INVALID_POOL_HANDLE};

        use super::*;

        #[test]
        fn pool_service_new_works() {
            PoolService::new();
            assert!(true, "No crashes on PoolService::new");
        }

        #[test]
        fn pool_service_drop_works() {
            fn drop_test() {
                PoolService::new();
            }

            drop_test();
            assert!(true, "No crashes on PoolService::drop");
        }

        #[test]
        fn pool_service_close_works() {
            test::cleanup_storage("pool_service_close_works");

            let ps = PoolService::new();
            let pool_id = next_pool_handle();
            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("pool_service_close_works");
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(Pool::new("", pool_id, PoolOpenConfig::default()), send_cmd_sock));
            let pool_mock = thread::spawn(move || {
                let recv = recv_cmd_sock.recv_multipart(0).unwrap();
                assert_eq!(recv.len(), 3);
                assert_eq!(COMMAND_EXIT, String::from_utf8(recv[0].clone()).unwrap());
                assert_eq!(pool_id, LittleEndian::read_i32(recv[1].as_slice()));
                PoolService::close_ack(pool_id, Ok(()));
            });

            block_on(ps.close(pool_id)).unwrap();
            pool_mock.join().unwrap();
        }

        #[test]
        fn pool_service_refresh_works() {
            test::cleanup_storage("pool_service_refresh_works");

            let ps = PoolService::new();
            let pool_id = next_pool_handle();
            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("pool_service_refresh_works");
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(Pool::new("", pool_id, PoolOpenConfig::default()), send_cmd_sock));
            let pool_mock = thread::spawn(move || {
                assert_eq!(1, zmq::poll(&mut [recv_cmd_sock.as_poll_item(zmq::POLLIN)], 10_000).unwrap());
                let recv = recv_cmd_sock.recv_multipart(zmq::DONTWAIT).unwrap();
                assert_eq!(recv.len(), 3);
                assert_eq!(COMMAND_REFRESH, String::from_utf8(recv[0].clone()).unwrap());
                let cmd_id = LittleEndian::read_i32(recv[1].as_slice());
                PoolService::refresh_ack(cmd_id, Ok(()));
            });
            block_on(ps.refresh(pool_id)).unwrap();
            pool_mock.join().unwrap();
        }

        #[test]
        fn pool_service_delete_works() {
            test::cleanup_storage("pool_service_delete_works");

            let ps = PoolService::new();
            let pool_name = "pool_service_delete_works";
            let path: path::PathBuf = environment::pool_path(pool_name);
            fs::create_dir_all(path.as_path()).unwrap();
            assert!(path.exists());
            ps.delete(pool_name).unwrap();
            assert!(!path.exists());

            test::cleanup_storage("pool_service_delete_works");
        }

        #[test]
        fn pool_service_delete_works_for_opened() {
            test::cleanup_storage("pool_service_delete_works_for_opened");

            let (send_cmd_sock, _recv_cmd_sock) = pool_create_pair_of_sockets("pool_service_delete_works_for_opened");
            let ps = PoolService::new();
            let pool_name = "pool_service_delete_works_for_opened";
            let path: path::PathBuf = environment::pool_path(pool_name);
            let pool_id = next_pool_handle();

            let pool = Pool::new(pool_name, pool_id, PoolOpenConfig::default());
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(pool, send_cmd_sock));

            fs::create_dir_all(path.as_path()).unwrap();
            assert!(path.exists());
            let res = ps.delete(pool_name);
            assert_eq!(IndyErrorKind::InvalidState, res.unwrap_err().kind());
            assert!(path.exists());

            test::cleanup_storage("pool_service_delete_works_for_opened");
        }

        #[test]
        fn pool_send_tx_works() {
            test::cleanup_storage("pool_send_tx_works");

            let name = "test";
            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("pool_send_tx_works");
            let pool_id = next_pool_handle();
            let pool = Pool::new(name, pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let test_data = "str_instead_of_tx_json";
            let pool_mock = thread::spawn(move || {
                assert_eq!(1, zmq::poll(&mut [recv_cmd_sock.as_poll_item(zmq::POLLIN)], 10_000).unwrap());
                assert_eq!(recv_cmd_sock.recv_string(zmq::DONTWAIT).unwrap().unwrap(), test_data);
                let cmd_id = LittleEndian::read_i32(recv_cmd_sock.recv_bytes(zmq::DONTWAIT).unwrap().as_slice());
                PoolService::submit_ack(cmd_id, Ok("".to_owned()));
            });
            block_on(ps.send_tx(pool_id, test_data)).unwrap();
            pool_mock.join().unwrap();
        }

        #[test]
        fn pool_send_tx_works_for_closed_socket() {
            test::cleanup_storage("pool_send_tx_works_for_closed_socket");

            let name = "test";
            let zmq_ctx = zmq::Context::new();
            let send_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();

            let pool_id = next_pool_handle();
            let pool = Pool::new(name, pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let res = block_on(ps.send_tx(pool_id, "test_data"));
            assert_eq!(IndyErrorKind::IOError, res.unwrap_err().kind());
        }

        #[test]
        fn pool_send_tx_works_for_invalid_handle() {
            test::cleanup_storage("pool_send_tx_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = block_on(ps.send_tx(INVALID_POOL_HANDLE, "txn"));
            assert_eq!(IndyErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_send_action_works() {
            test::cleanup_storage("pool_send_action_works");

            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("pool_send_action_works");
            let pool_id = next_pool_handle();
            let pool = Pool::new("pool_send_action_works", pool_id, PoolOpenConfig::default());
            let ps = PoolService::new();
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            let test_data = "str_instead_of_tx_json";
            let pool_mock = thread::spawn(move || {
                assert_eq!(1, zmq::poll(&mut [recv_cmd_sock.as_poll_item(zmq::POLLIN)], 10_000).unwrap());
                assert_eq!(recv_cmd_sock.recv_string(zmq::DONTWAIT).unwrap().unwrap(), test_data);
                let cmd_id = LittleEndian::read_i32(recv_cmd_sock.recv_bytes(zmq::DONTWAIT).unwrap().as_slice());
                PoolService::submit_ack(cmd_id, Ok("".to_owned()));
            });
            block_on(ps.send_action(pool_id, test_data, None, None)).unwrap();
            pool_mock.join().unwrap();
        }

        #[test]
        fn pool_close_works_for_invalid_handle() {
            test::cleanup_storage("pool_close_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = block_on(ps.close(INVALID_POOL_HANDLE));
            assert_eq!(IndyErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_refresh_works_for_invalid_handle() {
            test::cleanup_storage("pool_refresh_works_for_invalid_handle");
            let ps = PoolService::new();
            let res = block_on(ps.refresh(INVALID_POOL_HANDLE));
            assert_eq!(IndyErrorKind::InvalidPoolHandle, res.unwrap_err().kind());
        }

        #[test]
        fn pool_register_sp_parser_works() {
            test::cleanup_storage("pool_register_sp_parser_works");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            extern fn test_sp(_reply_from_node: *const c_char, _parsed_sp: *mut *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            extern fn test_free(_data: *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
        }

        #[test]
        fn pool_get_sp_parser_works() {
            test::cleanup_storage("pool_get_sp_parser_works");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            extern fn test_sp(_reply_from_node: *const c_char, _parsed_sp: *mut *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            extern fn test_free(_data: *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
            PoolService::get_sp_parser("test").unwrap();
        }

        #[test]
        fn pool_get_sp_parser_works_for_invalid_name() {
            test::cleanup_storage("pool_get_sp_parser_works_for_invalid_name");
            REGISTERED_SP_PARSERS.lock().unwrap().clear();
            assert_eq!(None, PoolService::get_sp_parser("test"));
        }
    }

    #[test]
    fn pool_drop_works_for_after_close() {
        use crate::utils::test;
        use std::time;

        test::cleanup_storage("pool_drop_works_for_after_close");

        fn drop_test() {
            _set_protocol_version(TEST_PROTOCOL_VERSION);
            let ps = PoolService::new();

            let pool_name = "pool_drop_works_for_after_close";
            let gen_txn = test::gen_txns()[0].clone();

            let (send_cmd_sock, recv_cmd_sock) = pool_create_pair_of_sockets("drop_test");

            // create minimal fs config stub before Pool::new()
            let mut pool_path = environment::pool_path(pool_name);
            fs::create_dir_all(&pool_path).unwrap();
            pool_path.push(pool_name);
            pool_path.set_extension("txn");
            let mut file = fs::File::create(pool_path).unwrap();
            file.write(&gen_txn.as_bytes()).unwrap();

            let pool_id = next_pool_handle();
            let mut pool = Pool::new(pool_name, pool_id, PoolOpenConfig::default());
            pool.work(recv_cmd_sock);
            ps.open_pools.borrow_mut().insert(pool_id, ZMQPool::new(pool, send_cmd_sock));
            thread::sleep(time::Duration::from_secs(1));
            block_on(ps.close(pool_id)).unwrap();
            thread::sleep(time::Duration::from_secs(1));
        }

        drop_test();
        test::cleanup_storage("pool_drop_works_for_after_close");
    }

    pub mod nodes_emulator {
        use rust_base58::{ToBase58, FromBase58};
        use indy_utils::crypto::ed25519_sign;

        use super::*;

        use ursa::bls::{Generator, SignKey, VerKey};
        use crate::services::pool::request_handler::DEFAULT_GENERATOR;

        pub static POLL_TIMEOUT: i64 = 1_000; *//* in ms *//*

        pub fn node() -> NodeTransactionV1 {
            let blskey = VerKey::new(&Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap(),
                                     &SignKey::new(None).unwrap()).unwrap().as_bytes().to_base58();

            NodeTransactionV1 {
                txn: Txn {
                    txn_type: "1".to_string(),
                    protocol_version: None,
                    data: TxnData {
                        data: NodeData {
                            alias: "n1".to_string(),
                            client_ip: Some("127.0.0.1".to_string()),
                            client_port: Some(9000),
                            node_ip: Some(String::new()),
                            node_port: Some(0),
                            services: Some(vec!["VALIDATOR".to_string()]),
                            blskey: Some(blskey.to_string()),
                            blskey_pop: None,
                        },
                        dest: "Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv".to_string(),
                        verkey: None,
                    },
                    metadata: TxnMetadata { req_id: None, from: String::new() },
                },
                txn_metadata: Metadata {
                    creation_time: None,
                    seq_no: None,
                    txn_id: None,
                },
                req_signature: ReqSignature { type_: None, values: None },
                ver: String::new(),
            }
        }

        pub fn node_2() -> NodeTransactionV1 {
            let blskey = VerKey::new(&Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap(),
                                     &SignKey::new(None).unwrap()).unwrap().as_bytes().to_base58();

            NodeTransactionV1 {
                txn: Txn {
                    txn_type: "1".to_string(),
                    protocol_version: None,
                    data: TxnData {
                        data: NodeData {
                            alias: "n2".to_string(),
                            client_ip: Some("127.0.0.1".to_string()),
                            client_port: Some(9001),
                            node_ip: Some(String::new()),
                            node_port: Some(0),
                            services: Some(vec!["VALIDATOR".to_string()]),
                            blskey: Some(blskey.to_string()),
                            blskey_pop: None,
                        },
                        dest: "Gw6pDLhcBcoQesN72qfotTgFa7cbuqZpkX3Xo6pLhPhv".to_string(),
                        verkey: None,
                    },
                    metadata: TxnMetadata { req_id: None, from: String::new() },
                },
                txn_metadata: Metadata {
                    creation_time: None,
                    seq_no: None,
                    txn_id: None,
                },
                req_signature: ReqSignature { type_: None, values: None },
                ver: String::new(),
            }
        }

        pub fn start(gt: &mut NodeTransactionV1) -> zmq::Socket {
            let (vk, sk) = ed25519_sign::create_key_pair_for_signature(None).unwrap();
            let pkc = ed25519_sign::vk_to_curve25519(&vk).expect("Invalid pkc");
            let skc = ed25519_sign::sk_to_curve25519(&sk).expect("Invalid skc");
            let ctx = zmq::Context::new();
            let s: zmq::Socket = ctx.socket(zmq::SocketType::ROUTER).unwrap();

            gt.txn.data.dest = (&vk[..]).to_base58();

            s.set_curve_publickey(&zmq::z85_encode(&pkc[..]).unwrap().as_bytes()).expect("set public key");
            s.set_curve_secretkey(&zmq::z85_encode(&skc[..]).unwrap().as_bytes()).expect("set secret key");
            s.set_curve_server(true).expect("set curve server");

            s.bind("tcp://127.0.0.1:*").expect("bind");

            let parts = s.get_last_endpoint().unwrap().unwrap();
            let parts = parts.rsplit(":").collect::<Vec<&str>>();

            gt.txn.data.data.client_port = Some(parts[0].parse::<u64>().unwrap());

            s
        }

        pub fn next(s: &zmq::Socket) -> Option<String> {
            let poll_res = s.poll(zmq::POLLIN, POLL_TIMEOUT).expect("poll");
            if poll_res == 1 {
                let v = s.recv_multipart(zmq::DONTWAIT).expect("recv mulp");
                trace!("Node emulator poll recv {:?}", v);
                s.send_multipart(&[v[0].as_slice(), "po".as_bytes()], zmq::DONTWAIT).expect("send mulp");
                Some(String::from_utf8(v[1].clone()).unwrap())
            } else {
                warn!("Node emulator poll return {}", poll_res);
                None
            }
        }
    }
}*/