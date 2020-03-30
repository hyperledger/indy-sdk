use super::response::GetReplyResultV1;
use crate::domain::anoncreds::credential_definition::{CredentialDefinitionData, SignatureType, CredentialDefinitionId};
use indy_vdr::ledger::identifiers::schema::SchemaId;
use indy_vdr::common::did::ShortDidValue;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetCredDefReplyResult {
    GetCredDefReplyResultV0(GetCredDefResultV0),
    GetCredDefReplyResultV1(GetReplyResultV1<GetCredDefResultDataV1>),
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
    pub data: CredentialDefinitionData,
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
    pub public_keys: CredentialDefinitionData,
}
