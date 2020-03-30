use serde_json;
use indy_vdr::common::did::ShortDidValue;

use std::collections::{HashSet, HashMap};

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub enum ResponseType {
    REQNACK,
    REPLY,
    REJECT
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct Response {
    pub op: ResponseType
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct Reply<T> {
    pub op: String,
    pub result: T,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAttribReplyResult {
    pub     identifier: String,
    pub   req_id: u64,
    #[serde(rename = "type")]
    pub   _type: String,
    pub   data: Option<String>,
    pub  dest: String,
    pub  seq_no: Option<i32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetValidatorInfoResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: Option<i32>,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: Option<serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnResult {
    pub identifier: String,
    #[serde(rename = "reqId")]
    pub req_id: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: Option<i32>,
    #[serde(rename = "type")]
    pub _type: String,
    pub data: Option<serde_json::Value>
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaData {
    pub name: String,
    pub version: String,
    pub attr_names: HashSet<String>
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialOfferInfo {
    pub cred_def_id: String
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct WalletRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub value: Option<String>,
    pub tags: Option<HashMap<String, String>>
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchRecords {
    pub total_count: Option<i32>,
    pub records: Option<Vec<WalletRecord>>
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct NymData {
    pub did: ShortDidValue,
    pub verkey: Option<String>,
    pub role: Option<String>,
}