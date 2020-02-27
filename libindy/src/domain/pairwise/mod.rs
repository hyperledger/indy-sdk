use named_type::NamedType;
use indy_api_types::domain::crypto::did::DidValue;

#[derive(Serialize, Deserialize, NamedType)]
pub struct Pairwise {
    pub my_did: DidValue,
    pub their_did: DidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PairwiseInfo {
    pub my_did: DidValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

impl From<Pairwise> for PairwiseInfo {
    fn from(pairwise: Pairwise) -> Self {
        PairwiseInfo {
            my_did: pairwise.my_did,
            metadata: pairwise.metadata
        }
    }
}