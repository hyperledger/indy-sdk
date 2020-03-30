use std::collections::HashMap;

use ursa::cl::{
    CredentialSignature,
    RevocationRegistry,
    SignatureCorrectnessProof,
    Witness
};

use indy_api_types::validation::Validatable;

use super::credential_definition::CredentialDefinitionId;
use super::revocation_registry_definition::RevocationRegistryId;
use super::schema::SchemaId;
use indy_vdr::utils::validation::Validatable as VdrValidatable;
use indy_vdr::common::error::VdrResultExt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Credential {
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryId>,
    pub values: CredentialValues,
    pub signature: CredentialSignature,
    pub signature_correctness_proof: SignatureCorrectnessProof,
    pub rev_reg: Option<RevocationRegistry>,
    pub witness: Option<Witness>
}

impl Credential {
    pub const QUALIFIABLE_TAGS: [&'static str; 5] = ["issuer_did", "cred_def_id", "schema_id", "schema_issuer_did", "rev_reg_id"];
    pub const EXTRA_TAG_SUFFIX: &'static str = "_short";

    pub fn add_extra_tag_suffix(tag: &str) -> String {
        format!("{}{}", tag, Self::EXTRA_TAG_SUFFIX)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialInfo {
    pub referent: String,
    pub attrs: ShortCredentialValues,
    pub schema_id: SchemaId,
    pub cred_def_id: CredentialDefinitionId,
    pub rev_reg_id: Option<RevocationRegistryId>,
    pub cred_rev_id: Option<String>
}

pub type ShortCredentialValues = HashMap<String, String>;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct CredentialValues(pub HashMap<String, AttributeValues>);

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AttributeValues {
    pub raw: String,
    pub encoded: String
}

impl Validatable for CredentialValues {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            return Err(String::from("CredentialValues validation failed: empty list has been passed"));
        }

        Ok(())
    }
}

impl Validatable for Credential {
    fn validate(&self) -> Result<(), String> {
        self.schema_id.validate().map_err_string()?;
        self.cred_def_id.validate().map_err_string()?;
        self.values.validate()?;

        if self.rev_reg_id.is_some() && (self.witness.is_none() || self.rev_reg.is_none()) {
            return Err(String::from("Credential validation failed: `witness` and `rev_reg` must be passed for revocable Credential"));
        }

        if self.values.0.is_empty() {
            return Err(String::from("Credential validation failed: `values` is empty"));
        }

        Ok(())
    }
}