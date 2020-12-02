use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::u64;

use rmp_serde;
use serde_json;
use serde_json::Value as SJsonValue;
use self::super::THRESHOLD;

use indy_api_types::errors::prelude::*;
use crate::services::ledger::merkletree::merkletree::MerkleTree;
use crate::services::pool::catchup::{build_catchup_req, CatchupProgress, check_cons_proofs, check_nodes_responses_on_status};
use crate::services::pool::events::NetworkerEvent;
use crate::services::pool::events::PoolEvent;
use crate::services::pool::events::RequestEvent;
use crate::services::pool::{get_last_signed_time, Nodes, PoolService};
use crate::services::pool::merkle_tree_factory;
use crate::services::pool::networker::Networker;
use crate::services::pool::state_proof;
use crate::services::pool::types::CatchupRep;
use crate::services::pool::types::HashableValue;

use super::ursa::bls::Generator;

use std::hash::{Hash, Hasher};
use log_derive::logfn;
use indy_api_types::CommandHandle;
use rust_base58::FromBase58;

struct RequestSM<T: Networker> {
    f: usize,
    cmd_ids: Vec<CommandHandle>,
    nodes: Nodes,
    generator: Generator,
    pool_name: String,
    timeout: i64,
    extended_timeout: i64,
    number_read_nodes: u8,
    state: RequestState<T>,
}

/// Transitions of request state
/// Start -> Start, Single, Consensus, CatchupSingle, CatchupConsensus, Full, Finish
/// Single -> Single, Finish
/// Consensus -> Consensus, Finish
/// CatchupSingle -> CatchupSingle, Finish
/// CatchupConsensus -> CatchupConsensus, Finish
/// Full -> Full, Finish
/// Finish -> Finish
enum RequestState<T: Networker> {
    Start(StartState<T>),
    Single(SingleState<T>),
    Consensus(ConsensusState<T>),
    CatchupSingle(CatchupSingleState<T>),
    CatchupConsensus(CatchupConsensusState<T>),
    Full(FullState<T>),
    Finish(FinishState),
}

/*
 The Generator is used for multi-signature verification.
 It must be the same as on the Ledger side otherwise signatures verification will fail.
*/
pub const DEFAULT_GENERATOR: &str = "3LHpUjiyFC2q2hD7MnwwNmVXiuaFbQx2XkAFJWzswCjgN1utjsCeLzHsKk1nJvFEaS4fcrUmVAkdhtPCYbrVyATZcmzwJReTcJqwqBCPTmTQ9uWPwz6rEncKb2pYYYFcdHa8N17HzVyTqKfgPi4X9pMetfT3A5xCHq54R2pDNYWVLDX";

impl<T: Networker> RequestSM<T> {
    pub fn new(networker: Rc<RefCell<T>>,
               f: usize,
               cmd_ids: &[CommandHandle],
               nodes: &Nodes,
               pool_name: &str, timeout: i64, extended_timeout: i64, number_read_nodes: u8) -> Self {
        let generator: Generator = Generator::from_bytes(&DEFAULT_GENERATOR.from_base58().unwrap()).unwrap();
        RequestSM {
            f,
            cmd_ids: cmd_ids.to_owned(),
            nodes: nodes.clone(),
            generator,
            pool_name: pool_name.to_string(),
            timeout,
            extended_timeout,
            number_read_nodes,
            state: RequestState::Start(StartState {
                networker
            }),
        }
    }

    pub fn step(f: usize,
                cmd_ids: Vec<CommandHandle>,
                nodes: Nodes,
                generator: Generator,
                pool_name: String,
                timeout: i64,
                extended_timeout: i64,
                number_read_nodes: u8,
                state: RequestState<T>) -> Self {
        RequestSM {
            f,
            cmd_ids,
            nodes,
            generator,
            pool_name,
            timeout,
            extended_timeout,
            number_read_nodes,
            state,
        }
    }
}

struct StartState<T: Networker> {
    networker: Rc<RefCell<T>>
}

struct ConsensusState<T: Networker> {
    denied_nodes: HashSet<String> /* FIXME should be map, may be merged with replies */,
    replies: HashMap<HashableValue, HashSet<String>>,
    timeout_nodes: HashSet<String>,
    networker: Rc<RefCell<T>>,
}

struct CatchupConsensusState<T: Networker> {
    replies: HashMap<(String, usize, Option<Vec<String>>), HashSet<String>>,
    networker: Rc<RefCell<T>>,
    merkle_tree: MerkleTree,
}

struct CatchupSingleState<T: Networker> {
    target_mt_root: Vec<u8>,
    target_mt_size: usize,
    merkle_tree: MerkleTree,
    networker: Rc<RefCell<T>>,
    req_id: String,
}

struct SingleState<T: Networker> {
    denied_nodes: HashSet<String> /* FIXME should be map, may be merged with replies */,
    replies: HashMap<HashableValue, HashSet<NodeResponse>>,
    timeout_nodes: HashSet<String>,
    networker: Rc<RefCell<T>>,
    sp_key: Option<Vec<u8>>,
    timestamps: (Option<u64>, Option<u64>),
}

struct FullState<T: Networker> {
    accum_reply: Option<HashableValue>,
    nodes_to_send: Option<Vec<String>>,
    networker: Rc<RefCell<T>>,
}

struct FinishState {}

impl<T: Networker> From<(StartState<T>, Option<Vec<u8>>, (Option<u64>, Option<u64>))> for SingleState<T> {
    fn from((state, sp_key, timestamps): (StartState<T>, Option<Vec<u8>>, (Option<u64>, Option<u64>))) -> Self {
        SingleState {
            denied_nodes: HashSet::new(),
            replies: HashMap::new(),
            timeout_nodes: HashSet::new(),
            networker: state.networker.clone(),
            sp_key,
            timestamps,
        }
    }
}

impl<T: Networker> From<StartState<T>> for ConsensusState<T> {
    fn from(state: StartState<T>) -> Self {
        ConsensusState {
            denied_nodes: HashSet::new(),
            replies: HashMap::new(),
            timeout_nodes: HashSet::new(),
            networker: state.networker.clone(),
        }
    }
}

impl<T: Networker> From<(MerkleTree, StartState<T>)> for CatchupConsensusState<T> {
    fn from((merkle_tree, state): (MerkleTree, StartState<T>)) -> Self {
        CatchupConsensusState {
            replies: HashMap::new(),
            networker: state.networker.clone(),
            merkle_tree,
        }
    }
}

impl<T: Networker> From<(MerkleTree, StartState<T>, Vec<u8>, usize, String)> for CatchupSingleState<T> {
    fn from((merkle_tree, state, target_mt_root, target_mt_size, req_id): (MerkleTree, StartState<T>, Vec<u8>, usize, String)) -> Self {
        CatchupSingleState {
            target_mt_root,
            target_mt_size,
            networker: state.networker.clone(),
            merkle_tree,
            req_id,
        }
    }
}

impl<T: Networker> From<StartState<T>> for FullState<T> {
    fn from(state: StartState<T>) -> Self {
        FullState {
            accum_reply: None,
            nodes_to_send: None,
            networker: state.networker.clone(),
        }
    }
}

impl<T: Networker> From<(Option<Vec<String>>, StartState<T>)> for FullState<T> {
    fn from((nodes_to_send, state): (Option<Vec<String>>, StartState<T>)) -> Self {
        FullState {
            accum_reply: None,
            nodes_to_send,
            networker: state.networker.clone(),
        }
    }
}

impl<T: Networker> RequestState<T> {
    fn finish() -> RequestState<T> {
        RequestState::Finish(FinishState {})
    }
}

struct NodeResponse {
    raw_msg: String,
    node_alias: String,
    timestamp: u64
}

impl PartialEq for NodeResponse {
    fn eq(&self, other: &NodeResponse) -> bool {
        self.node_alias == other.node_alias
    }
}

impl Eq for NodeResponse {}

impl Hash for NodeResponse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_alias.hash(state);
    }
}

impl<T: Networker> RequestSM<T> {
    fn handle_event(self, re: RequestEvent) -> (Self, Option<PoolEvent>) {
        let RequestSM { state, f, cmd_ids, nodes, generator, pool_name, timeout, extended_timeout, number_read_nodes } = self;
        let (state, event) = match state {
            RequestState::Start(state) => {
                match re {
                    RequestEvent::LedgerStatus(ls, _, Some(merkle)) => {
                        let req_id = ls.merkleRoot.clone();
                        let ne = Some(NetworkerEvent::SendAllRequest(serde_json::to_string(&super::types::Message::LedgerStatus(ls)).expect("FIXME"),
                                                                     req_id, extended_timeout, None));
                        trace!("start catchup, ne: {:?}", ne);
                        state.networker.borrow_mut().process_event(ne);
                        (RequestState::CatchupConsensus((merkle, state).into()), None)
                    }
                    RequestEvent::CatchupReq(merkle, target_mt_size, target_mt_root) => {
                        match build_catchup_req(&merkle, target_mt_size) {
                            Ok(Some((req_id, req_json))) => {
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendOneRequest(req_json, req_id.clone(), timeout)));
                                (RequestState::CatchupSingle((merkle, state, target_mt_root, target_mt_size, req_id).into()), None)
                            }
                            Ok(None) => {
                                warn!("No transactions to catch up!");
                                (RequestState::finish(), Some(PoolEvent::Synced(merkle)))
                            }
                            Err(e) => {
                                _send_replies(&cmd_ids, Err(e));
                                (RequestState::finish(), None)
                            }
                        }
                    }
                    RequestEvent::CustomSingleRequest(msg, req_id, sp_key, timestamps) => {
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendOneRequest(msg.clone(), req_id.clone(), timeout)));

                        for _ in 0..number_read_nodes - 1 {
                            state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone(), timeout)));
                        }

                        (RequestState::Single((state, sp_key, timestamps).into()), None)
                    }
                    RequestEvent::CustomFullRequest(msg, req_id, local_timeout, nodes_to_send) => {
                        let timeout = local_timeout.map(|to| to as i64).unwrap_or(extended_timeout);
                        if let Some(nodes_to_send) = nodes_to_send {
                            match serde_json::from_str::<Vec<String>>(&nodes_to_send) {
                                Ok(nodes_to_send) => {
                                    //TODO check empty list on API level?
                                    let is_nodes_to_send_known = !nodes_to_send.is_empty() && nodes_to_send.iter().all(|node| nodes.contains_key(node));
                                    if is_nodes_to_send_known {
                                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendAllRequest(msg, req_id, timeout, Some(nodes_to_send.clone()))));
                                        (RequestState::Full((Some(nodes_to_send), state).into()), None)
                                    } else {
                                        _send_replies(&cmd_ids, Err(err_msg(IndyErrorKind::InvalidStructure,
                                                                            format!("There is no known node in list to send {:?}, known nodes are {:?}",
                                                                                    nodes_to_send, nodes.keys()))));
                                        (RequestState::finish(), None)
                                    }
                                }
                                Err(err) => {
                                    _send_replies(&cmd_ids, Err(err.to_indy(IndyErrorKind::InvalidStructure, "Invalid list of nodes to send")));
                                    (RequestState::finish(), None)
                                }
                            }
                        } else {
                            state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendAllRequest(msg, req_id, timeout, None)));
                            (RequestState::Full((None, state).into()), None)
                        }
                    }
                    RequestEvent::CustomConsensusRequest(msg, req_id) => {
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::SendAllRequest(msg, req_id, timeout, None)));
                        (RequestState::Consensus(state.into()), None)
                    }
                    _ => {
                        (RequestState::Start(state), None)
                    }
                }
            }
            RequestState::Consensus(mut state) => {
                match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) |
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) |
                    RequestEvent::Reject(_, raw_msg, node_alias, req_id)
                    => {
                        if let Ok((_, result_without_proof)) = _get_msg_result_without_state_proof(&raw_msg) {
                            let hashable = HashableValue { inner: result_without_proof };

                            let cnt = {
                                let set = state.replies.entry(hashable).or_insert_with(HashSet::new);
                                set.insert(node_alias.clone());
                                set.len()
                            };

                            if cnt > f {
                                _send_ok_replies(&cmd_ids, &raw_msg);
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                (RequestState::finish(), None)
                            } else if state.is_consensus_reachable(f, nodes.len()) {
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                                (RequestState::Consensus(state), None)
                            } else {
                                //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
                                _send_replies(&cmd_ids, Err(err_msg(IndyErrorKind::PoolTimeout, "Consensus is impossible")));
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                (RequestState::finish(), None)
                            }
                        } else {
                            state.denied_nodes.insert(node_alias.clone());
                            if state.denied_nodes.len() + state.replies.len() == nodes.len() {
                                _send_replies(&cmd_ids, Err(err_msg(IndyErrorKind::PoolTimeout, "Consensus is impossible")));
                                (RequestState::finish(), None)
                            } else {
                                (RequestState::Consensus(state), None)
                            }
                        }
                    }
                    RequestEvent::ReqACK(_, _, node_alias, req_id) => {
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::ExtendTimeout(req_id, node_alias, extended_timeout)));
                        (RequestState::Consensus(state), None)
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        state.timeout_nodes.insert(node_alias.clone());
                        if state.is_consensus_reachable(f, nodes.len()) {
                            state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                            (RequestState::Consensus(state), None)
                        } else {
                            //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
                            _send_replies(&cmd_ids, Err(err_msg(IndyErrorKind::PoolTimeout, "Consensus is impossible")));
                            state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                            (RequestState::finish(), None)
                        }
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&cmd_ids);
                        (RequestState::finish(), None)
                    }
                    _ => (RequestState::Consensus(state), None)
                }
            }
            RequestState::Single(mut state) => {
                match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) |
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) |
                    RequestEvent::Reject(_, raw_msg, node_alias, req_id) => {
                        trace!("reply on single request");
                        state.timeout_nodes.remove(&node_alias);
                        if let Ok((result, result_without_proof)) = _get_msg_result_without_state_proof(&raw_msg) {
                            let hashable = HashableValue { inner: result_without_proof };

                            let last_write_time = get_last_signed_time(&raw_msg).unwrap_or(0);

                            let (cnt, soonest) = {
                                let set = state.replies.entry(hashable).or_insert_with(HashSet::new);
                                set.insert(NodeResponse { node_alias: node_alias.clone(), timestamp: last_write_time, raw_msg: raw_msg.clone() });
                                (
                                    set.len(),
                                    set.iter().max_by_key(|resp| resp.timestamp).map(|resp| &resp.raw_msg).unwrap_or(&raw_msg).clone()
                                )
                            };

                            if cnt > f
                                || _check_state_proof(&result, f, &generator, &nodes, &raw_msg, state.sp_key.as_ref().map(Vec::as_slice), state.timestamps, last_write_time) {
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                                _send_ok_replies(&cmd_ids, if cnt > f { &soonest } else { &raw_msg });
                                (RequestState::finish(), None)
                            } else {
                                (state.try_to_continue(req_id, node_alias, &cmd_ids, nodes.len(), timeout), None)
                            }
                        } else {
                            state.denied_nodes.insert(node_alias.clone());
                            (state.try_to_continue(req_id, node_alias, &cmd_ids, nodes.len(), timeout), None)
                        }
                    }
                    RequestEvent::ReqACK(_, _, node_alias, req_id) => {
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::ExtendTimeout(req_id, node_alias, extended_timeout)));
                        (RequestState::Single(state), None)
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        state.timeout_nodes.insert(node_alias.clone());
                        (state.try_to_continue(req_id, node_alias, &cmd_ids, nodes.len(), timeout), None)
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&cmd_ids);
                        (RequestState::finish(), None)
                    }
                    _ => (RequestState::Single(state), None)
                }
            }
            RequestState::CatchupConsensus(state) => {
                match re {
                    RequestEvent::LedgerStatus(ls, Some(node_alias), _) => {
                        RequestSM::_catchup_target_handle_consensus_state(
                            state,
                            ls.merkleRoot.clone(), ls.txnSeqNo, None,
                            node_alias, ls.merkleRoot, f, &nodes, &pool_name)
                    }
                    RequestEvent::ConsistencyProof(cp, node_alias) => {
                        RequestSM::_catchup_target_handle_consensus_state(
                            state,
                            cp.newMerkleRoot, cp.seqNoEnd, Some(cp.hashes),
                            node_alias, cp.oldMerkleRoot, f, &nodes, &pool_name)
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        RequestSM::_catchup_target_handle_consensus_state(
                            state,
                            "timeout".to_string(), 0, None,
                            node_alias, req_id, f, &nodes, &pool_name)
                    }

                    RequestEvent::Terminate => {
                        _finish_request(&cmd_ids);
                        (RequestState::finish(), None)
                    }
                    _ => (RequestState::CatchupConsensus(state), None)
                }
            }
            RequestState::CatchupSingle(state) => {
                match re {
                    RequestEvent::CatchupRep(mut cr, node_alias) => {
                        match _process_catchup_reply(&mut cr, &state.merkle_tree, &state.target_mt_root, state.target_mt_size, &pool_name) {
                            Ok(merkle) => {
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(state.req_id.clone(), None)));
                                (RequestState::finish(), Some(PoolEvent::Synced(merkle)))
                            }
                            Err(_) => {
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(state.req_id.clone(), timeout)));
                                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(state.req_id.clone(), Some(node_alias))));
                                (RequestState::CatchupSingle(state), None)
                            }
                        }
                    }
                    RequestEvent::Timeout(req_id, node_alias) => {
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(state.req_id.clone(), timeout)));
                        state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                        (RequestState::CatchupSingle(state), None)
                    }
                    RequestEvent::Terminate => {
                        _finish_request(&cmd_ids);
                        (RequestState::finish(), None)
                    }
                    _ => (RequestState::CatchupSingle(state), None)
                }
            }
            RequestState::Full(state) => {
                match re {
                    RequestEvent::Reply(_, raw_msg, node_alias, req_id) |
                    RequestEvent::ReqNACK(_, raw_msg, node_alias, req_id) |
                    RequestEvent::Reject(_, raw_msg, node_alias, req_id) =>
                        (RequestSM::_full_request_handle_consensus_state(
                            state, req_id, node_alias, raw_msg, &cmd_ids, &nodes), None),
                    RequestEvent::Timeout(req_id, node_alias) =>
                        (RequestSM::_full_request_handle_consensus_state(
                            state, req_id, node_alias, "timeout".to_string(), &cmd_ids, &nodes), None),

                    RequestEvent::Terminate => {
                        _finish_request(&cmd_ids);
                        (RequestState::finish(), None)
                    }
                    _ => (RequestState::Full(state), None),
                }
            }
            RequestState::Finish(state) => (RequestState::Finish(state), None)
        };
        (RequestSM::step(f, cmd_ids, nodes, generator, pool_name, timeout, extended_timeout, number_read_nodes, state), event)
    }

    fn is_terminal(&self) -> bool {
        match self.state {
            RequestState::Start(_) |
            RequestState::Consensus(_) |
            RequestState::Single(_) |
            RequestState::CatchupSingle(_) |
            RequestState::CatchupConsensus(_) |
            RequestState::Full(_) => false,
            RequestState::Finish(_) => true
        }
    }

    fn _full_request_handle_consensus_state(mut state: FullState<T>,
                                            req_id: String, node_alias: String, node_result: String,
                                            cmd_ids: &[CommandHandle],
                                            nodes: &Nodes) -> RequestState<T> {
        let is_first_resp = state.accum_reply.is_none();
        if is_first_resp {
            state.accum_reply = Some(HashableValue {
                inner: json!({node_alias.clone(): node_result})
            })
        } else {
            state.accum_reply.as_mut().unwrap()
                .inner.as_object_mut().unwrap()
                .insert(node_alias.clone(), SJsonValue::from(node_result));
        }

        let required_reply_cnt = state.nodes_to_send.as_ref().map(Vec::len).unwrap_or_else(|| nodes.len());

        let reply_cnt = state.accum_reply.as_ref().unwrap()
            .inner.as_object().unwrap().len();

        if reply_cnt == required_reply_cnt {
            state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
            let reply = state.accum_reply.as_ref().unwrap().inner.to_string();
            _send_ok_replies(&cmd_ids, &reply);
            RequestState::Finish(FinishState {})
        } else {
            state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
            RequestState::Full(state)
        }
    }

    fn _catchup_target_handle_consensus_state(mut state: CatchupConsensusState<T>,
                                              mt_root: String, sz: usize, cons_proof: Option<Vec<String>>,
                                              node_alias: String, req_id: String,
                                              f: usize, nodes: &Nodes,
                                              pool_name: &str) -> (RequestState<T>, Option<PoolEvent>) {
        let (finished, result) = RequestSM::_process_catchup_target(mt_root, sz, cons_proof,
                                                                    &node_alias, &mut state, f, nodes, pool_name);

        match (finished, result) {
            (true, result) => {
                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                (RequestState::finish(), result)
            }
            (false, Some(PoolEvent::CatchupRestart(merkle_tree))) => {
                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
                (RequestState::CatchupConsensus(state), Some(PoolEvent::CatchupRestart(merkle_tree)))
            }
            (false, result) => {
                state.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
                (RequestState::CatchupConsensus(state), result)
            }
        }
    }

    fn _process_catchup_target(merkle_root: String,
                               txn_seq_no: usize,
                               hashes: Option<Vec<String>>,
                               node_alias: &str,
                               state: &mut CatchupConsensusState<T>,
                               f: usize,
                               nodes: &Nodes,
                               pool_name: &str) -> (bool, Option<PoolEvent>) {
        let key = (merkle_root, txn_seq_no, hashes);
        let contains = state.replies.get_mut(&key)
            .map(|set| { set.insert(node_alias.to_string()); })
            .is_some();

        if !contains {
            state.replies.insert(key, HashSet::from_iter(vec![node_alias.to_string()]));
        }

        match check_nodes_responses_on_status(&state.replies,
                                              &state.merkle_tree,
                                              nodes.len(),
                                              f,
                                              &pool_name) {
            Ok(CatchupProgress::InProgress) => (false, None),
            Ok(CatchupProgress::NotNeeded(merkle_tree)) => (true, Some(PoolEvent::Synced(merkle_tree))),
            Ok(CatchupProgress::Restart(merkle_tree)) => (false, Some(PoolEvent::CatchupRestart(merkle_tree))),
            Ok(CatchupProgress::ShouldBeStarted(target_mt_root, target_mt_size, merkle_tree)) =>
                (true, Some(PoolEvent::CatchupTargetFound(target_mt_root, target_mt_size, merkle_tree))),
            Err(err) => (true, Some(PoolEvent::CatchupTargetNotFound(err))),
        }
    }
}

pub trait RequestHandler<T: Networker> {
    fn new(networker: Rc<RefCell<T>>, f: usize, cmd_ids: &[CommandHandle], nodes: &Nodes, pool_name: &str, timeout: i64, extended_timeout: i64, number_read_nodes: u8) -> Self;
    fn process_event(&mut self, ore: Option<RequestEvent>) -> Option<PoolEvent>;
    fn is_terminal(&self) -> bool;
}

pub struct RequestHandlerImpl<T: Networker> {
    request_wrapper: Option<RequestSM<T>>
}

impl<T: Networker> RequestHandler<T> for RequestHandlerImpl<T> {
    fn new(networker: Rc<RefCell<T>>, f: usize, cmd_ids: &[CommandHandle], nodes: &Nodes, pool_name: &str, timeout: i64, extended_timeout: i64, number_read_nodes: u8) -> Self {
        RequestHandlerImpl {
            request_wrapper: Some(RequestSM::new(networker, f, cmd_ids, nodes, pool_name, timeout, extended_timeout, number_read_nodes)),
        }
    }

    fn process_event(&mut self, ore: Option<RequestEvent>) -> Option<PoolEvent> {
        match ore {
            Some(re) => {
                if let Some((rw, res)) = self.request_wrapper.take().map(|w| w.handle_event(re)) {
                    self.request_wrapper = Some(rw);
                    res
                } else {
                    self.request_wrapper = None;
                    None
                }
            }
            None => None
        }
    }

    fn is_terminal(&self) -> bool {
        self.request_wrapper.as_ref().map(|w| w.is_terminal()).unwrap_or(true)
    }
}

impl<T: Networker> SingleState<T> {
    fn is_consensus_reachable(&self, total_nodes_cnt: usize) -> bool {
        (self.timeout_nodes.len() + self.denied_nodes.len() + self.replies.values().map(|set| set.len()).sum::<usize>())
            < total_nodes_cnt
    }

    fn try_to_continue(self, req_id: String, node_alias: String, cmd_ids: &[CommandHandle], nodes_cnt: usize, timeout: i64) -> RequestState<T> {
        if self.is_consensus_reachable(nodes_cnt) {
            self.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone(), timeout)));
            self.networker.borrow_mut().process_event(Some(NetworkerEvent::Resend(req_id.clone(), timeout)));
            self.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, Some(node_alias))));
            RequestState::Single(self)
        } else {
            //TODO: maybe we should change the error, but it was made to escape changing of ErrorCode returned to client
            _send_replies(cmd_ids, Err(err_msg(IndyErrorKind::PoolTimeout, "Consensus is impossible")));
            self.networker.borrow_mut().process_event(Some(NetworkerEvent::CleanTimeout(req_id, None)));
            RequestState::finish()
        }
    }
}

impl<T: Networker> ConsensusState<T> {
    fn is_consensus_reachable(&self, f: usize, total_nodes_cnt: usize) -> bool {
        let rep_no: usize = self.replies.values().map(|set| set.len()).sum();
        let max_no = self.replies.values().map(|set| set.len()).max().unwrap_or(0);
        max_no + total_nodes_cnt - rep_no - self.timeout_nodes.len() - self.denied_nodes.len() > f
    }
}

fn _parse_nack(denied_nodes: &mut HashSet<String>, f: usize, raw_msg: &str, cmd_ids: &[CommandHandle], node_alias: &str) -> bool {
    if denied_nodes.len() == f {
        _send_ok_replies(cmd_ids, raw_msg);
        true
    } else {
        denied_nodes.insert(node_alias.to_string());
        false
    }
}

fn _process_catchup_reply(rep: &mut CatchupRep, merkle: &MerkleTree, target_mt_root: &Vec<u8>, target_mt_size: usize, pool_name: &str) -> IndyResult<MerkleTree> {
    let mut txns_to_drop = vec![];
    let mut merkle = merkle.clone();

    while !rep.txns.is_empty() {
        let key = rep.min_tx()?;
        let txn = rep.txns.remove(&key.to_string()).unwrap();

        let txn = rmp_serde::to_vec_named(&txn)
            .to_indy(IndyErrorKind::InvalidStructure, "Invalid transaction -- can not transform to bytes")?;

        merkle.append(txn.clone())?;
        txns_to_drop.push(txn);
    }

    check_cons_proofs(&merkle, &rep.consProof, target_mt_root, target_mt_size)?;
    merkle_tree_factory::dump_new_txns(pool_name, &txns_to_drop)?;
    Ok(merkle)
}

fn _send_ok_replies(cmd_ids: &[CommandHandle], msg: &str) {
    _send_replies(cmd_ids, Ok(msg.to_string()))
}

fn _finish_request(cmd_ids: &[CommandHandle]) {
    _send_replies(cmd_ids, Err(err_msg(IndyErrorKind::PoolTerminated, "Pool is terminated")))
}

fn _send_replies(cmd_ids: &[CommandHandle], msg: IndyResult<String>) {
    cmd_ids.iter().for_each(|id| {
        PoolService::submit_ack(*id, msg.clone());
    });
}

fn _get_msg_result_without_state_proof(msg: &str) -> IndyResult<(SJsonValue, SJsonValue)> {
    let msg = serde_json::from_str::<SJsonValue>(msg)
        .to_indy(IndyErrorKind::InvalidStructure, "Response is malformed json")?;

    let msg_result = msg["result"].clone();

    let mut msg_result_without_proof: SJsonValue = msg_result.clone();
    msg_result_without_proof.as_object_mut().map(|obj| obj.remove("state_proof"));

    if msg_result_without_proof["data"].is_object() {
        msg_result_without_proof["data"].as_object_mut().map(|obj| obj.remove("stateProofFrom"));
    }

    Ok((msg_result, msg_result_without_proof))
}

fn _check_state_proof(msg_result: &SJsonValue, f: usize, gen: &Generator, bls_keys: &Nodes, raw_msg: &str, sp_key: Option<&[u8]>, requested_timestamps: (Option<u64>, Option<u64>), last_write_time: u64) -> bool {
    debug!("TransactionHandler::process_reply: Try to verify proof and signature >>");

    let proof_checking_res = match state_proof::parse_generic_reply_for_proof_checking(&msg_result, raw_msg, sp_key) {
        Some(parsed_sps) => {
            debug!("TransactionHandler::process_reply: Proof and signature are present");
            state_proof::verify_parsed_sp(parsed_sps, bls_keys, f, gen)
        }
        None => false
    };

    let res = proof_checking_res && _check_freshness(msg_result, requested_timestamps, last_write_time);

    debug!("TransactionHandler::process_reply: Try to verify proof and signature << {}", res);
    res
}

fn _check_freshness(msg_result: &SJsonValue, requested_timestamps: (Option<u64>, Option<u64>), last_write_time: u64) -> bool {
    debug!("TransactionHandler::_check_freshness: requested_timestamps: {:?} >>", requested_timestamps);

    let res = match requested_timestamps {
        (Some(from), Some(to)) => {
            let left_last_write_time = _extract_left_last_write_time(msg_result).unwrap_or(0);
            trace!("Last last signed time: {}", left_last_write_time);
            trace!("Last right signed time: {}", last_write_time);

            let left_time_for_freshness_check = from;
            let right_time_for_freshness_check = to;

            trace!("Left time for freshness check: {}", left_time_for_freshness_check);
            trace!("Right time for freshness check: {}", right_time_for_freshness_check);

            left_time_for_freshness_check <= _get_freshness_threshold() + left_last_write_time &&
                right_time_for_freshness_check <= _get_freshness_threshold() + last_write_time
        }
        (None, Some(to)) => {
            let time_for_freshness_check = to;

            trace!("Last signed time: {}", last_write_time);
            trace!("Time for freshness check: {}", time_for_freshness_check);

            time_for_freshness_check <= _get_freshness_threshold() + last_write_time
        }
        (Some(from), None) => {
            let left_last_write_time = _extract_left_last_write_time(msg_result).unwrap_or(0);

            trace!("Last last signed time: {}", left_last_write_time);
            trace!("Last right signed time: {}", last_write_time);

            let left_time_for_freshness_check = from;
            let time_for_freshness_check = _get_cur_time();

            trace!("Left time for freshness check: {}", left_time_for_freshness_check);
            trace!("Time for freshness check: {}", time_for_freshness_check);

            left_time_for_freshness_check <= _get_freshness_threshold() + left_last_write_time &&
                time_for_freshness_check <= _get_freshness_threshold() + last_write_time
        }
        (None, None) => {
            let time_for_freshness_check = _get_cur_time();

            trace!("Last signed time: {}", last_write_time);
            trace!("Time for freshness check: {}", time_for_freshness_check);

            time_for_freshness_check <= _get_freshness_threshold() + last_write_time
        }
    };

    debug!("TransactionHandler::_check_freshness << {:?} ", res);

    res
}

#[logfn(Trace)]
fn _extract_left_last_write_time(msg_result: &SJsonValue) -> Option<u64> {
    match msg_result["type"].as_str() {
        Some(crate::domain::ledger::constants::GET_REVOC_REG_DELTA) => {
            msg_result["data"]["stateProofFrom"]["multi_signature"]["value"]["timestamp"].as_u64()
        }
        _ => {
            None
        }
    }
}

fn _get_freshness_threshold() -> u64 {
    *THRESHOLD.lock().unwrap()
}

fn _get_cur_time() -> u64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has gone backwards");
    let res = since_epoch.as_secs();
    trace!("Current time: {}", res);
    res
}

#[cfg(test)]
pub mod tests {
    use crate::services::ledger::merkletree::tree::Tree;
    use crate::services::pool::networker::MockNetworker;
    use crate::services::pool::types::{ConsistencyProof, LedgerStatus, Reply, ReplyResultV1, ReplyTxnV1, ReplyV1, Response, ResponseMetadata, ResponseV1};
    use crate::utils::test;
    use crate::utils::test::test_pool_create_poolfile;
    use crate::domain::pool::NUMBER_READ_NODES;

    use super::*;
    use std::io::Write;

    const MESSAGE: &str = "message";
    const REQ_ID: &str = "1";
    const NODE: &str = "n1";
    const NODE_2: &str = "n2";
    const NODE_3: &str = "n3";
    const NODE_4: &str = "n4";
    const SIMPLE_REPLY: &str = r#"{"result":{}}"#;
    const REJECT_REPLY: &str = r#"{"op":"REJECT", "result": {"reason": "reject"}}"#;
    const NACK_REPLY: &str = r#"{"op":"REQNACK", "result": {"reason": "reqnack"}}"#;

    #[derive(Debug)]
    pub struct MockRequestHandler {}

    impl<T: Networker> RequestHandler<T> for MockRequestHandler {
        fn new(_networker: Rc<RefCell<T>>, _f: usize, _cmd_ids: &[CommandHandle], _nodes: &Nodes, _pool_name: &str, _timeout: i64, _extended_timeout: i64, _number_read_nodes: u8) -> Self {
            MockRequestHandler {}
        }

        fn process_event(&mut self, _ore: Option<RequestEvent>) -> Option<PoolEvent> {
            None
        }

        fn is_terminal(&self) -> bool {
            true
        }
    }

    impl Default for LedgerStatus {
        fn default() -> Self {
            LedgerStatus {
                txnSeqNo: 0,
                merkleRoot: String::new(),
                ledgerId: 0,
                ppSeqNo: None,
                viewNo: None,
                protocolVersion: None,
            }
        }
    }

    impl Default for Reply {
        fn default() -> Self {
            Reply::ReplyV1(ReplyV1 { result: ReplyResultV1 { txn: ReplyTxnV1 { metadata: ResponseMetadata { req_id: 1 } } } })
        }
    }

    impl Default for Response {
        fn default() -> Self {
            Response::ResponseV1(ResponseV1 { metadata: ResponseMetadata { req_id: 1 } })
        }
    }

    impl Default for ConsistencyProof {
        fn default() -> Self {
            ConsistencyProof {
                seqNoEnd: 0,
                seqNoStart: 0,
                ledgerId: 0,
                hashes: Vec::new(),
                oldMerkleRoot: String::new(),
                newMerkleRoot: String::new(),
            }
        }
    }

    impl Default for CatchupRep {
        fn default() -> Self {
            CatchupRep {
                ledgerId: 0,
                consProof: Vec::new(),
                txns: HashMap::new(),
            }
        }
    }

    fn _request_handler(pool_name: &str, f: usize, nodes_cnt: usize) -> RequestHandlerImpl<MockNetworker> {
        let networker = Rc::new(RefCell::new(MockNetworker::new(0, 0, vec![])));

        let mut default_nodes: Nodes = HashMap::new();
        default_nodes.insert(NODE.to_string(), None);

        let node_names = vec![NODE, NODE_2, "n3", "n4"];
        let mut nodes: Nodes = HashMap::with_capacity(nodes_cnt);

        for i in 0..nodes_cnt {
            nodes.insert(node_names[i].to_string(), None);
        }

        RequestHandlerImpl::new(networker,
                                f,
                                &vec![],
                                &nodes,
                                pool_name,
                                0,
                                0,
                                NUMBER_READ_NODES)
    }

    // required because of dumping txns to cache
    fn _create_pool(pool_name: &str, content: Option<String>) {
        let mut file = test_pool_create_poolfile(pool_name);
        file.write_all(content.unwrap_or("{}".to_string()).as_bytes()).unwrap();
    }

    #[test]
    fn request_handler_new_works() {
        let request_handler = _request_handler("request_handler_new_works", 0, 1);
        assert_match!(RequestState::Start(_), request_handler.request_wrapper.unwrap().state);
    }

    #[test]
    fn request_handler_process_event_works() {
        let mut request_handler = _request_handler("request_handler_process_event_works", 0, 1);
        request_handler.process_event(None);
    }

    mod start {
        use super::*;

        #[test]
        fn request_handler_process_ledger_status_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_ledger_status_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestState::CatchupConsensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_catchup_req_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_catchup_req_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            assert_match!(RequestState::CatchupSingle(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_catchup_req_event_from_start_works_for_no_transactions_to_catchup() {
            let mut request_handler = _request_handler("request_handler_process_catchup_req_event_from_start_works_for_no_transactions_to_catchup", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 0, vec![])));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_custom_single_req_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_custom_single_req_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_consensus_full_req_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, None)));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works_for_list_nodes() {
            let mut request_handler = _request_handler("request_handler_process_consensus_full_req_event_from_start_works_for_list_nodes", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, Some(format!(r#"["{}"]"#, NODE)))));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works_for_empty_list_nodes() {
            let mut request_handler = _request_handler("request_handler_process_consensus_full_req_event_from_start_works_for_empty_list_nodes", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, Some("[ ]".to_string()))));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works_for_list_nodes_contains_unknown_node() {
            let mut request_handler = _request_handler("request_handler_process_consensus_full_req_event_from_start_works_for_list_nodes_contains_unknown_node", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, Some("[Unknown Node]".to_string()))));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consensus_full_req_event_from_start_works_for_invalid_list_nodes_format() {
            let mut request_handler = _request_handler("request_handler_process_consensus_full_req_event_from_start_works_for_invalid_list_nodes_format", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, Some(format!(r#""{}""#, NODE)))));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_custom_consensus_req_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_custom_consensus_req_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_start_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_start_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Start(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod consensus_state {
        use super::*;

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached_with_mixed_msgs() {
            // the test will use 4 nodes, each node replying with a response to the "custom consensus request" message
            // some nodes accept, some reject and some nack.  the end result is consensus should not be reached
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached_with_mixed_msgs", 1, 4);

            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), REJECT_REPLY.to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), NACK_REPLY.to_string(), NODE_3.to_string(), REQ_ID.to_string())));

            // test state is already Finished because already have 2 nodes not in consensus
            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Consensus(_), request_handler_ref.state);
            }

            // send one more message to ensure state isn't affected
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), REJECT_REPLY.to_string(), NODE_4.to_string(), REQ_ID.to_string())));

            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached_with_0_concensus() {
            // the test will use 4 nodes, each node replying with a response to the "custom consensus request" message
            // some nodes accept, some reject and some nack.  the end result is consensus should not be reached
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_consensus_reached_with_0_concensus", 1, 4);

            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "".to_string(), NODE_3.to_string(), REQ_ID.to_string())));

            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Consensus(_), request_handler_ref.state);
            }

            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE_4.to_string(), REQ_ID.to_string())));

            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_consensus_reachable", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_consensus_not_reachable", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{}"#.to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_consensus_state_works_for_invalid_message() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_consensus_state_works_for_invalid_message", 1, 4);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqack_event_from_consensus_state_works() {
            let mut request_handler = _request_handler("request_handler_process_reqack_event_from_consensus_state_works", 1, 4);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reached", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_reachable", 1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_consensus_state_works_for_consensus_not_reachable", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_consensus_state_works_for_consensus_reached", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_consensus_state_works_for_consensus_reachable", 1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_consensus_state_works_for_consensus_not_reachable", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":{}}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_consensus_state_works_for_consensus_reachable() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_consensus_state_works_for_consensus_reachable", 1, 3);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Timeout(NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_consensus_state_works_for_consensus_not_reachable", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Timeout(NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_terminate_event_from_consensus_state_works_for_consensus_not_reachable() {
            let mut request_handler = _request_handler("request_handler_process_terminate_event_from_consensus_state_works_for_consensus_not_reachable", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_consensus_state_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_consensus_state_works", 1, 4);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Ping));
            assert_match!(RequestState::Consensus(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod single {
        use super::*;
        use crate::services::pool::set_freshness_threshold;

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_consensus_reached() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_consensus_reached", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        fn correct_state_proof_reply(timestamp: u64) -> String {
            json!({
                "result": {
                    "type": "test",
                    "ver": "1",
                    "multiSignature":{
                        "signedState": {
                            "stateMetadata": {
                                "timestamp": timestamp
                            }
                        }
                    }
                },
                "op": "REPLY",
            }).to_string()
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_state_proof() {
            set_freshness_threshold(600);
            add_state_proof_parser();
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_state_proof", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(
                RequestEvent::Reply(Reply::default(), correct_state_proof_reply(_get_cur_time() - 300), NODE.to_string(), REQ_ID.to_string()))
            );
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_state_proof_from_future() {
            set_freshness_threshold(600);
            add_state_proof_parser();
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_state_proof_from_future", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(
                Some(RequestEvent::Reply(Reply::default(), correct_state_proof_reply(_get_cur_time() + 300), NODE.to_string(), REQ_ID.to_string()))
            );
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        fn add_state_proof_parser() {
            use crate::services::pool::REGISTERED_SP_PARSERS;
            use indy_api_types::ErrorCode;
            use libc::c_char;
            use std::ffi::CString;

            REGISTERED_SP_PARSERS.lock().unwrap().clear();

            extern fn test_sp(_reply_from_node: *const c_char, parsed_sp: *mut *const c_char) -> ErrorCode {
                let sp: CString = CString::new("[]").unwrap();
                unsafe { *parsed_sp = sp.into_raw(); }
                ErrorCode::Success
            }
            extern fn test_free(_data: *const c_char) -> ErrorCode {
                ErrorCode::Success
            }
            PoolService::register_sp_parser("test", test_sp, test_free).unwrap();
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_freshness_filtering() {
            set_freshness_threshold(600);
            add_state_proof_parser();
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_freshness_filtering", 2, 4);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            //
            request_handler.process_event(Some(RequestEvent::Reply(
                Reply::default(),
                correct_state_proof_reply(_get_cur_time() - 700),
                NODE.to_string(),
                REQ_ID.to_string())));

            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Single(_), request_handler_ref.state);
            }

            request_handler.process_event(Some(RequestEvent::Reply(
                Reply::default(),
                correct_state_proof_reply(_get_cur_time() - 300),
                NODE.to_string(),
                REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_state_proof_from_past() {
            set_freshness_threshold(300);
            add_state_proof_parser();

            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_state_proof_from_past", 2, 4);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, Some(_get_cur_time() - 400)))));

            {
                request_handler.process_event(
                    Some(RequestEvent::Reply(Reply::default(), correct_state_proof_reply(_get_cur_time() - 800), NODE.to_string(), REQ_ID.to_string()))
                );

                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Single(_), request_handler_ref.state);
            }

            {
                request_handler.process_event(
                    Some(RequestEvent::Reply(Reply::default(), correct_state_proof_reply(_get_cur_time() - 400), NODE.to_string(), REQ_ID.to_string()))
                );
                assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
            }
        }


        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_freshness_filtering_from_env_variable() {
            set_freshness_threshold(300);
            // Register custom state proof parser
            add_state_proof_parser();

            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_freshness_filtering_from_env_variable", 2, 4);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            //
            request_handler.process_event(Some(RequestEvent::Reply(
                Reply::default(),
                correct_state_proof_reply(_get_cur_time() - 400),
                NODE.to_string(),
                REQ_ID.to_string())));

            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Single(_), request_handler_ref.state);
            }

            request_handler.process_event(Some(RequestEvent::Reply(
                Reply::default(),
                correct_state_proof_reply(_get_cur_time() - 200),
                NODE.to_string(),
                REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
            set_freshness_threshold(600);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_not_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_cannot_be_completed() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_cannot_be_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_invalid_message() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_invalid_message", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), "".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqack_event_from_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_reqack_event_from_single_state_works", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_single_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_single_state_works_for_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_single_state_works_for_not_completed", 1, 3);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_single_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_single_state_works_for_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_single_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_single_state_works_for_not_completed", 1, 3);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "{}".to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_single_state_works", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_single_state_works_for_cannot_be_completed() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_single_state_works_for_cannot_be_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_terminate_event_from_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_terminate_event_from_single_state_works", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_single_state_works", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestState::Single(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_consensus_reached_with_mixed_msgs() {
            // the test will use 4 nodes, each node replying with a response to the "custom consensus request" message
            // some nodes accept, some reject and some nack.  the end result is consensus should not be reached
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_consensus_reached_with_mixed_msgs", 1, 4);

            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), REJECT_REPLY.to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), NACK_REPLY.to_string(), NODE_3.to_string(), REQ_ID.to_string())));

            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Single(_), request_handler_ref.state);
            }

            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), REJECT_REPLY.to_string(), NODE_4.to_string(), REQ_ID.to_string())));

            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        // this test is marked ignore until https://jira.hyperledger.org/browse/IS-1137 is resolved
        #[test]
        fn request_handler_process_reply_event_from_single_state_works_for_consensus_reached_with_0_concensus() {
            // the test will use 4 nodes, each node replying with a response to the "custom consensus request" message
            // some nodes accept, some reject and some nack.  the end result is consensus should not be reached
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_single_state_works_for_consensus_reached_with_0_concensus", 1, 4);

            request_handler.process_event(Some(RequestEvent::CustomSingleRequest(MESSAGE.to_string(), REQ_ID.to_string(), None, (None, None))));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), SIMPLE_REPLY.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "".to_string(), NODE_2.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), "".to_string(), NODE_3.to_string(), REQ_ID.to_string())));

            // test state should be still Single because request handler has 3 different answers
            {
                let request_handler_ref = request_handler.request_wrapper.as_ref().unwrap();
                assert_match!(RequestState::Single(_), request_handler_ref.state);
            }

            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), "".to_string(), NODE_4.to_string(), REQ_ID.to_string())));

            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod catchup_consensus {
        use super::*;

        #[test]
        fn request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_completed() {
            let mut request_handler = _request_handler("request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_completed", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_catchup_not_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestState::CatchupConsensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_completed() {
            let mut request_handler = _request_handler("request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_completed", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::ConsistencyProof(ConsistencyProof::default(), NODE.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_consistency_proof_event_from_catchup_consensus_state_works_for_catchup_not_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::ConsistencyProof(ConsistencyProof::default(), NODE.to_string())));
            assert_match!(RequestState::CatchupConsensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_catchup_consensus_state_works", 1, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::CatchupConsensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_consensus_state_works_for_all_timeouts() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_catchup_consensus_state_works_for_all_timeouts", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_terminate_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler("request_handler_process_terminate_event_from_catchup_consensus_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_catchup_consensus_state_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_catchup_consensus_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestState::CatchupConsensus(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_splitted_pool() {
            let mut request_handler = _request_handler("request_handler_process_ledger_status_event_from_catchup_consensus_state_works_for_splitted_pool", 1, 4);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some(NODE.to_string()), Some(MerkleTree::default()))));
            assert_match!(&RequestState::CatchupConsensus(_), &request_handler.request_wrapper.as_ref().unwrap().state);

            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), "n1".to_string())));
            assert_match!(&RequestState::CatchupConsensus(_), &request_handler.request_wrapper.as_ref().unwrap().state);
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), "n2".to_string())));
            assert_match!(&RequestState::CatchupConsensus(_), &request_handler.request_wrapper.as_ref().unwrap().state);

            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some("n3".to_string()), Some(MerkleTree::default()))));
            assert_match!(&RequestState::Finish(_), &request_handler.request_wrapper.as_ref().unwrap().state);
            request_handler.process_event(Some(RequestEvent::LedgerStatus(LedgerStatus::default(), Some("n4".to_string()), Some(MerkleTree::default()))));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod catchup_single {
        use super::*;

        #[test]
        fn request_handler_process_catchup_reply_event_from_catchup_single_state_works() {
            test::cleanup_pool("request_handler_process_catchup_reply_event_from_catchup_single_state_works");
            _create_pool("request_handler_process_catchup_reply_event_from_catchup_single_state_works", None);

            let mut request_handler = _request_handler("request_handler_process_catchup_reply_event_from_catchup_single_state_works", 0, 1);

            let mt = MerkleTree {
                root: Tree::Leaf {
                    hash: vec![144, 26, 156, 60, 166, 79, 255, 53, 172, 15, 42, 186, 99, 222, 43, 53, 230, 243, 151, 105, 0, 233, 90, 151, 103, 149, 22, 172, 76, 124, 247, 62],
                    value: vec![132, 172, 114, 101, 113, 83, 105, 103, 110, 97, 116, 117, 114, 101, 128, 163, 116, 120, 110, 131, 164, 100, 97, 116, 97, 130, 164, 100, 97, 116, 97, 135, 165, 97, 108, 105, 97, 115, 165, 78, 111, 100, 101, 49, 166, 98, 108, 115, 107, 101, 121, 217, 175, 52, 78, 56, 97, 85, 78, 72, 83, 103, 106, 81, 86, 103, 107, 112, 109, 56, 110, 104, 78, 69, 102, 68, 102, 54, 116, 120, 72, 122, 110, 111, 89, 82, 69, 103, 57, 107, 105, 114, 109, 74, 114, 107, 105, 118, 103, 76, 52, 111, 83, 69, 105, 109, 70, 70, 54, 110, 115, 81, 54, 77, 52, 49, 81, 118, 104, 77, 50, 90, 51, 51, 110, 118, 101, 115, 53, 118, 102, 83, 110, 57, 110, 49, 85, 119, 78, 70, 74, 66, 89, 116, 87, 86, 110, 72, 89, 77, 65, 84, 110, 55, 54, 118, 76, 117, 76, 51, 122, 85, 56, 56, 75, 121, 101, 65, 89, 99, 72, 102, 115, 105, 104, 51, 72, 101, 54, 85, 72, 99, 88, 68, 120, 99, 97, 101, 99, 72, 86, 122, 54, 106, 104, 67, 89, 122, 49, 80, 50, 85, 90, 110, 50, 98, 68, 86, 114, 117, 76, 53, 119, 88, 112, 101, 104, 103, 66, 102, 66, 97, 76, 75, 109, 51, 66, 97, 169, 99, 108, 105, 101, 110, 116, 95, 105, 112, 168, 49, 48, 46, 48, 46, 48, 46, 50, 171, 99, 108, 105, 101, 110, 116, 95, 112, 111, 114, 116, 205, 37, 230, 167, 110, 111, 100, 101, 95, 105, 112, 168, 49, 48, 46, 48, 46, 48, 46, 50, 169, 110, 111, 100, 101, 95, 112, 111, 114, 116, 205, 37, 229, 168, 115, 101, 114, 118, 105, 99, 101, 115, 145, 169, 86, 65, 76, 73, 68, 65, 84, 79, 82, 164, 100, 101, 115, 116, 217, 44, 71, 119, 54, 112, 68, 76, 104, 99, 66, 99, 111, 81, 101, 115, 78, 55, 50, 113, 102, 111, 116, 84, 103, 70, 97, 55, 99, 98, 117, 113, 90, 112, 107, 88, 51, 88, 111, 54, 112, 76, 104, 80, 104, 118, 168, 109, 101, 116, 97, 100, 97, 116, 97, 129, 164, 102, 114, 111, 109, 182, 84, 104, 55, 77, 112, 84, 97, 82, 90, 86, 82, 89, 110, 80, 105, 97, 98, 100, 115, 56, 49, 89, 164, 116, 121, 112, 101, 161, 48, 171, 116, 120, 110, 77, 101, 116, 97, 100, 97, 116, 97, 130, 165, 115, 101, 113, 78, 111, 1, 165, 116, 120, 110, 73, 100, 217, 64, 102, 101, 97, 56, 50, 101, 49, 48, 101, 56, 57, 52, 52, 49, 57, 102, 101, 50, 98, 101, 97, 55, 100, 57, 54, 50, 57, 54, 97, 54, 100, 52, 54, 102, 53, 48, 102, 57, 51, 102, 57, 101, 101, 100, 97, 57, 53, 52, 101, 99, 52, 54, 49, 98, 50, 101, 100, 50, 57, 53, 48, 98, 54, 50, 163, 118, 101, 114, 161, 49],
                },
                height: 0,
                count: 1,
                nodes_count: 0,
            };

            request_handler.process_event(Some(RequestEvent::CatchupReq(mt, 2, vec![55, 104, 239, 91, 37, 160, 29, 25, 192, 253, 166, 135, 242, 53, 75, 41, 224, 4, 130, 27, 206, 133, 87, 231, 0, 133, 55, 159, 83, 105, 7, 237])));

            let mut txns: HashMap<String, SJsonValue> = HashMap::new();
            txns.insert("2".to_string(), serde_json::from_str::<SJsonValue>(r#"{"reqSignature":{},"txn":{"data":{"data":{"alias":"Node2","client_port":9704,"blskey":"37rAPpXVoxzKhz7d9gkUe52XuXryuLXoM6P6LbWDB7LSbG62Lsb33sfG7zqS8TK1MXwuCHj1FKNzVpsnafmqLG1vXN88rt38mNFs9TENzm4QHdBzsvCuoBnPH7rpYYDo9DZNJePaDvRvqJKByCabubJz3XXKbEeshzpz4Ma5QYpJqjk","node_port":9703,"node_ip":"10.0.0.2","services":["VALIDATOR"],"client_ip":"10.0.0.2"},"dest":"8ECVSk179mjsjKRLWiQtssMLgp6EPhWXtaYyStWPSGAb"},"metadata":{"from":"EbP4aYNeTHL6q385GuVpRV"},"type":"0"},"txnMetadata":{"seqNo":2,"txnId":"1ac8aece2a18ced660fef8694b61aac3af08ba875ce3026a160acbc3a3af35fc"},"ver":"1"}"#).unwrap());

            let cr = CatchupRep { ledgerId: 0, consProof: Vec::new(), txns };

            request_handler.process_event(Some(RequestEvent::CatchupRep(cr, "Node1".to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
            test::cleanup_pool("request_handler_process_catchup_reply_event_from_catchup_single_state_works");
        }

        #[test]
        fn request_handler_process_catchup_reply_event_from_catchup_single_state_works_for_error() {
            let mut request_handler = _request_handler("request_handler_process_catchup_reply_event_from_catchup_single_state_works_for_error", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::CatchupRep(CatchupRep::default(), NODE.to_string())));
            assert_match!(RequestState::CatchupSingle(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_catchup_single_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::CatchupSingle(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_terminate_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_terminate_event_from_catchup_single_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_catchup_single_state_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_catchup_single_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CatchupReq(MerkleTree::default(), 1, vec![])));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestState::CatchupSingle(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod full {
        use super::*;

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_full_state_works_for_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_full_state_works_for_not_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reply_event_from_full_state_works_for_different_replies() {
            let mut request_handler = _request_handler("request_handler_process_reply_event_from_full_state_works_for_different_replies", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":"11"}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Reply(Reply::default(), r#"{"result":"22"}"#.to_string(), "n2".to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_full_state_works_for_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqnack_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reqnack_event_from_full_state_works_for_not_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::ReqNACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_full_state_works_for_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reject_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_reject_event_from_full_state_works_for_not_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Reject(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_full_state_works_for_completed() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_full_state_works_for_completed", 1, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_timeout_event_from_full_state_works_for_not_completed() {
            let mut request_handler = _request_handler("request_handler_process_timeout_event_from_full_state_works_for_not_completed", 1, 2);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Timeout(REQ_ID.to_string(), NODE.to_string())));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_reqack_event_from_full_state_works() {
            let mut request_handler = _request_handler("request_handler_process_reqack_event_from_full_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::ReqACK(Response::default(), r#"{"result":""}"#.to_string(), NODE.to_string(), REQ_ID.to_string())));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_terminate_event_from_full_state_works() {
            let mut request_handler = _request_handler("request_handler_process_terminate_event_from_full_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Terminate));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }

        #[test]
        fn request_handler_process_other_event_from_full_state_works() {
            let mut request_handler = _request_handler("request_handler_process_other_event_from_full_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomFullRequest(r#"{"result":""}"#.to_string(), REQ_ID.to_string(), None, None)));
            request_handler.process_event(Some(RequestEvent::Pong));
            assert_match!(RequestState::Full(_), request_handler.request_wrapper.unwrap().state);
        }
    }

    mod finish {
        use super::*;

        #[test]
        fn request_handler_process_event_from_finish_state_works() {
            let mut request_handler = _request_handler("request_handler_process_event_from_finish_state_works", 0, 1);
            request_handler.process_event(Some(RequestEvent::CustomConsensusRequest(MESSAGE.to_string(), REQ_ID.to_string())));
            request_handler.process_event(Some(RequestEvent::Terminate));
            request_handler.process_event(Some(RequestEvent::Ping));
            assert_match!(RequestState::Finish(_), request_handler.request_wrapper.unwrap().state);
        }
    }
}
