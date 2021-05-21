use settings;
use connection;
use api::VcxStateType;
use messages::*;
use messages::message_type::MessageTypes;
use messages::payload::{Payloads, PayloadKinds};
use messages::thread::Thread;
use utils::{httpclient, constants};
use utils::uuid::uuid;
use error::prelude::*;
use utils::agent_info::get_agent_info;
use utils::httpclient::AgencyMock;

#[derive(Debug)]
pub struct SendMessageBuilder {
    mtype: RemoteMessageType,
    to_did: String,
    to_vk: String,
    agent_did: String,
    agent_vk: String,
    payload: Vec<u8>,
    ref_msg_id: Option<String>,
    status_code: MessageStatusCode,
    uid: Option<String>,
    title: Option<String>,
    detail: Option<String>,
    version: settings::ProtocolTypes,
}

impl SendMessageBuilder {
    pub fn create() -> SendMessageBuilder {
        trace!("SendMessage::create_message >>>");

        SendMessageBuilder {
            mtype: RemoteMessageType::Other(String::new()),
            to_did: String::new(),
            to_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
            payload: Vec::new(),
            ref_msg_id: None,
            status_code: MessageStatusCode::Created,
            uid: None,
            title: None,
            detail: None,
            version: settings::get_protocol_type(),
        }
    }

    pub fn msg_type(&mut self, msg: &RemoteMessageType) -> VcxResult<&mut Self> {
        //Todo: validate msg??
        self.mtype = msg.clone();
        Ok(self)
    }

    pub fn uid(&mut self, uid: Option<&str>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uid = uid.map(String::from);
        Ok(self)
    }

    pub fn status_code(&mut self, code: &MessageStatusCode) -> VcxResult<&mut Self> {
        //Todo: validate that it can be parsed to number??
        self.status_code = code.clone();
        Ok(self)
    }

    pub fn edge_agent_payload(&mut self, my_vk: &str, their_vk: &str, data: &str, payload_type: PayloadKinds, thread: Option<Thread>) -> VcxResult<&mut Self> {
        //todo: is this a json value, String??
        self.payload = Payloads::encrypt(my_vk, their_vk, data, payload_type, thread, &self.version)?;
        Ok(self)
    }


    pub fn ref_msg_id(&mut self, id: Option<String>) -> VcxResult<&mut Self> {
        self.ref_msg_id = id;
        Ok(self)
    }

    pub fn set_title(&mut self, title: &str) -> VcxResult<&mut Self> {
        self.title = Some(title.to_string());
        Ok(self)
    }

    pub fn set_detail(&mut self, detail: &str) -> VcxResult<&mut Self> {
        self.detail = Some(detail.to_string());
        Ok(self)
    }

    pub fn version(&mut self, version: Option<settings::ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version,
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<SendResponse> {
        trace!("SendMessage::send >>>");

        AgencyMock::set_next_response(constants::SEND_MESSAGE_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        let result = self.parse_response(response)?;

        Ok(result)
    }

    fn parse_response(&self, response: Vec<u8>) -> VcxResult<SendResponse> {
        let mut response = parse_response_from_agency(&response, &self.version)?;

        let index = match self.version {
            // TODO: THINK better
            settings::ProtocolTypes::V1 => {
                if response.len() <= 1 {
                    return Err(VcxError::from(VcxErrorKind::InvalidHttpResponse));
                }
                1
            }
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 |
            settings::ProtocolTypes::V4 => 0
        };

        match response.remove(index) {
            A2AMessage::Version1(A2AMessageV1::MessageSent(res)) =>
                Ok(SendResponse { uid: res.uid, uids: res.uids }),
            A2AMessage::Version2(A2AMessageV2::SendRemoteMessageResponse(res)) =>
                Ok(SendResponse { uid: Some(res.id.clone()), uids: if res.sent { vec![res.id] } else { vec![] } }),
            _ => Err(VcxError::from(VcxErrorKind::InvalidHttpResponse))
        }
    }
}

//Todo: Every GeneralMessage extension, duplicates code
impl GeneralMessage for SendMessageBuilder {
    type Msg = SendMessageBuilder;

    fn set_agent_did(&mut self, did: String) { self.agent_did = did; }
    fn set_agent_vk(&mut self, vk: String) { self.agent_vk = vk; }
    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        let messages =
            match self.version {
                settings::ProtocolTypes::V1 => {
                    let create = CreateMessage {
                        msg_type: MessageTypes::build_v1(A2AMessageKinds::CreateMessage),
                        mtype: self.mtype.clone(),
                        reply_to_msg_id: self.ref_msg_id.clone(),
                        send_msg: true,
                        uid: self.uid.clone(),
                    };
                    let detail = GeneralMessageDetail {
                        msg_type: MessageTypes::build_v1(A2AMessageKinds::MessageDetail),
                        msg: self.payload.clone(),
                        title: self.title.clone(),
                        detail: self.detail.clone(),
                    };
                    vec![A2AMessage::Version1(A2AMessageV1::CreateMessage(create)),
                         A2AMessage::Version1(A2AMessageV1::MessageDetail(MessageDetail::General(detail)))]
                }
                settings::ProtocolTypes::V2 |
                settings::ProtocolTypes::V3 |
                settings::ProtocolTypes::V4 => {
                    let msg: ::serde_json::Value = ::serde_json::from_slice(self.payload.as_slice())
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, err))?;

                    let message = SendRemoteMessage {
                        msg_type: MessageTypes::build_v2(A2AMessageKinds::SendRemoteMessage),
                        id: uuid(),
                        mtype: self.mtype.clone(),
                        reply_to_msg_id: self.ref_msg_id.clone(),
                        send_msg: true,
                        msg,
                        title: self.title.clone(),
                        detail: self.detail.clone(),
                    };
                    vec![A2AMessage::Version2(A2AMessageV2::SendRemoteMessage(message))]
                }
            };

        prepare_message_for_agent(messages, &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
    }
}

#[derive(Debug, PartialEq)]
pub struct SendResponse {
    uid: Option<String>,
    uids: Vec<String>,
}

impl SendResponse {
    pub fn get_msg_uid(&self) -> VcxResult<String> {
        self.uids
            .get(0)
            .map(|uid| uid.to_string())
            .ok_or(VcxError::from(VcxErrorKind::InvalidJson))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SendMessageOptions {
    pub msg_type: String,
    pub msg_title: String,
    pub ref_msg_id: Option<String>,
}

pub fn send_generic_message(connection_handle: u32, msg: &str, msg_options: &str) -> VcxResult<String> {
    if connection::get_state(connection_handle) != VcxStateType::VcxStateAccepted as u32 {
        return Err(VcxError::from(VcxErrorKind::NotReady));
    }

    let agent_info = get_agent_info()?.pw_info(connection_handle)?;

    let msg_options: SendMessageOptions = serde_json::from_str(msg_options).map_err(|_| {
        error!("Invalid SendMessage msg_options");
        VcxError::from(VcxErrorKind::InvalidConfiguration)
    })?;

    let response =
        send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Other(msg_options.msg_type.clone()))?
            .version(agent_info.version()?.clone())?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &msg,
                                PayloadKinds::Other(msg_options.msg_type.clone()),
                                None
            )?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .set_title(&msg_options.msg_title)?
            .set_detail(&msg_options.msg_title)?
            .ref_msg_id(msg_options.ref_msg_id.clone())?
            .status_code(&MessageStatusCode::Accepted)?
            .send_secure()?;

    let msg_uid = response.get_msg_uid()?;
    Ok(msg_uid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::constants::SEND_MESSAGE_RESPONSE;
    use utils::devsetup::*;

    #[test]
    fn test_msgpack() {
        let _setup = SetupMocks::init();

        let mut message = SendMessageBuilder {
            mtype: RemoteMessageType::CredOffer,
            to_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            to_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            agent_did: "8XFh8yBzrpJQmNyZzgoTqB".to_string(),
            agent_vk: "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A".to_string(),
            payload: vec![1, 2, 3, 4, 5, 6, 7, 8],
            ref_msg_id: Some("123".to_string()),
            status_code: MessageStatusCode::Created,
            uid: Some("123".to_string()),
            title: Some("this is the title".to_string()),
            detail: Some("this is the detail".to_string()),
            version: settings::get_protocol_type(),
        };

        /* just check that it doesn't panic */
        let _packed = message.prepare_request().unwrap();
    }

    #[test]
    fn test_parse_send_message_response() {
        let _setup = SetupMocks::init();

        let result = SendMessageBuilder::create().parse_response(SEND_MESSAGE_RESPONSE.to_vec()).unwrap();
        let expected = SendResponse {
            uid: None,
            uids: vec!["ntc2ytb".to_string()],
        };
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_send_message_bad_response() {
        let _setup = SetupMocks::init();

        let result = SendMessageBuilder::create().parse_response(::utils::constants::UPDATE_PROFILE_RESPONSE.to_vec());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_msg_uid() {
        let _setup = SetupDefaults::init();

        let test_val = "devin";
        let response = SendResponse {
            uid: None,
            uids: vec![test_val.to_string()],
        };

        let uid = response.get_msg_uid().unwrap();
        assert_eq!(test_val, uid);

        let response = SendResponse {
            uid: None,
            uids: vec![],
        };

        let uid = response.get_msg_uid().unwrap_err();
        assert_eq!(VcxErrorKind::InvalidJson, uid.kind());
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_send_generic_message() {
        let _setup = SetupLibraryAgencyV1::init();

        let (_faber, alice) = ::connection::tests::create_connected_connections();

        send_generic_message(alice, "this is the message", &json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap();

        ::utils::devsetup::set_consumer();
        let _all_messages = get_message::download_messages(None, None, None).unwrap();
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_send_message_and_download_response() {
        let _setup = SetupLibraryAgencyV1::init();

        let (faber, alice) = ::connection::tests::create_connected_connections();

        let msg_id = send_generic_message(alice, "this is the message", &json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap();

        ::utils::devsetup::set_consumer();
        let msg1 = get_message::download_messages(None, None, Some(vec![msg_id.clone()])).unwrap();
        println!("{}", serde_json::to_string(&msg1).unwrap());
        let msg_id_response = send_generic_message(faber, "this is the response", &json!({"msg_type":"response type", "msg_title": "test response", "ref_msg_id":msg_id}).to_string()).unwrap();

        ::utils::devsetup::set_institution();
        let msg1 = get_message::download_messages(None, None, Some(vec![msg_id.clone()])).unwrap();
        println!("{}", serde_json::to_string(&msg1).unwrap());

        let ref_msg_id = msg1[0].clone().msgs[0].clone().ref_msg_id.unwrap();
        assert_eq!(ref_msg_id, msg_id_response);

        let response = get_message::download_messages(None, None, Some(vec![ref_msg_id.clone()])).unwrap();
        println!("{}", serde_json::to_string(&response).unwrap());
        assert_eq!(response[0].clone().msgs[0].clone().msg_type, RemoteMessageType::Other("response type".to_string()));
    }

    #[test]
    fn test_send_generic_message_fails_with_invalid_connection() {
        let _setup = SetupMocks::init();

        let handle = ::connection::tests::build_test_connection();

        let err = send_generic_message(handle, "this is the message", &json!({"msg_type":"type", "msg_title": "title", "ref_msg_id":null}).to_string()).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::NotReady);
    }
}
