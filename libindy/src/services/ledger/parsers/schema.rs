use super::response::GetReplyResultV1;
use indy_vdr::common::did::ShortDidValue;
use indy_vdr::ledger::requests::schema::SchemaOperationData;
use indy_vdr::ledger::identifiers::schema::SchemaId;

use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetSchemaReplyResult {
    GetSchemaReplyResultV0(GetSchemaResultV0),
    GetSchemaReplyResultV1(GetReplyResultV1<GetSchemaResultDataV1>),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultV0 {
    pub seq_no: u32,
    pub data: SchemaOperationData,
    pub dest: ShortDidValue,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataV1 {
    pub ver: String,
    pub id: SchemaId,
    pub schema_name: String,
    pub schema_version: String,
    pub value: GetSchemaResultDataValueV1,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataValueV1 {
    pub attr_names: HashSet<String>
}
