use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use time;
use std::convert::TryInto;

use object_cache::ObjectCache;
use api::VcxStateType;
use error::prelude::*;

use connection;
use messages::{
    self,
    GeneralMessage,
    RemoteMessageType,
    payload::{
        Payloads,
        PayloadKinds,
    },
    thread::Thread,
    get_message::Message,
};
use messages::proofs::{
    proof_message::ProofMessage,
    proof_request::{
        ProofRequestMessage,
        ProofRequestData,
        NonRevokedInterval,
    },
};
use settings;
use utils::error;
use utils::constants::{CREDS_FROM_PROOF_REQ, DEFAULT_GENERATED_PROOF, DEFAULT_REJECTED_PROOF, NEW_PROOF_REQUEST_RESPONSE};
use utils::libindy::cache::{get_rev_reg_cache, set_rev_reg_cache, RevRegCache, RevState};
use utils::libindy::anoncreds;
use utils::libindy::anoncreds::{get_rev_reg_def_json, get_rev_reg_delta_json};

use v3::{
    messages::proof_presentation::presentation_request::PresentationRequest,
    handlers::proof_presentation::prover::prover::Prover,
};

use utils::agent_info::{get_agent_info, MyAgentInfo, get_agent_attr};
use utils::httpclient::AgencyMock;

lazy_static! {
    static ref HANDLE_MAP: ObjectCache<DisclosedProofs>  = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum DisclosedProofs {
    #[serde(rename = "3.0")]
    Pending(DisclosedProof),
    #[serde(rename = "1.0")]
    V1(DisclosedProof),
    #[serde(rename = "2.0")]
    V3(Prover),
}

impl Default for DisclosedProof {
    fn default() -> DisclosedProof
    {
        DisclosedProof {
            source_id: String::new(),
            state: VcxStateType::VcxStateNone,
            proof_request: None,
            proof: None,
            link_secret_alias: settings::DEFAULT_LINK_SECRET_ALIAS.to_string(),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
            thread: Some(Thread::new()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisclosedProof {
    source_id: String,
    state: VcxStateType,
    proof_request: Option<ProofRequestMessage>,
    proof: Option<ProofMessage>,
    link_secret_alias: String,
    my_did: Option<String>,
    my_vk: Option<String>,
    their_did: Option<String>,
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
    thread: Option<Thread>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestedCreds {
    pub self_attested_attributes: HashMap<String, String>,
    pub requested_attrs: HashMap<String, (String, bool)>,
    pub requested_predicates: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredInfo {
    pub requested_attr: String,
    pub referent: String,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_file: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn credential_def_identifiers(credentials: &str, proof_req: &ProofRequestData) -> VcxResult<Vec<CredInfo>> {
    let mut rtn = Vec::new();

    let credentials: Value = serde_json::from_str(credentials)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize credentials: {}", err)))?;

    if let Value::Object(ref attrs) = credentials["attrs"] {
        for (requested_attr, value) in attrs {
            if let (Some(referent), Some(schema_id), Some(cred_def_id)) =
            (value["credential"]["cred_info"]["referent"].as_str(),
             value["credential"]["cred_info"]["schema_id"].as_str(),
             value["credential"]["cred_info"]["cred_def_id"].as_str()) {
                let rev_reg_id = value["credential"]["cred_info"]["rev_reg_id"]
                    .as_str()
                    .map(|x| x.to_string());

                let cred_rev_id = value["credential"]["cred_info"]["cred_rev_id"]
                    .as_str()
                    .map(|x| x.to_string());

                let tails_file = value["tails_file"]
                    .as_str()
                    .map(|x| x.to_string());

                rtn.push(
                    CredInfo {
                        requested_attr: requested_attr.to_string(),
                        referent: referent.to_string(),
                        schema_id: schema_id.to_string(),
                        cred_def_id: cred_def_id.to_string(),
                        revocation_interval: _get_revocation_interval(&requested_attr, &proof_req)?,
                        timestamp: None,
                        rev_reg_id,
                        cred_rev_id,
                        tails_file,
                    }
                );
            } else { return Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, "Cannot get identifiers")); }
        }
    }

    Ok(rtn)
}

fn _get_revocation_interval(attr_name: &str, proof_req: &ProofRequestData) -> VcxResult<Option<NonRevokedInterval>> {
    if let Some(attr) = proof_req.requested_attributes.get(attr_name) {
        Ok(attr.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else if let Some(attr) = proof_req.requested_predicates.get(attr_name) {
        // Handle case for predicates
        Ok(attr.non_revoked.clone().or(proof_req.non_revoked.clone().or(None)))
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Attribute not found for: {}", attr_name)))
    }
}

// Also updates timestamp in credentials_identifiers
pub fn build_rev_states_json(credentials_identifiers: &mut Vec<CredInfo>) -> VcxResult<String> {
    let mut rtn: Value = json!({});
    let mut timestamps: HashMap<String, u64> = HashMap::new();

    for cred_info in credentials_identifiers.iter_mut() {
        if let (Some(rev_reg_id), Some(cred_rev_id), Some(tails_file)) =
        (&cred_info.rev_reg_id, &cred_info.cred_rev_id, &cred_info.tails_file) {
            if rtn.get(&rev_reg_id).is_none() {
                let (from, to) = if let Some(ref interval) = cred_info.revocation_interval
                { (interval.from, interval.to) } else { (None, None) };

                //                let from = from.unwrap_or(0);
                //                let to = to.unwrap_or(time::get_time().sec as u64);
                let cache = get_rev_reg_cache(&rev_reg_id);

                let (rev_state_json, timestamp) = if let Some(cached_rev_state) = cache.rev_state {
                    if cached_rev_state.timestamp >= from.unwrap_or(0)
                        && cached_rev_state.timestamp <= to.unwrap_or(time::get_time().sec as u64) {
                        (cached_rev_state.value, cached_rev_state.timestamp)
                    } else {
                        let from = match from {
                            Some(from) if from >= cached_rev_state.timestamp => {
                                Some(cached_rev_state.timestamp)
                            }
                            _ => None
                        };

                        let (_, rev_reg_def_json) = get_rev_reg_def_json(&rev_reg_id)?;

                        let (rev_reg_id, rev_reg_delta_json, timestamp) = get_rev_reg_delta_json(
                            &rev_reg_id,
                            from,
                            to,
                        )?;

                        let rev_state_json = anoncreds::libindy_prover_update_revocation_state(
                            &rev_reg_def_json,
                            &cached_rev_state.value,
                            &rev_reg_delta_json,
                            &cred_rev_id,
                            &tails_file,
                        )?;

                        if timestamp > cached_rev_state.timestamp {
                            let new_cache = RevRegCache {
                                rev_state: Some(RevState {
                                    timestamp,
                                    value: rev_state_json.clone(),
                                })
                            };
                            set_rev_reg_cache(&rev_reg_id, &new_cache);
                        }

                        (rev_state_json, timestamp)
                    }
                } else {
                    let (_, rev_reg_def_json) = get_rev_reg_def_json(&rev_reg_id)?;

                    let (rev_reg_id, rev_reg_delta_json, timestamp) = get_rev_reg_delta_json(
                        &rev_reg_id,
                        None,
                        to,
                    )?;

                    let rev_state_json = anoncreds::libindy_prover_create_revocation_state(
                        &rev_reg_def_json,
                        &rev_reg_delta_json,
                        &cred_rev_id,
                        &tails_file,
                    )?;

                    let new_cache = RevRegCache {
                        rev_state: Some(RevState {
                            timestamp,
                            value: rev_state_json.clone(),
                        })
                    };
                    set_rev_reg_cache(&rev_reg_id, &new_cache);

                    (rev_state_json, timestamp)
                };

                let rev_state_json: Value = serde_json::from_str(&rev_state_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RevocationState: {}", err)))?;

                // TODO: proover should be able to create multiple states of same revocation policy for different timestamps
                // see ticket IS-1108
                rtn[rev_reg_id.to_string()] = json!({timestamp.to_string(): rev_state_json});
                cred_info.timestamp = Some(timestamp);

                // Cache timestamp for future attributes that have the same rev_reg_id
                timestamps.insert(rev_reg_id.to_string(), timestamp);
            }

            // If the rev_reg_id is already in the map, timestamp may not be updated on cred_info
            if cred_info.timestamp.is_none() {
                cred_info.timestamp = timestamps.get(rev_reg_id).cloned();
            }
        }
    }

    Ok(rtn.to_string())
}

impl DisclosedProof {
    fn create_with_request(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProof> {
        trace!("create_with_request >>> source_id: {}, proof_req: {}", source_id, proof_req);

        let mut proof: DisclosedProof = Default::default();

        proof.set_source_id(source_id);
        proof.set_proof_request(proof_req)?;
        proof.set_state(VcxStateType::VcxStateRequestReceived);

        Ok(proof)
    }

    fn set_proof_request(&mut self, proof_req: &str) -> VcxResult<()> {
        let proof_req: ProofRequestMessage = serde_json::from_str(proof_req)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize proof request: {}", err)))?;
        self.proof_request = Some(proof_req);
        Ok(())
    }

    fn get_state(&self) -> u32 {
        trace!("DisclosedProof::get_state >>>");
        self.state as u32
    }
    fn set_state(&mut self, state: VcxStateType) {
        trace!("DisclosedProof::set_state >>> state: {:?}", state);
        self.state = state
    }

    fn retrieve_credentials(&self) -> VcxResult<String> {
        trace!("DisclosedProof::set_state >>>");
        if settings::indy_mocks_enabled() { return Ok(CREDS_FROM_PROOF_REQ.to_string()); }

        let proof_req = self.proof_request
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot get proot request"))?;

        let indy_proof_req = serde_json::to_string(&proof_req.proof_request_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize proof request: {}", err)))?;

        anoncreds::libindy_prover_get_credentials_for_proof_req(&indy_proof_req)
    }

    pub fn build_schemas_json(credentials_identifiers: &Vec<CredInfo>) -> VcxResult<String> {
        let mut rtn: Value = json!({});

        for ref cred_info in credentials_identifiers {
            if rtn.get(&cred_info.schema_id).is_none() {
                let (_, schema_json) = anoncreds::get_schema_json(&cred_info.schema_id)
                    .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

                let schema_json = serde_json::from_str(&schema_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot deserialize schema: {}", err)))?;

                rtn[cred_info.schema_id.to_owned()] = schema_json;
            }
        }
        Ok(rtn.to_string())
    }

    pub fn build_cred_def_json(credentials_identifiers: &Vec<CredInfo>) -> VcxResult<String> {
        let mut rtn: Value = json!({});

        for ref cred_info in credentials_identifiers {
            if rtn.get(&cred_info.cred_def_id).is_none() {
                let (_, credential_def) = anoncreds::get_cred_def_json(&cred_info.cred_def_id)
                    .map_err(|err| err.map(VcxErrorKind::InvalidProofCredentialData, "Cannot get credential definition"))?;

                let credential_def = serde_json::from_str(&credential_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Cannot deserialize credential definition: {}", err)))?;

                rtn[cred_info.cred_def_id.to_owned()] = credential_def;
            }
        }
        Ok(rtn.to_string())
    }

    pub fn build_requested_credentials_json(credentials_identifiers: &Vec<CredInfo>,
                                            self_attested_attrs: &str,
                                            proof_req: &ProofRequestData) -> VcxResult<String> {
        let mut rtn: Value = json!({
              "self_attested_attributes":{},
              "requested_attributes":{},
              "requested_predicates":{}
        });
        // do same for predicates and self_attested
        if let Value::Object(ref mut map) = rtn["requested_attributes"] {
            for ref cred_info in credentials_identifiers {
                if let Some(_) = proof_req.requested_attributes.get(&cred_info.requested_attr) {
                    let insert_val = json!({"cred_id": cred_info.referent, "revealed": true, "timestamp": cred_info.timestamp});
                    map.insert(cred_info.requested_attr.to_owned(), insert_val);
                }
            }
        }

        if let Value::Object(ref mut map) = rtn["requested_predicates"] {
            for ref cred_info in credentials_identifiers {
                if let Some(_) = proof_req.requested_predicates.get(&cred_info.requested_attr) {
                    let insert_val = json!({"cred_id": cred_info.referent, "timestamp": cred_info.timestamp});
                    map.insert(cred_info.requested_attr.to_owned(), insert_val);
                }
            }
        }

        // handle if the attribute is not revealed
        let self_attested_attrs: Value = serde_json::from_str(self_attested_attrs)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize self attested attributes: {}", err)))?;
        rtn["self_attested_attributes"] = self_attested_attrs;

        Ok(rtn.to_string())
    }

    fn generate_proof(&mut self, credentials: &str, self_attested_attrs: &str) -> VcxResult<u32> {
        trace!("DisclosedProof::generate_proof >>> credentials: {}, self_attested_attrs: {}", secret!(&credentials), secret!(&self_attested_attrs));

        debug!("generating proof {}", self.source_id);
        if settings::indy_mocks_enabled() { return Ok(error::SUCCESS.code_num); }

        let proof_req = self.proof_request.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::CreateProof, "Cannot get proof request"))?;

        let proof_req_data_json = serde_json::to_string(&proof_req.proof_request_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize proof request: {}", err)))?;

        let proof = DisclosedProof::generate_indy_proof(credentials, self_attested_attrs, &proof_req_data_json)?;

        let mut proof_msg = ProofMessage::new();
        proof_msg.libindy_proof = proof;
        self.proof = Some(proof_msg);

        Ok(error::SUCCESS.code_num)
    }

    pub fn generate_indy_proof(credentials: &str, self_attested_attrs: &str, proof_req_data_json: &str) -> VcxResult<String> {
        let proof_request: ProofRequestData = serde_json::from_str(&proof_req_data_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize proof request: {}", err)))?;

        let mut credentials_identifiers = credential_def_identifiers(credentials, &proof_request)?;

        let revoc_states_json = build_rev_states_json(&mut credentials_identifiers)?;
        let requested_credentials = DisclosedProof::build_requested_credentials_json(&credentials_identifiers,
                                                                                     self_attested_attrs,
                                                                                     &proof_request)?;

        let schemas_json = DisclosedProof::build_schemas_json(&credentials_identifiers)?;
        let credential_defs_json = DisclosedProof::build_cred_def_json(&credentials_identifiers)?;

        let proof = anoncreds::libindy_prover_create_proof(&proof_req_data_json,
                                                           &requested_credentials,
                                                           settings::DEFAULT_LINK_SECRET_ALIAS,
                                                           &schemas_json,
                                                           &credential_defs_json,
                                                           Some(&revoc_states_json))?;
        Ok(proof)
    }

    fn generate_proof_msg(&self) -> VcxResult<String> {
        let proof = match settings::indy_mocks_enabled() {
            false => {
                let proof: &ProofMessage = self.proof.as_ref().ok_or(VcxError::from(VcxErrorKind::CreateProof))?;
                serde_json::to_string(&proof)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize proof: {}", err)))?
            }
            true => DEFAULT_GENERATED_PROOF.to_string(),
        };

        Ok(proof)
    }

    fn _prep_proof_reference(&mut self, agent_info: &MyAgentInfo) -> VcxResult<String> {
        let proof_req = self.proof_request
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::CreateProof))?;

        let ref_msg_uid = proof_req.msg_ref_id
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::CreateProof))?;

        let their_did = get_agent_attr(&agent_info.their_pw_did)?;

        self.thread
            .as_mut()
            .map(|thread| thread.increment_receiver(&their_did));

        Ok(ref_msg_uid.to_string())
    }

    fn send_proof(&mut self, connection_handle: u32) -> VcxResult<u32> {
        trace!("DisclosedProof::send_proof >>> connection_handle: {}", connection_handle);

        debug!("sending proof {} via connection: {}",
               self.source_id, connection::get_source_id(connection_handle).unwrap_or_default()
        );

        let agent_info = get_agent_info()?.pw_info(connection_handle)?;
        apply_agent_info(self, &agent_info);

        let ref_msg_uid = self._prep_proof_reference(&agent_info)?;

        let proof = self.generate_proof_msg()?;

        messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Proof)?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .version(agent_info.version.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &proof,
                                PayloadKinds::Proof,
                                self.thread.clone()
            )
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::GeneralConnectionError,
                format!("Cannot encrypt payload: {}", err),
            ))?
            .ref_msg_id(Some(ref_msg_uid))?
            .send_secure()
            .map_err(|err| err.extend("Could not send proof"))?;

        self.state = VcxStateType::VcxStateAccepted;
        Ok(error::SUCCESS.code_num)
    }

    fn generate_reject_proof_msg(&self) -> VcxResult<String> {
        let msg = match settings::indy_mocks_enabled() {
            false => {
                let proof_reject = ProofMessage::new_reject();
                serde_json::to_string(&proof_reject)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize proof reject: {}", err)))?
            }
            true => DEFAULT_REJECTED_PROOF.to_string(),
        };

        Ok(msg)
    }

    fn reject_proof(&mut self, connection_handle: u32) -> VcxResult<u32> {
        trace!("DisclosedProof::reject_proof >>> connection_handle: {}", connection_handle);

        debug!("rejecting proof {} via connection: {}", self.source_id, connection::get_source_id(connection_handle).unwrap_or_default());
        // There feels like there's a much more rusty way to do the below.
        let agent_info = get_agent_info()?.pw_info(connection_handle)?;
        apply_agent_info(self, &agent_info);

        let ref_msg_uid = self._prep_proof_reference(&agent_info)?;

        let proof_reject = self.generate_reject_proof_msg()?;

        messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Proof)?
            .agent_did(&agent_info.agency_did)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .version(agent_info.version.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &proof_reject,
                                PayloadKinds::Proof,
                                self.thread.clone())
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::GeneralConnectionError,
                format!("Cannot encrypt payload: {}", err),
            ))?
            .ref_msg_id(Some(ref_msg_uid))?
            .send_secure()
            .map_err(|err| err.extend("Could not send proof reject"))?;

        self.state = VcxStateType::VcxStateRejected;
        return Ok(error::SUCCESS.code_num);
    }

    fn set_source_id(&mut self, id: &str) { self.source_id = id.to_string(); }

    fn get_source_id(&self) -> String { self.source_id.to_string() }

    #[cfg(test)] // TODO: REMOVE IT
    fn from_str(data: &str) -> VcxResult<DisclosedProof> {
        use messages::ObjectWithVersion;
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<DisclosedProof>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize DisclosedProof"))
    }
}

//********************************************
//         HANDLE FUNCTIONS
//********************************************
fn handle_err(err: VcxError) -> VcxError {
    if err.kind() == VcxErrorKind::InvalidHandle {
        VcxError::from(VcxErrorKind::InvalidDisclosedProofHandle)
    } else {
        err
    }
}

fn apply_agent_info(proof: &mut DisclosedProof, agent_info: &MyAgentInfo) {
    proof.my_did = agent_info.my_pw_did.clone();
    proof.my_vk = agent_info.my_pw_vk.clone();
    proof.their_did = agent_info.their_pw_did.clone();
    proof.their_vk = agent_info.their_pw_vk.clone();
    proof.agent_did = agent_info.pw_agent_did.clone();
    proof.agent_vk = agent_info.pw_agent_vk.clone();
}

fn create_proof_v3(source_id: &str, proof_req: &str) -> VcxResult<Option<DisclosedProofs>> {
    trace!("create_proof_v3 >>> source_id: {}, proof_req: {}", source_id, proof_req);

    // Received request of new format -- redirect to v3 folder
    if let Ok(presentation_request) = serde_json::from_str::<PresentationRequest>(proof_req) {
        let proof = Prover::create(source_id, presentation_request)?;
        return Ok(Some(DisclosedProofs::V3(proof)));
    }

    Ok(None)
}

fn create_pending_proof(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProofs> {
    trace!("create_pending_proof >>> source_id: {}, proof_req: {}", source_id, proof_req);

    let proof: DisclosedProof = DisclosedProof::create_with_request(source_id, proof_req)?;

    Ok(DisclosedProofs::Pending(proof))
}

fn create_proof_v1(source_id: &str, proof_req: &str) -> VcxResult<DisclosedProofs> {
    trace!("create_proof_v1 >>> source_id: {}, proof_req: {}", source_id, proof_req);

    let proof: DisclosedProof = DisclosedProof::create_with_request(source_id, proof_req)?;

    Ok(DisclosedProofs::V1(proof))
}

pub fn create_proof(source_id: &str, proof_req: &str) -> VcxResult<u32> {
    trace!("create_proof >>> source_id: {}, proof_req: {}", source_id, proof_req);

    debug!("creating disclosed proof with id: {}", source_id);

    // strict aries protocol is set. Presentation Request must be in aries format
    if settings::is_strict_aries_protocol_set() {
        let presentation_request: PresentationRequest = serde_json::from_str(proof_req)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Strict `aries` protocol is enabled. Can not parse `aries` formatted Presentation Request: {}", err)))?;

        let proof = Prover::create(source_id, presentation_request)?;
        return HANDLE_MAP.add(DisclosedProofs::V3(proof));
    }

    let proof =
        match create_proof_v3(source_id, &proof_req)? {
            Some(proof) => proof,
            None => {
                create_pending_proof(source_id, proof_req)?
            }
        };

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting proof {} into handle map", source_id);

    Ok(handle)
}

pub fn create_proof_with_msgid(source_id: &str, connection_handle: u32, msg_id: &str) -> VcxResult<(u32, String)> {
    let proof_request = get_proof_request(connection_handle, &msg_id)?;

    let proof = if connection::is_v3_connection(connection_handle)? {
        create_proof_v3(source_id, &proof_request)?
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("Connection can not be used for Proprietary Issuance protocol")))?
    } else {
        create_proof_v1(source_id, &proof_request)?
    };

    let handle = HANDLE_MAP.add(proof)?;

    debug!("inserting disclosed proof {} into handle map", source_id);
    Ok((handle, proof_request))
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(ref obj) => Ok(obj.get_state()),
            DisclosedProofs::V1(ref obj) => Ok(obj.get_state()),
            DisclosedProofs::V3(ref obj) => Ok(obj.state())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn update_state(handle: u32, message: Option<String>) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(obj) => {
                // update_state is just the same as get_state for disclosed_proof
                Ok(obj.get_state())
            }
            DisclosedProofs::V1(obj) => {
                // update_state is just the same as get_state for disclosed_proof
                Ok(obj.get_state())
            }
            DisclosedProofs::V3(ref mut obj) => {
                obj.update_state(message.as_ref().map(String::as_str))?;
                Ok(obj.state())
            }
        }
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |obj| {
        serde_json::to_string(obj)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize DisclosedProof object: {:?}", err)))
    })
}

pub fn from_string(proof_data: &str) -> VcxResult<u32> {
    let proof: DisclosedProofs = serde_json::from_str(proof_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("cannot deserialize DisclosedProofs object: {:?}", err)))?;

    HANDLE_MAP.add(proof)
}

pub fn release(handle: u32) -> VcxResult<()> {
    HANDLE_MAP.release(handle).map_err(handle_err)
}

pub fn release_all() {
    HANDLE_MAP.drain().ok();
}

pub fn generate_proof_msg(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(ref obj) => obj.generate_proof_msg(),
            DisclosedProofs::V1(ref obj) => obj.generate_proof_msg(),
            DisclosedProofs::V3(ref obj) => obj.generate_presentation_msg()
        }
    })
}

pub fn send_proof(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        let new_proof = match proof {
            DisclosedProofs::Pending(ref mut obj) => {
                // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                if ::connection::is_v3_connection(connection_handle)? {
                    let proof_request = obj.proof_request.clone()
                        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Can not get CredentialOffer of Credential object in Pending state"))?;

                    let proof = obj.proof.clone()
                        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Can not get proof message"))?;

                    let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                    prover.set_presentation(proof.try_into()?)?;
                    prover.send_presentation(connection_handle)?;

                    DisclosedProofs::V3(prover)
                } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                    obj.send_proof(connection_handle)?;
                    DisclosedProofs::V1(obj.clone())
                }
            }
            DisclosedProofs::V1(ref mut obj) => {
                obj.send_proof(connection_handle)?;
                DisclosedProofs::V1(obj.clone())
            }
            DisclosedProofs::V3(ref mut obj) => {
                obj.send_presentation(connection_handle)?;
                DisclosedProofs::V3(obj.clone())
            }
        };
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_reject_proof_msg(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(ref mut obj) => {
                obj.generate_reject_proof_msg()
            }
            DisclosedProofs::V1(ref mut obj) => {
                obj.generate_reject_proof_msg()
            }
            DisclosedProofs::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    })
}

pub fn reject_proof(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        let new_proof = match proof {
            DisclosedProofs::Pending(ref mut obj) => {
                // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                if ::connection::is_v3_connection(connection_handle)? {
                    let proof_request = obj.proof_request.clone()
                        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Can not get CredentialOffer of Credential object in Pending state"))?;

                    let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                    prover.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
                    DisclosedProofs::V3(prover)
                } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                    obj.reject_proof(connection_handle)?;
                    DisclosedProofs::V1(obj.clone())
                }
            }
            DisclosedProofs::V1(ref mut obj) => {
                obj.reject_proof(connection_handle)?;
                DisclosedProofs::V1(obj.clone())
            }
            DisclosedProofs::V3(ref mut obj) => {
                obj.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
                DisclosedProofs::V3(obj.clone())
            }
        };
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_proof(handle: u32, credentials: String, self_attested_attrs: String) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(ref mut obj) => {
                obj.generate_proof(&credentials, &self_attested_attrs)
            }
            DisclosedProofs::V1(ref mut obj) => {
                obj.generate_proof(&credentials, &self_attested_attrs)
            }
            DisclosedProofs::V3(ref mut obj) => {
                obj.generate_presentation(credentials.clone(), self_attested_attrs.clone())?;
                Ok(error::SUCCESS.code_num)
            }
        }
    }).map(|_| error::SUCCESS.code_num)
}

pub fn decline_presentation_request(handle: u32, connection_handle: u32, reason: Option<String>, proposal: Option<String>) -> VcxResult<u32> {
    HANDLE_MAP.get_mut(handle, |proof| {
        let new_proof = match proof {
            DisclosedProofs::Pending(ref mut obj) => {
                // if Aries connection is established --> Convert DisclosedProofs object to Aries presentation
                if ::connection::is_v3_connection(connection_handle)? {
                    let proof_request = obj.proof_request.clone()
                        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "Can not get CredentialOffer of Credential object in Pending state"))?;

                    let mut prover = Prover::create(&obj.get_source_id(), proof_request.try_into()?)?;
                    prover.decline_presentation_request(connection_handle, Some(String::from("Presentation Request was rejected")), None)?;
                    DisclosedProofs::V3(prover)
                } else { // else --> Convert DisclosedProofs object to Proprietary proof object
                    obj.reject_proof(connection_handle)?;
                    DisclosedProofs::V1(obj.clone())
                }
            }
            DisclosedProofs::V1(ref mut obj) => {
                obj.reject_proof(connection_handle)?;
                DisclosedProofs::V1(obj.clone())
            }
            DisclosedProofs::V3(ref mut obj) => {
                obj.decline_presentation_request(connection_handle, reason.clone(), proposal.clone())?;
                DisclosedProofs::V3(obj.clone())
            }
        };
        *proof = new_proof;
        Ok(error::SUCCESS.code_num)
    }).map(|_| error::SUCCESS.code_num)
}

pub fn retrieve_credentials(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get_mut(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(ref obj) => obj.retrieve_credentials(),
            DisclosedProofs::V1(ref obj) => obj.retrieve_credentials(),
            DisclosedProofs::V3(ref obj) => obj.retrieve_credentials()
        }
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    HANDLE_MAP.has_handle(handle)
}

//TODO one function with credential
fn get_proof_request(connection_handle: u32, msg_id: &str) -> VcxResult<String> {
    if connection::is_v3_connection(connection_handle)? {
        let presentation_request = Prover::get_presentation_request(connection_handle, msg_id)?;
        return serde_json::to_string_pretty(&presentation_request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize message: {}", err)));
    }

    trace!("get_proof_request >>> connection_handle: {}, msg_id: {}", connection_handle, msg_id);

    let agent_info = get_agent_info()?.pw_info(connection_handle)?;

    AgencyMock::set_next_response(NEW_PROOF_REQUEST_RESPONSE.to_vec());

    let message = messages::get_message::get_connection_messages(&agent_info.my_pw_did()?,
                                                                 &agent_info.my_pw_vk()?,
                                                                 &agent_info.pw_agent_did()?,
                                                                 &agent_info.pw_agent_vk()?,
                                                                 Some(vec![msg_id.to_string()]),
                                                                 None,
                                                                 &agent_info.version()?)?;

    if message[0].msg_type == RemoteMessageType::ProofReq {
        let request = _parse_proof_req_message(&message[0], &agent_info.my_pw_vk()?)?;

        serde_json::to_string_pretty(&request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize message: {}", err)))
    } else {
        Err(VcxError::from_msg(VcxErrorKind::InvalidMessages, "Message has different type"))
    }
}

//TODO one function with credential
pub fn get_proof_request_messages(connection_handle: u32, match_name: Option<&str>) -> VcxResult<String> {
    if connection::is_v3_connection(connection_handle)? {
        let presentation_requests = Prover::get_presentation_request_messages(connection_handle, match_name)?;

        // strict aries protocol is set. return aries formatted Proof Request.
        if settings::is_strict_aries_protocol_set() {
            return Ok(json!(presentation_requests).to_string());
        }

        let msgs: Vec<ProofRequestMessage> = presentation_requests
            .into_iter()
            .map(|presentation_request| presentation_request.try_into())
            .collect::<VcxResult<Vec<ProofRequestMessage>>>()?;

        return serde_json::to_string(&msgs).
            map_err(|err| {
                VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize ProofRequestMessage: {:?}", err))
            });
    }

    trace!("get_proof_request_messages >>> connection_handle: {}, match_name: {:?}", connection_handle, match_name);

    let agent_info = get_agent_info()?.pw_info(connection_handle)?;

    AgencyMock::set_next_response(NEW_PROOF_REQUEST_RESPONSE.to_vec());

    let payload = messages::get_message::get_connection_messages(&agent_info.my_pw_did()?,
                                                                 &agent_info.my_pw_vk()?,
                                                                 &agent_info.pw_agent_did()?,
                                                                 &agent_info.pw_agent_vk()?,
                                                                 None,
                                                                 None,
                                                                 &agent_info.version()?)?;

    let mut messages: Vec<ProofRequestMessage> = Default::default();

    for msg in payload {
        if msg.sender_did.eq(&agent_info.my_pw_did()?) { continue; }

        if msg.msg_type == RemoteMessageType::ProofReq {
            let req = _parse_proof_req_message(&msg, &agent_info.my_pw_vk()?)?;
            messages.push(req);
        }
    }

    serde_json::to_string_pretty(&messages)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidJson, format!("Cannot serialize proof request: {}", err),
        ))
}

fn _parse_proof_req_message(message: &Message, my_vk: &str) -> VcxResult<ProofRequestMessage> {
    let payload = message.payload.as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Cannot get payload"))?;

    let (request, thread) = Payloads::decrypt(&my_vk, payload)?;

    let mut request: ProofRequestMessage = serde_json::from_str(&request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, format!("Cannot deserialize proof request: {}", err)))?;

    request.msg_ref_id = Some(message.uid.to_owned());
    request.thread_id = thread.and_then(|tr| tr.thid.clone());

    Ok(request)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    HANDLE_MAP.get(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(obj) => Ok(obj.get_source_id()),
            DisclosedProofs::V1(obj) => Ok(obj.get_source_id()),
            DisclosedProofs::V3(ref obj) => Ok(obj.get_source_id())
        }
    }).map_err(handle_err)
}

pub fn get_presentation_status(handle: u32) -> VcxResult<u32> {
    HANDLE_MAP.get(handle, |obj| {
        match obj {
            DisclosedProofs::Pending(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidDisclosedProofHandle, "Cannot get presentation status for V1 DisclosedProof object")),
            DisclosedProofs::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidDisclosedProofHandle, "Cannot get presentation status for V1 DisclosedProof object")),
            DisclosedProofs::V3(ref obj) => Ok(obj.presentation_status())
        }
    })
}

#[cfg(test)]
mod tests {
    extern crate serde_json;

    use super::*;
    use serde_json::Value;
    use utils::{
        constants::{ADDRESS_CRED_ID, LICENCE_CRED_ID, ADDRESS_SCHEMA_ID,
                    ADDRESS_CRED_DEF_ID, CRED_DEF_ID, SCHEMA_ID, ADDRESS_CRED_REV_ID,
                    ADDRESS_REV_REG_ID, REV_REG_ID, CRED_REV_ID, TEST_TAILS_FILE, REV_STATE_JSON},
        get_temp_dir_path,
    };
    #[cfg(feature = "pool_tests")]
    use time;
    use utils::devsetup::*;

    fn proof_req_no_interval() -> ProofRequestData {
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": { "name": "address1" },
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height" }
            },
            "requested_predicates": {},
        }).to_string();

        serde_json::from_str(&proof_req).unwrap()
    }

    fn _get_proof_request_messages(connection_h: u32) -> String {
        let requests = get_proof_request_messages(connection_h, None).unwrap();
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = serde_json::to_string(&requests[0]).unwrap();
        requests
    }

    #[test]
    fn test_create_proof() {
        let _setup = SetupMocks::init();

        assert!(create_proof("1", ::utils::constants::PROOF_REQUEST_JSON).unwrap() > 0);
    }

    #[test]
    fn test_create_fails() {
        let _setup = SetupMocks::init();

        assert_eq!(create_proof("1", "{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_proof_cycle() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        send_proof(handle, connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(handle).unwrap());
    }

    #[test]
    fn test_proof_reject_cycle() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = _get_proof_request_messages(connection_h);

        let handle = create_proof("TEST_CREDENTIAL", &request).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap());

        reject_proof(handle, connection_h).unwrap();
        assert_eq!(VcxStateType::VcxStateRejected as u32, get_state(handle).unwrap());
    }

    #[test]
    fn get_state_test() {
        let _setup = SetupMocks::init();

        let proof: DisclosedProof = Default::default();
        assert_eq!(VcxStateType::VcxStateNone as u32, proof.get_state());

        let handle = create_proof("id", ::utils::constants::PROOF_REQUEST_JSON).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(handle).unwrap())
    }

    #[test]
    fn to_string_test() {
        let _setup = SetupMocks::init();

        let handle = create_proof("id", ::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let serialized = to_string(handle).unwrap();
        let j: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(j["version"], ::utils::constants::PENDING_OBJECT_SERIALIZE_VERSION);

        let handle_2 = from_string(&serialized).unwrap();
        assert_ne!(handle, handle_2);
    }

    #[test]
    fn test_deserialize_fails() {
        let _setup = SetupDefaults::init();

        assert_eq!(from_string("{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_deserialize_succeeds_with_self_attest_allowed() {
        let _setup = SetupDefaults::init();

        let handle = create_proof("id", ::utils::constants::PROOF_REQUEST_JSON).unwrap();

        let serialized = to_string(handle).unwrap();
        let p = DisclosedProof::from_str(&serialized).unwrap();
        assert_eq!(p.proof_request.unwrap().proof_request_data.requested_attributes.get("attr1_referent").unwrap().self_attest_allowed, Some(true))
    }

    #[test]
    fn test_find_schemas() {
        let _setup = SetupMocks::init();

        assert_eq!(DisclosedProof::build_schemas_json(&Vec::new()).unwrap(), "{}".to_string());

        let cred1 = CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let schemas = DisclosedProof::build_schemas_json(&creds).unwrap();
        assert!(schemas.len() > 0);
        assert!(schemas.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:2:test-licence:4.4.4","name":"test-licence""#));
    }

    #[test]
    fn test_find_schemas_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![CredInfo {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(DisclosedProof::build_schemas_json(&credential_ids).unwrap_err().kind(), VcxErrorKind::InvalidSchema);
    }

    #[test]
    fn test_find_credential_def() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let cred2 = CredInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        };
        let creds = vec![cred1, cred2];

        let credential_def = DisclosedProof::build_cred_def_json(&creds).unwrap();
        assert!(credential_def.len() > 0);
        assert!(credential_def.contains(r#""id":"2hoqvcwupRTUNkXn6ArYzs:3:CL:2471","schemaId":"2471""#));
    }

    #[test]
    fn test_find_credential_def_fails() {
        let _setup = SetupLibraryWallet::init();

        let credential_ids = vec![CredInfo {
            requested_attr: "1".to_string(),
            referent: "2".to_string(),
            schema_id: "3".to_string(),
            cred_def_id: "3".to_string(),
            rev_reg_id: Some("4".to_string()),
            cred_rev_id: Some("5".to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: None,
        }];
        assert_eq!(DisclosedProof::build_cred_def_json(&credential_ids).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);
    }

    #[test]
    fn test_build_requested_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let cred2 = CredInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: None,
            timestamp: Some(800),
        };
        let creds = vec![cred1, cred2];
        let self_attested_attrs = json!({
            "self_attested_attr_3": "my self attested 1",
            "self_attested_attr_4": "my self attested 2",
        }).to_string();

        let test: Value = json!({
              "self_attested_attributes":{
                  "self_attested_attr_3": "my self attested 1",
                  "self_attested_attr_4": "my self attested 2",
              },
              "requested_attributes":{
                  "height_1": {"cred_id": LICENCE_CRED_ID, "revealed": true, "timestamp": 800},
                  "zip_2": {"cred_id": ADDRESS_CRED_ID, "revealed": true, "timestamp": 800},
              },
              "requested_predicates":{}
        });

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "height_1": {
                    "name": "height_1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip_2" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequestData = serde_json::from_value(proof_req).unwrap();
        let requested_credential = DisclosedProof::build_requested_credentials_json(&creds, &self_attested_attrs, &proof_req).unwrap();
        assert_eq!(test.to_string(), requested_credential);
    }

    #[test]
    fn test_get_proof_request() {
        let _setup = SetupMocks::init();

        let connection_h = connection::tests::build_test_connection();

        let request = get_proof_request(connection_h, "123").unwrap();
        let _request: ProofRequestMessage = serde_json::from_str(&request).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_retrieve_credentials() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let (_, _, req, _) = ::utils::libindy::anoncreds::tests::create_proof();

        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req).unwrap();
        proof.proof_request = Some(proof_req);

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.len() > 500);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_retrieve_credentials_emtpy() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({}),
           "requested_predicates": json!({}),
        });
        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());

        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, "{}".to_string());

        req["requested_attributes"]["address1_1"] = json!({"name": "address1"});
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req);
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert_eq!(retrieved_creds, json!({"attrs":{"address1_1":[]}}).to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_case_for_proof_req_doesnt_matter_for_retrieve_creds() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let mut req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "zip_1": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               })
           }),
           "requested_predicates": json!({}),
        });

        let mut proof_req = ProofRequestMessage::create();
        let mut proof: DisclosedProof = Default::default();
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());

        // All lower case
        let retrieved_creds = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds.contains(r#""zip":"84000""#));
        let ret_creds_as_value: Value = serde_json::from_str(&retrieved_creds).unwrap();
        assert_eq!(ret_creds_as_value["attrs"]["zip_1"][0]["cred_info"]["attrs"]["zip"], "84000");
        // First letter upper
        req["requested_attributes"]["zip_1"]["name"] = json!("Zip");
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());
        let retrieved_creds2 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds2.contains(r#""zip":"84000""#));

        //entire word upper
        req["requested_attributes"]["zip_1"]["name"] = json!("ZIP");
        proof_req.proof_request_data = serde_json::from_str(&req.to_string()).unwrap();
        proof.proof_request = Some(proof_req.clone());
        let retrieved_creds3 = proof.retrieve_credentials().unwrap();
        assert!(retrieved_creds3.contains(r#""zip":"84000""#));
    }

    #[test]
    fn test_retrieve_credentials_fails_with_no_proof_req() {
        let _setup = SetupLibraryWallet::init();

        let proof: DisclosedProof = Default::default();
        assert_eq!(proof.retrieve_credentials().unwrap_err().kind(), VcxErrorKind::NotReady);
    }

    #[test]
    fn test_credential_def_identifiers() {
        let _setup = SetupDefaults::init();

        let cred1 = CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: Some(NonRevokedInterval { from: Some(123), to: Some(456) }),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        };
        let cred2 = CredInfo {
            requested_attr: "zip_2".to_string(),
            referent: ADDRESS_CRED_ID.to_string(),
            schema_id: ADDRESS_SCHEMA_ID.to_string(),
            cred_def_id: ADDRESS_CRED_DEF_ID.to_string(),
            rev_reg_id: Some(ADDRESS_REV_REG_ID.to_string()),
            cred_rev_id: Some(ADDRESS_CRED_REV_ID.to_string()),
            revocation_interval: Some(NonRevokedInterval { from: None, to: Some(987) }),
            tails_file: None,
            timestamp: None,
        };
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
              "zip_2":{
                "credential": {
                    "cred_info":{
                       "referent":ADDRESS_CRED_ID,
                       "attrs":{
                          "address1":"101 Tela Lane",
                          "address2":"101 Wilson Lane",
                          "zip":"87121",
                          "state":"UT",
                          "city":"SLC"
                       },
                       "schema_id":ADDRESS_SCHEMA_ID,
                       "cred_def_id":ADDRESS_CRED_DEF_ID,
                       "rev_reg_id":ADDRESS_REV_REG_ID,
                       "cred_rev_id":ADDRESS_CRED_REV_ID
                    },
                    "interval":null
                },
             }
           },
           "predicates":{ }
        });
        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "zip_2": { "name": "zip" },
                "height_1": { "name": "height", "non_revoked": {"from": 123, "to": 456} }
            },
            "requested_predicates": {},
            "non_revoked": {"to": 987}
        }).to_string();

        let creds = credential_def_identifiers(&selected_credentials.to_string(), &serde_json::from_str(&proof_req).unwrap()).unwrap();
        assert_eq!(creds, vec![cred1, cred2]);
    }

    #[test]
    fn test_credential_def_identifiers_failure() {
        let _setup = SetupDefaults::init();

        // selected credentials has incorrect json
        assert_eq!(credential_def_identifiers("", &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidJson);


        // No Creds
        assert_eq!(credential_def_identifiers("{}", &proof_req_no_interval()).unwrap(), Vec::new());
        assert_eq!(credential_def_identifiers(r#"{"attrs":{}}"#, &proof_req_no_interval()).unwrap(), Vec::new());

        // missing cred info
        let selected_credentials: Value = json!({
           "attrs":{
              "height_1":{ "interval":null }
           },
           "predicates":{

           }
        });
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Optional Revocation
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "schema_id": SCHEMA_ID,
                       "cred_def_id": CRED_DEF_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string(),
              },
           },
           "predicates":{ }
        });
        let creds = vec![CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            revocation_interval: None,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            timestamp: None,
        }];
        assert_eq!(&credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // rev_reg_id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["rev_reg_id"] = serde_json::Value::Null;
        assert_eq!(&credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap(), &creds);

        // Missing schema ID
        let mut selected_credentials: Value = json!({
           "attrs":{
              "height_1":{
                "credential": {
                    "cred_info":{
                       "referent":LICENCE_CRED_ID,
                       "attrs":{
                          "sex":"male",
                          "age":"111",
                          "name":"Bob",
                          "height":"4'11"
                       },
                       "cred_def_id": CRED_DEF_ID,
                       "rev_reg_id":REV_REG_ID,
                       "cred_rev_id":CRED_REV_ID
                    },
                    "interval":null
                },
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // Schema Id is null
        selected_credentials["attrs"]["height_1"]["cred_info"]["schema_id"] = serde_json::Value::Null;
        assert_eq!(credential_def_identifiers(&selected_credentials.to_string(), &proof_req_no_interval()).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let mut proof_req = ProofRequestMessage::create();
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": to}
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();

        let all_creds: Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ }
        });

        let self_attested: Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_self_attested_proof() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut proof_req = ProofRequestMessage::create();
        let indy_proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let selected_credentials: Value = json!({});

        let self_attested: Value = json!({
              "address1_1":"attested_address",
              "zip_2": "attested_zip"
        });

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();
        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());

        assert!(generated_proof.is_ok());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_proof_with_predicates() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
        let mut proof_req = ProofRequestMessage::create();
        let to = time::get_time().sec;
        let indy_proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "restrictions": [{"issuer_did": did}],
                    "non_revoked":  {"from": 123, "to": to}
                },
                "zip_2": { "name": "zip" }
            },
            "self_attested_attr_3": json!({
                   "name":"self_attested_attr",
             }),
            "requested_predicates": json!({
                "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
            }),
            "non_revoked": {"from": 098, "to": to}
        }).to_string();
        proof_req.proof_request_data = serde_json::from_str(&indy_proof_req).unwrap();

        let mut proof: DisclosedProof = Default::default();
        proof.proof_request = Some(proof_req);
        proof.link_secret_alias = "main".to_string();

        let all_creds: Value = serde_json::from_str(&proof.retrieve_credentials().unwrap()).unwrap();
        let selected_credentials: Value = json!({
           "attrs":{
              "address1_1": {
                "credential": all_creds["attrs"]["address1_1"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
              "zip_2": {
                "credential": all_creds["attrs"]["zip_2"][0],
                "tails_file": get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()
              },
           },
           "predicates":{ 
               "zip_3": {
                "credential": all_creds["attrs"]["zip_3"][0],
               }
           }
        });

        let self_attested: Value = json!({
              "self_attested_attr_3":"attested_val"
        });

        let generated_proof = proof.generate_proof(&selected_credentials.to_string(), &self_attested.to_string());
        assert!(generated_proof.is_ok());
    }

    #[test]
    fn test_generate_reject_proof() {
        let _setup = SetupMocks::init();

        let proof: DisclosedProof = Default::default();
        let generated_reject = proof.generate_reject_proof_msg();
        assert!(generated_reject.is_ok());
    }

    #[test]
    fn test_build_rev_states_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfo {
            requested_attr: "height".to_string(),
            referent: "abc".to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let mut cred_info = vec![cred1];
        let states = build_rev_states_json(cred_info.as_mut()).unwrap();
        let rev_state_json: Value = serde_json::from_str(REV_STATE_JSON).unwrap();
        let expected = json!({REV_REG_ID: {"1": rev_state_json}}).to_string();
        assert_eq!(states, expected);
        assert!(cred_info[0].timestamp.is_some());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_empty() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        // empty vector
        assert_eq!(build_rev_states_json(Vec::new().as_mut()).unwrap(), "{}".to_string());

        // no rev_reg_id
        let cred1 = CredInfo {
            requested_attr: "height_1".to_string(),
            referent: LICENCE_CRED_ID.to_string(),
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            cred_rev_id: Some(CRED_REV_ID.to_string()),
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        assert_eq!(build_rev_states_json(vec![cred1].as_mut()).unwrap(), "{}".to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_no_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = CredInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        // assert cache is empty
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache.rev_state, None);

        let states = build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache = get_rev_reg_cache(&rev_reg_id);
        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_cached() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_rev_state = "{\"some\": \"json\"}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = CredInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: None,
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: current_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is unchanged.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_with_older_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_timestamp = current_timestamp - 100;
        let cached_rev_state = "{\"witness\":{\"omega\":\"2 0BB3DE371F14384496D1F4FEB47B86A935C858BC21033B16251442FCBC5370A1 2 026F2848F2972B74079BEE16CDA9D48AD2FF7C7E39087515CB9B6E9B38D73BCB 2 10C48056D8C226141A8D7030E9FA17B7F02A39B414B9B64B6AECDDA5AFD1E538 2 11DCECD73A8FA6CFCD0468C659C2F845A9215842B69BA10355C1F4BF2D9A9557 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"rev_reg\":{\"accum\":\"2 033C0E6FAC660DF3582EF46021FAFDD93E111D1DC9DA59C4EA9B92BB21F8E0A4 2 02E0F749312228A93CF67BB5F86CA263FAE535A0F1CA449237D736939518EFF0 2 19BB82474D0BD0A1DDE72D377C8A965D6393071118B79D4220D4C9B93D090314 2 1895AAFD8050A8FAE4A93770C6C82881AB13134EE082C64CF6A7A379B3F6B217 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"timestamp\":100}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = CredInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: Some(NonRevokedInterval { from: Some(cached_timestamp + 1), to: None }),
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: cached_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is updated.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_ne!(cache, cached_data);

        // check if this value is in cache now.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_eq!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_eq!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_rev_states_json_real_with_newer_cache() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let current_timestamp = time::get_time().sec as u64;
        let cached_timestamp = current_timestamp + 100;
        let cached_rev_state = "{\"witness\":{\"omega\":\"2 0BB3DE371F14384496D1F4FEB47B86A935C858BC21033B16251442FCBC5370A1 2 026F2848F2972B74079BEE16CDA9D48AD2FF7C7E39087515CB9B6E9B38D73BCB 2 10C48056D8C226141A8D7030E9FA17B7F02A39B414B9B64B6AECDDA5AFD1E538 2 11DCECD73A8FA6CFCD0468C659C2F845A9215842B69BA10355C1F4BF2D9A9557 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"rev_reg\":{\"accum\":\"2 033C0E6FAC660DF3582EF46021FAFDD93E111D1DC9DA59C4EA9B92BB21F8E0A4 2 02E0F749312228A93CF67BB5F86CA263FAE535A0F1CA449237D736939518EFF0 2 19BB82474D0BD0A1DDE72D377C8A965D6393071118B79D4220D4C9B93D090314 2 1895AAFD8050A8FAE4A93770C6C82881AB13134EE082C64CF6A7A379B3F6B217 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000\"},\"timestamp\":100}".to_string();

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (schema_id, _, cred_def_id, _, _, _, _, cred_id, rev_reg_id, cred_rev_id) =
            ::utils::libindy::anoncreds::tests::create_and_store_credential(attrs, true);
        let cred2 = CredInfo {
            requested_attr: "height".to_string(),
            referent: cred_id,
            schema_id,
            cred_def_id,
            rev_reg_id: rev_reg_id.clone(),
            cred_rev_id,
            tails_file: Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string()),
            revocation_interval: Some(NonRevokedInterval { from: None, to: Some(cached_timestamp - 1) }),
            timestamp: None,
        };
        let rev_reg_id = rev_reg_id.unwrap();

        let cached_data = RevRegCache {
            rev_state: Some(RevState {
                timestamp: cached_timestamp,
                value: cached_rev_state.clone(),
            })
        };
        set_rev_reg_cache(&rev_reg_id, &cached_data);

        // assert data is successfully cached.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        let states = build_rev_states_json(vec![cred2].as_mut()).unwrap();
        assert!(states.contains(&rev_reg_id));

        // assert cached data is unchanged.
        let cache = get_rev_reg_cache(&rev_reg_id);
        assert_eq!(cache, cached_data);

        // check if this value is not in cache.
        let states: Value = serde_json::from_str(&states).unwrap();
        let state: HashMap<String, Value> = serde_json::from_value(states[&rev_reg_id].clone()).unwrap();

        let cache_rev_state = cache.rev_state.unwrap();
        let cache_rev_state_value: Value = serde_json::from_str(&cache_rev_state.value).unwrap();
        assert_ne!(cache_rev_state.timestamp, state.keys().next().unwrap().parse::<u64>().unwrap());
        assert_ne!(cache_rev_state_value.to_string(), state.values().next().unwrap().to_string());
    }

    #[test]
    fn test_get_credential_intervals_from_proof_req() {
        let _setup = SetupDefaults::init();

        let proof_req = json!({
            "nonce": "123432421212",
            "name": "proof_req_1",
            "version": "0.1",
            "requested_attributes": {
                "address1_1": {
                    "name": "address1",
                    "non_revoked":  {"from": 123, "to": 456}
                },
                "zip_2": { "name": "zip" }
            },
            "requested_predicates": {},
            "non_revoked": {"from": 098, "to": 123}
        });
        let proof_req: ProofRequestData = serde_json::from_value(proof_req).unwrap();

        // Attribute not found in proof req
        assert_eq!(_get_revocation_interval("not here", &proof_req).unwrap_err().kind(), VcxErrorKind::InvalidProofCredentialData);

        // attribute interval overrides proof request interval
        let interval = Some(NonRevokedInterval { from: Some(123), to: Some(456) });
        assert_eq!(_get_revocation_interval("address1_1", &proof_req).unwrap(), interval);

        // when attribute interval is None, defaults to proof req interval
        let interval = Some(NonRevokedInterval { from: Some(098), to: Some(123) });
        assert_eq!(_get_revocation_interval("zip_2", &proof_req).unwrap(), interval);

        // No interval provided for attribute or proof req
        assert_eq!(_get_revocation_interval("address1_1", &proof_req_no_interval()).unwrap(), None);
    }
}
