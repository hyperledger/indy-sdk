use api::VcxStateType;

use connection;
use v3::handlers::proof_presentation::prover::messages::ProverMessages;
use v3::messages::a2a::A2AMessage;
use v3::messages::proof_presentation::presentation_request::PresentationRequest;
use v3::messages::proof_presentation::presentation_proposal::{PresentationProposal, PresentationPreview};
use v3::messages::proof_presentation::presentation::Presentation;
use v3::messages::proof_presentation::presentation_ack::PresentationAck;
use v3::messages::error::ProblemReport;
use v3::messages::status::Status;

use std::collections::HashMap;
use disclosed_proof::DisclosedProof;

use error::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProverSM {
    source_id: String,
    thread_id: String,
    state: ProverState,
}

impl ProverSM {
    pub fn new(presentation_request: PresentationRequest, source_id: String) -> ProverSM {
        ProverSM { source_id, thread_id: presentation_request.id.0.clone(), state: ProverState::Initiated(InitialState { presentation_request }) }
    }
}

// Possible Transitions:
//
// Initial -> PresentationPrepared, PresentationPreparationFailedState, Finished
// PresentationPrepared -> PresentationSent, Finished
// PresentationPreparationFailedState -> Finished
// PresentationSent -> Finished
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProverState {
    Initiated(InitialState),
    PresentationPrepared(PresentationPreparedState),
    PresentationPreparationFailed(PresentationPreparationFailedState),
    PresentationSent(PresentationSentState),
    Finished(FinishedState),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitialState {
    presentation_request: PresentationRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparedState {
    presentation_request: PresentationRequest,
    presentation: Presentation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationPreparationFailedState {
    presentation_request: PresentationRequest,
    problem_report: ProblemReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresentationSentState {
    connection_handle: u32,
    presentation_request: PresentationRequest,
    presentation: Presentation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinishedState {
    connection_handle: u32,
    presentation_request: PresentationRequest,
    presentation: Presentation,
    status: Status,
}

impl From<(InitialState, Presentation)> for PresentationPreparedState {
    fn from((state, presentation): (InitialState, Presentation)) -> Self {
        trace!("transit state from InitialState to PresentationPreparedState");
        PresentationPreparedState {
            presentation_request: state.presentation_request,
            presentation,
        }
    }
}

impl From<(InitialState, ProblemReport)> for PresentationPreparationFailedState {
    fn from((state, problem_report): (InitialState, ProblemReport)) -> Self {
        trace!("transit state from InitialState to PresentationPreparationFailedState");
        PresentationPreparationFailedState {
            presentation_request: state.presentation_request,
            problem_report,
        }
    }
}

impl From<InitialState> for FinishedState {
    fn from(state: InitialState) -> Self {
        trace!("transit state from InitialState to FinishedState");
        FinishedState {
            connection_handle: 0,
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}

impl From<(PresentationPreparedState, u32)> for PresentationSentState {
    fn from((state, connection_handle): (PresentationPreparedState, u32)) -> Self {
        trace!("transit state from PresentationPreparedState to PresentationSentState");
        PresentationSentState {
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            connection_handle,
        }
    }
}

impl From<PresentationPreparedState> for FinishedState {
    fn from(state: PresentationPreparedState) -> Self {
        trace!("transit state from PresentationPreparedState to FinishedState");
        FinishedState {
            connection_handle: 0,
            presentation_request: state.presentation_request,
            presentation: Default::default(),
            status: Status::Declined,
        }
    }
}

impl From<(PresentationPreparationFailedState, u32)> for FinishedState {
    fn from((state, connection_handle): (PresentationPreparationFailedState, u32)) -> Self {
        trace!("transit state from PresentationPreparationFailedState to FinishedState");
        FinishedState {
            presentation_request: state.presentation_request,
            presentation: Presentation::create(),
            connection_handle,
            status: Status::Failed(state.problem_report),
        }
    }
}

impl From<(PresentationSentState, PresentationAck)> for FinishedState {
    fn from((state, _ack): (PresentationSentState, PresentationAck)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            connection_handle: state.connection_handle,
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            status: Status::Success,
        }
    }
}

impl From<(PresentationSentState, ProblemReport)> for FinishedState {
    fn from((state, problem_report): (PresentationSentState, ProblemReport)) -> Self {
        trace!("transit state from PresentationSentState to FinishedState");
        FinishedState {
            connection_handle: state.connection_handle,
            presentation_request: state.presentation_request,
            presentation: state.presentation,
            status: Status::Failed(problem_report),
        }
    }
}

impl InitialState {
    fn build_presentation(&self, credentials: &str, self_attested_attrs: &str) -> VcxResult<String> {
        DisclosedProof::generate_indy_proof(credentials,
                                            self_attested_attrs,
                                            &self.presentation_request.request_presentations_attach.content()?)
    }
}

impl ProverSM {
    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        trace!("Prover::find_message_to_handle >>> messages: {:?}", messages);

        for (uid, message) in messages {
            match self.state {
                ProverState::Initiated(_) => {
                    match message {
                        A2AMessage::PresentationRequest(_) => {
                            // ignore it here??
                        }
                        _ => {}
                    }
                }
                ProverState::PresentationPrepared(_) => {
                    // do not process messages
                }
                ProverState::PresentationPreparationFailed(_) => {
                    // do not process messages
                }
                ProverState::PresentationSent(_) => {
                    match message {
                        A2AMessage::Ack(ack) | A2AMessage::PresentationAck(ack) => {
                            if ack.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::PresentationAck(ack)));
                            }
                        }
                        A2AMessage::CommonProblemReport(problem_report) => {
                            if problem_report.from_thread(&self.thread_id) {
                                return Some((uid, A2AMessage::CommonProblemReport(problem_report)));
                            }
                        }
                        _ => {}
                    }
                }
                ProverState::Finished(_) => {
                    // do not process messages
                }
            };
        }

        None
    }

    pub fn step(self, message: ProverMessages) -> VcxResult<ProverSM> {
        trace!("ProverSM::step >>> message: {:?}", message);

        let ProverSM { source_id, state, thread_id } = self;

        let state = match state {
            ProverState::Initiated(state) => {
                match message {
                    ProverMessages::SetPresentation(presentation) => {
                        let presentation = presentation.set_thread_id(&thread_id);
                        ProverState::PresentationPrepared((state, presentation).into())
                    }
                    ProverMessages::PreparePresentation((credentials, self_attested_attrs)) => {
                        match state.build_presentation(&credentials, &self_attested_attrs) {
                            Ok(presentation) => {
                                let presentation = Presentation::create()
                                    .ask_for_ack()
                                    .set_thread_id(&thread_id)
                                    .set_presentations_attach(presentation)?;

                                ProverState::PresentationPrepared((state, presentation).into())
                            }
                            Err(err) => {
                                let problem_report =
                                    ProblemReport::create()
                                        .set_comment(err.to_string())
                                        .set_thread_id(&thread_id);

                                ProverState::PresentationPreparationFailed((state, problem_report).into())
                            }
                        }
                    }
                    ProverMessages::RejectPresentationRequest((connection_handle, reason)) => {
                        Self::_handle_reject_presentation_request(connection_handle, &reason, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    ProverMessages::ProposePresentation((connection_handle, preview)) => {
                        Self::_handle_presentation_proposal(connection_handle, preview, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    _ => {
                        ProverState::Initiated(state)
                    }
                }
            }
            ProverState::PresentationPrepared(state) => {
                match message {
                    ProverMessages::SendPresentation(connection_handle) => {
                        match state.presentation_request.service.clone() {
                            None => {
                                connection::send_message(connection_handle, state.presentation.to_a2a_message())?;
                                connection::remove_pending_message(connection_handle, &state.presentation_request.id)?;
                                ProverState::PresentationSent((state, connection_handle).into())
                            }
                            Some(service) => {
                                connection::send_message_to_self_endpoint(state.presentation.to_a2a_message(), &service.into())?;
                                ProverState::Finished(state.into())
                            }
                        }
                    }
                    ProverMessages::RejectPresentationRequest((connection_handle, reason)) => {
                        Self::_handle_reject_presentation_request(connection_handle, &reason, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    ProverMessages::ProposePresentation((connection_handle, preview)) => {
                        Self::_handle_presentation_proposal(connection_handle, preview, &state.presentation_request, &thread_id)?;
                        ProverState::Finished(state.into())
                    }
                    _ => {
                        ProverState::PresentationPrepared(state)
                    }
                }
            }
            ProverState::PresentationPreparationFailed(state) => {
                match message {
                    ProverMessages::SendPresentation(connection_handle) => {
                        match state.presentation_request.service.clone() {
                            None => {
                                connection::send_message(connection_handle, state.problem_report.to_a2a_message())?;
                            }
                            Some(service) => {
                                connection::send_message_to_self_endpoint(state.problem_report.to_a2a_message(), &service.into())?;
                            }
                        }

                        ProverState::Finished((state, connection_handle).into())
                    }
                    _ => {
                        ProverState::PresentationPreparationFailed(state)
                    }
                }
            }
            ProverState::PresentationSent(state) => {
                match message {
                    ProverMessages::PresentationAckReceived(ack) => {
                        ProverState::Finished((state, ack).into())
                    }
                    ProverMessages::PresentationRejectReceived(problem_report) => {
                        ProverState::Finished((state, problem_report).into())
                    }
                    ProverMessages::RejectPresentationRequest(_) => {
                        return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Presentation is already sent"));
                    }
                    _ => {
                        ProverState::PresentationSent(state)
                    }
                }
            }
            ProverState::Finished(state) => ProverState::Finished(state)
        };

        Ok(ProverSM { source_id, state, thread_id })
    }

    fn _handle_reject_presentation_request(connection_handle: u32, reason: &str, presentation_request: &PresentationRequest, thread_id: &str) -> VcxResult<()> {
        let problem_report = ProblemReport::create()
            .set_comment(reason.to_string())
            .set_thread_id(thread_id);

        match presentation_request.service.clone() {
            None => connection::send_message(connection_handle, problem_report.to_a2a_message())?,
            Some(service) => connection::send_message_to_self_endpoint(problem_report.to_a2a_message(), &service.into())?
        }

        connection::remove_pending_message(connection_handle, &presentation_request.id)?;
        Ok(())
    }

    fn _handle_presentation_proposal(connection_handle: u32, preview: PresentationPreview, presentation_request: &PresentationRequest, thread_id: &str) -> VcxResult<()> {
        let proposal = PresentationProposal::create()
            .set_presentation_preview(preview)
            .set_thread_id(thread_id);

        match presentation_request.service.clone() {
            None => connection::send_message(connection_handle, proposal.to_a2a_message())?,
            Some(service) => connection::send_message_to_self_endpoint(proposal.to_a2a_message(), &service.into())?
        }

        connection::remove_pending_message(connection_handle, &presentation_request.id)?;
        Ok(())
    }

    pub fn source_id(&self) -> String { self.source_id.clone() }

    pub fn state(&self) -> u32 {
        match self.state {
            ProverState::Initiated(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPrepared(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationPreparationFailed(_) => VcxStateType::VcxStateRequestReceived as u32,
            ProverState::PresentationSent(_) => VcxStateType::VcxStateOfferSent as u32, // TODO: maybe VcxStateType::VcxStateAccepted
            ProverState::Finished(ref status) => {
                match status.status {
                    Status::Success => VcxStateType::VcxStateAccepted as u32,
                    _ => VcxStateType::VcxStateNone as u32,
                }
            }
        }
    }

    pub fn has_transitions(&self) -> bool {
        match self.state {
            ProverState::Initiated(_) => false,
            ProverState::PresentationPrepared(_) => true,
            ProverState::PresentationPreparationFailed(_) => true,
            ProverState::PresentationSent(_) => true,
            ProverState::Finished(_) => false,
        }
    }

    pub fn presentation_status(&self) -> u32 {
        match self.state {
            ProverState::Finished(ref state) => state.status.code(),
            _ => Status::Undefined.code()
        }
    }

    pub fn connection_handle(&self) -> VcxResult<u32> {
        match self.state {
            ProverState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Connection handle isn't set")),
            ProverState::PresentationPrepared(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Connection handle isn't set")),
            ProverState::PresentationPreparationFailed(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Connection handle isn't set")),
            ProverState::PresentationSent(ref state) => Ok(state.connection_handle),
            ProverState::Finished(ref state) => Ok(state.connection_handle),
        }
    }

    pub fn presentation_request(&self) -> &PresentationRequest {
        match self.state {
            ProverState::Initiated(ref state) => &state.presentation_request,
            ProverState::PresentationPrepared(ref state) => &state.presentation_request,
            ProverState::PresentationPreparationFailed(ref state) => &state.presentation_request,
            ProverState::PresentationSent(ref state) => &state.presentation_request,
            ProverState::Finished(ref state) => &state.presentation_request,
        }
    }

    pub fn presentation(&self) -> VcxResult<&Presentation> {
        match self.state {
            ProverState::Initiated(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverState::PresentationPrepared(ref state) => Ok(&state.presentation),
            ProverState::PresentationPreparationFailed(_) => Err(VcxError::from_msg(VcxErrorKind::NotReady, "Presentation is not created yet")),
            ProverState::PresentationSent(ref state) => Ok(&state.presentation),
            ProverState::Finished(ref state) => Ok(&state.presentation),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use utils::devsetup::SetupAriesMocks;
    use v3::handlers::connection::tests::mock_connection;
    use v3::test::source_id;
    use v3::messages::proof_presentation::test::{_ack, _problem_report};
    use v3::messages::proof_presentation::presentation_request::tests::{_presentation_request, _presentation_request_with_service};
    use v3::messages::proof_presentation::presentation::tests::_presentation;
    use v3::messages::proof_presentation::presentation_proposal::tests::{_presentation_proposal, _presentation_preview};

    pub fn _prover_sm() -> ProverSM {
        ProverSM::new(_presentation_request(), source_id())
    }

    impl ProverSM {
        fn to_presentation_prepared_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self
        }

        fn to_presentation_sent_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self = self.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            self
        }

        fn to_finished_state(mut self) -> ProverSM {
            self = self.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            self = self.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            self = self.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            self
        }
    }

    fn _credentials() -> String {
        json!({
            "attrs":{
            "attribute_0":{
                "credential":{
                    "cred_info":{
                        "attrs":{"name": "alice"},
                        "cred_def_id": "V4SGRU86Z58d6TV7PBUe6f:3:CL:419:tag",
                        "referent": "a1991de8-8317-43fd-98b3-63bac40b9e8b",
                        "schema_id": "V4SGRU86Z58d6TV7PBUe6f:2:QcimrRShWQniqlHUtIDddYP0n:1.0"
                        }
                    }
                }
            }
        }).to_string()
    }

    fn _self_attested() -> String {
        json!({}).to_string()
    }

    mod new {
        use super::*;

        #[test]
        fn test_prover_new() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm();

            assert_match!(ProverState::Initiated(_), prover_sm.state);
            assert_eq!(source_id(), prover_sm.source_id());
        }
    }

    mod step {
        use super::*;

        #[test]
        fn test_prover_init() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm();
            assert_match!(ProverState::Initiated(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_prepare_presentation_message_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();

            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_prepare_presentation_message_from_initiated_state_for_invalid_credentials() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();

            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject request")))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_propose_presentation_message_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_other_messages_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::Initiated(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::Initiated(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();

            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_prepared_state_for_presentation_request_contains_service_decorator() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = ProverSM::new(_presentation_request_with_service(), source_id());
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::PresentationPrepared(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();
            prover_sm = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject request")))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_propose_presentation_message_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm().to_presentation_prepared_state();
            prover_sm = prover_sm.step(ProverMessages::ProposePresentation((mock_connection(), _presentation_preview()))).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_send_presentation_message_from_presentation_preparation_failed_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::default()).code(), prover_sm.presentation_status());
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_preparation_failed_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation(("invalid".to_string(), _self_attested()))).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::PresentationPreparationFailed(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_ack_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Success.code(), prover_sm.presentation_status());
        }

        #[test]
        fn test_prover_handle_reject_presentation_request_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let prover_sm = _prover_sm().to_presentation_sent_state();
            let err = prover_sm.step(ProverMessages::RejectPresentationRequest((mock_connection(), String::from("reject")))).unwrap_err();
            assert_eq!(VcxErrorKind::ActionNotSupported, err.kind());
        }

        #[test]
        fn test_prover_handle_presentation_reject_message_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();

            assert_match!(ProverState::Finished(_), prover_sm.state);
            assert_eq!(Status::Failed(ProblemReport::create()).code(), prover_sm.presentation_status());
        }

        #[test]
        fn test_prover_handle_other_messages_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            assert_match!(ProverState::PresentationSent(_), prover_sm.state);
        }

        #[test]
        fn test_prover_handle_messages_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let mut prover_sm = _prover_sm();
            prover_sm = prover_sm.step(ProverMessages::PreparePresentation((_credentials(), _self_attested()))).unwrap();
            prover_sm = prover_sm.step(ProverMessages::SendPresentation(mock_connection())).unwrap();
            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();

            prover_sm = prover_sm.step(ProverMessages::PresentationAckReceived(_ack())).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);

            prover_sm = prover_sm.step(ProverMessages::PresentationRejectReceived(_problem_report())).unwrap();
            assert_match!(ProverState::Finished(_), prover_sm.state);
        }
    }

    mod find_message_to_handle {
        use super::*;

        #[test]
        fn test_prover_find_message_to_handle_from_initiated_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_presentation_prepared_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_presentation_prepared_state();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_presentation_sent_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_presentation_sent_state();

            // Ack
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::PresentationAck(_), message);
            }

            // Problem Report
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_3".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                let (uid, message) = prover.find_message_to_handle(messages).unwrap();
                assert_eq!("key_3", uid);
                assert_match!(A2AMessage::CommonProblemReport(_), message);
            }

            // No messages for different Thread ID
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal().set_thread_id("")),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation().set_thread_id("")),
                    "key_3".to_string() => A2AMessage::PresentationAck(_ack().set_thread_id("")),
                    "key_4".to_string() => A2AMessage::CommonProblemReport(_problem_report().set_thread_id(""))
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::PresentationRequest(_presentation_request())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }

        #[test]
        fn test_prover_find_message_to_handle_from_finished_state() {
            let _setup = SetupAriesMocks::init();

            let prover = _prover_sm().to_finished_state();

            // No messages
            {
                let messages = map!(
                    "key_1".to_string() => A2AMessage::PresentationProposal(_presentation_proposal()),
                    "key_2".to_string() => A2AMessage::Presentation(_presentation()),
                    "key_3".to_string() => A2AMessage::PresentationRequest(_presentation_request()),
                    "key_4".to_string() => A2AMessage::PresentationAck(_ack()),
                    "key_5".to_string() => A2AMessage::CommonProblemReport(_problem_report())
                );

                assert!(prover.find_message_to_handle(messages).is_none());
            }
        }
    }

    mod get_state {
        use super::*;

        #[test]
        fn test_get_state() {
            let _setup = SetupAriesMocks::init();

            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, _prover_sm().to_presentation_prepared_state().state());
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, _prover_sm().to_presentation_sent_state().state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, _prover_sm().to_finished_state().state());
        }
    }
}