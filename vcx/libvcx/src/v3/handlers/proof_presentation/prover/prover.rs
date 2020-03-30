use messages::get_message::Message;
use error::prelude::*;
use utils::libindy::anoncreds;
use std::convert::TryInto;
use std::collections::HashMap;

use v3::handlers::proof_presentation::prover::states::ProverSM;
use v3::handlers::proof_presentation::prover::messages::ProverMessages;
use v3::messages::a2a::{A2AMessage, MessageId};
use v3::messages::proof_presentation::presentation_proposal::PresentationPreview;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use connection;

use messages::proofs::proof_message::ProofMessage;

use std::sync::Mutex;
use v3::messages::proof_presentation::presentation::Presentation;

lazy_static! {
    pub static ref PENDING_PRESENTATION_REQUESTS: Mutex<HashMap<MessageId, String>> = Default::default();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prover {
    prover_sm: ProverSM
}

impl Prover {
    pub fn create(source_id: &str, presentation_request: PresentationRequest) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}, presentation_request: {:?}", source_id, presentation_request);
        Ok(Prover {
            prover_sm: ProverSM::new(presentation_request, source_id.to_string()),
        })
    }

    pub fn state(&self) -> u32 { self.prover_sm.state() }

    pub fn presentation_status(&self) -> u32 {
        trace!("Prover::presentation_state >>>");
        self.prover_sm.presentation_status()
    }

    pub fn retrieve_credentials(&self) -> VcxResult<String> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.prover_sm.presentation_request().request_presentations_attach.content()?;
        anoncreds::libindy_prover_get_credentials_for_proof_req(&presentation_request)
    }

    pub fn generate_presentation(&mut self, credentials: String, self_attested_attrs: String) -> VcxResult<()> {
        trace!("Prover::generate_presentation >>> credentials: {}, self_attested_attrs: {:?}", credentials, self_attested_attrs);
        self.step(ProverMessages::PreparePresentation((credentials, self_attested_attrs)))
    }

    pub fn generate_presentation_msg(&self) -> VcxResult<String> {
        trace!("Prover::generate_presentation_msg >>>");

        let proof: ProofMessage = self.prover_sm.presentation()?.clone().try_into()?;

        ::serde_json::to_string(&proof)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize ProofMessage: {:?}", err)))
    }

    pub fn set_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.step(ProverMessages::SetPresentation(presentation))
    }

    pub fn send_presentation(&mut self, connection_handle: u32) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.step(ProverMessages::SendPresentation(connection_handle))
    }

    pub fn update_state(&mut self, message: Option<&str>) -> VcxResult<()> {
        trace!("Prover::update_state >>> message: {:?}", message);

        if !self.prover_sm.has_transitions() { return Ok(()); }

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let connection_handle = self.prover_sm.connection_handle()?;
        let messages = connection::get_messages(connection_handle)?;

        if let Some((uid, message)) = self.prover_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            connection::update_message_status(connection_handle, uid)?;
        };

        Ok(())
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<()> {
        trace!("Prover::update_state_with_message >>> message: {:?}", message);

        let message: Message = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot updated state with message: Message deserialization failed: {:?}", err)))?;

        let connection_handle = self.prover_sm.connection_handle()?;

        let uid = message.uid.clone();
        let a2a_message = connection::decode_message(connection_handle, message)?;

        self.handle_message(a2a_message.into())?;
        connection::update_message_status(connection_handle, uid)?;

        Ok(())
    }

    pub fn handle_message(&mut self, message: ProverMessages) -> VcxResult<()> {
        trace!("Prover::handle_message >>> message: {:?}", message);
        self.step(message)
    }

    pub fn get_presentation_request(connection_handle: u32, msg_id: &str) -> VcxResult<PresentationRequest> {
        trace!("Prover::get_presentation_request >>> connection_handle: {:?}, msg_id: {:?}", connection_handle, msg_id);

        let message = connection::get_message_by_id(connection_handle, msg_id.to_string())?;

        let (id, presentation_request): (MessageId, PresentationRequest) = match message {
            A2AMessage::PresentationRequest(presentation_request) => (presentation_request.id.clone(), presentation_request),
            msg => return Err(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message of different type was received: {:?}", msg)))
        };

        connection::add_pending_messages(connection_handle, map! { id => msg_id.to_string() })?;

        Ok(presentation_request)
    }

    pub fn get_presentation_request_messages(connection_handle: u32, match_name: Option<&str>) -> VcxResult<Vec<PresentationRequest>> {
        trace!("Prover::get_presentation_request_messages >>> connection_handle: {:?}, match_name: {:?}", connection_handle, match_name);

        let (uids, presentation_requests): (HashMap<MessageId, String>, Vec<PresentationRequest>) =
            connection::get_messages(connection_handle)?
                .into_iter()
                .filter_map(|(uid, message)| {
                    match message {
                        A2AMessage::PresentationRequest(presentation_request) => {
                            Some((uid, presentation_request.id.clone(), presentation_request))
                        }
                        _ => None,
                    }
                }).fold((HashMap::new(), Vec::new()), |(mut uids, mut messages), (uid, id, presentation_request)| {
                uids.insert(id, uid);
                messages.push(presentation_request);
                (uids, messages)
            });

        connection::add_pending_messages(connection_handle, uids)?;

        Ok(presentation_requests)
    }

    pub fn get_source_id(&self) -> String { self.prover_sm.source_id() }

    pub fn step(&mut self, message: ProverMessages) -> VcxResult<()> {
        self.prover_sm = self.prover_sm.clone().step(message)?;
        Ok(())
    }

    pub fn decline_presentation_request(&mut self, connection_handle: u32, reason: Option<String>, proposal: Option<String>) -> VcxResult<()> {
        trace!("Prover::decline_presentation_request >>> connection_handle: {}, reason: {:?}, proposal: {:?}", connection_handle, reason, proposal);
        match (reason, proposal) {
            (Some(reason), None) => {
                self.step(ProverMessages::RejectPresentationRequest((connection_handle, reason)))
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize Presentation Preview: {:?}", err)))?;

                self.step(ProverMessages::ProposePresentation((connection_handle, presentation_preview)))
            }
            (None, None) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Either `reason` or `proposal` parameter must be specified."));
            }
            (Some(_), Some(_)) => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "Only one of `reason` or `proposal` parameters must be specified."));
            }
        }
    }
}