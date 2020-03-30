use indy_api_types::validation::Validatable;

use ursa::cl::{
    CredentialPrivateKey,
    CredentialKeyCorrectnessProof
};
pub use indy_vdr::ledger::requests::cred_def::{CredentialDefinition, SignatureType, CredentialDefinitionV1, CredentialDefinitionData};
pub use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;

use std::collections::HashMap;

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
pub struct TemporaryCredentialDefinition {
    pub cred_def: CredentialDefinition,
    pub cred_def_priv_key: CredentialDefinitionPrivateKey,
    pub cred_def_correctness_proof: CredentialDefinitionCorrectnessProof
}

pub type CredentialDefinitions = HashMap<CredentialDefinitionId, CredentialDefinition>;

pub fn cred_defs_map_to_cred_defs_v1_map(cred_defs: CredentialDefinitions) -> HashMap<CredentialDefinitionId, CredentialDefinitionV1> {
    cred_defs
        .into_iter()
        .map(|(cred_def_id, cred_def)|
            match cred_def {
                CredentialDefinition::CredentialDefinitionV1(cred_def) => (cred_def_id, cred_def)
            }
        ).collect()
}



#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionPrivateKey {
    pub value: CredentialPrivateKey
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinitionCorrectnessProof {
    pub value: CredentialKeyCorrectnessProof
}

impl Validatable for CredentialDefinitionConfig {}

#[cfg(test)]
mod tests {
    use super::*;
    use indy_vdr::common::did::DidValue;
    use indy_vdr::ledger::identifiers::schema::SchemaId;
    use indy_vdr::utils::validation::Validatable;

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
        use indy_vdr::utils::qualifier::Qualifiable;

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
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_without_tag() {
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_unqualified_without_tag().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_unqualified(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq() {
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_unqualified_with_schema_as_seq_no().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_unqualified_with_schema_as_seq_without_tag() {
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_unqualified_with_schema_as_seq_no_without_tag().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_seq_no(), schema_id);
            assert_eq!(String::new(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified() {
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_signature_type(), signature_type);
            assert_eq!(_schema_id_qualified(), schema_id);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_cred_def_id_parts_for_id_as_qualified_with_schema_as_seq() {
            let (_, did, signature_type, schema_id, tag) = _cred_def_id_qualified_with_schema_as_seq_no().parts().unwrap();
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