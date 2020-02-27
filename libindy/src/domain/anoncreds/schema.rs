use super::DELIMITER;

use indy_api_types::domain::crypto::did::DidValue;

use std::collections::{HashMap, HashSet};
use named_type::NamedType;

use indy_api_types::validation::Validatable;
use indy_api_types::qualifier;

pub const MAX_ATTRIBUTES_COUNT: usize = 125;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaV1 {
    pub id: SchemaId,
    pub name: String,
    pub version: String,
    #[serde(rename = "attrNames")]
    pub attr_names: AttributeNames,
    pub seq_no: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
#[serde(tag = "ver")]
pub enum Schema {
    #[serde(rename = "1.0")]
    SchemaV1(SchemaV1)
}

impl Schema {
    pub fn to_unqualified(self) -> Schema {
        match self {
            Schema::SchemaV1(schema) => {
                Schema::SchemaV1(SchemaV1 {
                    id: schema.id.to_unqualified(),
                    name: schema.name,
                    version: schema.version,
                    attr_names: schema.attr_names,
                    seq_no: schema.seq_no,
                })
            }
        }
    }
}

impl From<Schema> for SchemaV1 {
    fn from(schema: Schema) -> Self {
        match schema {
            Schema::SchemaV1(schema) => schema
        }
    }
}

pub type Schemas = HashMap<SchemaId, Schema>;

pub fn schemas_map_to_schemas_v1_map(schemas: Schemas) -> HashMap<SchemaId, SchemaV1> {
    schemas.into_iter().map(|(schema_id, schema)| { (schema_id, SchemaV1::from(schema)) }).collect()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeNames(pub HashSet<String>);

#[allow(dead_code)]
impl AttributeNames {
    pub fn new() -> Self {
        AttributeNames(HashSet::new())
    }
}

impl From<HashSet<String>> for AttributeNames {
    fn from(attrs: HashSet<String>) -> Self {
        AttributeNames(attrs)
    }
}

impl Into<HashSet<String>> for AttributeNames {
    fn into(self) -> HashSet<String> {
        self.0
    }
}

impl Validatable for Schema {
    fn validate(&self) -> Result<(), String> {
        match self {
            Schema::SchemaV1(schema) => {
                schema.attr_names.validate()?;
                schema.id.validate()?;
                if let Some((_, name, version)) = schema.id.parts() {
                    if name != schema.name {
                        return Err(format!("Inconsistent Schema Id and Schema Name: {:?} and {}", schema.id, schema.name))
                    }
                    if version != schema.version {
                        return Err(format!("Inconsistent Schema Id and Schema Version: {:?} and {}", schema.id, schema.version))
                    }
                }
                Ok(())
            }
        }
    }
}

impl Validatable for AttributeNames {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            return Err(String::from("Empty list of Schema attributes has been passed"));
        }

        if self.0.len() > MAX_ATTRIBUTES_COUNT {
            return Err(format!("The number of Schema attributes {} cannot be greater than {}", self.0.len(), MAX_ATTRIBUTES_COUNT));
        }
        Ok(())
    }
}

qualifiable_type!(SchemaId);

impl SchemaId {
    pub const PREFIX: &'static str = "schema";
    pub const MARKER: &'static str = "2";

    pub fn new(did: &DidValue, name: &str, version: &str) -> SchemaId {
        let id = SchemaId(format!("{}{}{}{}{}{}{}", did.0, DELIMITER, Self::MARKER, DELIMITER, name, DELIMITER, version));
        match did.get_method() {
            Some(method) => id.set_method(&method),
            None => id
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, String)> {
        let parts = self.0.split_terminator(DELIMITER).collect::<Vec<&str>>();

        if parts.len() == 1 {
            // 1
            return None;
        }

        if parts.len() == 4 {
            // NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let name = parts[2].to_string();
            let version = parts[3].to_string();
            return Some((DidValue(did), name, version));
        }

        if parts.len() == 8 {
            // schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[2..5].join(DELIMITER);
            let name = parts[6].to_string();
            let version = parts[7].to_string();
            return Some((DidValue(did), name, version));
        }

        None
    }

    pub fn qualify(&self, method: &str) -> SchemaId {
        match self.parts() {
            Some((did, name, version)) => {
                SchemaId::new(&did.qualify(method), &name, &version)
            }
            None => self.clone()
        }
    }

    pub fn to_unqualified(&self) -> SchemaId {
        match self.parts() {
            Some((did, name, version)) => {
                SchemaId::new(&did.to_unqualified(), &name, &version)
            }
            None => self.clone()
        }
    }
}

impl Validatable for SchemaId {
    fn validate(&self) -> Result<(), String> {
        if self.0.parse::<i32>().is_ok() {
            return Ok(());
        }

        self.parts().ok_or(format!("SchemaId validation failed: {:?}, doesn't match pattern", self.0))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _did_qualified() -> DidValue {
        DidValue("did:sov:NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _schema_id_seq_no() -> SchemaId {
        SchemaId("1".to_string())
    }

    fn _schema_id_unqualified() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _schema_id_qualified() -> SchemaId {
        SchemaId("schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _schema_id_invalid() -> SchemaId {
        SchemaId("NcYxiDXkpYi6ov5FcYDi1e:2".to_string())
    }

    mod to_unqualified {
        use super::*;

        #[test]
        fn test_schema_id_unqualify_for_id_as_seq_no() {
            assert_eq!(_schema_id_seq_no(), _schema_id_seq_no().to_unqualified());
        }

        #[test]
        fn test_schema_id_parts_for_id_as_unqualified() {
            assert_eq!(_schema_id_unqualified(), _schema_id_unqualified().to_unqualified());
        }

        #[test]
        fn test_schema_id_parts_for_id_as_qualified() {
            assert_eq!(_schema_id_unqualified(), _schema_id_qualified().to_unqualified());
        }

        #[test]
        fn test_schema_id_parts_for_invalid_unqualified() {
            assert_eq!(_schema_id_invalid(), _schema_id_invalid().to_unqualified());
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_schema_id_parts_for_id_as_seq_no() {
            assert!(_schema_id_seq_no().parts().is_none());
        }

        #[test]
        fn test_schema_id_parts_for_id_as_unqualified() {
            let (did, _, _) = _schema_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
        }

        #[test]
        fn test_schema_id_parts_for_id_as_qualified() {
            let (did, _, _) = _schema_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
        }

        #[test]
        fn test_schema_id_parts_for_invalid_unqualified() {
            assert!(_schema_id_invalid().parts().is_none());
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn test_validate_schema_id_as_seq_no() {
            _schema_id_seq_no().validate().unwrap();
        }

        #[test]
        fn test_validate_schema_id_as_unqualified() {
            _schema_id_unqualified().validate().unwrap();
        }

        #[test]
        fn test_validate_schema_id_as_fully_qualified() {
            _schema_id_qualified().validate().unwrap();
        }

        #[test]
        fn test_validate_schema_id_for_invalid_unqualified() {
            _schema_id_invalid().validate().unwrap_err();
        }

        #[test]
        fn test_validate_schema_id_for_invalid_fully_qualified() {
            let id = SchemaId("schema:sov:NcYxiDXkpYi6ov5FcYDi1e:2:1.0".to_string());
            id.validate().unwrap_err();
        }
    }

    mod test_schema_validation {
        use super::*;

        #[test]
        fn test_valid_schema() {
            let schema_json = json!({
                "id": _schema_id_qualified(),
                "name": "gvt",
                "ver": "1.0",
                "version": "1.0",
                "attrNames": ["aaa", "bbb", "ccc"],
            }).to_string();

            let schema: Schema = serde_json::from_str(&schema_json).unwrap();
            schema.validate().unwrap();
            match schema {
                Schema::SchemaV1(schema) => {
                    assert_eq!(schema.name, "gvt");
                    assert_eq!(schema.version, "1.0");
                }
            }
        }

        #[test]
        fn test_invalid_name_schema() {
            let schema_json = json!({
                "id": _schema_id_qualified(),
                "name": "gvt1",
                "ver": "1.0",
                "version": "1.0",
                "attrNames": ["aaa", "bbb", "ccc"],
            }).to_string();

            let schema: Schema = serde_json::from_str(&schema_json).unwrap();
            schema.validate().unwrap_err();
        }

        #[test]
        fn test_invalid_version_schema() {
            let schema_json = json!({
                "id": _schema_id_qualified(),
                "name": "gvt",
                "ver": "1.0",
                "version": "1.1",
                "attrNames": ["aaa", "bbb", "ccc"],
            }).to_string();

            let schema: Schema = serde_json::from_str(&schema_json).unwrap();
            schema.validate().unwrap_err();
        }
    }
}