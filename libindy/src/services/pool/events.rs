/*
use serde_json;
use serde_json::Value as SJsonValue;

use crate::domain::ledger::constants;
use indy_api_types::errors::prelude::*;
use crate::services::ledger::merkletree::merkletree::MerkleTree;
use crate::services::pool::{PoolService, types:: *};
use indy_api_types::CommandHandle;

pub const REQUESTS_FOR_STATE_PROOFS: [&str; 11] = [
    constants::GET_NYM,
    constants::GET_TXN_AUTHR_AGRMT,
    constants::GET_TXN_AUTHR_AGRMT_AML,
    constants::GET_SCHEMA,
    constants::GET_CRED_DEF,
    constants::GET_ATTR,
    constants::GET_REVOC_REG,
    constants::GET_REVOC_REG_DEF,
    constants::GET_REVOC_REG_DELTA,
    constants::GET_AUTH_RULE,
    constants::GET_TXN,
];

const REQUEST_FOR_FULL: [&str; 2] = [
    constants::POOL_RESTART,
    constants::GET_VALIDATOR_INFO,
];


pub const REQUESTS_FOR_STATE_PROOFS_IN_THE_PAST: [&str; 5] = [
    constants::GET_REVOC_REG,
    constants::GET_REVOC_REG_DELTA,
    constants::GET_TXN_AUTHR_AGRMT,
    constants::GET_TXN_AUTHR_AGRMT_AML,
    constants::GET_TXN,
];

pub const REQUESTS_FOR_MULTI_STATE_PROOFS: [&str; 1] = [
    constants::GET_REVOC_REG_DELTA,
];

#[derive(Debug, Clone)]
pub enum NetworkerEvent {
    SendOneRequest(
        String, //msg
        String, //req_id
        i64, //timeout
    ),
    SendAllRequest(
        String, //msg
        String, //req_id
        i64, //timeout
        Option<Vec<String>>, //nodes
    ),
    Resend(
        String, //req_id
        i64, //timeout
    ),
    NodesStateUpdated(Vec<RemoteNode>),
    ExtendTimeout(
        String, //req_id
        String, //node_alias
        i64, //timeout
    ),
    CleanTimeout(
        String, //req_id
        Option<String>, //node_alias
    ),
    Timeout,
}

pub const COMMAND_EXIT : &str = "exit";
pub const COMMAND_CONNECT : &str = "connect";
pub const COMMAND_REFRESH : &str = "refresh";

#[derive(Clone, Debug)]
pub enum PoolEvent {
    CheckCache(CommandHandle),
    NodeReply(
        String, // reply
        String, // node alias
    ),
    Close(
        CommandHandle
    ),
    Refresh(
        CommandHandle
    ),
    CatchupTargetFound(
        Vec<u8>, //target_mt_root
        usize, //target_mt_size
        MerkleTree,
    ),
    CatchupRestart(
        MerkleTree,
    ),
    CatchupTargetNotFound(IndyError),
    #[allow(dead_code)] //FIXME
    PoolOutdated,
    Synced(
        MerkleTree
    ),
    #[allow(dead_code)] //FIXME
    NodesBlacklisted,
    SendRequest(
        CommandHandle,
        String, // request
        Option<i32>, // timeout
        Option<String>, // node list
    ),
    Timeout(
        String, //req_id
        String, //node alias
    ),
}

#[derive(Clone, Debug)]
pub enum RequestEvent {
    LedgerStatus(
        LedgerStatus,
        Option<String>, //node alias
        Option<MerkleTree>,
    ),
    CatchupReq(
        MerkleTree,
        usize, // target mt size
        Vec<u8>, // target mt root
    ),
    Timeout(
        String, //req_id
        String, //node_alias
    ),
    CatchupRep(
        CatchupRep,
        String, // node_alias
    ),
    CustomSingleRequest(
        String, // message
        String, // req_id
        Option<Vec<u8>>, // expected key for State Proof in Reply,
        (Option<u64>, Option<u64>) // expected timestamps for freshness comparison
    ),
    CustomConsensusRequest(
        String, // message
        String, // req_id
    ),
    CustomFullRequest(
        String, // message
        String, // req_id
        Option<i32>, // timeout
        Option<String>, // nodes
    ),
    ConsistencyProof(
        ConsistencyProof,
        String, //node alias
    ),
    Reply(
        Reply,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    ReqACK(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    ReqNACK(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    Reject(
        Response,
        String, //raw_msg
        String, //node alias
        String, //req_id
    ),
    PoolLedgerTxns,
    Ping,
    Pong,
    Terminate,
}

impl RequestEvent {
    pub fn get_req_id(&self) -> String {
        match *self {
            RequestEvent::CustomSingleRequest(_, ref id, _, _) => id.to_string(),
            RequestEvent::CustomConsensusRequest(_, ref id) => id.to_string(),
            RequestEvent::CustomFullRequest(_, ref id, _, _) => id.to_string(),
            RequestEvent::Reply(_, _, _, ref id) => id.to_string(),
            RequestEvent::ReqACK(_, _, _, ref id) => id.to_string(),
            RequestEvent::ReqNACK(_, _, _, ref id) => id.to_string(),
            RequestEvent::Reject(_, _, _, ref id) => id.to_string(),
            _ => "".to_string()
        }
    }
}

impl Into<Option<RequestEvent>> for PoolEvent {
    fn into(self) -> Option<RequestEvent> {
        match self {
            PoolEvent::NodeReply(msg, node_alias) => {
                _parse_msg(&msg).map(|parsed|
                    match parsed {
                        //TODO change mapping for CatchupReq. May be return None
                        Message::CatchupReq(_) => RequestEvent::CatchupReq(
                            MerkleTree::default(), 0, vec![]),
                        Message::CatchupRep(rep) => RequestEvent::CatchupRep(rep, node_alias),
                        Message::LedgerStatus(ls) => RequestEvent::LedgerStatus(ls, Some(node_alias), None),
                        Message::ConsistencyProof(cp) => RequestEvent::ConsistencyProof(cp, node_alias),
                        Message::Reply(rep) => {
                            let req_id = rep.req_id();
                            RequestEvent::Reply(rep, msg, node_alias, req_id.to_string())
                        }
                        Message::ReqACK(rep) => {
                            let req_id = rep.req_id();
                            RequestEvent::ReqACK(rep, msg, node_alias, req_id.to_string())
                        }
                        Message::ReqNACK(rep) => {
                            let req_id = rep.req_id();
                            RequestEvent::ReqNACK(rep, msg, node_alias, req_id.to_string())
                        }
                        Message::Reject(rep) => {
                            let req_id = rep.req_id();
                            RequestEvent::Reject(rep, msg, node_alias, req_id.to_string())
                        }
                        Message::PoolLedgerTxns(_) => RequestEvent::PoolLedgerTxns,
                        Message::Ping => RequestEvent::Ping,
                        Message::Pong => RequestEvent::Pong,
                    })
            }
            PoolEvent::SendRequest(_, msg, timeout, nodes) => {
                let parsed_req = _parse_req_id_and_op(&msg);
                if let Ok((ref req, ref req_id, ref op)) = parsed_req {
                    if REQUEST_FOR_FULL.contains(&op.as_str()) {
                        Some(RequestEvent::CustomFullRequest(msg, req_id.clone(), timeout, nodes))
                    } else if timeout.is_some() || nodes.is_some() {
                        error!("Timeout {:?} or nodes {:?} is specified for non-supported request operation type {}",
                               timeout, nodes, op);
                        None
                    } else if REQUESTS_FOR_STATE_PROOFS.contains(&op.as_str()) {
                        let key = super::state_proof::parse_key_from_request_for_builtin_sp(&req);
                        let timestamps = _parse_timestamp_from_req_for_builtin_sp(req, &op);
                        Some(RequestEvent::CustomSingleRequest(msg, req_id.clone(), key, timestamps))
                    } else if PoolService::get_sp_parser(&op.as_str()).is_some() {
                        Some(RequestEvent::CustomSingleRequest(msg, req_id.clone(), None, (None, None)))
                    } else {
                        Some(RequestEvent::CustomConsensusRequest(msg, req_id.clone()))
                    }
                } else {
                    error!("Can't parse parsed_req or op from message {}", msg);
                    None
                }
            }
            PoolEvent::Timeout(req_id, node_alias) => Some(RequestEvent::Timeout(req_id, node_alias)),
            _ => None
        }
    }
}

fn _parse_timestamp_from_req_for_builtin_sp(req: &SJsonValue, op: &str) -> (Option<u64>, Option<u64>) {
    if !REQUESTS_FOR_STATE_PROOFS_IN_THE_PAST.contains(&op) {
        return (None, None);
    }

    if op == constants::GET_TXN {
        return (None, Some(0));
    }

    match op {
        constants::GET_REVOC_REG | constants::GET_TXN_AUTHR_AGRMT | constants::GET_TXN_AUTHR_AGRMT_AML => {
            (None, req["operation"]["timestamp"].as_u64())
        }
        constants::GET_REVOC_REG_DELTA => {
            (req["operation"]["from"].as_u64(), req["operation"]["to"].as_u64())
        }
        _ => { (None, None) }
    }
}

fn _parse_msg(msg: &str) -> Option<Message> {
    Message::from_raw_str(msg).map_err(map_err_trace!()).ok()
}

fn _parse_req_id_and_op(msg: &str) -> IndyResult<(SJsonValue, String, String)> {
    let req_json = _get_req_json(msg)?;

    let req_id = req_json["reqId"]
        .as_u64()
        .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "No reqId in request"))?
        .to_string();

    let op = req_json["operation"]["type"]
        .as_str()
        .ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "No operation type in request"))?
        .to_string();

    Ok((req_json, req_id, op))
}

fn _get_req_json(msg: &str) -> IndyResult<SJsonValue> {
    serde_json::from_str(msg)
        .to_indy(IndyErrorKind::InvalidStructure, "Invalid request json") // FIXME: Review kind
}
*/
