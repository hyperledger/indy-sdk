use settings;
use messages::*;
use messages::message_type::MessageTypes;
use messages::MessageStatusCode;
use messages::payload::Payloads;
use utils::{httpclient, constants};
use error::prelude::*;
use settings::ProtocolTypes;
use utils::httpclient::AgencyMock;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessages {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "excludePayload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uids: Option<Vec<String>>,
    #[serde(rename = "statusCodes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    status_codes: Option<Vec<MessageStatusCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pairwiseDIDs")]
    pairwise_dids: Option<Vec<String>>,
}

impl GetMessages {
    fn build(kind: A2AMessageKinds, exclude_payload: Option<String>, uids: Option<Vec<String>>,
             status_codes: Option<Vec<MessageStatusCode>>, pairwise_dids: Option<Vec<String>>) -> GetMessages {
        GetMessages {
            msg_type: MessageTypes::build(kind),
            exclude_payload,
            uids,
            status_codes,
            pairwise_dids,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    msgs: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "msgsByConns")]
    #[serde(default)]
    msgs: Vec<MessageByConnection>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageByConnection {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub msgs: Vec<Message>,
}

#[derive(Debug)]
pub struct GetMessagesBuilder {
    to_did: String,
    to_vk: String,
    agent_did: String,
    agent_vk: String,
    exclude_payload: Option<String>,
    uids: Option<Vec<String>>,
    status_codes: Option<Vec<MessageStatusCode>>,
    pairwise_dids: Option<Vec<String>>,
    version: ProtocolTypes,
}

impl GetMessagesBuilder {
    pub fn create() -> GetMessagesBuilder {
        trace!("GetMessages::create_message >>>");

        GetMessagesBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
            uids: None,
            exclude_payload: None,
            status_codes: None,
            pairwise_dids: None,
            version: settings::get_protocol_type(),
        }
    }

    #[cfg(test)]
    pub fn create_v1() -> GetMessagesBuilder {
        let mut builder = GetMessagesBuilder::create();
        builder.version = settings::ProtocolTypes::V1;
        builder
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> VcxResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn pairwise_dids(&mut self, pairwise_dids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.pairwise_dids = pairwise_dids;
        Ok(self)
    }

    pub fn include_edge_payload(&mut self, payload: &str) -> VcxResult<&mut Self> {
        //todo: is this a json value, String??
        self.exclude_payload = Some(payload.to_string());
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<Vec<Message>> {
        trace!("GetMessages::send >>>");

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        if settings::agency_mocks_enabled() && response.len() == 0 {
            return Ok(Vec::new());
        }

        self.parse_response(response)
    }

    fn parse_response(&self, response: Vec<u8>) -> VcxResult<Vec<Message>> {
        trace!("parse_get_messages_response >>>");

        let mut response = parse_response_from_agency(&response, &self.version)?;

        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::GetMessagesResponse(res)) => Ok(res.msgs),
            A2AMessage::Version2(A2AMessageV2::GetMessagesResponse(res)) => Ok(res.msgs),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesResponse"))
        }
    }

    pub fn download_messages(&mut self) -> VcxResult<Vec<MessageByConnection>> {
        trace!("GetMessages::download >>>");

        let data = self.prepare_download_request()?;

        let response = httpclient::post_u8(&data)?;

        if settings::agency_mocks_enabled() && response.len() == 0 {
            return Ok(Vec::new());
        }

        let response = self.parse_download_messages_response(response)?;

        Ok(response)
    }

    fn prepare_download_request(&self) -> VcxResult<Vec<u8>> {
        let message = match self.version {
            settings::ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::GetMessages(
                        GetMessages::build(A2AMessageKinds::GetMessagesByConnections,
                                           self.exclude_payload.clone(),
                                           self.uids.clone(),
                                           self.status_codes.clone(),
                                           self.pairwise_dids.clone()))
                ),
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetMessages(
                        GetMessages::build(A2AMessageKinds::GetMessagesByConnections,
                                           self.exclude_payload.clone(),
                                           self.uids.clone(),
                                           self.status_codes.clone(),
                                           self.pairwise_dids.clone()))
                ),

        };

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_download_messages_response(&self, response: Vec<u8>) -> VcxResult<Vec<MessageByConnection>> {
        trace!("parse_download_messages_response >>>");
        let mut response = parse_response_from_agency(&response, &self.version)?;

        trace!("parse_download_messages_response: parsed response {:?}", response);
        let msgs = match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::GetMessagesByConnectionsResponse(res)) => res.msgs,
            A2AMessage::Version2(A2AMessageV2::GetMessagesByConnectionsResponse(res)) => res.msgs,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of GetMessagesByConnectionsResponse"))
        };

        msgs
            .iter()
            .map(|connection| {
                ::utils::libindy::signus::get_local_verkey(&connection.pairwise_did)
                    .map(|vk| MessageByConnection {
                        pairwise_did: connection.pairwise_did.clone(),
                        msgs: connection.msgs.iter().map(|message| message.decrypt(&vk)).collect(),
                    })
            })
            .collect()
    }
}

//Todo: Every GeneralMessage extension, duplicates code
impl GeneralMessage for GetMessagesBuilder {
    type Msg = GetMessagesBuilder;

    fn set_agent_did(&mut self, did: String) { self.agent_did = did; }
    fn set_agent_vk(&mut self, vk: String) { self.agent_vk = vk; }
    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        let message = match self.version {
            settings::ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::GetMessages(
                        GetMessages::build(A2AMessageKinds::GetMessages,
                                           self.exclude_payload.clone(),
                                           self.uids.clone(),
                                           self.status_codes.clone(),
                                           self.pairwise_dids.clone()))
                ),
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetMessages(
                        GetMessages::build(A2AMessageKinds::GetMessages,
                                           self.exclude_payload.clone(),
                                           self.uids.clone(),
                                           self.status_codes.clone(),
                                           self.pairwise_dids.clone()))
                ),
        };

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetails {
    to: String,
    status_code: String,
    last_updated_date_time: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum MessagePayload {
    V1(Vec<i8>),
    V2(::serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "statusCode")]
    pub status_code: MessageStatusCode,
    pub payload: Option<MessagePayload>,
    #[serde(rename = "senderDID")]
    pub sender_did: String,
    pub uid: String,
    #[serde(rename = "type")]
    pub msg_type: RemoteMessageType,
    pub ref_msg_id: Option<String>,
    #[serde(skip_deserializing)]
    pub delivery_details: Vec<DeliveryDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypted_payload: Option<String>,
}

impl Message {
    pub fn payload<'a>(&'a self) -> VcxResult<Vec<u8>> {
        match self.payload {
            Some(MessagePayload::V1(ref payload)) => Ok(to_u8(payload)),
            Some(MessagePayload::V2(ref payload)) => serde_json::to_vec(payload).map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, err)),
            _ => Err(VcxError::from(VcxErrorKind::InvalidState)),
        }
    }

    pub fn decrypt(&self, vk: &str) -> Message {
        // TODO: must be Result
        let mut new_message = self.clone();
        if let Some(ref payload) = self.payload {
            let decrypted_payload = match payload {
                MessagePayload::V1(payload) => Payloads::decrypt_payload_v1(&vk, &payload)
                    .map(Payloads::PayloadV1),
                MessagePayload::V2(payload) => Payloads::decrypt_payload_v2(&vk, &payload)
                    .map(Payloads::PayloadV2)
            };

            if let Ok(decrypted_payload) = decrypted_payload {
                new_message.decrypted_payload = ::serde_json::to_string(&decrypted_payload).ok();
            } else if let Ok(decrypted_payload) = self._decrypt_v3_message() {
                new_message.decrypted_payload = ::serde_json::to_string(&json!(decrypted_payload)).ok()
            } else {
                new_message.decrypted_payload = ::serde_json::to_string(&json!(null)).ok();
            }
        }
        new_message.payload = None;
        new_message
    }

    fn _decrypt_v3_message(&self) -> VcxResult<::messages::payload::PayloadV1> {
        use v3::messages::a2a::A2AMessage;
        use v3::utils::encryption_envelope::EncryptionEnvelope;
        use ::issuer_credential::{CredentialOffer, CredentialMessage};
        use ::messages::payload::{PayloadTypes, PayloadV1, PayloadKinds};
        use std::convert::TryInto;

        let a2a_message = EncryptionEnvelope::open(self.payload()?)?;

        let (kind, msg) = match a2a_message {
            A2AMessage::PresentationRequest(presentation_request) => {
                let proof_req: ProofRequestMessage = presentation_request.try_into()?;

                (PayloadKinds::ProofRequest, json!(&proof_req).to_string())
            }
            A2AMessage::CredentialOffer(offer) => {
                let cred_offer: CredentialOffer = offer.try_into()?;

                (PayloadKinds::CredOffer, json!(vec![cred_offer]).to_string())
            }
            A2AMessage::Credential(credential) => {
                let credential: CredentialMessage = credential.try_into()?;

                (PayloadKinds::Cred, json!(&credential).to_string())
            }
            msg => {
                let msg = json!(&msg).to_string();
                (PayloadKinds::Other(String::from("aries")), msg)
            }
        };

        Ok(PayloadV1 {
            type_: PayloadTypes::build_v1(kind, "json"),
            msg,
        })
    }
}

pub fn get_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>, version: &Option<ProtocolTypes>) -> VcxResult<Vec<Message>> {
    trace!("get_connection_messages >>> pw_did: {}, pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           pw_did, pw_vk, agent_vk, msg_uid);

    let response = get_messages()
        .to(&pw_did)?
        .to_vk(&pw_vk)?
        .agent_did(&agent_did)?
        .agent_vk(&agent_vk)?
        .uid(msg_uid)?
        .status_codes(status_codes)?
        .version(version)?
        .send_secure()
        .map_err(|err| err.map(VcxErrorKind::PostMessageFailed, "Cannot get messages"))?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

pub fn get_ref_msg(msg_id: &str, pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str) -> VcxResult<(String, MessagePayload)> {
    trace!("get_ref_msg >>> msg_id: {}, pw_did: {}, pw_vk: {}, agent_did: {}, agent_vk: {}",
           msg_id, pw_did, pw_vk, agent_did, agent_vk);

    let message: Vec<Message> = get_connection_messages(pw_did, pw_vk, agent_did, agent_vk, Some(vec![msg_id.to_string()]), None, &None)?; // TODO: FIXME version should be param
    trace!("checking for ref_msg: {:?}", message);

    let msg_id = match message.get(0).as_ref().and_then(|message| message.ref_msg_id.as_ref()) {
        Some(ref ref_msg_id) if message[0].status_code == MessageStatusCode::Accepted => ref_msg_id.to_string(),
        _ => return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot find referent message")),
    };

    let message: Vec<Message> = get_connection_messages(pw_did, pw_vk, agent_did, agent_vk, Some(vec![msg_id]), None, &None)?;  // TODO: FIXME version should be param

    trace!("checking for pending message: {:?}", message);

    // this will work for both credReq and proof types
    match message.get(0).as_ref().and_then(|message| message.payload.as_ref()) {
        Some(payload) if message[0].status_code == MessageStatusCode::Received => {
            // TODO: check returned verkey
            Ok((message[0].uid.clone(), payload.to_owned()))
        }
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Cannot find referent message"))
    }
}

fn _parse_status_code(status_codes: Option<Vec<String>>) -> VcxResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code|
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code))
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse message status code: {}", err)))
                ).collect::<VcxResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None)
    }
}

pub fn download_messages(pairwise_dids: Option<Vec<String>>, status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> VcxResult<Vec<MessageByConnection>> {
    trace!("download_messages >>> pairwise_dids: {:?}, status_codes: {:?}, uids: {:?}",
           pairwise_dids, status_codes, uids);

    AgencyMock::set_next_response(constants::GET_ALL_MESSAGES_RESPONSE.to_vec());

    let status_codes = _parse_status_code(status_codes)?;

    let response =
        get_messages()
            .uid(uids)?
            .status_codes(status_codes)?
            .pairwise_dids(pairwise_dids)?
            .version(&Some(::settings::get_protocol_type()))?
            .download_messages()?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

pub fn download_agent_messages(status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> VcxResult<Vec<Message>> {
    trace!("download_messages >>> status_codes: {:?}, uids: {:?}", status_codes, uids);

    AgencyMock::set_next_response(constants::GET_ALL_MESSAGES_RESPONSE.to_vec());

    let status_codes = _parse_status_code(status_codes)?;

    let response =
        get_messages()
            .to(&::settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_DID)?)?
            .to_vk(&::settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?)?
            .agent_did(&::settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?)?
            .agent_vk(&::settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY)?)?
            .uid(uids)?
            .status_codes(status_codes)?
            .send_secure()?;

    trace!("message returned: {:?}", response);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::constants::{GET_MESSAGES_RESPONSE, GET_ALL_MESSAGES_RESPONSE};
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use std::thread;
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use std::time::Duration;
    use utils::devsetup::*;

    #[test]
    fn test_parse_get_messages_response() {
        let _setup = SetupMocks::init();

        let result = GetMessagesBuilder::create_v1().parse_response(GET_MESSAGES_RESPONSE.to_vec()).unwrap();
        assert_eq!(result.len(), 3)
    }

    #[test]
    fn test_parse_get_connection_messages_response() {
        let _setup = SetupMocks::init();

        let result = GetMessagesBuilder::create().version(&Some(ProtocolTypes::V1)).unwrap().parse_download_messages_response(GET_ALL_MESSAGES_RESPONSE.to_vec()).unwrap();
        assert_eq!(result.len(), 1)
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    #[ignore] // Dummy cloud agent has not implemented this functionality yet
    fn test_download_agent_messages() {
        let _setup = SetupLibraryAgencyV1::init();

        let (_faber, alice) = ::connection::tests::create_connected_connections();

        // AS CONSUMER GET MESSAGES
        ::utils::devsetup::set_consumer();
        let all_messages = download_agent_messages(None, None).unwrap();
        assert_eq!(all_messages.len(), 0);

        let _hello_uid = ::connection::send_generic_message(alice, "hello", &json!({"msg_type":"hello", "msg_title": "hello", "ref_msg_id": null}).to_string()).unwrap();
        thread::sleep(Duration::from_millis(2000));
        let all_messages = download_agent_messages(None, None).unwrap();
        assert_eq!(all_messages.len(), 1);

        let invalid_status_code = "abc".to_string();
        let bad_req = download_agent_messages(Some(vec![invalid_status_code]),  None);
        assert!(bad_req.is_err());
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_download_messages() {
        let _setup = SetupLibraryAgencyV1::init();

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (_faber, alice) = ::connection::tests::create_connected_connections();

        let (_, cred_def_handle) = ::credential_def::tests::create_cred_def_real(false);

        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let credential_offer = ::issuer_credential::issuer_credential_create(cred_def_handle,
                                                                             "1".to_string(),
                                                                             institution_did.clone(),
                                                                             "credential_name".to_string(),
                                                                             credential_data.to_owned(),
                                                                             1).unwrap();

        ::issuer_credential::send_credential_offer(credential_offer, alice).unwrap();

        thread::sleep(Duration::from_millis(1000));

        let hello_uid = ::connection::send_generic_message(alice, "hello", &json!({"msg_type":"hello", "msg_title": "hello", "ref_msg_id": null}).to_string()).unwrap();

        // AS CONSUMER GET MESSAGES
        ::utils::devsetup::set_consumer();

        let _all_messages = download_messages(None, None, None).unwrap();

        let pending = download_messages(None, Some(vec!["MS-103".to_string()]), None).unwrap();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].msgs[0].decrypted_payload.is_some());

        let accepted = download_messages(None, Some(vec!["MS-104".to_string()]), None).unwrap();
        assert_eq!(accepted[0].msgs.len(), 2);

        let specific = download_messages(None, None, Some(vec![accepted[0].msgs[0].uid.clone()])).unwrap();
        assert_eq!(specific.len(), 1);

        // No pending will return empty list
        let empty = download_messages(None, Some(vec!["MS-103".to_string()]), Some(vec![accepted[0].msgs[0].uid.clone()])).unwrap();
        assert_eq!(empty.len(), 1);

        let hello_msg = download_messages(None, None, Some(vec![hello_uid])).unwrap();
        assert_eq!(hello_msg[0].msgs[0].decrypted_payload, Some("{\"@type\":{\"name\":\"hello\",\"ver\":\"1.0\",\"fmt\":\"json\"},\"@msg\":\"hello\"}".to_string()));

        // Agency returns a bad request response for invalid dids
        let invalid_did = "abc".to_string();
        let bad_req = download_messages(Some(vec![invalid_did]), None, None);
        assert_eq!(bad_req.unwrap_err().kind(), VcxErrorKind::PostMessageFailed);
    }
}
