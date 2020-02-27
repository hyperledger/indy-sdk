use super::constants::{SCHEMA, GET_SCHEMA};
use super::response::{GetReplyResultV1, ReplyType};
use indy_api_types::domain::crypto::did::ShortDidValue;
use super::super::anoncreds::schema::SchemaId;

use std::collections::HashSet;

#[derive(Serialize, PartialEq, Debug)]
pub struct SchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: SchemaOperationData,
}

impl SchemaOperation {
    pub fn new(data: SchemaOperationData) -> SchemaOperation {
        SchemaOperation {
            data,
            _type: SCHEMA.to_string()
        }
    }
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub struct SchemaOperationData {
    pub name: String,
    pub version: String,
    pub attr_names: HashSet<String>
}

impl SchemaOperationData {
    pub fn new(name: String, version: String, attr_names: HashSet<String>) -> SchemaOperationData {
        SchemaOperationData {
            name,
            version,
            attr_names
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct GetSchemaOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    pub data: GetSchemaOperationData
}

impl GetSchemaOperation {
    pub fn new(dest: ShortDidValue, data: GetSchemaOperationData) -> GetSchemaOperation {
        GetSchemaOperation {
            _type: GET_SCHEMA.to_string(),
            dest,
            data
        }
    }
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub struct GetSchemaOperationData {
    pub name: String,
    pub version: String
}

impl GetSchemaOperationData {
    pub fn new(name: String, version: String) -> GetSchemaOperationData {
        GetSchemaOperationData {
            name,
            version
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GetSchemaReplyResult {
    GetSchemaReplyResultV0(GetSchemaResultV0),
    GetSchemaReplyResultV1(GetReplyResultV1<GetSchemaResultDataV1>)
}

impl ReplyType for GetSchemaReplyResult {
    fn get_type<'a>() -> &'a str {
        GET_SCHEMA
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultV0 {
    pub seq_no: u32,
    pub data: SchemaOperationData,
    pub dest: ShortDidValue
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataV1 {
    pub ver: String,
    pub id: SchemaId,
    pub schema_name: String,
    pub schema_version: String,
    pub value: GetSchemaResultDataValueV1
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaResultDataValueV1 {
    pub attr_names: HashSet<String>
}
