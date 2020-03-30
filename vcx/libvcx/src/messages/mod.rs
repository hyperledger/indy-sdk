pub mod create_key;
pub mod invite;
pub mod validation;
pub mod get_message;
pub mod send_message;
pub mod update_profile;
pub mod proofs;
pub mod agent_utils;
pub mod update_connection;
pub mod update_message;
pub mod message_type;
pub mod payload;
#[macro_use]
pub mod thread;

use std::u8;
use settings;
use utils::libindy::crypto;
use self::create_key::{CreateKeyBuilder, CreateKey, CreateKeyResponse};
use self::update_connection::{DeleteConnectionBuilder, UpdateConnection, UpdateConnectionResponse};
use self::update_profile::{UpdateProfileDataBuilder, UpdateConfigs, UpdateConfigsResponse};
use self::invite::{
    SendInviteBuilder, ConnectionRequest, SendInviteMessageDetails, SendInviteMessageDetailsResponse, ConnectionRequestResponse,
    RedirectConnectionMessageDetails, ConnectionRequestRedirect, ConnectionRequestRedirectResponse,
    AcceptInviteBuilder, RedirectConnectionBuilder, ConnectionRequestAnswer, AcceptInviteMessageDetails, ConnectionRequestAnswerResponse
};
use self::get_message::{GetMessagesBuilder, GetMessages, GetMessagesResponse, MessagesByConnections};
use self::send_message::SendMessageBuilder;
use self::update_message::{UpdateMessageStatusByConnections, UpdateMessageStatusByConnectionsResponse};
use self::proofs::proof_request::ProofRequestMessage;
use self::agent_utils::{Connect, ConnectResponse, SignUp, SignUpResponse, CreateAgent, CreateAgentResponse, UpdateComMethod, ComMethodUpdated};
use self::message_type::*;
use error::prelude::*;

use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;
use settings::ProtocolTypes;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum A2AMessageV1 {
    /// routing
    Forward(Forward),

    /// onbording
    Connect(Connect),
    ConnectResponse(ConnectResponse),
    SignUp(SignUp),
    SignUpResponse(SignUpResponse),
    CreateAgent(CreateAgent),
    CreateAgentResponse(CreateAgentResponse),

    /// PW Connection
    CreateKey(CreateKey),
    CreateKeyResponse(CreateKeyResponse),

    CreateMessage(CreateMessage),
    MessageDetail(MessageDetail),
    MessageCreated(MessageCreated),
    MessageSent(MessageSent),

    GetMessages(GetMessages),
    GetMessagesResponse(GetMessagesResponse),
    GetMessagesByConnections(GetMessages),
    GetMessagesByConnectionsResponse(MessagesByConnections),

    UpdateConnection(UpdateConnection),
    UpdateConnectionResponse(UpdateConnectionResponse),
    UpdateMessageStatusByConnections(UpdateMessageStatusByConnections),
    UpdateMessageStatusByConnectionsResponse(UpdateMessageStatusByConnectionsResponse),

    /// Configs
    UpdateConfigs(UpdateConfigs),
    UpdateConfigsResponse(UpdateConfigsResponse),
    UpdateComMethod(UpdateComMethod),
    ComMethodUpdated(ComMethodUpdated),
}

impl<'de> Deserialize<'de> for A2AMessageV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        let message_type: MessageTypeV1 = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;

        match message_type.name.as_str() {
            "FWD" => {
                Forward::deserialize(value)
                    .map(A2AMessageV1::Forward)
                    .map_err(de::Error::custom)
            }
            "CONNECT" => {
                Connect::deserialize(value)
                    .map(A2AMessageV1::Connect)
                    .map_err(de::Error::custom)
            }
            "CONNECTED" => {
                ConnectResponse::deserialize(value)
                    .map(A2AMessageV1::ConnectResponse)
                    .map_err(de::Error::custom)
            }
            "SIGNUP" => {
                SignUp::deserialize(value)
                    .map(A2AMessageV1::SignUp)
                    .map_err(de::Error::custom)
            }
            "SIGNED_UP" => {
                SignUpResponse::deserialize(value)
                    .map(A2AMessageV1::SignUpResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_AGENT" => {
                CreateAgent::deserialize(value)
                    .map(A2AMessageV1::CreateAgent)
                    .map_err(de::Error::custom)
            }
            "AGENT_CREATED" => {
                CreateAgentResponse::deserialize(value)
                    .map(A2AMessageV1::CreateAgentResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_COM_METHOD" => {
                UpdateComMethod::deserialize(value)
                    .map(A2AMessageV1::UpdateComMethod)
                    .map_err(de::Error::custom)
            }
            "COM_METHOD_UPDATED" => {
                ComMethodUpdated::deserialize(value)
                    .map(A2AMessageV1::ComMethodUpdated)
                    .map_err(de::Error::custom)
            }
            "CREATE_KEY" => {
                CreateKey::deserialize(value)
                    .map(A2AMessageV1::CreateKey)
                    .map_err(de::Error::custom)
            }
            "KEY_CREATED" => {
                CreateKeyResponse::deserialize(value)
                    .map(A2AMessageV1::CreateKeyResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV1::GetMessages)
                    .map_err(de::Error::custom)
            }
            "MSGS" => {
                GetMessagesResponse::deserialize(value)
                    .map(A2AMessageV1::GetMessagesResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS_BY_CONNS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV1::GetMessagesByConnections)
                    .map_err(de::Error::custom)
            }
            "MSGS_BY_CONNS" => {
                MessagesByConnections::deserialize(value)
                    .map(A2AMessageV1::GetMessagesByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_MSG" => {
                CreateMessage::deserialize(value)
                    .map(A2AMessageV1::CreateMessage)
                    .map_err(de::Error::custom)
            }
            "MSG_DETAIL" => {
                MessageDetail::deserialize(value)
                    .map(A2AMessageV1::MessageDetail)
                    .map_err(de::Error::custom)
            }
            "MSG_CREATED" => {
                MessageCreated::deserialize(value)
                    .map(A2AMessageV1::MessageCreated)
                    .map_err(de::Error::custom)
            }
            "MSG_SENT" | "MSGS_SENT" => {
                MessageSent::deserialize(value)
                    .map(A2AMessageV1::MessageSent)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONN_STATUS" => {
                UpdateConnection::deserialize(value)
                    .map(A2AMessageV1::UpdateConnection)
                    .map_err(de::Error::custom)
            }
            "CONN_STATUS_UPDATED" => {
                UpdateConnectionResponse::deserialize(value)
                    .map(A2AMessageV1::UpdateConnectionResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_MSG_STATUS_BY_CONNS" => {
                UpdateMessageStatusByConnections::deserialize(value)
                    .map(A2AMessageV1::UpdateMessageStatusByConnections)
                    .map_err(de::Error::custom)
            }
            "MSG_STATUS_UPDATED_BY_CONNS" => {
                UpdateMessageStatusByConnectionsResponse::deserialize(value)
                    .map(A2AMessageV1::UpdateMessageStatusByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONFIGS" => {
                UpdateConfigs::deserialize(value)
                    .map(A2AMessageV1::UpdateConfigs)
                    .map_err(de::Error::custom)
            }
            "CONFIGS_UPDATED" => {
                UpdateConfigsResponse::deserialize(value)
                    .map(A2AMessageV1::UpdateConfigsResponse)
                    .map_err(de::Error::custom)
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum A2AMessageV2 {
    /// routing
    Forward(ForwardV2),

    /// onbording
    Connect(Connect),
    ConnectResponse(ConnectResponse),
    SignUp(SignUp),
    SignUpResponse(SignUpResponse),
    CreateAgent(CreateAgent),
    CreateAgentResponse(CreateAgentResponse),

    /// PW Connection
    CreateKey(CreateKey),
    CreateKeyResponse(CreateKeyResponse),
    ConnectionRequest(ConnectionRequest),
    ConnectionRequestResponse(ConnectionRequestResponse),

    SendRemoteMessage(SendRemoteMessage),
    SendRemoteMessageResponse(SendRemoteMessageResponse),

    GetMessages(GetMessages),
    GetMessagesResponse(GetMessagesResponse),
    GetMessagesByConnections(GetMessages),
    GetMessagesByConnectionsResponse(MessagesByConnections),

    ConnectionRequestAnswer(ConnectionRequestAnswer),
    ConnectionRequestAnswerResponse(ConnectionRequestAnswerResponse),

    ConnectionRequestRedirect(ConnectionRequestRedirect),
    ConnectionRequestRedirectResponse(ConnectionRequestRedirectResponse),

    UpdateConnection(UpdateConnection),
    UpdateConnectionResponse(UpdateConnectionResponse),
    UpdateMessageStatusByConnections(UpdateMessageStatusByConnections),
    UpdateMessageStatusByConnectionsResponse(UpdateMessageStatusByConnectionsResponse),

    /// config
    UpdateConfigs(UpdateConfigs),
    UpdateConfigsResponse(UpdateConfigsResponse),
    UpdateComMethod(UpdateComMethod),
    ComMethodUpdated(ComMethodUpdated),
}

impl<'de> Deserialize<'de> for A2AMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        let message_type: MessageTypeV2 = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;

        match message_type.type_.as_str() {
            "FWD" => {
                ForwardV2::deserialize(value)
                    .map(A2AMessageV2::Forward)
                    .map_err(de::Error::custom)
            }
            "CONNECT" => {
                Connect::deserialize(value)
                    .map(A2AMessageV2::Connect)
                    .map_err(de::Error::custom)
            }
            "CONNECTED" => {
                ConnectResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectResponse)
                    .map_err(de::Error::custom)
            }
            "SIGNUP" => {
                SignUp::deserialize(value)
                    .map(A2AMessageV2::SignUp)
                    .map_err(de::Error::custom)
            }
            "SIGNED_UP" => {
                SignUpResponse::deserialize(value)
                    .map(A2AMessageV2::SignUpResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_AGENT" => {
                CreateAgent::deserialize(value)
                    .map(A2AMessageV2::CreateAgent)
                    .map_err(de::Error::custom)
            }
            "AGENT_CREATED" => {
                CreateAgentResponse::deserialize(value)
                    .map(A2AMessageV2::CreateAgentResponse)
                    .map_err(de::Error::custom)
            }
            "CREATE_KEY" => {
                CreateKey::deserialize(value)
                    .map(A2AMessageV2::CreateKey)
                    .map_err(de::Error::custom)
            }
            "KEY_CREATED" => {
                CreateKeyResponse::deserialize(value)
                    .map(A2AMessageV2::CreateKeyResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV2::GetMessages)
                    .map_err(de::Error::custom)
            }
            "MSGS" => {
                GetMessagesResponse::deserialize(value)
                    .map(A2AMessageV2::GetMessagesResponse)
                    .map_err(de::Error::custom)
            }
            "GET_MSGS_BY_CONNS" => {
                GetMessages::deserialize(value)
                    .map(A2AMessageV2::GetMessagesByConnections)
                    .map_err(de::Error::custom)
            }
            "MSGS_BY_CONNS" => {
                MessagesByConnections::deserialize(value)
                    .map(A2AMessageV2::GetMessagesByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "CONN_REQUEST" => {
                ConnectionRequest::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequest)
                    .map_err(de::Error::custom)
            }
            "CONN_REQUEST_RESP" => {
                ConnectionRequestResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestResponse)
                    .map_err(de::Error::custom)
            }
            "CONN_REQUEST_ANSWER" => {
                ConnectionRequestAnswer::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestAnswer)
                    .map_err(de::Error::custom)
            }
            "ACCEPT_CONN_REQ" => {
                ConnectionRequestAnswer::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestAnswer)
                    .map_err(de::Error::custom)
            }
            "CONN_REQUEST_ANSWER_RESP" => {
                ConnectionRequestAnswerResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestAnswerResponse)
                    .map_err(de::Error::custom)
            }
            "ACCEPT_CONN_REQ_RESP" => {
                ConnectionRequestAnswerResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestAnswerResponse)
                    .map_err(de::Error::custom)
            }
            "REDIRECT_CONN_REQ" => {
                ConnectionRequestRedirect::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestRedirect)
                    .map_err(de::Error::custom)
            }
            "CONN_REQ_REDIRECTED" => {
                ConnectionRequestRedirectResponse::deserialize(value)
                    .map(A2AMessageV2::ConnectionRequestRedirectResponse)
                    .map_err(de::Error::custom)
            }
            "SEND_REMOTE_MSG" => {
                SendRemoteMessage::deserialize(value)
                    .map(A2AMessageV2::SendRemoteMessage)
                    .map_err(de::Error::custom)
            }
            "REMOTE_MSG_SENT" => {
                SendRemoteMessageResponse::deserialize(value)
                    .map(A2AMessageV2::SendRemoteMessageResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONN_STATUS" => {
                UpdateConnection::deserialize(value)
                    .map(A2AMessageV2::UpdateConnection)
                    .map_err(de::Error::custom)
            }
            "CONN_STATUS_UPDATED" => {
                UpdateConnectionResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateConnectionResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_MSG_STATUS_BY_CONNS" => {
                UpdateMessageStatusByConnections::deserialize(value)
                    .map(A2AMessageV2::UpdateMessageStatusByConnections)
                    .map_err(de::Error::custom)
            }
            "MSG_STATUS_UPDATED_BY_CONNS" => {
                UpdateMessageStatusByConnectionsResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateMessageStatusByConnectionsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_CONFIGS" => {
                UpdateConfigs::deserialize(value)
                    .map(A2AMessageV2::UpdateConfigs)
                    .map_err(de::Error::custom)
            }
            "CONFIGS_UPDATED" => {
                UpdateConfigsResponse::deserialize(value)
                    .map(A2AMessageV2::UpdateConfigsResponse)
                    .map_err(de::Error::custom)
            }
            "UPDATE_COM_METHOD" => {
                UpdateComMethod::deserialize(value)
                    .map(A2AMessageV2::UpdateComMethod)
                    .map_err(de::Error::custom)
            }
            "COM_METHOD_UPDATED" => {
                ComMethodUpdated::deserialize(value)
                    .map(A2AMessageV2::ComMethodUpdated)
                    .map_err(de::Error::custom)
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

#[derive(Debug)]
pub enum A2AMessage {
    Version1(A2AMessageV1),
    Version2(A2AMessageV2),
}

impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            A2AMessage::Version1(msg) => msg.serialize(serializer).map_err(ser::Error::custom),
            A2AMessage::Version2(msg) => msg.serialize(serializer).map_err(ser::Error::custom)
        }
    }
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        let message_type: MessageTypes = serde_json::from_value(value["@type"].clone()).map_err(de::Error::custom)?;

        match message_type {
            MessageTypes::MessageTypeV1(_) =>
                A2AMessageV1::deserialize(value)
                    .map(A2AMessage::Version1)
                    .map_err(de::Error::custom),
            MessageTypes::MessageTypeV2(_) =>
                A2AMessageV2::deserialize(value)
                    .map(A2AMessage::Version2 )
                    .map_err(de::Error::custom)
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Forward {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV1,
    #[serde(rename = "@fwd")]
    fwd: String,
    #[serde(rename = "@msg")]
    msg: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ForwardV2 {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV2,
    #[serde(rename = "@fwd")]
    fwd: String,
    #[serde(rename = "@msg")]
    msg: Value,
}

impl Forward {
    fn new(fwd: String, msg: Vec<u8>, version: ProtocolTypes) -> VcxResult<A2AMessage> {
        match version {
            settings::ProtocolTypes::V1 => {
                Ok(A2AMessage::Version1(A2AMessageV1::Forward(
                    Forward {
                        msg_type: MessageTypes::build_v1(A2AMessageKinds::Forward),
                        fwd,
                        msg,
                    }
                )))
            }
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 => {
                let msg = serde_json::from_slice(msg.as_slice())
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, err))?;

                Ok(A2AMessage::Version2(A2AMessageV2::Forward(
                    ForwardV2 {
                        msg_type: MessageTypes::build_v2(A2AMessageKinds::Forward),
                        fwd,
                        msg,
                    }
                )))
            }
        }
    }
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreateMessage {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV1,
    mtype: RemoteMessageType,
    #[serde(rename = "sendMsg")]
    send_msg: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "replyToMsgId")]
    reply_to_msg_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralMessageDetail {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV1,
    #[serde(rename = "@msg")]
    msg: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageCreated {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV1,
    pub uid: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageSent {
    #[serde(rename = "@type")]
    msg_type: MessageTypeV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(default)]
    pub uids: Vec<String>,
}

#[serde(untagged)]
#[derive(Debug, Deserialize, Serialize)]
pub enum MessageDetail {
    ConnectionRequestAnswer(AcceptInviteMessageDetails),
    ConnectionRequestRedirect(RedirectConnectionMessageDetails),
    ConnectionRequest(SendInviteMessageDetails),
    ConnectionRequestResp(SendInviteMessageDetailsResponse),
    General(GeneralMessageDetail),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRemoteMessage {
    #[serde(rename = "@type")]
    pub msg_type: MessageTypeV2,
    #[serde(rename = "@id")]
    pub id: String,
    pub mtype: RemoteMessageType,
    #[serde(rename = "replyToMsgId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_msg_id: Option<String>,
    #[serde(rename = "sendMsg")]
    pub send_msg: bool,
    #[serde(rename = "@msg")]
    msg: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRemoteMessageResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "@id")]
    pub id: String,
    pub sent: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoteMessageType {
    Other(String),
    ConnReq,
    ConnReqAnswer,
    ConnReqRedirect,
    CredOffer,
    CredReq,
    Cred,
    ProofReq,
    Proof,
}

impl Serialize for RemoteMessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            RemoteMessageType::ConnReq => "connReq",
            RemoteMessageType::ConnReqAnswer => "connReqAnswer",
            RemoteMessageType::ConnReqRedirect => "connReqRedirect",
            RemoteMessageType::CredOffer => "credOffer",
            RemoteMessageType::CredReq => "credReq",
            RemoteMessageType::Cred => "cred",
            RemoteMessageType::ProofReq => "proofReq",
            RemoteMessageType::Proof => "proof",
            RemoteMessageType::Other(_type) => _type,
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RemoteMessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("connReq") => Ok(RemoteMessageType::ConnReq),
            Some("connReqAnswer") | Some("CONN_REQ_ACCEPTED") => Ok(RemoteMessageType::ConnReqAnswer),
            Some("connReqRedirect") | Some("CONN_REQ_REDIRECTED") | Some("connReqRedirected")  => Ok(RemoteMessageType::ConnReqRedirect),
            Some("credOffer") => Ok(RemoteMessageType::CredOffer),
            Some("credReq") => Ok(RemoteMessageType::CredReq),
            Some("cred") => Ok(RemoteMessageType::Cred),
            Some("proofReq") => Ok(RemoteMessageType::ProofReq),
            Some("proof") => Ok(RemoteMessageType::Proof),
            Some(_type) => Ok(RemoteMessageType::Other(_type.to_string())),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageStatusCode {
    Created,
    Sent,
    Received,
    Accepted,
    Rejected,
    Reviewed,
    Redirected,
}

impl MessageStatusCode {
    pub fn message(&self) -> &'static str {
        match self {
            MessageStatusCode::Created => "message created",
            MessageStatusCode::Sent => "message sent",
            MessageStatusCode::Received => "message received",
            MessageStatusCode::Redirected => "message redirected",
            MessageStatusCode::Accepted => "message accepted",
            MessageStatusCode::Rejected => "message rejected",
            MessageStatusCode::Reviewed => "message reviewed",
        }
    }
}

impl std::string::ToString for MessageStatusCode {
    fn to_string(&self) -> String {
        match self {
            MessageStatusCode::Created => "MS-101",
            MessageStatusCode::Sent => "MS-102",
            MessageStatusCode::Received => "MS-103",
            MessageStatusCode::Accepted => "MS-104",
            MessageStatusCode::Rejected => "MS-105",
            MessageStatusCode::Reviewed => "MS-106",
            MessageStatusCode::Redirected => "MS-107",
        }.to_string()
    }
}

impl Serialize for MessageStatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = self.to_string();
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MessageStatusCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("MS-101") => Ok(MessageStatusCode::Created),
            Some("MS-102") => Ok(MessageStatusCode::Sent),
            Some("MS-103") => Ok(MessageStatusCode::Received),
            Some("MS-104") => Ok(MessageStatusCode::Accepted),
            Some("MS-105") => Ok(MessageStatusCode::Rejected),
            Some("MS-106") => Ok(MessageStatusCode::Reviewed),
            Some("MS-107") => Ok(MessageStatusCode::Redirected),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum A2AMessageKinds {
    Forward,
    Connect,
    Connected,
    SignUp,
    SignedUp,
    CreateAgent,
    AgentCreated,
    CreateKey,
    KeyCreated,
    CreateMessage,
    MessageDetail,
    MessageCreated,
    MessageSent,
    GetMessages,
    GetMessagesByConnections,
    Messages,
    UpdateMessageStatusByConnections,
    MessageStatusUpdatedByConnections,
    UpdateConnectionStatus,
    UpdateConfigs,
    ConfigsUpdated,
    UpdateComMethod,
    ComMethodUpdated,
    ConnectionRequest,
    ConnectionRequestAnswer,
    ConnectionRequestRedirect,
    SendRemoteMessage,
    SendRemoteMessageResponse,
}

impl A2AMessageKinds {
    pub fn family(&self) -> MessageFamilies {
        match self {
            A2AMessageKinds::Forward => MessageFamilies::Routing,
            A2AMessageKinds::Connect => MessageFamilies::Onboarding,
            A2AMessageKinds::Connected => MessageFamilies::Onboarding,
            A2AMessageKinds::CreateAgent => MessageFamilies::Onboarding,
            A2AMessageKinds::AgentCreated => MessageFamilies::Onboarding,
            A2AMessageKinds::SignUp => MessageFamilies::Onboarding,
            A2AMessageKinds::SignedUp => MessageFamilies::Onboarding,
            A2AMessageKinds::CreateKey => MessageFamilies::Pairwise,
            A2AMessageKinds::KeyCreated => MessageFamilies::Pairwise,
            A2AMessageKinds::CreateMessage => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageDetail => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageCreated => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageSent => MessageFamilies::Pairwise,
            A2AMessageKinds::GetMessages => MessageFamilies::Pairwise,
            A2AMessageKinds::GetMessagesByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::Messages => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateConnectionStatus => MessageFamilies::Pairwise,
            A2AMessageKinds::ConnectionRequest => MessageFamilies::Pairwise,
            A2AMessageKinds::ConnectionRequestAnswer => MessageFamilies::Pairwise,
            A2AMessageKinds::ConnectionRequestRedirect => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateMessageStatusByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::MessageStatusUpdatedByConnections => MessageFamilies::Pairwise,
            A2AMessageKinds::UpdateConfigs => MessageFamilies::Configs,
            A2AMessageKinds::ConfigsUpdated => MessageFamilies::Configs,
            A2AMessageKinds::UpdateComMethod => MessageFamilies::Configs,
            A2AMessageKinds::ComMethodUpdated => MessageFamilies::Configs,
            A2AMessageKinds::SendRemoteMessage => MessageFamilies::Routing,
            A2AMessageKinds::SendRemoteMessageResponse => MessageFamilies::Routing,
        }
    }

    pub fn name(&self) -> String {
        match self {
            A2AMessageKinds::Forward => "FWD".to_string(),
            A2AMessageKinds::Connect => "CONNECT".to_string(),
            A2AMessageKinds::Connected => "CONNECTED".to_string(),
            A2AMessageKinds::CreateAgent => "CREATE_AGENT".to_string(),
            A2AMessageKinds::AgentCreated => "AGENT_CREATED".to_string(),
            A2AMessageKinds::SignUp => "SIGNUP".to_string(),
            A2AMessageKinds::SignedUp => "SIGNED_UP".to_string(),
            A2AMessageKinds::CreateKey => "CREATE_KEY".to_string(),
            A2AMessageKinds::KeyCreated => "KEY_CREATED".to_string(),
            A2AMessageKinds::CreateMessage => "CREATE_MSG".to_string(),
            A2AMessageKinds::MessageDetail => "MSG_DETAIL".to_string(),
            A2AMessageKinds::MessageCreated => "MSG_CREATED".to_string(),
            A2AMessageKinds::MessageSent => "MSGS_SENT".to_string(),
            A2AMessageKinds::GetMessages => "GET_MSGS".to_string(),
            A2AMessageKinds::GetMessagesByConnections => "GET_MSGS_BY_CONNS".to_string(),
            A2AMessageKinds::UpdateMessageStatusByConnections => "UPDATE_MSG_STATUS_BY_CONNS".to_string(),
            A2AMessageKinds::MessageStatusUpdatedByConnections => "MSG_STATUS_UPDATED_BY_CONNS".to_string(),
            A2AMessageKinds::Messages => "MSGS".to_string(),
            A2AMessageKinds::UpdateConnectionStatus => "UPDATE_CONN_STATUS".to_string(),
            A2AMessageKinds::ConnectionRequest => "CONN_REQUEST".to_string(),
            A2AMessageKinds::ConnectionRequestAnswer => "ACCEPT_CONN_REQ".to_string(),
            A2AMessageKinds::ConnectionRequestRedirect => "REDIRECT_CONN_REQ".to_string(),
            A2AMessageKinds::UpdateConfigs => "UPDATE_CONFIGS".to_string(),
            A2AMessageKinds::ConfigsUpdated => "CONFIGS_UPDATED".to_string(),
            A2AMessageKinds::UpdateComMethod => "UPDATE_COM_METHOD".to_string(),
            A2AMessageKinds::ComMethodUpdated => "COM_METHOD_UPDATED".to_string(),
            A2AMessageKinds::SendRemoteMessage => "SEND_REMOTE_MSG".to_string(),
            A2AMessageKinds::SendRemoteMessageResponse => "REMOTE_MSG_SENT".to_string(),
        }
    }
}

pub fn prepare_message_for_agency(message: &A2AMessage, agency_did: &str, version: &ProtocolTypes) -> VcxResult<Vec<u8>> {
    match version {
        settings::ProtocolTypes::V1 => bundle_for_agency_v1(message, &agency_did),
        settings::ProtocolTypes::V2 |
        settings::ProtocolTypes::V3 => pack_for_agency_v2(message, agency_did)
    }
}

fn bundle_for_agency_v1(message: &A2AMessage, agency_did: &str) -> VcxResult<Vec<u8>> {
    let agent_vk = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY)?;
    let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let message = rmp_serde::to_vec_named(&message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::UnknownError, format!("Cannot encode message: {}", err)))?;

    let message = Bundled::create(message).encode()?;

    let message = crypto::prep_msg(&my_vk, &agent_vk, &message[..])?;

    prepare_forward_message(message, agency_did, ProtocolTypes::V1)
}

fn pack_for_agency_v2(message: &A2AMessage, agency_did: &str) -> VcxResult<Vec<u8>> {
    let agent_vk = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY)?;
    let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

    let message = ::serde_json::to_string(&message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize A2A message: {}", err)))?;

    let receiver_keys = ::serde_json::to_string(&vec![&agent_vk])
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize receiver keys: {}", err)))?;

    let message = crypto::pack_message(Some(&my_vk), &receiver_keys, message.as_bytes())?;

    prepare_forward_message(message, agency_did, ProtocolTypes::V2)
}

fn parse_response_from_agency(response: &Vec<u8>, version: &ProtocolTypes) -> VcxResult<Vec<A2AMessage>> {
    match version {
        settings::ProtocolTypes::V1 => parse_response_from_agency_v1(response),
        settings::ProtocolTypes::V2 |
        settings::ProtocolTypes::V3 => parse_response_from_agency_v2(response)
    }
}

fn parse_response_from_agency_v1(response: &Vec<u8>) -> VcxResult<Vec<A2AMessage>> {
    let verkey = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;
    let (_, data) = crypto::parse_msg(&verkey, &response)?;
    let bundle: Bundled<Vec<u8>> = bundle_from_u8(data)?;
    bundle.bundled
        .iter()
        .map(|msg| rmp_serde::from_slice(msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize response: {}", err))))
        .collect::<VcxResult<Vec<A2AMessage>>>()
}

pub fn parse_message_from_response(response: &Vec<u8>) -> VcxResult<String> {
    let unpacked_msg = crypto::unpack_message(&response[..])?;

    let message: Value = ::serde_json::from_slice(unpacked_msg.as_slice())
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize response: {}", err)))?;

    Ok(message["message"].as_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Cannot find `message` field on response"))?.to_string())
}

fn parse_response_from_agency_v2(response: &Vec<u8>) -> VcxResult<Vec<A2AMessage>> {
    let message = parse_message_from_response(response)?;

    let message: A2AMessage = serde_json::from_str(&message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize A2A message: {}", err)))?;

    Ok(vec![message])
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Bundled<T> {
    bundled: Vec<T>,
}

impl<T> Bundled<T> {
    pub fn create(bundled: T) -> Bundled<T> {
        let mut vec = Vec::new();
        vec.push(bundled);
        Bundled {
            bundled: vec,
        }
    }

    pub fn encode(&self) -> VcxResult<Vec<u8>> where T: serde::Serialize {
        rmp_serde::to_vec_named(self)
            .map_err(|err| {
                error!("Could not convert bundle to messagepack: {}", err);
                VcxError::from_msg(VcxErrorKind::InvalidMessagePack, format!("Could not encode bundle: {}", err))
            })
    }
}

pub fn try_i8_bundle(data: Vec<u8>) -> VcxResult<Bundled<Vec<u8>>> {
    let bundle: Bundled<Vec<i8>> =
        rmp_serde::from_slice(&data[..])
            .map_err(|_| {
                trace!("could not deserialize bundle with i8, will try u8");
                VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Could not deserialize bundle with i8")
            })?;

    let mut new_bundle: Bundled<Vec<u8>> = Bundled { bundled: Vec::new() };
    for i in bundle.bundled {
        let mut buf: Vec<u8> = Vec::new();
        for j in i { buf.push(j as u8); }
        new_bundle.bundled.push(buf);
    }
    Ok(new_bundle)
}

pub fn to_u8(bytes: &Vec<i8>) -> Vec<u8> {
    bytes.iter().map(|i| *i as u8).collect()
}

pub fn to_i8(bytes: &Vec<u8>) -> Vec<i8> {
    bytes.iter().map(|i| *i as i8).collect()
}

pub fn bundle_from_u8(data: Vec<u8>) -> VcxResult<Bundled<Vec<u8>>> {
    try_i8_bundle(data.clone())
        .or_else(|_| rmp_serde::from_slice::<Bundled<Vec<u8>>>(&data[..]))
        .map_err(|err| {
            error!("could not deserialize bundle with i8 or u8: {}", err);
            VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Could not deserialize bundle with i8 or u8")
        })
}

fn prepare_forward_message(message: Vec<u8>, did: &str, version: ProtocolTypes) -> VcxResult<Vec<u8>> {
    let agency_vk = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY)?;

    let message = Forward::new(did.to_string(), message, version)?;

    match message {
        A2AMessage::Version1(A2AMessageV1::Forward(msg)) => prepare_forward_message_for_agency_v1(&msg, &agency_vk),
        A2AMessage::Version2(A2AMessageV2::Forward(msg)) => prepare_forward_message_for_agency_v2(&msg, &agency_vk),
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidState, "Invalid message type"))
    }
}

fn prepare_forward_message_for_agency_v1(message: &Forward, agency_vk: &str) -> VcxResult<Vec<u8>> {
    let message = rmp_serde::to_vec_named(message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::UnknownError, format!("Cannot serialize Forward message: {}", err)))?;
    let message = Bundled::create(message).encode()?;
    crypto::prep_anonymous_msg(agency_vk, &message[..])
}

fn prepare_forward_message_for_agency_v2(message: &ForwardV2, agency_vk: &str) -> VcxResult<Vec<u8>> {
    let message = serde_json::to_string(message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize Forward message: {}", err)))?;

    let receiver_keys = serde_json::to_string(&vec![agency_vk])
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize receiver keys: {}", err)))?;

    crypto::pack_message(None, &receiver_keys, message.as_bytes())
}

pub fn prepare_message_for_agent(messages: Vec<A2AMessage>, pw_vk: &str, agent_did: &str, agent_vk: &str, version: &ProtocolTypes) -> VcxResult<Vec<u8>> {
    match version {
        settings::ProtocolTypes::V1 => prepare_message_for_agent_v1(messages, pw_vk, agent_did, agent_vk),
        settings::ProtocolTypes::V2 |
        settings::ProtocolTypes::V3 => prepare_message_for_agent_v2(messages, pw_vk, agent_did, agent_vk)
    }
}

fn prepare_message_for_agent_v1(messages: Vec<A2AMessage>, pw_vk: &str, agent_did: &str, agent_vk: &str) -> VcxResult<Vec<u8>> {
    let message = messages
        .iter()
        .map(|msg| rmp_serde::to_vec_named(msg))
        .collect::<Result<Vec<_>, _>>()
        .map(|msgs| Bundled { bundled: msgs })
        .and_then(|bundle| rmp_serde::to_vec_named(&bundle))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize A2A message: {}", err)))?;

    let message = crypto::prep_msg(&pw_vk, agent_vk, &message[..])?;

    /* forward to did */
    let message = Forward::new(agent_did.to_owned(), message, ProtocolTypes::V1)?;

    let to_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

    bundle_for_agency_v1(&message, &to_did)
}

fn prepare_message_for_agent_v2(messages: Vec<A2AMessage>, pw_vk: &str, agent_did: &str, agent_vk: &str) -> VcxResult<Vec<u8>> {
    let message = messages.get(0)
        .ok_or(VcxError::from_msg(VcxErrorKind::SerializationError, "Cannot get message"))?;

    let message = serde_json::to_string(message)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize A2A message: {}", err)))?;

    let receiver_keys = serde_json::to_string(&vec![&agent_vk])
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot receiver keys: {}", err)))?;

    let message = crypto::pack_message(Some(pw_vk), &receiver_keys, message.as_bytes())?;

    /* forward to did */
    let message = Forward::new(agent_did.to_owned(), message, ProtocolTypes::V2)?;

    let to_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

    pack_for_agency_v2(&message, &to_did)
}

pub trait GeneralMessage {
    type Msg;

    //todo: deserialize_message

    fn to(&mut self, to_did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(to_did)?;
        self.set_to_did(to_did.to_string());
        Ok(self)
    }

    fn to_vk(&mut self, to_vk: &str) -> VcxResult<&mut Self> {
        validation::validate_verkey(to_vk)?;
        self.set_to_vk(to_vk.to_string());
        Ok(self)
    }

    fn agent_did(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.set_agent_did(did.to_string());
        Ok(self)
    }

    fn agent_vk(&mut self, to_vk: &str) -> VcxResult<&mut Self> {
        validation::validate_verkey(to_vk)?;
        self.set_agent_vk(to_vk.to_string());
        Ok(self)
    }

    fn set_to_vk(&mut self, to_vk: String);
    fn set_to_did(&mut self, to_did: String);
    fn set_agent_did(&mut self, did: String);
    fn set_agent_vk(&mut self, vk: String);

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectWithVersion<'a, T> {
    pub version: &'a str,
    pub data: T
}

impl<'a, 'de, T> ObjectWithVersion<'a, T> where T: ::serde::Serialize + ::serde::de::DeserializeOwned {
    pub fn new(version: &'a str, data: T) -> ObjectWithVersion<'a, T> {
        ObjectWithVersion { version, data }
    }

    pub fn serialize(&self) -> VcxResult<String> {
        ::serde_json::to_string(self)
            .to_vcx(VcxErrorKind::InvalidState, "Cannot serialize object")
    }

    pub fn deserialize(data: &str) -> VcxResult<ObjectWithVersion<T>> where T: ::serde::de::DeserializeOwned {
        ::serde_json::from_str(data)
            .to_vcx(VcxErrorKind::InvalidJson, "Cannot deserialize object")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SerializableObjectWithState<T, P> {
    #[serde(rename = "1.0")]
    V1 { data: T },
    #[serde(rename = "2.0")]
    V2 { data: T, state: P },
}

pub fn create_keys() -> CreateKeyBuilder { CreateKeyBuilder::create() }

pub fn send_invite() -> SendInviteBuilder { SendInviteBuilder::create() }

pub fn delete_connection() -> DeleteConnectionBuilder { DeleteConnectionBuilder::create() }

pub fn accept_invite() -> AcceptInviteBuilder { AcceptInviteBuilder::create() }

pub fn redirect_connection() -> RedirectConnectionBuilder { RedirectConnectionBuilder::create() }

pub fn update_data() -> UpdateProfileDataBuilder { UpdateProfileDataBuilder::create() }

pub fn get_messages() -> GetMessagesBuilder { GetMessagesBuilder::create() }

pub fn send_message() -> SendMessageBuilder { SendMessageBuilder::create() }

pub fn proof_request() -> ProofRequestMessage { ProofRequestMessage::create() }

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::*;

    #[test]
    fn test_to_u8() {
        let _setup = SetupDefaults::init();

        let vec: Vec<i8> = vec![-127, -89, 98, 117, 110, 100, 108, 101, 100, -111, -36, 5, -74];

        let buf = to_u8(&vec);
        println!("new bundle: {:?}", buf);
    }

    #[test]
    fn test_to_i8() {
        let _setup = SetupDefaults::init();

        let vec: Vec<u8> = vec![129, 167, 98, 117, 110, 100, 108, 101, 100, 145, 220, 19, 13];
        let buf = to_i8(&vec);
        println!("new bundle: {:?}", buf);
    }

    #[test]
    fn test_general_message_null_parameters() {
        let _setup = SetupDefaults::init();

        let details = GeneralMessageDetail {
            msg_type: MessageTypeV1 {
                name: "Name".to_string(),
                ver: "1.0".to_string()
            },
            msg: vec![1, 2, 3],
            title: None,
            detail: None
        };

        let string: String = serde_json::to_string(&details).unwrap();
        assert!(!string.contains("title"));
        assert!(!string.contains("detail"));
    }

    #[test]
    fn test_create_message_null_parameters() {
        let _setup = SetupDefaults::init();

        let details = CreateMessage {
            msg_type: MessageTypeV1 {
                name: "Name".to_string(),
                ver: "1.0".to_string()
            },
            mtype: RemoteMessageType::ProofReq,
            send_msg: true,
            uid: None,
            reply_to_msg_id: None
        };

        let string: String = serde_json::to_string(&details).unwrap();
        assert!(!string.contains("uid"));
        assert!(!string.contains("replyToMsgId"));
    }
}
