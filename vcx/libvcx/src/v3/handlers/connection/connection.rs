use messages::get_message::Message;
use error::prelude::*;

use v3::handlers::connection::states::{DidExchangeSM, Actor, ActorDidExchangeState, DidExchangeState, InvitedState};
use v3::handlers::connection::messages::DidExchangeMessages;
use v3::handlers::connection::agent::AgentInfo;
use v3::messages::a2a::A2AMessage;
use v3::messages::connection::invite::{Invitation, PairwiseInvitation, PublicInvitation};
use v3::messages::connection::request::Request;

use std::collections::HashMap;
use v3::messages::connection::did_doc::DidDoc;
use v3::messages::basic_message::message::BasicMessage;
use v3::messages::discovery::disclose::ProtocolDescriptor;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    connection_sm: DidExchangeSM
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn from_opt_str(options: &Option<String>) -> VcxResult<ConnectionOptions> {
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

impl Connection {
    pub fn create(source_id: &str) -> Connection {
        trace!("Connection::create >>> source_id: {}", source_id);

        Connection {
            connection_sm: DidExchangeSM::new(Actor::Inviter, source_id),
        }
    }

    pub fn from_parts(source_id: String, agent_info: AgentInfo, state: ActorDidExchangeState) -> Connection {
        Connection { connection_sm: DidExchangeSM::from(source_id, agent_info, state) }
    }

    pub fn create_with_invite(source_id: &str, invitation: Invitation) -> VcxResult<Connection> {
        trace!("Connection::create_with_invite >>> source_id: {}", source_id);

        let mut connection = Connection {
            connection_sm: DidExchangeSM::new(Actor::Invitee, source_id),
        };

        connection.process_invite(invitation)?;

        Ok(connection)
    }

    pub fn create_connection_with_connection_request_message(source_id: &str, request_msg: &str) -> VcxResult<Connection> {
        let request_msg: A2AMessage = serde_json::from_str(request_msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize connection request message: {:?}", err)))?;
        match request_msg {
            request @ A2AMessage::ConnectionRequest(_) => match request.into() {
               DidExchangeMessages::ExchangeRequestReceived(request) => Connection::create_connection_with_connection_request(source_id, request),
                _ => Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize connection request message")))
            },
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize connection request message")))
        }
    }

    fn create_connection_with_connection_request(source_id: &str, request: Request) -> VcxResult<Connection> {
        trace!("Connection::create_with_conn_request >>> source_id: {}, request: {:?}", source_id, request);

        let pub_agent = AgentInfo::get_pub_agent()?;

        let pub_invite = PublicInvitation::create()
                             .set_label(source_id.to_string())
                             .set_public_did(String::from(&pub_agent.pw_did));

        let mut connection = Connection {
            connection_sm: DidExchangeSM::from(String::from(source_id), pub_agent, ActorDidExchangeState::Inviter(DidExchangeState::Invited(InvitedState { invitation: Invitation::Public(pub_invite) })))
        };

        connection.process_conn_request(request)?;

        Ok(connection)
    }

    pub fn create_connection_with_obtained_connection_request(source_id: &str) -> VcxResult<Option<Connection>> {
        let request = match Connection::find_request_message_to_handle()? {
            Some(message) => match message.into() {
               DidExchangeMessages::ExchangeRequestReceived(request) => Some(request),
               _ => None
            }
            None => None
        };
        match request {
            Some(request) => Connection::create_connection_with_connection_request(&source_id, request).map(|conn| Ok(Some(conn)))?,
            None => Ok(None)
        }
    }

    fn find_request_message_to_handle() -> VcxResult<Option<A2AMessage>> {
        for (_, message) in AgentInfo::get_pub_agent()?.get_messages()? {
            match message {
                request @ A2AMessage::ConnectionRequest(_) => {
                    debug!("Inviter received ConnectionRequest message");
                    return Ok(Some(request));
                }
              _ => {}
            }
        }
        Ok(None)
    }

    pub fn source_id(&self) -> String { self.connection_sm.source_id().to_string() }

    pub fn state(&self) -> u32 { self.connection_sm.state() }

    pub fn agent_info(&self) -> &AgentInfo { self.connection_sm.agent_info() }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.connection_sm.remote_did()
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.connection_sm.remote_vk()
    }

    pub fn state_object<'a>(&'a self) -> &'a ActorDidExchangeState {
        &self.connection_sm.state_object()
    }

    pub fn get_source_id(&self) -> String {
        self.connection_sm.source_id().to_string()
    }

    pub fn process_invite(&mut self, invitation: Invitation) -> VcxResult<()> {
        trace!("Connection::process_invite >>> invitation: {:?}", invitation);
        self.step(DidExchangeMessages::InvitationReceived(invitation))
    }

    pub fn process_conn_request(&mut self, request: Request) -> VcxResult<()> {
        trace!("Connection::process_conn_request >>> request: {:?}", request);
        self.step(DidExchangeMessages::ExchangeRequestReceived(request))
    }


    pub fn get_invite_details(&self) -> VcxResult<String> {
        trace!("Connection::get_invite_details >>>");
        if let Some(invitation) = self.connection_sm.get_invitation() {
            match invitation {
                Invitation::Pairwise(invitation) => Ok(json!(invitation.to_a2a_message()).to_string()),
                Invitation::Public(invitation) => Ok(json!(invitation.to_a2a_message()).to_string())
            }
        } else if let Some(did_doc) = self.connection_sm.did_doc() {
            Ok(json!(PairwiseInvitation::from(did_doc).to_a2a_message()).to_string())
        } else {
            Ok(json!({}).to_string())
        }
    }

    pub fn actor(&self) -> Actor {
        self.connection_sm.actor()
    }

    pub fn connect(&mut self, options: ConnectionOptions) -> VcxResult<()> {
        trace!("Connection::connect >>> source_id: {}, options: {:?}", self.connection_sm.source_id(), options);
        self.step(DidExchangeMessages::Connect())
    }

    pub fn update_state(&mut self, message: Option<&str>) -> VcxResult<()> {
        trace!("Connection::update_state >>> message: {:?}", message);

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let messages = self.get_messages()?;
        let agent_info = self.agent_info().clone();

        if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            agent_info.update_message_status(uid)?;
        };

        if let Some(prev_agent_info) = self.connection_sm.prev_agent_info().cloned() {
            let messages = prev_agent_info.get_messages()?;

            if let Some((uid, message)) = self.connection_sm.find_message_to_handle(messages) {
                self.handle_message(message.into())?;
                prev_agent_info.update_message_status(uid)?;
            }
        }

        Ok(())
    }

    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Connection::update_message_status >>> uid: {:?}", uid);
        self.connection_sm.agent_info().update_message_status(uid)
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<()> {
        trace!("Connection: update_state_with_message: {}", message);

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption,
                                              format!("Cannot updated state with messages: Message deserialization failed: {:?}", err)))?;

        self.handle_message(message.into())?;

        Ok(())
    }

    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Connection: get_messages >>>");
        self.agent_info().get_messages()
    }

    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Connection: get_message_by_id >>>");
        self.agent_info().get_message_by_id(msg_id)
    }

    pub fn handle_message(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        trace!("Connection: handle_message >>> {:?}", message);
        self.step(message)
    }

    pub fn decode_message(&self, message: &Message) -> VcxResult<A2AMessage> {
        match message.decrypted_payload {
            Some(ref payload) => {
                let message: ::messages::payload::PayloadV1 = ::serde_json::from_str(&payload)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize message: {}", err)))?;

                ::serde_json::from_str::<A2AMessage>(&message.msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))
            }
            None => self.agent_info().decode_message(message)
        }
    }

    pub fn send_message(&self, message: &A2AMessage) -> VcxResult<()> {
        trace!("Connection::send_message >>> message: {:?}", message);

        let did_doc = self.connection_sm.did_doc()
            .ok_or(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot send message: Remote Connection information is not set"))?;

        self.agent_info().send_message(message, &did_doc)
    }

    pub fn send_message_to_self_endpoint(message: &A2AMessage, did_doc: &DidDoc) -> VcxResult<()> {
        trace!("Connection::send_message_to_self_endpoint >>> message: {:?}, did_doc: {:?}", message, did_doc);

        AgentInfo::send_message_anonymously(message, did_doc)
    }

    fn parse_generic_message(message: &str, _message_options: &str) -> A2AMessage {
        match ::serde_json::from_str::<A2AMessage>(message) {
            Ok(a2a_message) => a2a_message,
            Err(_) => {
                BasicMessage::create()
                    .set_content(message.to_string())
                    .set_time()
                    .to_a2a_message()
            }
        }
    }

    pub fn send_generic_message(&self, message: &str, _message_options: &str) -> VcxResult<String> {
        trace!("Connection::send_generic_message >>> message: {:?}", message);

        let message = Connection::parse_generic_message(message, _message_options);
        self.send_message(&message).map(|_| String::new())
    }

    pub fn send_ping(&mut self, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_ping >>> comment: {:?}", comment);
        self.handle_message(DidExchangeMessages::SendPing(comment))
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Connection: delete >>> {:?}", self.connection_sm.source_id());
        self.agent_info().delete()
    }

    fn step(&mut self, message: DidExchangeMessages) -> VcxResult<()> {
        self.connection_sm = self.connection_sm.clone().step(message)?;
        Ok(())
    }

    pub fn send_discovery_features(&mut self, query: Option<String>, comment: Option<String>) -> VcxResult<()> {
        trace!("Connection::send_discovery_features_query >>> query: {:?}, comment: {:?}", query, comment);
        self.handle_message(DidExchangeMessages::DiscoverFeatures((query, comment)))
    }

    pub fn get_connection_info(&self) -> VcxResult<String> {
        trace!("Connection::get_connection_info >>>");

        let agent_info = self.agent_info().clone();

        let current = SideConnectionInfo {
            did: agent_info.pw_did.clone(),
            recipient_keys: agent_info.recipient_keys().clone(),
            routing_keys: agent_info.routing_keys()?,
            service_endpoint: agent_info.agency_endpoint()?,
            protocols: Some(self.connection_sm.get_protocols()),
        };

        let remote = match self.connection_sm.did_doc() {
            Some(did_doc) =>
                Some(SideConnectionInfo {
                    did: did_doc.id.clone(),
                    recipient_keys: did_doc.recipient_keys(),
                    routing_keys: did_doc.routing_keys(),
                    service_endpoint: did_doc.get_endpoint(),
                    protocols: self.connection_sm.get_remote_protocols(),
                }),
            None => None
        };

        let connection_info = ConnectionInfo { my: current, their: remote };

        let connection_info_json = serde_json::to_string(&connection_info)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("Cannot serialize ConnectionInfo: {:?}", err)))?;

        return Ok(connection_info_json);
    }
}

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    my: SideConnectionInfo,
    their: Option<SideConnectionInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SideConnectionInfo {
    did: String,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocols: Option<Vec<ProtocolDescriptor>>,
}

#[cfg(test)]
mod tests {
    use v3::messages::a2a::A2AMessage;
    use v3::handlers::connection::connection::Connection;

    #[test]
    fn test_parse_generic_message_plain_string_should_be_parsed_as_basic_msg() -> Result<(), String> {
        let message = "Some plain text message";
        let result = Connection::parse_generic_message(message, "");
        match result {
            A2AMessage::BasicMessage(basic_msg) => {
                assert_eq!(basic_msg.content, message);
                Ok(())
            }
            other => Err(format!("Result is not BasicMessage, but: {:?}", other))
        }
    }

    #[test]
    fn test_parse_generic_message_json_msg_should_be_parsed_as_generic() -> Result<(), String> {
        let message = json!({
            "@id": "some id",
            "@type": "some type",
            "content": "some content"
        }).to_string();
        let result = Connection::parse_generic_message(&message, "");
        match result {
            A2AMessage::Generic(value) => {
                assert_eq!(value.to_string(), message);
                Ok(())
            }
            other => Err(format!("Result is not Generic, but: {:?}", other))
        }
    }
}
