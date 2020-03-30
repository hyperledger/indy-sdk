use std::collections::HashMap;

use rmp_serde;
use serde_json;
use serde_json::Value;

use api::VcxStateType;
use error::prelude::*;
use messages;
use messages::{GeneralMessage, MessageStatusCode, RemoteMessageType, SerializableObjectWithState};
use messages::invite::{InviteDetail, RedirectDetail, SenderDetail, Payload as ConnectionPayload, AcceptanceDetails, RedirectionDetails};
use messages::payload::{Payloads, PayloadKinds};
use messages::thread::Thread;
use messages::send_message::SendMessageOptions;
use messages::get_message::{Message, MessagePayload};
use object_cache::ObjectCache;
use settings;
use utils::error;
use utils::libindy::signus::create_and_store_my_did;
use utils::libindy::crypto;
use utils::json::mapped_key_rewrite;
use utils::json::KeyMatch;

use v3::handlers::connection::connection::Connection as ConnectionV3;
use v3::handlers::connection::states::ActorDidExchangeState;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::connection::invite::Invitation as InvitationV3;
use settings::ProtocolTypes;

lazy_static! {
    static ref CONNECTION_MAP: ObjectCache<Connections> = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum Connections {
    #[serde(rename = "1.0")]
    V1(Connection),
    #[serde(rename = "2.0")]
    V3(ConnectionV3),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionOptions {
    #[serde(default)]
    pub connection_type: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    pub use_public_did: Option<bool>,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        ConnectionOptions {
            connection_type: None,
            phone: None,
            use_public_did: None,
        }
    }
}

impl ConnectionOptions {
    pub fn from_opt_str(options: Option<String>) -> VcxResult<ConnectionOptions> {
        Ok(
            match options.as_ref().map(|opt| opt.trim()) {
                None => ConnectionOptions::default(),
                Some(opt) if opt.is_empty() => ConnectionOptions::default(),
                Some(opt) => {
                    serde_json::from_str(&opt)
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot deserialize ConnectionOptions: {}", err)))?
                }
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Connection {
    source_id: String,
    pw_did: String,
    pw_verkey: String,
    state: VcxStateType,
    uuid: String,
    endpoint: String,
    // For QR code invitation
    invite_detail: Option<InviteDetail>,
    redirect_detail: Option<RedirectDetail>,
    invite_url: Option<String>,
    agent_did: String,
    agent_vk: String,
    their_pw_did: String,
    their_pw_verkey: String,
    // used by proofs/credentials when sending to edge device
    public_did: Option<String>,
    their_public_did: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<settings::ProtocolTypes>,
}

impl Connection {
    fn _connect_send_invite(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        debug!("sending invite for connection {}", self.source_id);

        let (invite, url) =
            messages::send_invite()
                .to(&self.pw_did)?
                .to_vk(&self.pw_verkey)?
                .phone_number(options.phone.as_ref().map(String::as_str))?
                .agent_did(&self.agent_did)?
                .agent_vk(&self.agent_vk)?
                .public_did(self.public_did.as_ref().map(String::as_str))?
                .thread(&Thread::new())?
                .version(&Some(::settings::get_protocol_type()))?
                .send_secure()
                .map_err(|err| err.extend("Cannot send invite"))?;

        self.state = VcxStateType::VcxStateOfferSent;
        self.invite_detail = Some(invite);
        self.invite_url = Some(url);

        Ok(error::SUCCESS.code_num)
    }

    pub fn delete_connection(&mut self) -> VcxResult<u32> {
        trace!("Connection::delete_connection >>>");

        messages::delete_connection()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .version(&self.version)?
            .send_secure()
            .map_err(|err| err.extend("Cannot delete connection"))?;

        self.state = VcxStateType::VcxStateNone;

        Ok(error::SUCCESS.code_num)
    }

    fn _connect_accept_invite(&mut self) -> VcxResult<u32> {
        debug!("accepting invite for connection {}", self.source_id);

        let details: &InviteDetail = self.invite_detail.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Invite details not found for: {}", self.source_id)))?;

        messages::accept_invite()
            .to(&self.pw_did)?
            .to_vk(&self.pw_verkey)?
            .agent_did(&self.agent_did)?
            .agent_vk(&self.agent_vk)?
            .sender_details(&details.sender_detail)?
            .sender_agency_details(&details.sender_agency_detail)?
            .answer_status_code(&MessageStatusCode::Accepted)?
            .reply_to(&details.conn_req_id)?
            .thread(&self._build_thread(&details))?
            .version(self.version.clone())?
            .send_secure()
            .map_err(|err| err.extend("Cannot accept invite"))?;

        self.state = VcxStateType::VcxStateAccepted;

        Ok(error::SUCCESS.code_num)
    }

    fn _build_thread(&self, invite_detail: &InviteDetail) -> Thread {
        let mut received_orders = HashMap::new();
        received_orders.insert(invite_detail.sender_detail.did.clone(), 0);
        Thread {
            thid: invite_detail.thread_id.clone(),
            pthid: None,
            sender_order: 0,
            received_orders,
        }
    }

    fn connect(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        trace!("Connection::connect >>> options: {:?}", options);
        match self.state {
            VcxStateType::VcxStateInitialized
            | VcxStateType::VcxStateOfferSent => self._connect_send_invite(options),
            VcxStateType::VcxStateRequestReceived => self._connect_accept_invite(),
            _ => {
                warn!("connection {} in state {} not ready to connect", self.source_id, self.state as u32);
                // TODO: Refactor Error
                // TODO: Implement Correct Error
                Err(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Connection {} in state {} not ready to connect", self.source_id, self.state as u32)))
            }
        }
    }

    fn redirect(&mut self, redirect_to: &Connection) -> VcxResult<u32> {
        trace!("Connection::redirect >>> redirect_to: {:?}", redirect_to);

        let details: &InviteDetail = self.invite_detail.as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Invite details not found for: {}", self.source_id)))?;

        match self.state {
            VcxStateType::VcxStateRequestReceived => {
                messages::redirect_connection()
                    .to(&self.pw_did)?
                    .to_vk(&self.pw_verkey)?
                    .agent_did(&self.agent_did)?
                    .agent_vk(&self.agent_vk)?
                    .sender_details(&details.sender_detail)?
                    .sender_agency_details(&details.sender_agency_detail)?
                    .redirect_details(&redirect_to.generate_redirect_details()?)?
                    .answer_status_code(&MessageStatusCode::Redirected)?
                    .reply_to(&details.conn_req_id)?
                    .thread(&self._build_thread(&details))?
                    .version(self.version.clone())?
                    .send_secure()
                    .map_err(|err| err.extend("Cannot send redirect"))?;

                self.state = VcxStateType::VcxStateRedirected;

                Ok(error::SUCCESS.code_num)
            }
            _ => {
                warn!("connection {} in state {} not ready to redirect", self.source_id, self.state as u32);
                // TODO: Refactor Error
                // TODO: Implement Correct Error
                Err(VcxError::from_msg(VcxErrorKind::GeneralConnectionError, format!("Connection {} in state {} not ready to redirect", self.source_id, self.state as u32)))
            }
        }
    }

    fn generate_redirect_details(&self) -> VcxResult<RedirectDetail> {
        let signature = format!("{}{}", self.pw_did, self.pw_verkey);
        let signature = ::utils::libindy::crypto::sign(&self.pw_verkey, signature.as_bytes())?;
        let signature = base64::encode(&signature);

        Ok(RedirectDetail {
            their_did: self.pw_did.clone(),
            their_verkey: self.pw_verkey.clone(),
            their_public_did: self.public_did.clone(),
            did: self.their_pw_did.clone(),
            verkey: self.their_pw_verkey.clone(),
            public_did: self.their_public_did.clone(),
            signature,
        })
    }

    fn get_state(&self) -> u32 {
        trace!("Connection::get_state >>>");
        self.state as u32
    }
    fn set_state(&mut self, state: VcxStateType) {
        trace!("Connection::set_state >>> state: {:?}", state);
        self.state = state;
    }

    fn get_pw_did(&self) -> &String { &self.pw_did }
    fn set_pw_did(&mut self, did: &str) { self.pw_did = did.to_string(); }

    fn get_their_pw_did(&self) -> &String { &self.their_pw_did }
    fn set_their_pw_did(&mut self, did: &str) { self.their_pw_did = did.to_string(); }

    fn set_their_public_did(&mut self, did: &str) { self.their_public_did = Some(did.to_string()); }
    fn get_their_public_did(&self) -> Option<String> { self.their_public_did.clone() }

    fn get_agent_did(&self) -> &String { &self.agent_did }
    fn set_agent_did(&mut self, did: &str) { self.agent_did = did.to_string(); }

    fn get_pw_verkey(&self) -> &String { &self.pw_verkey }
    fn set_pw_verkey(&mut self, verkey: &str) { self.pw_verkey = verkey.to_string(); }

    fn get_their_pw_verkey(&self) -> &String { &self.their_pw_verkey }
    fn set_their_pw_verkey(&mut self, verkey: &str) { self.their_pw_verkey = verkey.to_string(); }

    fn get_agent_verkey(&self) -> &String { &self.agent_vk }
    fn set_agent_verkey(&mut self, verkey: &str) { self.agent_vk = verkey.to_string(); }

    fn get_uuid(&self) -> &String { &self.uuid }
    fn set_uuid(&mut self, uuid: &str) { self.uuid = uuid.to_string(); }

    fn get_endpoint(&self) -> &String { &self.endpoint }
    fn set_endpoint(&mut self, endpoint: &str) { self.endpoint = endpoint.to_string(); }

    fn get_invite_detail(&self) -> &Option<InviteDetail> { &self.invite_detail }
    fn set_invite_detail(&mut self, id: InviteDetail) {
        self.version = match id.version.is_some() {
            true => Some(settings::ProtocolTypes::from(id.version.clone().unwrap())),
            false => Some(settings::get_connecting_protocol_version()),
        };
        self.invite_detail = Some(id);
    }

    #[allow(dead_code)]
    fn get_redirect_detail(&self) -> &Option<RedirectDetail> { &self.redirect_detail }
    fn set_redirect_detail(&mut self, rd: RedirectDetail) { self.redirect_detail = Some(rd); }

    fn get_version(&self) -> Option<settings::ProtocolTypes> {
        self.version.clone()
    }

    fn get_source_id(&self) -> &String { &self.source_id }

    #[allow(dead_code)]
    fn ready_to_connect(&self) -> bool {
        self.state != VcxStateType::VcxStateNone && self.state != VcxStateType::VcxStateAccepted
    }

    fn create_agent_pairwise(&mut self) -> VcxResult<u32> {
        debug!("creating pairwise keys on agent for connection {}", self.source_id);

        let (for_did, for_verkey) = messages::create_keys()
            .for_did(&self.pw_did)?
            .for_verkey(&self.pw_verkey)?
            .version(&Some(settings::get_protocol_type()))?
            .send_secure()
            .map_err(|err| err.extend("Cannot create pairwise keys"))?;

        debug!("create key for connection: {} with did {:?}, vk: {:?}", self.source_id, for_did, for_verkey);
        self.set_agent_did(&for_did);
        self.set_agent_verkey(&for_verkey);

        Ok(error::SUCCESS.code_num)
    }

    fn update_agent_profile(&mut self, options: &ConnectionOptions) -> VcxResult<u32> {
        debug!("updating agent config for connection {}", self.source_id);

        if let Some(true) = options.use_public_did {
            self.public_did = Some(settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?);
        };

        let webhook_url = settings::get_config_value(settings::CONFIG_WEBHOOK_URL).ok();

        if let Ok(name) = settings::get_config_value(settings::CONFIG_INSTITUTION_NAME) {
            messages::update_data()
                .to(&self.pw_did)?
                .name(&name)?
                .logo_url(&settings::get_config_value(settings::CONFIG_INSTITUTION_LOGO_URL)?)?
                .webhook_url(&webhook_url)?
                .use_public_did(&self.public_did)?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.extend("Cannot update agent profile"))?;
        }

        Ok(error::SUCCESS.code_num)
    }

    pub fn update_state(&mut self, _message: Option<String>) -> VcxResult<u32> {
        debug!("updating state for connection {}", self.source_id);

        if self.state == VcxStateType::VcxStateInitialized ||
            self.state == VcxStateType::VcxStateAccepted ||
            self.state == VcxStateType::VcxStateRedirected {
            return Ok(error::SUCCESS.code_num);
        }

        let response =
            messages::get_messages()
                .to(&self.pw_did)?
                .to_vk(&self.pw_verkey)?
                .agent_did(&self.agent_did)?
                .agent_vk(&self.agent_vk)?
                .version(&self.version)?
                .send_secure()
                .map_err(|err| err.map(VcxErrorKind::PostMessageFailed, format!("Could not update state for connection {}", self.source_id)))?;

        debug!("connection {} update state response: {:?}", self.source_id, response);
        if self.state == VcxStateType::VcxStateOfferSent || self.state == VcxStateType::VcxStateInitialized {
            for message in response {
                if message.status_code == MessageStatusCode::Accepted && message.msg_type == RemoteMessageType::ConnReqAnswer {
                    let rc = self.process_acceptance_message(&message);
                    if rc.is_err() {
                        self.force_v2_parse_acceptance_details(&message)?;
                    }
                } else if message.status_code == MessageStatusCode::Redirected && message.msg_type == RemoteMessageType::ConnReqRedirect {
                    let rc = self.process_redirect_message(&message);
                    if rc.is_err() {
                        self.force_v2_parse_redirection_details(&message)?;
                    }
                } else {
                    warn!("Unexpected message: {:?}", message);
                }
            }
        };

        Ok(error::SUCCESS.code_num)
    }

    pub fn process_acceptance_message(&mut self, message: &Message) -> VcxResult<u32> {
        let details = parse_acceptance_details(message)
            .map_err(|err| err.extend("Cannot parse acceptance details"))?;

        self.set_their_pw_did(&details.did);
        self.set_their_pw_verkey(&details.verkey);
        self.set_state(VcxStateType::VcxStateAccepted);

        Ok(error::SUCCESS.code_num)
    }


    pub fn send_generic_message(&self, message: &str, msg_options: &str) -> VcxResult<String> {
        if self.state != VcxStateType::VcxStateAccepted {
            return Err(VcxError::from(VcxErrorKind::NotReady));
        }

        let msg_options: SendMessageOptions = serde_json::from_str(msg_options).map_err(|_| {
            error!("Invalid SendMessage msg_options");
            VcxError::from(VcxErrorKind::InvalidConfiguration)
        })?;

        let response =
            ::messages::send_message()
                .to(&self.get_pw_did())?
                .to_vk(&self.get_pw_verkey())?
                .msg_type(&RemoteMessageType::Other(msg_options.msg_type.clone()))?
                .edge_agent_payload(&self.get_pw_verkey(), &self.get_their_pw_verkey(), &message, PayloadKinds::Other(msg_options.msg_type.clone()), None)?
                .agent_did(&self.get_agent_did())?
                .agent_vk(&self.get_agent_verkey())?
                .set_title(&msg_options.msg_title)?
                .set_detail(&msg_options.msg_title)?
                .ref_msg_id(msg_options.ref_msg_id.clone())?
                .status_code(&MessageStatusCode::Accepted)?
                .send_secure()?;

        let msg_uid = response.get_msg_uid()?;
        return Ok(msg_uid);
    }
}

pub fn create_agent_keys(source_id: &str, pw_did: &str, pw_verkey: &str) -> VcxResult<(String, String)> {
    /*
        Create User Pairwise Agent in old way.
        Send Messages corresponding to V2 Protocol version to avoid code changes on Agency side.
    */
    debug!("creating pairwise keys on agent for connection {}", source_id);

    let (agent_did, agent_verkey) = messages::create_keys()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .version(&Some(settings::get_protocol_type()))?
        .send_secure()
        .map_err(|err| err.extend("Cannot create pairwise keys"))?;

    Ok((agent_did, agent_verkey))
}

pub fn is_valid_handle(handle: u32) -> bool {
    CONNECTION_MAP.has_handle(handle)
}

pub fn set_agent_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_agent_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_agent_did().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_pw_did().to_string()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_did.to_string())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_ver_str(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_version().as_ref().map(ProtocolTypes::to_string)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_did(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_did().to_string()),
            Connections::V3(ref connection) => connection.remote_did()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_public_did(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => Ok(connection.set_their_public_did(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_public_did(handle: u32) -> VcxResult<Option<String>> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_public_did()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_their_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => Ok(connection.get_their_pw_verkey().to_string()),
            Connections::V3(ref connection) => connection.remote_vk()
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_their_pw_verkey(handle: u32, did: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_their_pw_verkey(did)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_uuid(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_uuid().to_string()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_uuid(handle: u32, uuid: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_uuid(uuid)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

// TODO: Add NO_ENDPOINT error to connection error
pub fn get_endpoint(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_endpoint().to_string()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::NoEndpoint)))
}

pub fn set_endpoint(handle: u32, endpoint: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_endpoint(endpoint)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_agent_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_agent_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().agent_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_version(handle: u32) -> VcxResult<Option<ProtocolTypes>> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_version()),
            Connections::V3(_) => Ok(Some(settings::get_protocol_type()))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_agent_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_agent_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_pw_verkey(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_pw_verkey().clone()),
            Connections::V3(ref connection) => Ok(connection.agent_info().pw_vk.clone())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_pw_verkey(handle: u32, verkey: &str) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_pw_verkey(verkey).clone()),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_state(handle: u32) -> u32 {
    CONNECTION_MAP.get(handle, |cxn| {
        debug!("get state for connection");
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_state()),
            Connections::V3(ref connection) => Ok(connection.state())
        }
    }).unwrap_or(0)
}


pub fn set_state(handle: u32, state: VcxStateType) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |cxn| {
        match cxn {
            Connections::V1(ref mut connection) => Ok(connection.set_state(state)),
            Connections::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(ref connection) => Ok(connection.get_source_id().clone()),
            Connections::V3(ref connection) => Ok(connection.get_source_id())
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

fn store_connection(connection: Connections) -> VcxResult<u32> {
    CONNECTION_MAP.add(connection)
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}
pub fn create_test_connection(did: String, verkey: String, label: String, endpoint: String) -> VcxResult<u32> {
    if settings::ARIES_COMMUNICATION_METHOD.to_string() != settings::get_communication_method().unwrap_or_default() {
        return Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Cannot create test connection for not `aries` communication method"));
    }

    let connection = ConnectionV3::create_test_connection(did, verkey, label, endpoint)?;
    CONNECTION_MAP.add(Connections::V3(connection))
        .or(Err(VcxError::from(VcxErrorKind::CreateConnection)))
}

pub fn get_connection_info(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |cxn| {
        match cxn {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref connection) => connection.get_connection_info()
        }
    })
}

fn create_connection_v1(source_id: &str) -> VcxResult<Connection> {
    let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();

    let (pw_did, pw_verkey) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

    Ok(Connection {
        source_id: source_id.to_string(),
        pw_did,
        pw_verkey,
        state: VcxStateType::VcxStateInitialized,
        uuid: String::new(),
        endpoint: String::new(),
        invite_detail: None,
        redirect_detail: None,
        invite_url: None,
        agent_did: String::new(),
        agent_vk: String::new(),
        their_pw_did: String::new(),
        their_pw_verkey: String::new(),
        public_did: None,
        their_public_did: None,
        version: Some(settings::get_connecting_protocol_version()),
    })
}

pub fn create_connection(source_id: &str) -> VcxResult<u32> {
    trace!("create_connection >>> source_id: {}", source_id);

    // Initiate connection of new format -- redirect to v3 folder
    if settings::is_aries_protocol_set() {
        let connection = Connections::V3(ConnectionV3::create(source_id));
        return store_connection(connection);
    }

    let connection = create_connection_v1(source_id)?;

    store_connection(Connections::V1(connection))
}

pub fn create_connection_with_invite(source_id: &str, details: &str) -> VcxResult<u32> {
    debug!("create connection {} with invite {}", source_id, details);

    // Invitation of new format -- redirect to v3 folder
    if let Ok(invitation) = serde_json::from_str::<InvitationV3>(details) {
        let connection = Connections::V3(ConnectionV3::create_with_invite(source_id, invitation)?);
        return store_connection(connection);
    }

    let details: Value = serde_json::from_str(&details)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize invite details: {}", err)))?;

    let invite_details: InviteDetail = match serde_json::from_value(details.clone()) {
        Ok(x) => x,
        Err(_) => {
            // Try converting to abbreviated
            let details = unabbrv_event_detail(details)?;
            serde_json::from_value(details)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize invite details: {}", err)))?
        }
    };

    let mut connection = create_connection_v1(source_id)?;

    connection.set_their_pw_did(invite_details.sender_detail.did.as_str());
    connection.set_their_pw_verkey(invite_details.sender_detail.verkey.as_str());

    if let Some(did) = invite_details.sender_detail.public_did.as_ref() {
        connection.set_their_public_did(did);
    }

    connection.set_invite_detail(invite_details);
    connection.set_state(VcxStateType::VcxStateRequestReceived);

    store_connection(Connections::V1(connection))
}

pub fn parse_acceptance_details(message: &Message) -> VcxResult<SenderDetail> {
    let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let payload = message.payload
        .as_ref()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Payload not found"))?;

    match payload {
        MessagePayload::V1(payload) => {
            // TODO: check returned verkey
            let (_, payload) = crypto::parse_msg(&my_vk, &messages::to_u8(&payload))
                .map_err(|err| err.map(VcxErrorKind::InvalidMessagePack, "Cannot decrypt connection payload"))?;

            let response: ConnectionPayload = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse connection payload: {}", err)))?;

            let payload = messages::to_u8(&response.msg);

            let response: AcceptanceDetails = rmp_serde::from_slice(&payload[..])
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

            Ok(response.sender_detail)
        }
        MessagePayload::V2(payload) => {
            let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
            let response: AcceptanceDetails = serde_json::from_str(&payload.msg)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

            Ok(response.sender_detail)
        }
    }
}

impl Connection {
    pub fn parse_redirection_details(&self, message: &Message) -> VcxResult<RedirectDetail> {
        debug!("connection {} parsing redirect details for message {:?}", self.source_id, message);
        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Payload not found"))?;

        match payload {
            MessagePayload::V1(payload) => {
                // TODO: check returned verkey
                let (_, payload) = crypto::parse_msg(&my_vk, &messages::to_u8(&payload))
                    .map_err(|err| err.map(VcxErrorKind::InvalidMessagePack, "Cannot decrypt connection payload"))?;

                let response: ConnectionPayload = rmp_serde::from_slice(&payload[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot parse connection payload: {}", err)))?;

                let payload = messages::to_u8(&response.msg);

                let response: RedirectionDetails = rmp_serde::from_slice(&payload[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot deserialize RedirectDetails: {}", err)))?;

                Ok(response.redirect_detail)
            }
            MessagePayload::V2(payload) => {
                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: RedirectionDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RedirectDetails: {}", err)))?;

                Ok(response.redirect_detail)
            }
        }
    }

    pub fn force_v2_parse_acceptance_details(&mut self, message: &Message) -> VcxResult<SenderDetail> {
        debug!("forcing connection {} parsing acceptance details for message {:?}", self.source_id, message);
        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Payload not found"))?;

        match payload {
            MessagePayload::V1(payload) => {
                let vec = messages::to_u8(payload);
                let json: Value = serde_json::from_slice(&vec[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot deserialize SenderDetails: {}", err)))?;

                let payload = Payloads::decrypt_payload_v12(&my_vk, &json)?;
                let response: AcceptanceDetails = serde_json::from_value(payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

                self.set_their_pw_did(&response.sender_detail.did);
                self.set_their_pw_verkey(&response.sender_detail.verkey);
                self.set_state(VcxStateType::VcxStateAccepted);

                Ok(response.sender_detail)
            }
            MessagePayload::V2(payload) => {
                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: AcceptanceDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize AcceptanceDetails: {}", err)))?;

                Ok(response.sender_detail)
            }
        }
    }
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => connection.send_generic_message(&msg, &msg_options),
            Connections::V3(ref connection) => connection.send_generic_message(msg, msg_options)
        }
    })
}

pub fn update_state_with_message(handle: u32, message: Message) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                if message.status_code == MessageStatusCode::Redirected && message.msg_type == RemoteMessageType::ConnReqRedirect {
                    connection.process_redirect_message(&message)
                } else {
                    connection.process_acceptance_message(&message)
                }
            }
            Connections::V3(ref mut connection) => {
                connection.update_state(Some(&json!(message).to_string()))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

impl Connection {
    pub fn force_v2_parse_redirection_details(&mut self, message: &Message) -> VcxResult<RedirectDetail> {
        debug!("forcing connection {} parsing redirection details for message {:?}", self.source_id, message);
        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        let payload = message.payload
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Payload not found"))?;

        match payload {
            MessagePayload::V1(payload) => {
                let vec = messages::to_u8(payload);
                let json: Value = serde_json::from_slice(&vec[..])
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Cannot deserialize SenderDetails: {}", err)))?;

                let payload = Payloads::decrypt_payload_v12(&my_vk, &json)?;
                let response: RedirectionDetails = serde_json::from_value(payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RedirectionDetails: {}", err)))?;

                self.set_redirect_detail(response.redirect_detail.clone());
                self.set_state(VcxStateType::VcxStateRedirected);

                Ok(response.redirect_detail)
            }
            MessagePayload::V2(payload) => {
                let payload = Payloads::decrypt_payload_v2(&my_vk, &payload)?;
                let response: RedirectionDetails = serde_json::from_str(&payload.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize RedirectionDetails: {}", err)))?;

                Ok(response.redirect_detail)
            }
        }
    }
}

pub fn update_state(handle: u32, message: Option<String>) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.update_state(message.clone())
            }
            Connections::V3(ref mut connection) => {
                connection.update_state(message.as_ref().map(String::as_str))?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

impl Connection {
    pub fn process_redirect_message(&mut self, message: &Message) -> VcxResult<u32> {
        let details = self.parse_redirection_details(&message)
            .map_err(|err| err.extend("Cannot parse redirection details"))?;

        self.set_redirect_detail(details);
        self.set_state(VcxStateType::VcxStateRedirected);

        Ok(error::SUCCESS.code_num)
    }
}

pub fn process_redirect_message(handle: u32, message: &Message) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.process_redirect_message(&message.clone())
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    })
}


pub fn delete_connection(handle: u32) -> VcxResult<u32> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.delete_connection()
            }
            Connections::V3(ref mut connection) => {
                connection.delete()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
        .map(|_| error::SUCCESS.code_num)
        .or(Err(VcxError::from(VcxErrorKind::DeleteConnection)))
        .and(release(handle))
        .and_then(|_| Ok(error::SUCCESS.code_num))
}

pub fn connect(handle: u32, options: Option<String>) -> VcxResult<u32> {
    let options_obj: ConnectionOptions = ConnectionOptions::from_opt_str(options)?;

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                debug!("establish connection {}", connection.source_id);
                connection.update_agent_profile(&options_obj)?;
                connection.create_agent_pairwise()?;
                connection.connect(&options_obj)
            }
            Connections::V3(ref mut connection) => {
                connection.connect()?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
}

pub fn redirect(handle: u32, redirect_handle: u32) -> VcxResult<u32> {
    let rc = CONNECTION_MAP.get(redirect_handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                Ok(connection.clone())
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    })?;

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                debug!("redirecting connection {}", connection.get_source_id());
                connection.update_agent_profile(&ConnectionOptions::default())?;
                connection.create_agent_pairwise()?;
                connection.redirect(&rc)
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    })
}


pub fn to_string(handle: u32) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                let object: SerializableObjectWithState<Connection, ConnectionV3> = SerializableObjectWithState::V1 { data: connection.to_owned() };

                ::serde_json::to_string(&object)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
            }
            Connections::V3(ref connection) => {
                let (data, state) = connection.to_owned().into();
                let object = SerializableObjectWithState::V2 { data, state };

                ::serde_json::to_string(&object)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize Connection: {:?}", err)))
            }
        }
    })
}

pub fn from_string(connection_data: &str) -> VcxResult<u32> {
    let object: SerializableObjectWithState<Connection, ::v3::handlers::connection::states::ActorDidExchangeState> = ::serde_json::from_str(connection_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Connection: {:?}", err)))?;

    let handle = match object {
        SerializableObjectWithState::V1 { data, .. } => {
            CONNECTION_MAP.add(Connections::V1(data))?
        }
        SerializableObjectWithState::V2 { data, state } => {
            CONNECTION_MAP.add(Connections::V3((data, state).into()))?
        }
    };

    Ok(handle)
}

pub fn release(handle: u32) -> VcxResult<()> {
    CONNECTION_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn release_all() {
    CONNECTION_MAP.drain().ok();
}

pub fn get_invite_details(handle: u32, abbreviated: bool) -> VcxResult<String> {
    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                match abbreviated {
                    false => {
                        serde_json::to_string(&connection.get_invite_detail())
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidInviteDetail, format!("Cannot serialize InviteDetail: {}", err)))
                    }
                    true => {
                        let details = serde_json::to_value(&connection.get_invite_detail())
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidInviteDetail, format!("Cannot serialize InviteDetail: {}", err)))?;
                        let abbr = abbrv_event_detail(details)?;
                        serde_json::to_string(&abbr)
                            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidInviteDetail, format!("Cannot serialize abbreviated InviteDetail: {}", err)))
                    }
                }
            }
            Connections::V3(ref connection) => {
                connection.get_invite_details()
            }
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_invite_details(handle: u32, invite_detail: &InviteDetail) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.set_invite_detail(invite_detail.clone());
                Ok(())
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
            }
        }
    })
        .or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn get_redirect_details(handle: u32) -> VcxResult<String> {
    debug!("get redirect details for connection {}", get_source_id(handle).unwrap_or_default());

    CONNECTION_MAP.get(handle, |connection| {
        match connection {
            Connections::V1(ref connection) => {
                serde_json::to_string(&connection.redirect_detail)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidRedirectDetail, format!("Cannot serialize RedirectDetail: {}", err)))
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn set_redirect_details(handle: u32, redirect_detail: &RedirectDetail) -> VcxResult<()> {
    debug!("set redirect details for connection {}", get_source_id(handle).unwrap_or_default());

    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(ref mut connection) => {
                connection.set_redirect_detail(redirect_detail.clone());
                Ok(())
            }
            Connections::V3(_) => {
                Err(VcxError::from(VcxErrorKind::ActionNotSupported))
            }
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}


//**********
// Code to convert InviteDetails to Abbreviated String
//**********

impl KeyMatch for (String, Option<String>) {
    fn matches(&self, key: &String, context: &Vec<String>) -> bool {
        if key.eq(&self.0) {
            match context.last() {
                Some(parent) => {
                    if let Some(ref expected_parent) = self.1 {
                        return parent.eq(expected_parent);
                    }
                }
                None => {
                    return self.1.is_none();
                }
            }
        }
        false
    }
}


lazy_static! {
    static ref ABBREVIATIONS: Vec<(String, String)> = {
        vec![
        ("statusCode".to_string(),          "sc".to_string()),
        ("connReqId".to_string(),           "id".to_string()),
        ("senderDetail".to_string(),        "s".to_string()),
        ("name".to_string(),                "n".to_string()),
        ("agentKeyDlgProof".to_string(),    "dp".to_string()),
        ("agentDID".to_string(),            "d".to_string()),
        ("agentDelegatedKey".to_string(),   "k".to_string()),
        ("signature".to_string(),           "s".to_string()),
        ("DID".to_string(), "d".to_string()),
        ("logoUrl".to_string(), "l".to_string()),
        ("verKey".to_string(), "v".to_string()),
        ("senderAgencyDetail".to_string(), "sa".to_string()),
        ("endpoint".to_string(), "e".to_string()),
        ("targetName".to_string(), "t".to_string()),
        ("statusMsg".to_string(), "sm".to_string()),
        ]
    };
}

lazy_static! {
    static ref UNABBREVIATIONS: Vec<((String, Option<String>), String)> = {
        vec![
        (("sc".to_string(), None),                                  "statusCode".to_string()),
        (("id".to_string(), None),                                  "connReqId".to_string()),
        (("s".to_string(), None),                                   "senderDetail".to_string()),
        (("n".to_string(), Some("senderDetail".to_string())),       "name".to_string()),
        (("dp".to_string(), Some("senderDetail".to_string())),      "agentKeyDlgProof".to_string()),
        (("d".to_string(), Some("agentKeyDlgProof".to_string())),   "agentDID".to_string()),
        (("k".to_string(), Some("agentKeyDlgProof".to_string())),   "agentDelegatedKey".to_string()),
        (("s".to_string(), Some("agentKeyDlgProof".to_string())),   "signature".to_string()),
        (("d".to_string(), Some("senderDetail".to_string())),       "DID".to_string()),
        (("l".to_string(), Some("senderDetail".to_string())),       "logoUrl".to_string()),
        (("v".to_string(), Some("senderDetail".to_string())),       "verKey".to_string()),
        (("sa".to_string(), None),                                  "senderAgencyDetail".to_string()),
        (("d".to_string(), Some("senderAgencyDetail".to_string())), "DID".to_string()),
        (("v".to_string(), Some("senderAgencyDetail".to_string())), "verKey".to_string()),
        (("e".to_string(), Some("senderAgencyDetail".to_string())), "endpoint".to_string()),
        (("t".to_string(), None),                                   "targetName".to_string()),
        (("sm".to_string(), None),                                  "statusMsg".to_string()),
        ]
    };
}

fn abbrv_event_detail(val: Value) -> VcxResult<Value> {
    mapped_key_rewrite(val, &ABBREVIATIONS)
}

fn unabbrv_event_detail(val: Value) -> VcxResult<Value> {
    mapped_key_rewrite(val, &UNABBREVIATIONS)
        .map_err(|err| err.extend("Cannot unabbreviate event detail"))
}

impl Into<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn into(self) -> (Connection, ActorDidExchangeState) {
        let data = Connection {
            source_id: self.source_id().clone(),
            pw_did: self.agent_info().pw_did.clone(),
            pw_verkey: self.agent_info().pw_vk.clone(),
            state: VcxStateType::from_u32(self.state()),
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            redirect_detail: None,
            invite_url: None,
            agent_did: self.agent_info().agent_did.clone(),
            agent_vk: self.agent_info().agent_vk.clone(),
            their_pw_did: self.remote_did().unwrap_or_default(),
            their_pw_verkey: self.remote_vk().unwrap_or_default(),
            public_did: None,
            their_public_did: None,
            version: Some(ProtocolTypes::V2), // TODO check correctness
        };

        (data, self.state_object().to_owned())
    }
}

impl From<(Connection, ActorDidExchangeState)> for ConnectionV3 {
    fn from((connection, state): (Connection, ActorDidExchangeState)) -> ConnectionV3 {
        let agent_info = AgentInfo {
            pw_did: connection.get_pw_did().to_string(),
            pw_vk: connection.get_pw_verkey().to_string(),
            agent_did: connection.get_agent_did().to_string(),
            agent_vk: connection.get_agent_verkey().to_string(),
        };

        ConnectionV3::from_parts(connection.get_source_id().to_string(), agent_info, state)
    }
}

use v3::messages::a2a::{A2AMessage, MessageId};
use v3::messages::connection::did_doc::DidDoc;

pub fn get_messages(handle: u32) -> VcxResult<HashMap<String, A2AMessage>> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_messages(),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn update_message_status(handle: u32, uid: String) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.update_message_status(uid.clone()),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn get_message_by_id(handle: u32, msg_id: String) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.get_message_by_id(&msg_id),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn decode_message(handle: u32, message: Message) -> VcxResult<A2AMessage> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V3(ref mut connection) => connection.decode_message(&message),
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
        }
    })
}

pub fn send_message(handle: u32, message: A2AMessage) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)),
            Connections::V3(ref mut connection) => connection.send_message(&message)
        }
    })
}

pub fn send_message_to_self_endpoint(message: A2AMessage, did_doc: &DidDoc) -> VcxResult<()> {
    ConnectionV3::send_message_to_self_endpoint(&message, did_doc)
}

pub fn add_pending_messages(handle: u32, messages: HashMap<MessageId, String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)),
            Connections::V3(ref mut connection) => connection.add_pending_messages(messages.clone())
        }
    })
}

pub fn remove_pending_message(handle: u32, id: &MessageId) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)),
            Connections::V3(ref mut connection) => connection.remove_pending_message(id.clone())
        }
    })
}

pub fn is_v3_connection(connection_handle: u32) -> VcxResult<bool> {
    CONNECTION_MAP.get(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Ok(false),
            Connections::V3(_) => Ok(true)
        }
    }).or(Err(VcxError::from(VcxErrorKind::InvalidConnectionHandle)))
}

pub fn send_ping(connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => connection.send_ping(comment.clone())
        }
    })
}

pub fn send_discovery_features(connection_handle: u32, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
    CONNECTION_MAP.get_mut(connection_handle, |connection| {
        match connection {
            Connections::V1(_) => Err(VcxError::from(VcxErrorKind::ActionNotSupported)),
            Connections::V3(ref mut connection) => connection.send_discovery_features(query.clone(), comment.clone())
        }
    })
}

#[cfg(test)]
pub mod tests {
    use std::thread;
    use std::time::Duration;

    use messages::get_message::*;
    use utils::constants::*;
    use utils::constants::INVITE_DETAIL_STRING;

    use super::*;
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use utils::constants;

    pub fn build_test_connection() -> u32 {
        let handle = create_connection("alice").unwrap();
        connect(handle, Some("{}".to_string())).unwrap();
        handle
    }

    pub fn create_connected_connections() -> (u32, u32) {
        ::utils::devsetup::set_institution();

        let alice = create_connection("alice").unwrap();
        let my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let options = json!({"use_public_did": true}).to_string();

        connect(alice, Some(options)).unwrap();
        let details = get_invite_details(alice, false).unwrap();

        //BE CONSUMER AND ACCEPT INVITE FROM INSTITUTION
        ::utils::devsetup::set_consumer();

        let faber = create_connection_with_invite("faber", &details).unwrap();

        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber));

        connect(faber, Some("{}".to_string())).unwrap();
        let public_did = get_their_public_did(faber).unwrap().unwrap();
        assert_eq!(my_public_did, public_did);

        //BE INSTITUTION AND CHECK THAT INVITE WAS ACCEPTED
        ::utils::devsetup::set_institution();

        thread::sleep(Duration::from_millis(500));

        update_state(alice, None).unwrap();
        assert_eq!(VcxStateType::VcxStateAccepted as u32, get_state(alice));
        (faber, alice)
    }

    #[test]
    fn test_build_connection_failures_with_no_wallet() {
        let _setup = SetupDefaults::init();

        assert_eq!(create_connection("This Should Fail").unwrap_err().kind(), VcxErrorKind::InvalidWalletHandle);

        assert_eq!(create_connection_with_invite("This Should Fail", "BadDetailsFoobar").unwrap_err().kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_create_connection_agency_failure() {
        let _setup = SetupIndyMocks::init();

        let handle = create_connection("invalid").unwrap();
        let rc = connect(handle, None);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::PostMessageFailed);
    }

    #[test]
    fn test_create_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_connection").unwrap();

        assert_eq!(get_pw_did(handle).unwrap(), constants::DID);
        assert_eq!(get_pw_verkey(handle).unwrap(), constants::VERKEY);
        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, Some("{}".to_string())).unwrap();

        AgencyMock::set_next_response(GET_MESSAGES_INVITE_ACCEPTED_RESPONSE.to_vec());
        update_state(handle, None).unwrap();
        assert_eq!(get_state(handle), VcxStateType::VcxStateAccepted as u32);

        AgencyMock::set_next_response(DELETE_CONNECTION_RESPONSE.to_vec());
        assert_eq!(delete_connection(handle).unwrap(), 0);

        // This errors b/c we release handle in delete connection
        assert!(release(handle).is_err());
    }

    #[test]
    fn test_create_drop_create() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);
        let did1 = get_pw_did(handle).unwrap();

        release(handle).unwrap();

        let handle2 = create_connection("test_create_drop_create").unwrap();

        assert_eq!(get_state(handle2), VcxStateType::VcxStateInitialized as u32);
        let did2 = get_pw_did(handle2).unwrap();

        assert_ne!(handle, handle2);
        assert_eq!(did1, did2);

        release(handle2).unwrap();
    }

    #[test]
    fn test_connection_release_fails() {
        let _setup = SetupEmpty::init();

        let rc = release(1);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_get_state_fails() {
        let _setup = SetupEmpty::init();

        let state = get_state(1);
        assert_eq!(state, VcxStateType::VcxStateNone as u32);
    }

    #[test]
    fn test_get_string_fails() {
        let _setup = SetupEmpty::init();

        let rc = to_string(0);
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[test]
    fn test_get_qr_code_data() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_get_qr_code_data").unwrap();

        connect(handle, None).unwrap();

        let details = get_invite_details(handle, true).unwrap();
        assert!(details.contains("\"dp\":"));

        assert_eq!(get_invite_details(0, true).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());

        // Aries connection
        ::settings::set_config_value(::settings::COMMUNICATION_METHOD, "aries");

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let first_string = to_string(handle).unwrap();
        assert!(release(handle).is_ok());
        let handle = from_string(&first_string).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);

        assert!(release(handle).is_ok());
    }

    #[test]
    fn test_deserialize_existing() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let first_string = to_string(handle).unwrap();

        let handle = from_string(&first_string).unwrap();

        let _pw_did = get_pw_did(handle).unwrap();
        let second_string = to_string(handle).unwrap();

        assert_eq!(first_string, second_string);
    }

    #[test]
    fn test_retry_connection() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_serialize_deserialize").unwrap();

        assert_eq!(get_state(handle), VcxStateType::VcxStateInitialized as u32);

        connect(handle, None).unwrap();
        connect(handle, None).unwrap();
    }

    #[test]
    fn test_parse_redirect_details() {
        let _setup = SetupMocks::init();
        let test_name = "test_parse_acceptance_details";

        let response = Message {
            status_code: MessageStatusCode::Redirected,
            payload: Some(MessagePayload::V1(vec![-110, -109, -81, 99, 111, 110, 110, 82, 101, 113, 82, 101, 100, 105, 114, 101, 99, 116, -93, 49, 46, 48, -84, 105, 110, 100, 121, 46, 109, 115, 103, 112, 97, 99, 107, -36, 0, -24, -48, -111, -48, -105, -48, -74, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, -48, -39, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, -48, -64, -48, -74, 56, 88, 70, 104, 56, 121, 66, 122, 114, 112, 74, 81, 109, 78, 121, 90, 122, 103, 111, 84, 113, 66, -48, -39, 44, 69, 107, 86, 84, 97, 55, 83, 67, 74, 53, 83, 110, 116, 112, 89, 121, 88, 55, 67, 83, 98, 50, 112, 99, 66, 104, 105, 86, 71, 84, 57, 107, 87, 83, 97, 103, 65, 56, 97, 57, 84, 54, 57, 65, -48, -64, -48, -39, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61])),
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqRedirect,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let c = Connection {
            source_id: test_name.to_string(),
            pw_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            pw_verkey: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            state: VcxStateType::VcxStateOfferSent,
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            redirect_detail: None,
            invite_url: None,
            agent_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            agent_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            their_pw_did: String::new(),
            their_pw_verkey: String::new(),
            public_did: None,
            their_public_did: None,
            version: None,
        };

        c.parse_redirection_details(&response).unwrap();

        // test that it fails
        let bad_response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: None,
            // This will cause an error
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let e = c.parse_redirection_details(&bad_response).unwrap_err();
        // TODO: Refactor Error
        // TODO: Fix this test to be a correct Error Type
        assert_eq!(e.kind(), VcxErrorKind::InvalidMessagePack);
    }

    #[test]
    fn test_parse_acceptance_details() {
        let _setup = SetupMocks::init();

        let test_name = "test_parse_acceptance_details";

        let response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: Some(MessagePayload::V1(vec![-126, -91, 64, 116, 121, 112, 101, -125, -92, 110, 97, 109, 101, -83, 99, 111, 110, 110, 82, 101, 113, 65, 110, 115, 119, 101, 114, -93, 118, 101, 114, -93, 49, 46, 48, -93, 102, 109, 116, -84, 105, 110, 100, 121, 46, 109, 115, 103, 112, 97, 99, 107, -92, 64, 109, 115, 103, -36, 1, 53, -48, -127, -48, -84, 115, 101, 110, 100, 101, 114, 68, 101, 116, 97, 105, 108, -48, -125, -48, -93, 68, 73, 68, -48, -74, 67, 113, 85, 88, 113, 53, 114, 76, 105, 117, 82, 111, 100, 55, 68, 67, 52, 97, 86, 84, 97, 115, -48, -90, 118, 101, 114, 75, 101, 121, -48, -39, 44, 67, 70, 86, 87, 122, 118, 97, 103, 113, 65, 99, 117, 50, 115, 114, 68, 106, 117, 106, 85, 113, 74, 102, 111, 72, 65, 80, 74, 66, 111, 65, 99, 70, 78, 117, 49, 55, 113, 117, 67, 66, 57, 118, 71, -48, -80, 97, 103, 101, 110, 116, 75, 101, 121, 68, 108, 103, 80, 114, 111, 111, 102, -48, -125, -48, -88, 97, 103, 101, 110, 116, 68, 73, 68, -48, -74, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, -48, -79, 97, 103, 101, 110, 116, 68, 101, 108, 101, 103, 97, 116, 101, 100, 75, 101, 121, -48, -39, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, -48, -87, 115, 105, 103, 110, 97, 116, 117, 114, 101, -48, -39, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61])),
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let c = Connections::V1(Connection {
            source_id: test_name.to_string(),
            pw_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            pw_verkey: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            state: VcxStateType::VcxStateOfferSent,
            uuid: String::new(),
            endpoint: String::new(),
            invite_detail: None,
            redirect_detail: None,
            invite_url: None,
            agent_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            agent_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            their_pw_did: String::new(),
            their_pw_verkey: String::new(),
            public_did: None,
            their_public_did: None,
            version: None,
        });

        CONNECTION_MAP.add(c).unwrap();

        parse_acceptance_details(&response).unwrap();

        // test that it fails
        let bad_response = Message {
            status_code: MessageStatusCode::Accepted,
            payload: None,
            // This will cause an error
            sender_did: "H4FBkUidRG8WLsWa7M6P38".to_string(),
            uid: "yzjjywu".to_string(),
            msg_type: RemoteMessageType::ConnReqAnswer,
            ref_msg_id: None,
            delivery_details: Vec::new(),
            decrypted_payload: None,
        };

        let e = parse_acceptance_details(&bad_response).unwrap_err();
        // TODO: Refactor Error
        // TODO: Fix this test to be a correct Error Type
        assert_eq!(e.kind(), VcxErrorKind::InvalidMessagePack);
    }

    #[test]
    fn test_invite_detail_abbr() {
        let _setup = SetupEmpty::init();

        let un_abbr = json!({
          "statusCode":"MS-102",
          "connReqId":"yta2odh",
          "senderDetail":{
            "name":"ent-name",
            "agentKeyDlgProof":{
              "agentDID":"N2Uyi6SVsHZq1VWXuA3EMg",
              "agentDelegatedKey":"CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
              "signature":"/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg=="
            },
            "DID":"F2axeahCaZfbUYUcKefc3j",
            "logoUrl":"ent-logo-url",
            "verKey":"74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx"
          },
          "senderAgencyDetail":{
            "DID":"BDSmVkzxRYGE4HKyMKxd1H",
            "verKey":"6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
            "endpoint":"52.38.32.107:80/agency/msg"
          },
          "targetName":"there",
          "statusMsg":"message sent"
        });

        let abbr = json!({
          "sc":"MS-102",
          "id": "yta2odh",
          "s": {
            "n": "ent-name",
            "dp": {
              "d": "N2Uyi6SVsHZq1VWXuA3EMg",
              "k": "CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
              "s":
                "/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg==",
            },
            "d": "F2axeahCaZfbUYUcKefc3j",
            "l": "ent-logo-url",
            "v": "74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx",
          },
          "sa": {
            "d": "BDSmVkzxRYGE4HKyMKxd1H",
            "v": "6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
            "e": "52.38.32.107:80/agency/msg",
          },
          "t": "there",
          "sm":"message sent"
        });
        let processed = abbrv_event_detail(un_abbr.clone()).unwrap();
        assert_eq!(processed, abbr);
        let unprocessed = unabbrv_event_detail(processed).unwrap();
        assert_eq!(unprocessed, un_abbr);
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_connection("rel1").unwrap();
        let h2 = create_connection("rel2").unwrap();
        let h3 = create_connection("rel3").unwrap();
        let h4 = create_connection("rel4").unwrap();
        let h5 = create_connection("rel5").unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_create_with_valid_invite_details() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();
        connect(handle, None).unwrap();

        let handle_2 = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();
        connect(handle_2, None).unwrap();
    }

    #[test]
    fn test_process_acceptance_message() {
        let _setup = SetupMocks::init();

        let handle = create_connection("test_process_acceptance_message").unwrap();
        let message = serde_json::from_str(INVITE_ACCEPTED_RESPONSE).unwrap();
        assert_eq!(error::SUCCESS.code_num, update_state_with_message(handle, message).unwrap());
    }

    #[test]
    fn test_create_with_invalid_invite_details() {
        let _setup = SetupMocks::init();

        let bad_details = r#"{"id":"mtfjmda","s":{"d":"abc"},"l":"abc","n":"Evernym","v":"avc"},"sa":{"d":"abc","e":"abc","v":"abc"},"sc":"MS-101","sm":"message created","t":"there"}"#;
        let err = create_connection_with_invite("alice", &bad_details).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_void_functions_actually_have_results() {
        let _setup = SetupDefaults::init();

        assert_eq!(set_their_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_state(1, VcxStateType::VcxStateNone).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_their_pw_did(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_uuid(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_endpoint(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_agent_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        let invite_details: InviteDetail = serde_json::from_str(INVITE_DETAIL_STRING).unwrap();
        assert_eq!(set_invite_details(1, &invite_details).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        let redirect_details: RedirectDetail = serde_json::from_str(REDIRECT_DETAIL_STRING).unwrap();
        assert_eq!(set_redirect_details(1, &redirect_details).unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);

        assert_eq!(set_pw_verkey(1, "blah").unwrap_err().kind(), VcxErrorKind::InvalidConnectionHandle);
    }

    #[test]
    fn test_different_protocol_version() {
        let _setup = SetupMocks::init();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Ok(()),
                Connections::V3(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V1")),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();

        let handle = create_connection_with_invite("alice", INVITE_DETAIL_V3_STRING).unwrap();

        CONNECTION_MAP.get_mut(handle, |connection| {
            match connection {
                Connections::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "It is suppose to be V3")),
                Connections::V3(_) => Ok(()),
            }
        }).unwrap();

        let _serialized = to_string(handle).unwrap();
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_connection_redirection_real() {
        let _setup = SetupLibraryAgencyV1::init();

        //0. Create initial connection
        let (faber, alice) = ::connection::tests::create_connected_connections();

        //1. Faber sends another invite
        ::utils::devsetup::set_institution(); //Faber to Alice
        let alice2 = create_connection("alice2").unwrap();
        let my_public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let options = json!({"use_public_did": true}).to_string();
        connect(alice2, Some(options)).unwrap();
        let details_for_alice2 = get_invite_details(alice2, false).unwrap();
        println!("alice2 details: {}", details_for_alice2);

        //2. Alice receives (recognizes that there is already a connection), calls different api (redirect rather than regular connect)
        //BE CONSUMER AND REDIRECT INVITE FROM INSTITUTION
        ::utils::devsetup::set_consumer();
        let faber_duplicate = create_connection_with_invite("faber_duplicate", &details_for_alice2).unwrap();
        assert_eq!(VcxStateType::VcxStateRequestReceived as u32, get_state(faber_duplicate));
        redirect(faber_duplicate, faber).unwrap();
        let public_did = get_their_public_did(faber_duplicate).unwrap().unwrap();
        assert_eq!(my_public_did, public_did);

        //3. Faber waits for redirect state change
        //BE INSTITUTION AND CHECK THAT INVITE WAS ACCEPTED
        ::utils::devsetup::set_institution();
        thread::sleep(Duration::from_millis(2000));
        update_state(alice2, None).unwrap();
        assert_eq!(VcxStateType::VcxStateRedirected as u32, get_state(alice2));

        //4. Faber calls 'get_redirect_data' and based on data, finds old connection  (business logic of enterprise)
        let redirect_data = get_redirect_details(alice2).unwrap();
        println!("redirect_data: {}", redirect_data);

        let rd: RedirectDetail = serde_json::from_str(&redirect_data).unwrap();
        let alice_serialized = to_string(alice).unwrap();

        let to_alice_old: Connection = ::messages::ObjectWithVersion::deserialize(&alice_serialized)
            .map(|obj: ::messages::ObjectWithVersion<Connection>| obj.data).unwrap();


        // Assert redirected data match old connection to alice
        assert_eq!(rd.did, to_alice_old.pw_did);
        assert_eq!(rd.verkey, to_alice_old.pw_verkey);
        assert_eq!(rd.public_did, to_alice_old.public_did);
        assert_eq!(rd.their_did, to_alice_old.their_pw_did);
        assert_eq!(rd.their_verkey, to_alice_old.their_pw_verkey);
        assert_eq!(rd.their_public_did, to_alice_old.their_public_did);
    }
}
