use std::collections::HashMap;
use std::fmt;
use ursa::cl::Nonce;

use indy_api_types::validation::Validatable;

use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;
use crate::utils::wql::Query;

use super::credential::Credential;
use indy_api_types::domain::crypto::did::DidValue;
use super::credential_definition::CredentialDefinitionId;
use super::revocation_registry_definition::RevocationRegistryId;
use super::schema::SchemaId;
use indy_api_types::qualifier;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProofRequestPayload {
    pub nonce: Nonce,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttributeInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub non_revoked: Option<NonRevocedInterval>
}

#[derive(Debug)]
pub enum ProofRequest {
    ProofRequestV1(ProofRequestPayload),
    ProofRequestV2(ProofRequestPayload),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ProofRequestsVersion {
    V1,
    V2,
}

impl ProofRequest {
    pub fn value<'a>(&'a self) -> &'a ProofRequestPayload {
        match self {
            ProofRequest::ProofRequestV1(proof_req) => proof_req,
            ProofRequest::ProofRequestV2(proof_req) => proof_req,
        }
    }

    pub fn version(&self) -> ProofRequestsVersion {
        match self {
            ProofRequest::ProofRequestV1(_) => ProofRequestsVersion::V1,
            ProofRequest::ProofRequestV2(_) => ProofRequestsVersion::V2,
        }
    }
}

impl<'de> Deserialize<'de> for ProofRequest
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct Helper {
            ver: Option<String>,
            nonce: String,
        }

        let v = Value::deserialize(deserializer)?;

        let helper = Helper::deserialize(&v).map_err(de::Error::custom)?;
        let nonce_cleaned = helper.nonce.replace(" ", "").replace("_", "");

        let proof_req = match helper.ver {
            Some(version) => {
                match version.as_ref() {
                    "1.0" => {
                        let proof_request = ProofRequestPayload::deserialize(v).map_err(de::Error::custom)?;
                        ProofRequest::ProofRequestV1(proof_request)
                    }
                    "2.0" => {
                        let proof_request = ProofRequestPayload::deserialize(v).map_err(de::Error::custom)?;
                        ProofRequest::ProofRequestV2(proof_request)
                    }
                    _ => return Err(de::Error::unknown_variant(&version, &["2.0"]))
                }
            }
            None => {
                let proof_request = ProofRequestPayload::deserialize(v).map_err(de::Error::custom)?;
                ProofRequest::ProofRequestV1(proof_request)
            }
        };
        let nonce_parsed = match &proof_req {
            ProofRequest::ProofRequestV1(payload) => payload.nonce.to_dec().map_err(de::Error::custom)?,
            ProofRequest::ProofRequestV2(payload) => payload.nonce.to_dec().map_err(de::Error::custom)?
        };
        if nonce_cleaned != nonce_parsed {
            Err(de::Error::custom(format!("Invalid nonce provided: {}", nonce_cleaned)))
        } else {
            Ok(proof_req)
        }
    }
}

impl Serialize for ProofRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ProofRequest::ProofRequestV1(proof_req) => {
                let mut value = ::serde_json::to_value(proof_req).map_err(ser::Error::custom)?;
                value.as_object_mut().unwrap().insert("ver".into(), json!("1.0"));
                value
            },
            ProofRequest::ProofRequestV2(proof_req) => {
                let mut value = ::serde_json::to_value(proof_req).map_err(ser::Error::custom)?;
                value.as_object_mut().unwrap().insert("ver".into(), json!("2.0"));
                value
            }
        };

        value.serialize(serializer)
    }
}

pub type ProofRequestExtraQuery = HashMap<String, Query>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NonRevocedInterval {
    pub from: Option<u64>,
    pub to: Option<u64>
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AttributeInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    pub restrictions: Option<Query>,
    pub non_revoked: Option<NonRevocedInterval>
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct PredicateInfo {
    pub name: String,
    pub p_type: PredicateTypes,
    pub p_value: i32,
    pub restrictions: Option<Query>,
    pub non_revoked: Option<NonRevocedInterval>
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum PredicateTypes {
    #[serde(rename = ">=")]
    GE,
    #[serde(rename = "<=")]
    LE,
    #[serde(rename = ">")]
    GT,
    #[serde(rename = "<")]
    LT
}

impl fmt::Display for PredicateTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PredicateTypes::GE => write!(f, "GE"),
            PredicateTypes::GT => write!(f, "GT"),
            PredicateTypes::LE => write!(f, "LE"),
            PredicateTypes::LT => write!(f, "LT")
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestedAttributeInfo {
    pub attr_referent: String,
    pub attr_info: AttributeInfo,
    pub revealed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestedPredicateInfo {
    pub predicate_referent: String,
    pub predicate_info: PredicateInfo
}

impl Validatable for ProofRequest {
    fn validate(&self) -> Result<(), String> {
        let value = self.value();
        let version = self.version();

        if value.requested_attributes.is_empty() && value.requested_predicates.is_empty() {
            return Err(String::from("Proof Request validation failed: both `requested_attributes` and `requested_predicates` are empty"));
        }

        for (_, requested_attribute) in value.requested_attributes.iter() {
            let has_name = !requested_attribute.name.as_ref().map(String::is_empty).unwrap_or(true);
            let has_names = !requested_attribute.names.as_ref().map(Vec::is_empty).unwrap_or(true);
            if  !has_name && !has_names {
                return Err(format!("Proof Request validation failed: there is empty requested attribute: {:?}", requested_attribute));
            }

            if has_name && has_names {
                return Err(format!("Proof request validation failed: there is a requested attribute with both name and names: {:?}", requested_attribute));
            }

            if let Some(ref restrictions) = requested_attribute.restrictions {
                _process_operator(&restrictions, &version)?;
            }
        }

        for (_, requested_predicate) in value.requested_predicates.iter() {
            if requested_predicate.name.is_empty() {
                return Err(format!("Proof Request validation failed: there is empty requested attribute: {:?}", requested_predicate));
            }
            if let Some(ref restrictions) = requested_predicate.restrictions {
                _process_operator(&restrictions, &version)?;
            }
        }

        Ok(())
    }
}

impl ProofRequest {
    pub fn to_unqualified(self) -> ProofRequest {
        let convert = |proof_request: &mut ProofRequestPayload| {
            for (_, requested_attribute) in proof_request.requested_attributes.iter_mut() {
                requested_attribute.restrictions = requested_attribute.restrictions.as_mut().map(|ref mut restrictions| _convert_query_to_unqualified(&restrictions));
            }
            for (_, requested_predicate) in proof_request.requested_predicates.iter_mut() {
                requested_predicate.restrictions = requested_predicate.restrictions.as_mut().map(|ref mut restrictions| _convert_query_to_unqualified(&restrictions));
            }
        };

        match self {
            ProofRequest::ProofRequestV2(mut proof_request) => {
                convert(&mut proof_request);
                ProofRequest::ProofRequestV2(proof_request)
            }
            ProofRequest::ProofRequestV1(mut proof_request) => {
                convert(&mut proof_request);
                ProofRequest::ProofRequestV1(proof_request)
            }
        }
    }
}

fn _convert_query_to_unqualified(query: &Query) -> Query {
    match query {
        Query::Eq(tag_name, ref tag_value) => { Query::Eq(tag_name.to_string(), _convert_value_to_unqualified(tag_name, tag_value)) }
        Query::Neq(ref tag_name, ref tag_value) => { Query::Neq(tag_name.to_string(), _convert_value_to_unqualified(tag_name, tag_value)) }
        Query::In(ref tag_name, ref tag_values) => {
            Query::In(tag_name.to_string(),
                      tag_values
                          .iter()
                          .map(|tag_value| _convert_value_to_unqualified(tag_name, tag_value))
                          .collect::<Vec<String>>()
            )
        }
        Query::And(ref queries) => {
            Query::And(
                queries
                    .iter()
                    .map(|query| _convert_query_to_unqualified(query))
                    .collect::<Vec<Query>>()
            )
        }
        Query::Or(ref queries) => {
            Query::Or(
                queries
                    .iter()
                    .map(|query| _convert_query_to_unqualified(query))
                    .collect::<Vec<Query>>()
            )
        }
        Query::Not(ref query) => { _convert_query_to_unqualified(query) }
        query => query.clone()
    }
}

fn _convert_value_to_unqualified(tag_name: &str, tag_value: &str) -> String {
    match tag_name {
        "issuer_did" | "schema_issuer_did" => DidValue(tag_value.to_string()).to_unqualified().0,
        "schema_id" => SchemaId(tag_value.to_string()).to_unqualified().0,
        "cred_def_id" => CredentialDefinitionId(tag_value.to_string()).to_unqualified().0,
        "rev_reg_id" => RevocationRegistryId(tag_value.to_string()).to_unqualified().0,
        _ => tag_value.to_string()
    }
}

fn _process_operator(restriction_op: &Query, version: &ProofRequestsVersion) -> Result<(), String> {
    match restriction_op {
        Query::Eq(ref tag_name, ref tag_value) |
        Query::Neq(ref tag_name, ref tag_value) |
        Query::Gt(ref tag_name, ref tag_value) |
        Query::Gte(ref tag_name, ref tag_value) |
        Query::Lt(ref tag_name, ref tag_value) |
        Query::Lte(ref tag_name, ref tag_value) |
        Query::Like(ref tag_name, ref tag_value) => {
            _check_restriction(tag_name, tag_value, version)
        }
        Query::In(ref tag_name, ref tag_values) => {
            tag_values
                .iter()
                .map(|tag_value| _check_restriction(tag_name, tag_value, version))
                .collect::<Result<Vec<()>, String>>()?;
            Ok(())
        }
        Query::And(ref operators) | Query::Or(ref operators) => {
            operators
                .iter()
                .map(|operator| _process_operator(operator, version))
                .collect::<Result<Vec<()>, String>>()?;
            Ok(())
        }
        Query::Not(ref operator) => {
            _process_operator(operator, version)
        }
    }
}

fn _check_restriction(tag_name: &str, tag_value: &str, version: &ProofRequestsVersion) -> Result<(), String> {
    if *version == ProofRequestsVersion::V1 &&
        Credential::QUALIFIABLE_TAGS.contains(&tag_name) &&
        qualifier::is_fully_qualified(tag_value) {
        return Err("Proof Request validation failed: fully qualified identifiers can not be used for Proof Request of the first version. \
                    Please, set \"ver\":\"2.0\" to use fully qualified identifiers.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod invalid_nonce {
        use super::*;

        #[test]
        fn proof_request_valid_nonce() {
            let proof_req_json = json!({
                "nonce": "123456",
                "name": "name",
                "version": "2.0",
                "requested_attributes": {},
                "requested_predicates": {},
            }).to_string();

            let proof_req: ProofRequest = serde_json::from_str(&proof_req_json).unwrap();
            let payload = match proof_req {
                ProofRequest::ProofRequestV1(p) => p,
                ProofRequest::ProofRequestV2(p) => p,
            };

            assert_eq!(payload.nonce.to_dec().unwrap(), "123456");
        }

        #[test]
        fn proof_request_invalid_nonce() {
            let proof_req_json = json!({
                "nonce": "123abc",
                "name": "name",
                "version": "2.0",
                "requested_attributes": {},
                "requested_predicates": {},
            }).to_string();

            serde_json::from_str::<ProofRequest>(&proof_req_json).unwrap_err();
        }
    }

    mod to_unqualified {
        use super::*;

        const DID_QUALIFIED: &str = "did:sov:NcYxiDXkpYi6ov5FcYDi1e";
        const DID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e";
        const SCHEMA_ID_QUALIFIED: &str = "schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
        const SCHEMA_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0";
        const CRED_DEF_ID_QUALIFIED: &str = "creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag";
        const CRED_DEF_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag";
        const REV_REG_ID_QUALIFIED: &str = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
        const REV_REG_ID_UNQUALIFIED: &str = "NcYxiDXkpYi6ov5FcYDi1e:4:NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";

        #[test]
        fn proof_request_to_unqualified() {
            let mut requested_attributes: HashMap<String, AttributeInfo> = HashMap::new();
            requested_attributes.insert("attr1_referent".to_string(), AttributeInfo {
                name: Some("name".to_string()),
                names: None,
                restrictions: Some(Query::And(vec![
                    Query::Eq("issuer_did".to_string(), DID_QUALIFIED.to_string()),
                    Query::Eq("schema_id".to_string(), SCHEMA_ID_QUALIFIED.to_string()),
                    Query::Eq("cred_def_id".to_string(), CRED_DEF_ID_QUALIFIED.to_string()),
                ])),
                non_revoked: None,
            });

            let mut requested_predicates: HashMap<String, PredicateInfo> = HashMap::new();
            requested_predicates.insert("predicate1_referent".to_string(), PredicateInfo {
                name: "age".to_string(),
                p_type: PredicateTypes::GE,
                p_value: 0,
                restrictions: Some(Query::And(vec![
                    Query::Eq("schema_issuer_did".to_string(), DID_QUALIFIED.to_string()),
                    Query::Eq("rev_reg_id".to_string(), REV_REG_ID_QUALIFIED.to_string()),
                ])),
                non_revoked: None,
            });

            let proof_request = ProofRequest::ProofRequestV2(ProofRequestPayload {
                nonce: Nonce::new().unwrap(),
                name: "proof_request_to_unqualified".to_string(),
                version: "1.0".to_string(),
                requested_attributes,
                requested_predicates,
                non_revoked: None,
            });

            let mut expected_requested_attributes: HashMap<String, AttributeInfo> = HashMap::new();
            expected_requested_attributes.insert("attr1_referent".to_string(), AttributeInfo {
                name: Some("name".to_string()),
                names: None,
                restrictions: Some(Query::And(vec![
                    Query::Eq("issuer_did".to_string(), DID_UNQUALIFIED.to_string()),
                    Query::Eq("schema_id".to_string(), SCHEMA_ID_UNQUALIFIED.to_string()),
                    Query::Eq("cred_def_id".to_string(), CRED_DEF_ID_UNQUALIFIED.to_string()),
                ])),
                non_revoked: None,
            });


            let mut expected_requested_predicates: HashMap<String, PredicateInfo> = HashMap::new();
            expected_requested_predicates.insert("predicate1_referent".to_string(), PredicateInfo {
                name: "age".to_string(),
                p_type: PredicateTypes::GE,
                p_value: 0,
                restrictions: Some(Query::And(vec![
                    Query::Eq("schema_issuer_did".to_string(), DID_UNQUALIFIED.to_string()),
                    Query::Eq("rev_reg_id".to_string(), REV_REG_ID_UNQUALIFIED.to_string()),
                ])),
                non_revoked: None,
            });

            let proof_request = proof_request.to_unqualified();
            assert_eq!(expected_requested_attributes, proof_request.value().requested_attributes);
            assert_eq!(expected_requested_predicates, proof_request.value().requested_predicates);
            assert_eq!(ProofRequestsVersion::V2, proof_request.version());
        }
    }
}