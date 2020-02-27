use super::constants::{CRED_DEF, GET_CRED_DEF};
use super::response::{GetReplyResultV1, ReplyType};
use super::super::anoncreds::credential_definition::{CredentialDefinitionData, CredentialDefinitionV1, SignatureType, CredentialDefinitionId};
use super::super::anoncreds::schema::SchemaId;
use super::super::ledger::request::ProtocolVersion;
use indy_api_types::domain::crypto::did::ShortDidValue;

#[derive(Serialize, Debug)]
pub struct CredDefOperation {
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub data: CredentialDefinitionData,
    #[serde(rename = "type")]
    pub _type: String,
    pub signature_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>
}

impl CredDefOperation {
    pub fn new(data: CredentialDefinitionV1) -> CredDefOperation {
        CredDefOperation {
            _ref: data.schema_id.0.parse::<i32>().unwrap_or(0),
            signature_type: data.signature_type.to_str().to_string(),
            data: data.value,
            tag: if ProtocolVersion::is_node_1_3() { None } else { Some(data.tag.clone()) },
            _type: CRED_DEF.to_string()
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetCredDefOperation {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "ref")]
    pub _ref: i32,
    pub signature_type: String,
    pub origin: ShortDidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>
}

impl GetCredDefOperation {
    pub fn new(_ref: i32, signature_type: String, origin: ShortDidValue, tag: Option<String>) -> GetCredDefOperation {
        GetCredDefOperation {
            _type: GET_CRED_DEF.to_string(),
            _ref,
            signature_type,
            origin,
            tag
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetCredDefReplyResult {
    GetCredDefReplyResultV0(GetCredDefResultV0),
    GetCredDefReplyResultV1(GetReplyResultV1<GetCredDefResultDataV1>)
}

impl ReplyType for GetCredDefReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_CRED_DEF
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetCredDefResultV0 {
    pub identifier: ShortDidValue,
    #[serde(rename = "ref")]
    pub ref_: u64,
    #[serde(rename = "seqNo")]
    pub seq_no: i32,
    pub signature_type: SignatureType,
    pub origin: ShortDidValue,
    pub tag: Option<String>,
    pub data: CredentialDefinitionData
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCredDefResultDataV1 {
    pub ver: String,
    pub id: CredentialDefinitionId,
    #[serde(rename = "type")]
    pub type_: SignatureType,
    pub tag: String,
    pub schema_ref: SchemaId,
    pub public_keys: CredentialDefinitionData
}
