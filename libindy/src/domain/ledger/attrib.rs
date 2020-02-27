use super::constants::{ATTRIB, GET_ATTR};
use super::response::GetReplyResultV1;
use indy_api_types::domain::crypto::did::ShortDidValue;

use named_type::NamedType;

#[derive(Serialize, PartialEq, Debug)]
pub struct AttribOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc: Option<String>
}

impl AttribOperation {
    pub fn new(dest: ShortDidValue, hash: Option<String>, raw: Option<String>,
               enc: Option<String>) -> AttribOperation {
        AttribOperation {
            _type: ATTRIB.to_string(),
            dest,
            hash,
            raw,
            enc,
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetAttribOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc: Option<String>
}

impl GetAttribOperation {
    pub fn new(dest: ShortDidValue, raw: Option<&str>, hash: Option<&str>, enc: Option<&str>) -> GetAttribOperation {
        GetAttribOperation {
            _type: GET_ATTR.to_string(),
            dest,
            raw: raw.map(String::from),
            hash: hash.map(String::from),
            enc: enc.map(String::from)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetAttrReplyResult {
    GetAttrReplyResultV0(GetAttResultV0),
    GetAttrReplyResultV1(GetReplyResultV1<GetAttResultDataV1>)
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAttResultV0 {
    pub  identifier: ShortDidValue,
    pub  data: String,
    pub  dest: ShortDidValue,
    pub  raw: String
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct GetAttResultDataV1 {
    pub ver: String,
    pub id: String,
    pub did: ShortDidValue,
    pub raw: String,
}

#[derive(Deserialize, Debug)]
pub struct AttribData {
    pub endpoint: Endpoint
}

#[derive(Serialize, Deserialize, Clone, Debug, NamedType)]
pub struct Endpoint {
    pub ha: String, // indy-node and indy-plenum restrict this to ip-address:port
    pub verkey: Option<String>
}

impl Endpoint {
    pub fn new(ha: String, verkey: Option<String>) -> Endpoint {
        Endpoint {
            ha,
            verkey
        }
    }
}
