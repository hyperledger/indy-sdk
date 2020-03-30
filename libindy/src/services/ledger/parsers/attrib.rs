use super::response::GetReplyResultV1;
use indy_vdr::common::did::ShortDidValue;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
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
