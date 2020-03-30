use ursa::cl::RevocationKeyPrivate;

use std::collections::{HashMap, HashSet};

use indy_api_types::validation::Validatable;

pub use indy_vdr::ledger::requests::rev_reg_def::{
    CL_ACCUM,
    IssuanceType,
    RegistryType,
    RevocationRegistryDefinitionValue,
    RevocationRegistryDefinitionValuePublicKeys,
    RevocationRegistryDefinitionV1,
    RevocationRegistryDefinition,
};
pub use indy_vdr::ledger::identifiers::rev_reg::RevocationRegistryId;

#[derive(Deserialize, Debug, Serialize)]
pub struct RevocationRegistryConfig {
    pub issuance_type: Option<IssuanceType>,
    pub max_cred_num: Option<u32>,
}

pub type RevocationRegistryDefinitions = HashMap<RevocationRegistryId, RevocationRegistryDefinition>;

pub fn rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs: RevocationRegistryDefinitions) -> HashMap<RevocationRegistryId, RevocationRegistryDefinitionV1> {
    rev_reg_defs
        .into_iter()
        .map(|(rev_reg_id, rev_reg_def)|
            match rev_reg_def {
                RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => (rev_reg_id, rev_reg_def)
            }
        ).collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevocationRegistryDefinitionPrivate {
    pub value: RevocationKeyPrivate
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>,
}

impl Validatable for RevocationRegistryConfig {
    fn validate(&self) -> Result<(), String> {
        if let Some(num_) = self.max_cred_num {
            if num_ == 0 {
                return Err(String::from("RevocationRegistryConfig validation failed: `max_cred_num` must be greater than 0"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indy_vdr::common::did::DidValue;
    use indy_vdr::ledger::identifiers::cred_def::CredentialDefinitionId;
    use indy_vdr::utils::validation::Validatable;

    fn _did() -> DidValue {
        DidValue("NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _rev_reg_type() -> String { "CL_ACCUM".to_string() }

    fn _tag() -> String { "TAG_1".to_string() }

    fn _did_qualified() -> DidValue {
        DidValue("did:sov:NcYxiDXkpYi6ov5FcYDi1e".to_string())
    }

    fn _cred_def_id_unqualified() -> CredentialDefinitionId {
        CredentialDefinitionId("NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    fn _cred_def_id_qualified() -> CredentialDefinitionId {
        CredentialDefinitionId("creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag".to_string())
    }

    fn _rev_reg_id_unqualified() -> RevocationRegistryId {
        RevocationRegistryId("NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string())
    }

    fn _rev_reg_id_qualified() -> RevocationRegistryId {
        RevocationRegistryId("revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1".to_string())
    }

    mod to_unqualified {
        use super::*;
        use indy_vdr::utils::qualifier::Qualifiable;

        #[test]
        fn test_rev_reg_id_parts_for_id_as_unqualified() {
            assert_eq!(_rev_reg_id_unqualified(), _rev_reg_id_unqualified().to_unqualified());
        }

        #[test]
        fn test_rev_reg_id_parts_for_id_as_qualified() {
            assert_eq!(_rev_reg_id_unqualified(), _rev_reg_id_qualified().to_unqualified());
        }
    }

    mod parts {
        use super::*;

        #[test]
        fn test_rev_reg_id_parts_for_id_as_unqualified() {
            let (did, cred_def_id, rev_reg_type, tag) = _rev_reg_id_unqualified().parts().unwrap();
            assert_eq!(_did(), did);
            assert_eq!(_cred_def_id_unqualified(), cred_def_id);
            assert_eq!(_rev_reg_type(), rev_reg_type);
            assert_eq!(_tag(), tag);
        }

        #[test]
        fn test_rev_reg_id_parts_for_id_as_qualified() {
            let (did, cred_def_id, rev_reg_type, tag) = _rev_reg_id_qualified().parts().unwrap();
            assert_eq!(_did_qualified(), did);
            assert_eq!(_cred_def_id_qualified(), cred_def_id);
            assert_eq!(_rev_reg_type(), rev_reg_type);
            assert_eq!(_tag(), tag);
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn test_validate_rev_reg_id_as_unqualified() {
            _rev_reg_id_unqualified().validate().unwrap();
        }

        #[test]
        fn test_validate_rev_reg_id_as_fully_qualified() {
            _rev_reg_id_qualified().validate().unwrap();
        }
    }
}
