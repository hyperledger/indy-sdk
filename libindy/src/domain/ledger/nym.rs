use super::constants::{NYM, GET_NYM};
use super::response::{GetReplyResultV0, GetReplyResultV1, ReplyType};
use indy_api_types::domain::crypto::did::ShortDidValue;

#[derive(Serialize, PartialEq, Debug)]
pub struct NymOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<::serde_json::Value>,
}

impl NymOperation {
    pub fn new(dest: ShortDidValue, verkey: Option<String>, alias: Option<String>, role: Option<::serde_json::Value>) -> NymOperation {
        NymOperation {
            _type: NYM.to_string(),
            dest,
            verkey,
            alias,
            role,
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetNymOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue
}

impl GetNymOperation {
    pub fn new(dest: ShortDidValue) -> GetNymOperation {
        GetNymOperation {
            _type: GET_NYM.to_string(),
            dest
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetNymReplyResult {
    GetNymReplyResultV0(GetReplyResultV0<String>),
    GetNymReplyResultV1(GetReplyResultV1<GetNymResultDataV1>)
}

impl ReplyType for GetNymReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_NYM
    }
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetNymResultDataV0 {
    pub identifier: Option<ShortDidValue>,
    pub dest: ShortDidValue,
    pub role: Option<String>,
    pub verkey: Option<String>
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetNymResultDataV1 {
    pub ver: String,
    pub id: String,
    pub did: ShortDidValue,
    pub verkey: Option<String>,
    pub role: Option<String>
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct NymData {
    pub did: ShortDidValue,
    pub verkey: Option<String>,
    pub role: Option<String>,
}
