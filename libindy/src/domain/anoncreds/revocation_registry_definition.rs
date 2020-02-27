use ursa::cl::{RevocationKeyPublic, RevocationKeyPrivate};

use super::DELIMITER;
use super::credential_definition::CredentialDefinitionId;
use indy_api_types::domain::crypto::did::DidValue;

use std::collections::{HashMap, HashSet};
use named_type::NamedType;

use indy_api_types::validation::Validatable;
use indy_api_types::qualifier;

pub const CL_ACCUM: &str = "CL_ACCUM";
pub const REV_REG_DEG_MARKER: &str = "4";

use regex::Regex;

lazy_static! {
    static ref QUALIFIED_REV_REG_ID: Regex = Regex::new("(^revreg:(?P<method>[a-z0-9]+):)?(?P<did>.+):4:(?P<cred_def_id>.+):(?P<rev_reg_type>.+):(?P<tag>.+)$").unwrap();
}

#[derive(Deserialize, Debug, Serialize)]
pub struct RevocationRegistryConfig {
    pub issuance_type: Option<IssuanceType>,
    pub max_cred_num: Option<u32>
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum IssuanceType {
    ISSUANCE_BY_DEFAULT,
    ISSUANCE_ON_DEMAND
}

impl IssuanceType {
    pub fn to_bool(&self) -> bool {
        self.clone() == IssuanceType::ISSUANCE_BY_DEFAULT
    }
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub enum RegistryType {
    CL_ACCUM,
}

impl RegistryType {
    pub fn to_str(&self) -> &'static str {
        match *self {
            RegistryType::CL_ACCUM => CL_ACCUM
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: IssuanceType,
    pub max_cred_num: u32,
    pub public_keys: RevocationRegistryDefinitionValuePublicKeys,
    pub tails_hash: String,
    pub tails_location: String
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValuePublicKeys {
    pub accum_key: RevocationKeyPublic
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionV1 {
    pub id: RevocationRegistryId,
    pub revoc_def_type: RegistryType,
    pub tag: String,
    pub cred_def_id: CredentialDefinitionId,
    pub value: RevocationRegistryDefinitionValue
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
#[serde(tag = "ver")]
pub enum RevocationRegistryDefinition {
    #[serde(rename = "1.0")]
    RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1)
}

impl RevocationRegistryDefinition {
    pub fn to_unqualified(self) -> RevocationRegistryDefinition {
        match self {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_ref_def) => {
                RevocationRegistryDefinition::RevocationRegistryDefinitionV1(RevocationRegistryDefinitionV1 {
                    id: rev_ref_def.id.to_unqualified(),
                    revoc_def_type: rev_ref_def.revoc_def_type,
                    tag: rev_ref_def.tag,
                    cred_def_id: rev_ref_def.cred_def_id.to_unqualified(),
                    value: rev_ref_def.value,
                })
            }
        }
    }
}

impl From<RevocationRegistryDefinition> for RevocationRegistryDefinitionV1 {
    fn from(rev_reg_def: RevocationRegistryDefinition) -> Self {
        match rev_reg_def {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(rev_reg_def) => rev_reg_def
        }
    }
}

pub type RevocationRegistryDefinitions = HashMap<RevocationRegistryId, RevocationRegistryDefinition>;

pub fn rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs: RevocationRegistryDefinitions) -> HashMap<RevocationRegistryId, RevocationRegistryDefinitionV1> {
    rev_reg_defs
        .into_iter()
        .map(|(rev_reg_id, rev_reg_def)| (rev_reg_id, RevocationRegistryDefinitionV1::from(rev_reg_def)))
        .collect()
}

#[derive(Debug, Serialize, Deserialize, NamedType)]
pub struct RevocationRegistryDefinitionPrivate {
    pub value: RevocationKeyPrivate
}

#[derive(Debug, Deserialize, Serialize, Clone, NamedType)]
pub struct RevocationRegistryInfo {
    pub id: RevocationRegistryId,
    pub curr_id: u32,
    pub used_ids: HashSet<u32>
}

qualifiable_type!(RevocationRegistryId);

impl RevocationRegistryId {
    pub const PREFIX: &'static str = "revreg";

    pub fn new(did: &DidValue, cred_def_id: &CredentialDefinitionId, rev_reg_type: &str, tag: &str) -> RevocationRegistryId {
        let id = RevocationRegistryId(format!("{}{}{}{}{}{}{}{}{}", did.0, DELIMITER, REV_REG_DEG_MARKER, DELIMITER, cred_def_id.0, DELIMITER, rev_reg_type, DELIMITER, tag));
        match did.get_method() {
            Some(method) => RevocationRegistryId(qualifier::qualify(&id.0, Self::PREFIX, &method)),
            None => id
        }
    }

    pub fn parts(&self) -> Option<(DidValue, CredentialDefinitionId, String, String)> {
        match QUALIFIED_REV_REG_ID.captures(&self.0) {
            Some(caps) => {
                Some((DidValue(caps["did"].to_string()), CredentialDefinitionId(caps["cred_def_id"].to_string()), caps["rev_reg_type"].to_string(), caps["tag"].to_string()))
            }
            None => None
        }
    }

    pub fn to_unqualified(&self) -> RevocationRegistryId {
        match self.parts() {
            Some((did, cred_def_id, rev_reg_type, tag)) => RevocationRegistryId::new(&did.to_unqualified(), &cred_def_id.to_unqualified(), &rev_reg_type, &tag),
            None => self.clone()
        }
    }
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

impl Validatable for RevocationRegistryId {
    fn validate(&self) -> Result<(), String> {
        self.parts().ok_or(format!("Revocation Registry Id validation failed: {:?}, doesn't match pattern", self.0))?;
        Ok(())
    }
}

impl Validatable for RevocationRegistryDefinition {
    fn validate(&self) -> Result<(), String> {
        match self {
            RevocationRegistryDefinition::RevocationRegistryDefinitionV1(revoc_reg_def) => {
                revoc_reg_def.id.validate()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
