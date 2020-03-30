use serde_json;

use std::collections::HashMap;
use std::vec::Vec;

use messages::validation;
use error::prelude::*;
use utils::libindy::anoncreds;
use utils::qualifier;
use v3::messages::connection::service::Service;

static PROOF_REQUEST: &str = "PROOF_REQUEST";
static PROOF_DATA: &str = "proof_request_data";
pub const PROOF_REQUEST_V2: &str = "2.0";

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct ProofType {
    name: String,
    #[serde(rename = "version")]
    type_version: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd)]
struct ProofTopic {
    mid: u32,
    tid: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Restrictions {
    V1(Vec<Filter>),
    V2(::serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AttrInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Restrictions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_attest_allowed: Option<bool>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Filter {
    pub schema_id: Option<String>,
    pub schema_issuer_did: Option<String>,
    pub schema_name: Option<String>,
    pub schema_version: Option<String>,
    pub issuer_did: Option<String>,
    pub cred_def_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PredicateInfo {
    pub name: String,
    //Todo: Update p_type to use Enum
    pub p_type: String,
    pub p_value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Restrictions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_revoked: Option<NonRevokedInterval>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProofPredicates {
    predicates: Vec<PredicateInfo>
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NonRevokedInterval {
    pub from: Option<u64>,
    pub to: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProofRequestData {
    pub nonce: String,
    pub name: String,
    #[serde(rename = "version")]
    pub data_version: String,
    #[serde(default)]
    pub requested_attributes: HashMap<String, AttrInfo>,
    #[serde(default)]
    pub requested_predicates: HashMap<String, PredicateInfo>,
    pub non_revoked: Option<NonRevokedInterval>,
    pub ver: Option<ProofRequestVersion>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ProofRequestMessage {
    #[serde(rename = "@type")]
    type_header: ProofType,
    #[serde(rename = "@topic")]
    topic: ProofTopic,
    pub proof_request_data: ProofRequestData,
    pub msg_ref_id: Option<String>,
    from_timestamp: Option<u64>,
    to_timestamp: Option<u64>,
    pub thread_id: Option<String>,
    #[serde(rename = "~service")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

impl ProofPredicates {
    pub fn create() -> ProofPredicates {
        ProofPredicates {
            predicates: Vec::new()
        }
    }
}

impl ProofRequestMessage {
    pub fn create() -> ProofRequestMessage {
        ProofRequestMessage {
            type_header: ProofType {
                name: String::from(PROOF_REQUEST),
                type_version: String::new(),
            },
            topic: ProofTopic {
                tid: 0,
                mid: 0,
            },
            proof_request_data: ProofRequestData {
                nonce: String::new(),
                name: String::new(),
                data_version: String::new(),
                requested_attributes: HashMap::new(),
                requested_predicates: HashMap::new(),
                non_revoked: None,
                ver: None,
            },
            msg_ref_id: None,
            from_timestamp: None,
            to_timestamp: None,
            thread_id: None,
            service: None,
        }
    }

    pub fn type_version(&mut self, version: &str) -> VcxResult<&mut Self> {
        self.type_header.type_version = String::from(version);
        Ok(self)
    }

    pub fn tid(&mut self, tid: u32) -> VcxResult<&mut Self> {
        self.topic.tid = tid;
        Ok(self)
    }

    pub fn mid(&mut self, mid: u32) -> VcxResult<&mut Self> {
        self.topic.mid = mid;
        Ok(self)
    }

    pub fn nonce(&mut self, nonce: &str) -> VcxResult<&mut Self> {
        let nonce = validation::validate_nonce(nonce)?;
        self.proof_request_data.nonce = nonce;
        Ok(self)
    }

    pub fn proof_name(&mut self, name: &str) -> VcxResult<&mut Self> {
        self.proof_request_data.name = String::from(name);
        Ok(self)
    }

    pub fn proof_request_format_version(&mut self, version: Option<ProofRequestVersion>) -> VcxResult<&mut Self> {
        self.proof_request_data.ver = version;
        Ok(self)
    }

    pub fn proof_data_version(&mut self, version: &str) -> VcxResult<&mut Self> {
        self.proof_request_data.data_version = String::from(version);
        Ok(self)
    }


    pub fn requested_attrs(&mut self, attrs: &str) -> VcxResult<&mut Self> {
        let mut check_req_attrs: HashMap<String, AttrInfo> = HashMap::new();
        let proof_attrs: Vec<AttrInfo> = serde_json::from_str(attrs)
            .map_err(|err| {
                debug!("Cannot parse attributes: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse attributes: {}", err))
            })?;

        let mut index = 1;
        for mut attr in proof_attrs.into_iter() {
            let attr_name = match (attr.name.as_ref(), attr.names.as_ref()) {
                (Some(name), None) => { name.clone() }
                (None, Some(names)) => {
                    if names.is_empty(){
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest, "Proof Request validation failed: there is empty request attribute names"))
                    }
                    names.join(",")
                }
                (Some(_), Some(_)) => {
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                                  format!("Proof Request validation failed: there is empty requested attribute: {:?}", attrs)));
                }
                (None, None) => {
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                                  format!("Proof request validation failed: there is a requested attribute with both name and names: {:?}", attrs)));
                }
            };

            attr.restrictions = self.process_restrictions(attr.restrictions);

            if check_req_attrs.contains_key(&attr_name) {
                check_req_attrs.insert(format!("{}_{}", attr_name, index), attr);
            } else {
                check_req_attrs.insert(attr_name, attr);
            }
            index = index + 1;
        }
        self.proof_request_data.requested_attributes = check_req_attrs;
        Ok(self)
    }

    pub fn requested_predicates(&mut self, predicates: &str) -> VcxResult<&mut Self> {
        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_values: Vec<PredicateInfo> = serde_json::from_str(predicates)
            .map_err(|err| {
                debug!("Cannot parse predicates: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse predicates: {}", err))
            })?;

        let mut index = 1;
        for mut attr in attr_values.into_iter() {
            attr.restrictions = self.process_restrictions(attr.restrictions);

            if check_predicates.contains_key(&attr.name) {
                check_predicates.insert(format!("{}_{}", attr.name, index), attr);
            } else {
                check_predicates.insert(attr.name.clone(), attr);
            }
            index = index + 1;
        }

        self.proof_request_data.requested_predicates = check_predicates;
        Ok(self)
    }

    fn process_restrictions(&self, restrictions: Option<Restrictions>) -> Option<Restrictions> {
        match restrictions {
            Some(Restrictions::V2(restrictions)) => Some(Restrictions::V2(restrictions)),
            Some(Restrictions::V1(restrictions)) => {
                Some(Restrictions::V1(
                    restrictions
                        .into_iter()
                        .map(|filter| {
                            Filter {
                                schema_id: filter.schema_id.as_ref().and_then(|schema_id| anoncreds::libindy_to_unqualified(&schema_id).ok()),
                                schema_issuer_did: filter.schema_issuer_did.as_ref().and_then(|schema_issuer_did| anoncreds::libindy_to_unqualified(&schema_issuer_did).ok()),
                                schema_name: filter.schema_name,
                                schema_version: filter.schema_version,
                                issuer_did: filter.issuer_did.as_ref().and_then(|issuer_did| anoncreds::libindy_to_unqualified(&issuer_did).ok()),
                                cred_def_id: filter.cred_def_id.as_ref().and_then(|cred_def_id| anoncreds::libindy_to_unqualified(&cred_def_id).ok()),
                            }
                        })
                        .collect()
                ))
            }
            None => None
        }
    }

    pub fn from_timestamp(&mut self, from: Option<u64>) -> VcxResult<&mut Self> {
        self.from_timestamp = from;
        Ok(self)
    }

    pub fn to_timestamp(&mut self, to: Option<u64>) -> VcxResult<&mut Self> {
        self.to_timestamp = to;
        Ok(self)
    }

    pub fn set_proof_request_data(&mut self, proof_request_data: ProofRequestData) -> VcxResult<&mut Self> {
        self.proof_request_data = proof_request_data;
        Ok(self)
    }


    pub fn set_thread_id(&mut self, thid: String) -> VcxResult<&mut Self> {
        self.thread_id = Some(thid);
        Ok(self)
    }

    pub fn set_service(&mut self, service: Option<Service>) -> VcxResult<&mut Self> {
        self.service = service;
        Ok(self)
    }

    pub fn serialize_message(&mut self) -> VcxResult<String> {
        serde_json::to_string(self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize proof request: {}", err)))
    }

    pub fn get_proof_request_data(&self) -> String {
        json!(self)[PROOF_DATA].to_string()
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize proof request: {}", err)))
    }
}

impl ProofRequestData {
    const DEFAULT_VERSION: &'static str = "1.0";

    pub fn create() -> ProofRequestData {
        ProofRequestData::default()
    }

    pub fn set_name(mut self, name: String) -> ProofRequestData {
        self.name = name;
        self
    }

    pub fn set_version(mut self, version: String) -> ProofRequestData {
        self.data_version = version;
        self
    }

    pub fn set_format_version(mut self, version: ProofRequestVersion) -> ProofRequestData {
        self.ver = Some(version);
        self
    }

    pub fn set_nonce(mut self) -> VcxResult<ProofRequestData> {
        self.nonce = anoncreds::generate_nonce()?;
        Ok(self)
    }

    pub fn set_requested_attributes(mut self, requested_attrs: String) -> VcxResult<ProofRequestData> {
        let requested_attributes: Vec<AttrInfo> = ::serde_json::from_str(&requested_attrs)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Requested Attributes: {:?}, err: {:?}", requested_attrs, err)))?;

        self.requested_attributes = requested_attributes
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("attribute_{}", index), attribute))
            .collect();
        Ok(self)
    }

    pub fn set_requested_predicates(mut self, requested_predicates: String) -> VcxResult<ProofRequestData> {
        let requested_predicates: Vec<PredicateInfo> = ::serde_json::from_str(&requested_predicates)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Requested Attributes: {:?}, err: {:?}", requested_predicates, err)))?;

        self.requested_predicates = requested_predicates
            .into_iter()
            .enumerate()
            .map(|(index, attribute)| (format!("predicate_{}", index), attribute))
            .collect();
        Ok(self)
    }

    pub fn set_not_revoked_interval(mut self, non_revoc_interval: String) -> VcxResult<ProofRequestData> {
        let non_revoc_interval: NonRevokedInterval = ::serde_json::from_str(&non_revoc_interval)
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Revocation Interval: {:?}", non_revoc_interval)))?;

        self.non_revoked = match (non_revoc_interval.from, non_revoc_interval.to) {
            (None, None) => None,
            (from, to) => Some(NonRevokedInterval { from, to })
        };

        Ok(self)
    }

    pub fn set_format_version_for_did(mut self, my_did: &str, remote_did: &str) -> VcxResult<ProofRequestData> {
        if qualifier::is_fully_qualified(&my_did) && qualifier::is_fully_qualified(&remote_did) {
            self.ver = Some(ProofRequestVersion::V2)
        } else {
            let proof_request_json = serde_json::to_string(&self)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize ProofRequestData: {:?}", err)))?;

            let proof_request_json = anoncreds::libindy_to_unqualified(&proof_request_json)?;

            self = serde_json::from_str(&proof_request_json)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize ProofRequestData: {:?}", err)))?;

            self.ver = Some(ProofRequestVersion::V1)
        }
        Ok(self)
    }
}

impl Default for ProofRequestData {
    fn default() -> ProofRequestData {
        ProofRequestData {
            nonce: String::new(),
            name: String::new(),
            data_version: String::from(ProofRequestData::DEFAULT_VERSION),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None,
            ver: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProofRequestVersion {
    #[serde(rename = "1.0")]
    V1,
    #[serde(rename = "2.0")]
    V2,
}

impl Default for ProofRequestVersion {
    fn default() -> ProofRequestVersion {
        ProofRequestVersion::V1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messages::proof_request;
    use utils::constants::{REQUESTED_ATTRS, REQUESTED_PREDICATES};
    use utils::devsetup::SetupDefaults;

    #[test]
    fn test_create_proof_request_data() {
        let _setup = SetupDefaults::init();

        let request = proof_request();
        let proof_data = ProofRequestData {
            nonce: String::new(),
            name: String::new(),
            data_version: String::new(),
            requested_attributes: HashMap::new(),
            requested_predicates: HashMap::new(),
            non_revoked: None,
            ver: None,
        };
        assert_eq!(request.proof_request_data, proof_data);
    }

    #[test]
    fn test_proof_request_msg() {
        let _setup = SetupDefaults::init();

        //proof data
        let data_name = "Test";
        let nonce = "123432421212";
        let data_version = "3.75";
        let version = "1.3";
        let tid = 89;
        let mid = 98;

        let mut request = proof_request()
            .type_version(version).unwrap()
            .tid(tid).unwrap()
            .mid(mid).unwrap()
            .nonce(nonce).unwrap()
            .proof_request_format_version(Some(ProofRequestVersion::V2)).unwrap()
            .proof_name(data_name).unwrap()
            .proof_data_version(data_version).unwrap()
            .requested_attrs(REQUESTED_ATTRS).unwrap()
            .requested_predicates(REQUESTED_PREDICATES).unwrap()
            .to_timestamp(Some(100)).unwrap()
            .from_timestamp(Some(1)).unwrap()
            .clone();

        let serialized_msg = request.serialize_message().unwrap();
        assert!(serialized_msg.contains(r#""@type":{"name":"PROOF_REQUEST","version":"1.3"}"#));
        assert!(serialized_msg.contains(r#"@topic":{"mid":98,"tid":89}"#));
        assert!(serialized_msg.contains(r#"proof_request_data":{"nonce":"123432421212","name":"Test","version":"3.75","requested_attributes""#));

        assert!(serialized_msg.contains(r#""age":{"name":"age","restrictions":[{"schema_id":"6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11","schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB","schema_name":"Faber Student Info","schema_version":"1.0","issuer_did":"8XFh8yBzrpJQmNyZzgoTqB","cred_def_id":"8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766"},{"schema_id":"5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11","schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB","schema_name":"BYU Student Info","schema_version":"1.0","issuer_did":"66Fh8yBzrpJQmNyZzgoTqB","cred_def_id":"66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}"#));
        assert!(serialized_msg.contains(r#""to_timestamp":100"#));
        assert!(serialized_msg.contains(r#""from_timestamp":1"#));
        assert!(serialized_msg.contains(r#""ver":"2.0""#));
    }

    #[test]
    fn test_requested_attrs_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_req_attrs: HashMap<String, AttrInfo> = HashMap::new();
        let attr_info1: AttrInfo = serde_json::from_str(r#"{ "name":"age", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();
        let attr_info2: AttrInfo = serde_json::from_str(r#"{ "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();

        check_req_attrs.insert("age".to_string(), attr_info1);
        check_req_attrs.insert("name".to_string(), attr_info2);

        let request = proof_request().requested_attrs(REQUESTED_ATTRS).unwrap().clone();
        assert_eq!(request.proof_request_data.requested_attributes, check_req_attrs);
    }

    #[test]
    fn test_requested_predicates_constructed_correctly() {
        let _setup = SetupDefaults::init();

        let mut check_predicates: HashMap<String, PredicateInfo> = HashMap::new();
        let attr_info1: PredicateInfo = serde_json::from_str(r#"{ "name":"age","p_type":"GE","p_value":22, "restrictions":[ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }"#).unwrap();
        check_predicates.insert("age".to_string(), attr_info1);

        let request = proof_request().requested_predicates(REQUESTED_PREDICATES).unwrap().clone();
        assert_eq!(request.proof_request_data.requested_predicates, check_predicates);
    }

    #[test]
    fn test_requested_attrs_constructed_correctly_for_names() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({ "names":["name", "age", "email"], "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });
        let attr_info_2 = json!({ "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });

        let requested_attrs = json!([ attr_info, attr_info_2 ]).to_string();

        let request = proof_request().requested_attrs(&requested_attrs).unwrap().clone();

        let mut expected_req_attrs: HashMap<String, AttrInfo> = HashMap::new();
        expected_req_attrs.insert("name,age,email".to_string(), serde_json::from_value(attr_info).unwrap());
        expected_req_attrs.insert("name".to_string(), serde_json::from_value(attr_info_2).unwrap());

        assert_eq!(request.proof_request_data.requested_attributes, expected_req_attrs);
    }

    #[test]
    fn test_requested_attrs_constructed_correctly_for_name_and_names_passed_together() {
        let _setup = SetupDefaults::init();

        let attr_info = json!({ "name":"name", "names":["name", "age", "email"], "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11" } ] });

        let requested_attrs = json!([ attr_info ]).to_string();

        let err = proof_request().requested_attrs(&requested_attrs).unwrap_err();
        assert_eq!(VcxErrorKind::InvalidProofRequest, err.kind());
    }

    #[test]
    fn test_indy_proof_req_parses_correctly() {
        let _setup = SetupDefaults::init();

        let _proof_req: ProofRequestData = serde_json::from_str(::utils::constants::INDY_PROOF_REQ_JSON).unwrap();
    }
}