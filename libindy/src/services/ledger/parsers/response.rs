#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub req_id: u64,
    pub reason: String
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply<T> {
    ReplyV0(ReplyV0<T>),
    ReplyV1(ReplyV1<T>)
}

impl<T> Reply<T> {
    pub fn result(self) -> T {
        match self {
            Reply::ReplyV0(reply) => reply.result,
            Reply::ReplyV1(mut reply) => reply.data.result.remove(0).result
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0<T> {
    pub result: T
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1<T> {
    pub data: ReplyDataV1<T>
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1<T> {
    pub  result: Vec<ReplyV0<T>>
}

#[derive(Debug, Deserialize)]
pub struct GetReplyResultV0<T> {
    pub  data: Option<T>
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReplyResultV1<T> {
    pub  txn: GetReplyTxnV1<T>,
    pub  txn_metadata: TxnMetadata,
}

#[derive(Debug, Deserialize)]
pub struct GetReplyTxnV1<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxnMetadata {
    pub seq_no: u32,
    pub creation_time: u64,
}

#[serde(tag = "op")]
#[derive(Deserialize, Debug)]
pub enum Message<T> {
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(rename = "REPLY")]
    Reply(Reply<T>),
    #[serde(rename = "REJECT")]
    Reject(Response)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_no: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_txn_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seq_no: Option<u64>,
}