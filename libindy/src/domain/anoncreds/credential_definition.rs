use super::DELIMITER;
use super::schema::SchemaId;
use super::super::ledger::request::ProtocolVersion;
use indy_api_types::domain::crypto::did::DidValue;

use indy_api_types::validation::Validatable;

use indy_api_types::qualifier;

use ursa::cl::{
    CredentialPrimaryPublicKey,
    CredentialRevocationPublicKey,
    CredentialPrivateKey,
    CredentialKeyCorrectnessProof
};

use std::collections::HashMap;
use named_type::NamedType;

pub const CL_SIGNATURE_TYPE: &str = "CL";

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum SignatureType {
    CL
}

impl SignatureType {
    pub fn to_str(&self) -> &'static str {
        match *self {
            SignatureType::CL => CL_SIGNATURE_TYPE
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialDefinitionConfig {
    #[serde(default)]
    pub support_revocation: bool
}

impl Default for CredentialDefinitionConfig {
    fn default() -> Self {
        CredentialDefinitionConfig {
            support_revocation: false
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionData {
    pub primary: CredentialPrimaryPublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation: Option<CredentialRevocationPublicKey>
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialDefinitionV1 {
    pub id: CredentialDefinitionId,
    pub schema_id: SchemaId,
    #[serde(rename = "type")]
    pub signature_type: SignatureType,
    pub tag: String,
    pub value: CredentialDefinitionData
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
#[serde(tag = "ver")]
pub enum CredentialDefinition {
    #[serde(rename = "1.0")]
    CredentialDefinitionV1(CredentialDefinitionV1)
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
pub struct TemporaryCredentialDefinition {
    pub cred_def: CredentialDefinition,
    pub cred_def_priv_key: CredentialDefinitionPrivateKey,
    pub cred_def_correctness_proof: CredentialDefinitionCorrectnessProof
}

impl CredentialDefinition {
    pub fn to_unqualified(self) -> CredentialDefinition {
        match self {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => {
                CredentialDefinition::CredentialDefinitionV1(CredentialDefinitionV1 {
                    id: cred_def.id.to_unqualified(),
                    schema_id: cred_def.schema_id.to_unqualified(),
                    signature_type: cred_def.signature_type,
                    tag: cred_def.tag,
                    value: cred_def.value,
                })
            }
        }
    }
}

impl From<CredentialDefinition> for CredentialDefinitionV1 {
    fn from(cred_def: CredentialDefinition) -> Self {
        match cred_def {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => cred_def
        }
    }
}

pub type CredentialDefinitions = HashMap<CredentialDefinitionId, CredentialDefinition>;

pub fn cred_defs_map_to_cred_defs_v1_map(cred_defs: CredentialDefinitions) -> HashMap<CredentialDefinitionId, CredentialDefinitionV1> {
    cred_defs
        .into_iter()
        .map(|(cred_def_id, cred_def)| (cred_def_id, CredentialDefinitionV1::from(cred_def)))
        .collect()
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
pub struct CredentialDefinitionPrivateKey {
    pub value: CredentialPrivateKey
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
pub struct CredentialDefinitionCorrectnessProof {
    pub value: CredentialKeyCorrectnessProof
}

impl Validatable for CredentialDefinition {
    fn validate(&self) -> Result<(), String> {
        match self {
            CredentialDefinition::CredentialDefinitionV1(cred_def) => {
                cred_def.id.validate()?;
                cred_def.schema_id.validate()?;
                Ok(())
            }
        }
    }
}

qualifiable_type!(CredentialDefinitionId);

impl CredentialDefinitionId {
    pub const PREFIX: &'static str = "creddef";
    pub const MARKER: &'static str = "3";

    pub fn new(did: &DidValue, schema_id: &SchemaId, signature_type: &str, tag: &str) -> CredentialDefinitionId {
        let id = if ProtocolVersion::is_node_1_3() {
            CredentialDefinitionId(format!("{}{}{}{}{}{}{}", did.0, DELIMITER, Self::MARKER, DELIMITER, signature_type, DELIMITER, schema_id.0))
        } else {
            let tag = if tag.is_empty() { format!("") } else { format!("{}{}", DELIMITER, tag) };
            CredentialDefinitionId(format!("{}{}{}{}{}{}{}{}", did.0, DELIMITER, Self::MARKER, DELIMITER, signature_type, DELIMITER, schema_id.0, tag))
        };
        match did.get_method() {
            Some(method) => id.set_method(&method),
            None => id
        }
    }

    pub fn parts(&self) -> Option<(DidValue, String, SchemaId, String)> {
        let parts = self.0.split_terminator(DELIMITER).collect::<Vec<&str>>();

        if parts.len() == 4 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 5 {
            // Th7MpTaRZVRYnPiabds81Y:3:CL:1:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3].to_string();
            let tag = parts[4].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 7 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(DELIMITER);
            let tag = String::new();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 8 {
            // NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[0].to_string();
            let signature_type = parts[2].to_string();
            let schema_id = parts[3..7].join(DELIMITER);
            let tag = parts[7].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 9 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:3:tag
            let did = parts[2..5].join(DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7].to_string();
            let tag = parts[8].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        if parts.len() == 16 {
            // creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag
            let did = parts[2..5].join(DELIMITER);
            let signature_type = parts[6].to_string();
            let schema_id = parts[7..15].join(DELIMITER);
            let tag = parts[15].to_string();
            return Some((DidValue(did), signature_type, SchemaId(schema_id), tag));
        }

        None
    }

    pub fn issuer_did(&self) -> Option<DidValue> {
        self.parts().map(|(did, _, _, _)| did)
    }

    pub fn qualify(&self, method: &str) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => {
                CredentialDefinitionId::new(&did.qualify(method), &schema_id.qualify(method), &signature_type, &tag)
            }
            None => self.clone()
        }
    }

    pub fn to_unqualified(&self) -> CredentialDefinitionId {
        match self.parts() {
            Some((did, signature_type, schema_id, tag)) => {
                CredentialDefinitionId::new(&did.to_unqualified(), &schema_id.to_unqualified(), &signature_type, &tag)
            }
            None => self.clone()
        }
    }
}

impl Validatable for CredentialDefinitionId {
    fn validate(&self) -> Result<(), String> {
        self.parts().ok_or(format!("Credential Definition Id validation failed: {:?}, doesn't match pattern", self.0))?;
        Ok(())
    }
}

impl Validatable for CredentialDefinitionConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _signature_type() -> String { "CL".to_string() }

    fn _tag() -> String { "tag".to_string() }

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

    fn _cred_def_id_unqualified() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    fn _cred_def_id_unqualified_with_schema_as_seq_no() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _cred_def_id_unqualified_with_schema_as_seq_no_without_tag() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:1".to_string())
    }

    fn _cred_def_id_unqualified_without_tag() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string())
    }

    fn _cred_def_id_qualified_with_schema_as_seq_no() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:1:tag".to_string())
    }

    fn _cred_def_id_qualified() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    mod to_unqualified {
        use super::*;

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified() {
            assert_eq!(_cred_def_id_unqualified(), _cred_def_id_unqualified().to_unqualified());
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag() {
            assert_eq!(_cred_def_id_unqualified_without_tag(), _cred_def_id_unqualified_without_tag().to_unqualified());
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag_with_schema_as_seq_no() {
            assert_eq!(_cred_def_id_unqualified_with_schema_as_seq_no(), _cred_def_id_unqualified_with_schema_as_seq_no().to_unqualified());
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag_with_schema_as_seq_no_without_tag() {
            assert_eq!(_cred_def_id_unqualified_with_schema_as_seq_no_without_tag(), _cred_def_id_unqualified_with_schema_as_seq_no_without_tag().to_unqualified());
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified() {
            assert_eq!(_cred_def_id_unqualified(), _cred_def_id_qualified().to_unqualified());
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified_with_schema_as_seq_no() {
            assert_eq!(_cred_def_id_unqualified_with_schema_as_seq_no(), _cred_def_id_qualified_with_schema_as_seq_no().to_unqualified());
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_unqualified_without_tag().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_unqualified_with_schema_as_seq_no().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq_without_tag() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_unqualified_with_schema_as_seq_no_without_tag().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_qualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified_with_schema_as_seq() {
            let (did, signature_type, schema_id, tag) = _cred_def_id_qualified_with_schema_as_seq_no().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(_tag(), tag);
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn test_validate_cred_def_id_as_unqualified() {
            _cred_def_id_unqualified().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_without_tag() {
            _cred_def_id_unqualified_without_tag().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_with_schema_as_seq_no() {
            _cred_def_id_unqualified_with_schema_as_seq_no().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_unqualified_with_schema_as_seq_no_without_tag() {
            _cred_def_id_unqualified_with_schema_as_seq_no_without_tag().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_fully_qualified() {
            _cred_def_id_qualified().validate().unwrap();
        }

        #[test]
        fn test_validate_cred_def_id_as_fully_qualified_with_schema_as_seq_no() {
            _cred_def_id_qualified_with_schema_as_seq_no().validate().unwrap();
        }
    }
}