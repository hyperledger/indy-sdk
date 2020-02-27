use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use indy_api_types::errors::prelude::*;
use indy_utils::crypto::verkey_builder::build_full_verkey;
use indy_api_types::CommandHandle;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NodeData {
    pub alias: String,
    pub client_ip: Option<String>,
    #[serde(deserialize_with = "string_or_number")]
    #[serde(default)]
    pub client_port: Option<u64>,
    pub node_ip: Option<String>,
    #[serde(deserialize_with = "string_or_number")]
    #[serde(default)]
    pub node_port: Option<u64>,
    pub services: Option<Vec<String>>,
    pub blskey: Option<String>,
    pub blskey_pop: Option<String>,
}

fn string_or_number<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where D: serde::Deserializer<'de>
{
    let deser_res: Result<serde_json::Value, _> = serde::Deserialize::deserialize(deserializer);

    match deser_res {
        Ok(serde_json::Value::String(s)) => match s.parse::<u64>() {
            Ok(num) => Ok(Some(num)),
            Err(err) => Err(serde::de::Error::custom(format!("Invalid Node transaction: {:?}", err)))
        },
        Ok(serde_json::Value::Number(n)) => match n.as_u64() {
            Some(num) => Ok(Some(num)),
            None => Err(serde::de::Error::custom("Invalid Node transaction".to_string()))
        },
        Ok(serde_json::Value::Null) => Ok(None),
        _ => Err(serde::de::Error::custom("Invalid Node transaction".to_string())),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeTransaction {
    NodeTransactionV0(NodeTransactionV0),
    NodeTransactionV1(NodeTransactionV1),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NodeTransactionV0 {
    pub data: NodeData,
    pub dest: String,
    pub identifier: String,
    #[serde(rename = "txnId")]
    pub txn_id: Option<String>,
    pub verkey: Option<String>,
    #[serde(rename = "type")]
    pub txn_type: String,
}

impl NodeTransactionV0 {
    pub const VERSION: &'static str = "1.3";
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeTransactionV1 {
    pub txn: Txn,
    pub txn_metadata: Metadata,
    pub req_signature: ReqSignature,
    pub ver: String,
}


impl NodeTransactionV1 {
    pub const VERSION: &'static str = "1.4";
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Txn {
    #[serde(rename = "type")]
    pub txn_type: String,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: Option<i32>,
    pub data: TxnData,
    pub metadata: TxnMetadata,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub creation_time: Option<u64>,
    pub seq_no: Option<i32>,
    pub txn_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReqSignature {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub values: Option<Vec<ReqSignatureValue>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ReqSignatureValue {
    pub from: Option<String>,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct TxnData {
    pub data: NodeData,
    pub dest: String,
    pub verkey: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxnMetadata {
    pub req_id: Option<i64>,
    pub from: String,
}

impl From<NodeTransactionV0> for NodeTransactionV1 {
    fn from(node_txn: NodeTransactionV0) -> Self {
        {
            let txn = Txn {
                txn_type: node_txn.txn_type,
                protocol_version: None,
                data: TxnData {
                    data: node_txn.data,
                    dest: node_txn.dest,
                    verkey: node_txn.verkey,
                },
                metadata: TxnMetadata {
                    req_id: None,
                    from: node_txn.identifier,
                },
            };
            NodeTransactionV1 {
                txn,
                txn_metadata: Metadata {
                    seq_no: None,
                    txn_id: node_txn.txn_id,
                    creation_time: None,
                },
                req_signature: ReqSignature {
                    type_: None,
                    values: None,
                },
                ver: "1".to_string(),
            }
        }
    }
}

impl NodeTransactionV1 {
    pub fn update(&mut self, other: &mut NodeTransactionV1) -> IndyResult<()> {
        assert_eq!(self.txn.data.dest, other.txn.data.dest);
        assert_eq!(self.txn.data.data.alias, other.txn.data.data.alias);

        if let Some(ref mut client_ip) = other.txn.data.data.client_ip {
            self.txn.data.data.client_ip = Some(client_ip.to_owned());
        }

        if let Some(ref mut client_port) = other.txn.data.data.client_port {
            self.txn.data.data.client_port = Some(client_port.to_owned());
        }

        if let Some(ref mut node_ip) = other.txn.data.data.node_ip {
            self.txn.data.data.node_ip = Some(node_ip.to_owned());
        }

        if let Some(ref mut node_port) = other.txn.data.data.node_port {
            self.txn.data.data.node_port = Some(node_port.to_owned());
        }

        if let Some(ref mut blskey) = other.txn.data.data.blskey {
            self.txn.data.data.blskey = Some(blskey.to_owned());
        }

        if let Some(ref mut blskey_pop) = other.txn.data.data.blskey_pop {
            self.txn.data.data.blskey_pop = Some(blskey_pop.to_owned());
        }

        if let Some(ref mut services) = other.txn.data.data.services {
            self.txn.data.data.services = Some(services.to_owned());
        }

        if other.txn.data.verkey.is_some() {
            self.txn.data.verkey = Some(build_full_verkey(&self.txn.data.dest, other.txn.data.verkey.as_ref().map(String::as_str))?);
        }

        Ok(())
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerStatus {
    pub txnSeqNo: usize,
    pub merkleRoot: String,
    pub ledgerId: u8,
    pub ppSeqNo: Option<u32>,
    pub viewNo: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocolVersion: Option<usize>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsistencyProof {
    //TODO almost all fields Option<> or find better approach
    pub seqNoEnd: usize,
    pub seqNoStart: usize,
    pub ledgerId: usize,
    pub hashes: Vec<String>,
    pub oldMerkleRoot: String,
    pub newMerkleRoot: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CatchupReq {
    pub ledgerId: usize,
    pub seqNoStart: usize,
    pub seqNoEnd: usize,
    pub catchupTill: usize,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CatchupRep {
    pub ledgerId: usize,
    pub consProof: Vec<String>,
    pub txns: HashMap<String, serde_json::Value>,
}

impl CatchupRep {
    pub fn min_tx(&self) -> IndyResult<usize> {
        let mut min = None;

        for (k, _) in self.txns.iter() {
            let val = k.parse::<usize>()
                .to_indy(IndyErrorKind::InvalidStructure, "Invalid key in catchup reply")?;

            match min {
                None => min = Some(val),
                Some(m) => if val < m { min = Some(val) }
            }
        }

        min.ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "Empty map"))
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

impl Reply {
    pub fn req_id(&self) -> u64 {
        match *self {
            Reply::ReplyV0(ref reply) => reply.result.req_id,
            Reply::ReplyV1(ref reply) => reply.result.txn.metadata.req_id
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyV0 {
    pub result: ResponseMetadata
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyV1 {
    pub result: ReplyResultV1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyResultV1 {
    pub txn: ReplyTxnV1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplyTxnV1 {
    pub metadata: ResponseMetadata
}

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Response {
    ResponseV0(ResponseV0),
    ResponseV1(ResponseV1),
}

impl Response {
    pub fn req_id(&self) -> u64 {
        match *self {
            Response::ResponseV0(ref res) => res.req_id,
            Response::ResponseV1(ref res) => res.metadata.req_id
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseV0 {
    pub req_id: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseV1 {
    pub metadata: ResponseMetadata
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    pub req_id: u64
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(untagged)]
pub enum PoolLedgerTxn {
    PoolLedgerTxnV0(PoolLedgerTxnV0),
    PoolLedgerTxnV1(PoolLedgerTxnV1),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnV0 {
    pub txn: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnV1 {
    pub txn: PoolLedgerTxnDataV1,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoolLedgerTxnDataV1 {
    pub txn: Response,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SimpleRequest {
    pub req_id: u64,
}

#[serde(tag = "op")]
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    #[serde(rename = "CONSISTENCY_PROOF")]
    ConsistencyProof(ConsistencyProof),
    #[serde(rename = "LEDGER_STATUS")]
    LedgerStatus(LedgerStatus),
    #[serde(rename = "CATCHUP_REQ")]
    CatchupReq(CatchupReq),
    #[serde(rename = "CATCHUP_REP")]
    CatchupRep(CatchupRep),
    #[serde(rename = "REQACK")]
    ReqACK(Response),
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(rename = "REPLY")]
    Reply(Reply),
    #[serde(rename = "REJECT")]
    Reject(Response),
    #[serde(rename = "POOL_LEDGER_TXNS")]
    PoolLedgerTxns(PoolLedgerTxn),
    Ping,
    Pong,
}

impl Message {
    pub fn from_raw_str(str: &str) -> IndyResult<Message> {
        match str {
            "po" => Ok(Message::Pong),
            "pi" => Ok(Message::Ping),
            _ => serde_json::from_str::<Message>(str)
                .to_indy(IndyErrorKind::InvalidStructure, "Malformed message json"),
        }
    }
}

/**
 Single item to verification:
 - SP Trie with RootHash
 - BLS MS
 - set of key-value to verify
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedSP {
    /// encoded SP Trie transferred from Node to Client
    pub proof_nodes: String,
    /// RootHash of the Trie, start point for verification. Should be same with appropriate filed in BLS MS data
    pub root_hash: String,
    /// entities to verification against current SP Trie
    pub kvs_to_verify: KeyValuesInSP,
    /// BLS MS data for verification
    pub multi_signature: serde_json::Value,
}

/**
 Variants of representation for items to verify against SP Trie
 Right now 2 options are specified:
 - simple array of key-value pair
 - whole subtrie
*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum KeyValuesInSP {
    Simple(KeyValueSimpleData),
    SubTrie(KeyValuesSubTrieData),
}

/**
 Simple variant of `KeyValuesInSP`.

 All required data already present in parent SP Trie (built from `proof_nodes`).
 `kvs` can be verified directly in parent trie

 Encoding of `key` in `kvs` is defined by verification type
*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct KeyValueSimpleData {
    pub kvs: Vec<(String /* key */, Option<String /* val */>)>,
    #[serde(default)]
    pub verification_type: KeyValueSimpleDataVerificationType
}

/**
 Options of common state proof check process
*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum KeyValueSimpleDataVerificationType {
    /* key should be base64-encoded string */
    Simple,
    /* key should be plain string */
    NumericalSuffixAscendingNoGaps(NumericalSuffixAscendingNoGapsData),
    /* nodes are from a simple merkle tree */
    MerkleTree(u64)
}

impl Default for KeyValueSimpleDataVerificationType {
    fn default() -> Self {
        KeyValueSimpleDataVerificationType::Simple
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct NumericalSuffixAscendingNoGapsData {
    pub from: Option<u64>,
    pub next: Option<u64>,
    pub prefix: String
}

/**
 Subtrie variant of `KeyValuesInSP`.

 In this case Client (libindy) should construct subtrie and append it into trie based on `proof_nodes`.
 After this preparation each kv pair can be checked.
*/
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct KeyValuesSubTrieData {
    /// base64-encoded common prefix of each pair in `kvs`. Should be used to correct merging initial trie and subtrie
    pub sub_trie_prefix: Option<String>,
    pub kvs: Vec<(String /* b64-encoded key_suffix */, Option<String /* val */>)>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RemoteNode {
    pub name: String,
    pub public_key: Vec<u8>,
    pub zaddr: String,
    pub is_blacklisted: bool,
}

pub trait MinValue {
    fn get_min_index(&self) -> IndyResult<usize>;
}

impl MinValue for Vec<(CatchupRep, usize)> {
    fn get_min_index(&self) -> IndyResult<usize> {
        let mut res = None;

        for (index, &(ref catchup_rep, _)) in self.iter().enumerate() {
            match res {
                None => { res = Some((catchup_rep, index)); }
                Some((min_rep, _)) => if catchup_rep.min_tx()? < min_rep.min_tx()? {
                    res = Some((catchup_rep, index));
                }
            }
        }

        Ok(res.ok_or_else(|| err_msg(IndyErrorKind::InvalidStructure, "Element not Found"))?.1)
    }
}

#[derive(Debug)]
pub struct HashableValue {
    pub inner: serde_json::Value
}

impl Eq for HashableValue {}

impl Hash for HashableValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        serde_json::to_string(&self.inner).unwrap().hash(state); //TODO
    }
}

impl PartialEq for HashableValue {
    fn eq(&self, other: &HashableValue) -> bool {
        self.inner.eq(&other.inner)
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct ResendableRequest {
    pub request: String,
    pub start_node: usize,
    pub next_node: usize,
    pub next_try_send_time: Option<time::Tm>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandProcess {
    pub nack_cnt: usize,
    pub replies: HashMap<HashableValue, usize>,
    pub accum_replies: Option<HashableValue>,
    pub parent_cmd_ids: Vec<CommandHandle>,
    pub resendable_request: Option<ResendableRequest>,
    pub full_cmd_timeout: Option<time::Tm>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestToSend {
    pub request: String,
    pub id: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageToProcess {
    pub message: String,
    pub node_idx: usize,
}
