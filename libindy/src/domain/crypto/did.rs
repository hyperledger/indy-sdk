use indy_api_types::validation::Validatable;
pub use indy_vdr::utils::validation::Validatable as VdrValidatable;
pub use indy_vdr::common::did::{DidValue, ShortDidValue, DidMethod};
use indy_vdr::common::error::VdrResultExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MyDidInfo {
    pub did: Option<DidValue>,
    pub seed: Option<String>,
    pub crypto_type: Option<String>,
    pub cid: Option<bool>,
    pub method_name: Option<DidMethod>,
}

impl Validatable for MyDidInfo {
    fn validate(&self) -> Result<(), String> {
        if let Some(ref did) = self.did {
            did.validate().map_err_string()?;
        }
        if let Some(ref name) = self.method_name {
            name.validate().map_err_string()?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TheirDidInfo {
    pub did: DidValue,
    pub verkey: Option<String>,
}

impl TheirDidInfo {
    pub fn new(did: DidValue, verkey: Option<String>) -> TheirDidInfo {
        TheirDidInfo {
            did,
            verkey,
        }
    }
}

impl Validatable for TheirDidInfo {
    fn validate(&self) -> Result<(), String> {
        self.did.validate().map_err_string()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Did {
    pub did: DidValue,
    pub verkey: String,
}

impl Did {
    pub fn new(did: DidValue, verkey: String) -> Did {
        Did {
            did,
            verkey,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidMetadata {
    pub value: String
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidWithMeta {
    pub did: DidValue,
    pub verkey: String,
    pub temp_verkey: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TheirDid {
    pub did: DidValue,
    pub verkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemporaryDid {
    pub did: DidValue,
    pub verkey: String,
}

impl From<TemporaryDid> for Did {
    fn from(temp_did: TemporaryDid) -> Self {
        Did {
            did: temp_did.did,
            verkey: temp_did.verkey,
        }
    }
}
