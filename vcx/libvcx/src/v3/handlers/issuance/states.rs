use v3::messages::a2a::MessageId;
use v3::messages::issuance::credential_request::CredentialRequest;
use v3::messages::issuance::credential_offer::CredentialOffer;
use v3::messages::issuance::credential::Credential;
use v3::messages::status::Status;
use v3::messages::error::ProblemReport;

// Possible Transitions:
// Initial -> OfferSent
// Initial -> Finished
// OfferSent -> CredentialSent
// OfferSent -> Finished
// CredentialSent -> Finished
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IssuerState {
    Initial(InitialState),
    OfferSent(OfferSentState),
    RequestReceived(RequestReceivedState),
    CredentialSent(CredentialSentState),
    Finished(FinishedState)
}

impl IssuerState {
    pub fn get_connection_handle(&self) -> u32 {
        match self {
            IssuerState::Initial(_) => 0,
            IssuerState::OfferSent(state) => state.connection_handle,
            IssuerState::RequestReceived(state) => state.connection_handle,
            IssuerState::CredentialSent(state) => state.connection_handle,
            IssuerState::Finished(_) => 0
        }
    }

    pub fn thread_id(&self) -> String {
        match self {
            IssuerState::Initial(_) => String::new(),
            IssuerState::OfferSent(state) => state.thread_id.clone(),
            IssuerState::RequestReceived(state) => state.thread_id.clone(),
            IssuerState::CredentialSent(state) => state.thread_id.clone(),
            IssuerState::Finished(state) => state.thread_id.clone(),
        }
    }
}

impl InitialState {
    pub fn new(cred_def_id: &str, credential_json: &str, rev_reg_id: Option<String>, tails_file: Option<String>, credential_name: Option<String>) -> Self {
        InitialState {
            cred_def_id: cred_def_id.to_string(),
            credential_json: credential_json.to_string(),
            rev_reg_id,
            tails_file,
            credential_name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitialState {
    pub cred_def_id: String,
    pub credential_json: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferSentState {
    pub offer: String,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub connection_handle: u32,
    pub thread_id: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestReceivedState {
    pub offer: String,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub connection_handle: u32,
    pub request: CredentialRequest,
    pub thread_id: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RevocationInfoV1 {
	pub cred_rev_id: Option<String>,
	pub rev_reg_id: Option<String>,
	pub tails_file: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialSentState {
    pub connection_handle: u32,
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub thread_id: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedState {
    pub cred_id: Option<String>,
    pub thread_id: String,
    pub revocation_info_v1: Option<RevocationInfoV1>,
    pub status: Status
}

impl From<(InitialState, String, u32, MessageId)> for OfferSentState {
    fn from((state, offer, connection_handle, sent_id): (InitialState, String, u32, MessageId)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data: state.credential_json,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            connection_handle,
            thread_id: sent_id.0,
        }
    }
}

impl From<InitialState> for FinishedState {
    fn from(_state: InitialState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: String::new(),
            revocation_info_v1: None,
            status: Status::Undefined,
        }
    }
}

impl From<(OfferSentState, CredentialRequest)> for RequestReceivedState {
    fn from((state, request): (OfferSentState, CredentialRequest)) -> Self {
        trace!("SM is now in Request Received state");
        RequestReceivedState {
            offer: state.offer,
            cred_data: state.cred_data,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            connection_handle: state.connection_handle,
            request,
            thread_id: state.thread_id,
        }
    }
}

impl From<(RequestReceivedState, MessageId)> for CredentialSentState {
    fn from((state, _sent_id): (RequestReceivedState, MessageId)) -> Self {
        trace!("SM is now in CredentialSent state");
        CredentialSentState {
            connection_handle: state.connection_handle,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            thread_id: state.thread_id,
        }
    }
}

impl From<OfferSentState> for FinishedState {
    fn from(state: OfferSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Undefined,
        }
    }
}

impl From<(OfferSentState, ProblemReport)> for FinishedState {
    fn from((state, err): (OfferSentState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Failed(err),
        }
    }
}

impl From<(RequestReceivedState, Option<String>)> for FinishedState {
    fn from((state, cred_rev_id): (RequestReceivedState, Option<String>)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: cred_rev_id,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Success,
        }
    }
}

impl From<(RequestReceivedState, ProblemReport)> for FinishedState {
    fn from((state, err): (RequestReceivedState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Failed(err),
        }
    }
}

impl From<CredentialSentState> for FinishedState {
    fn from(state: CredentialSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: state.thread_id,
            revocation_info_v1: state.revocation_info_v1,
            status: Status::Success,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HolderState {
    OfferReceived(OfferReceivedState),
    RequestSent(RequestSentState),
    Finished(FinishedHolderState)
}

impl HolderState {
    pub fn get_connection_handle(&self) -> u32 {
        match self {
            HolderState::OfferReceived(_) => 0,
            HolderState::RequestSent(state) => state.connection_handle,
            HolderState::Finished(_) => 0
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub req_meta: String,
    pub cred_def_json: String,
    pub connection_handle: u32
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer
}

impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        OfferReceivedState {
            offer,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedHolderState {
    pub cred_id: Option<String>,
    pub credential: Option<Credential>,
    pub status: Status,
    pub rev_reg_def_json: Option<String>
}

impl From<(OfferReceivedState, String, String, u32)> for RequestSentState {
    fn from((_state, req_meta, cred_def_json, connection_handle): (OfferReceivedState, String, String, u32)) -> Self {
        trace!("SM is now in RequestSent state");
        RequestSentState {
            req_meta,
            cred_def_json,
            connection_handle,
        }
    }
}

impl From<(RequestSentState, String, Credential, Option<String>)> for FinishedHolderState {
    fn from((_, cred_id, credential, rev_reg_def_json): (RequestSentState, String, Credential, Option<String>)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            rev_reg_def_json: rev_reg_def_json
        }
    }
}

impl From<(RequestSentState, ProblemReport)> for FinishedHolderState {
    fn from((_, problem_report): (RequestSentState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None
        }
    }
}

impl From<(OfferReceivedState, ProblemReport)> for FinishedHolderState {
    fn from((_state, problem_report): (OfferReceivedState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None
        }
    }
}
