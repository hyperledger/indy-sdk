use messages::update_message::{UIDsByConn, update_messages as update_messages_status};
use messages::MessageStatusCode;
use messages::get_message::{Message, get_connection_messages};
use messages::update_connection::send_delete_connection_message;

use v3::messages::connection::did_doc::DidDoc;
use v3::messages::a2a::A2AMessage;

use v3::utils::encryption_envelope::EncryptionEnvelope;

use std::collections::HashMap;

use connection::create_agent_keys;
use utils::httpclient;
use utils::libindy::signus::create_and_store_my_did;
use settings;
use error::prelude::*;
use settings::ProtocolTypes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub pw_did: String,
    pub pw_vk: String,
    pub agent_did: String,
    pub agent_vk: String,
}

impl Default for AgentInfo {
    fn default() -> AgentInfo {
        AgentInfo {
            pw_did: String::new(),
            pw_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
        }
    }
}

impl AgentInfo {
    pub fn create_agent(&self) -> VcxResult<AgentInfo> {
        trace!("Agent::create_agent >>>");

        let method_name = settings::get_config_value(settings::CONFIG_DID_METHOD).ok();
        let (pw_did, pw_vk) = create_and_store_my_did(None, method_name.as_ref().map(String::as_str))?;

        /*
            Create User Pairwise Agent in old way.
            Send Messages corresponding to V2 Protocol to avoid code changes on Agency side.
        */
        let (agent_did, agent_vk) = create_agent_keys("", &pw_did, &pw_vk)?;

        Ok(AgentInfo { pw_did, pw_vk, agent_did, agent_vk })
    }

    pub fn agency_endpoint(&self) -> VcxResult<String> {
        settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)
            .map(|str| format!("{}/agency/msg", str))
    }

    pub fn routing_keys(&self) -> VcxResult<Vec<String>> {
        let agency_vk = settings::get_config_value(settings::CONFIG_AGENCY_VERKEY)?;
        Ok(vec![self.agent_vk.to_string(), agency_vk])
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        vec![self.pw_vk.to_string()]
    }

    pub fn update_message_status(&self, uid: String) -> VcxResult<()> {
        trace!("Agent::update_message_status >>> uid: {:?}", uid);

        let messages_to_update = vec![UIDsByConn {
            pairwise_did: self.pw_did.clone(),
            uids: vec![uid],
        }];

        update_messages_status(MessageStatusCode::Reviewed, messages_to_update)
    }

    pub fn get_messages(&self) -> VcxResult<HashMap<String, A2AMessage>> {
        trace!("Agent::get_messages >>>");

        let messages = get_connection_messages(&self.pw_did,
                                               &self.pw_vk,
                                               &self.agent_did,
                                               &self.agent_vk,
                                               None,
                                               Some(vec![MessageStatusCode::Received]),
                                               &Some(ProtocolTypes::V2))?;


        let mut a2a_messages: HashMap<String, A2AMessage> = HashMap::new();

        for message in messages {
            a2a_messages.insert(message.uid.clone(), self.decode_message(&message)?);
        }

        Ok(a2a_messages)
    }

    pub fn get_message_by_id(&self, msg_id: &str) -> VcxResult<A2AMessage> {
        trace!("Agent::get_message_by_id >>> msg_id: {:?}", msg_id);

        let mut messages = get_connection_messages(&self.pw_did,
                                                   &self.pw_vk,
                                                   &self.agent_did,
                                                   &self.agent_vk,
                                                   Some(vec![msg_id.to_string()]),
                                                   None,
                                                   &Some(ProtocolTypes::V2))?;

        let message =
            messages
                .pop()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message not found for id: {:?}", msg_id)))?;

        let message = self.decode_message(&message)?;

        Ok(message)
    }

    pub fn decode_message(&self, message: &Message) -> VcxResult<A2AMessage> {
        trace!("Agent::decode_message >>>");

        EncryptionEnvelope::open(message.payload()?)
    }

    pub fn send_message(&self, message: &A2AMessage, did_dod: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message >>> message: {:?}, did_doc: {:?}", message, did_dod);
        let envelope = EncryptionEnvelope::create(&message, Some(&self.pw_vk), &did_dod)?;
        httpclient::post_message(&envelope.0, &did_dod.get_endpoint())?;
        Ok(())
    }

    pub fn send_message_anonymously(message: &A2AMessage, did_dod: &DidDoc) -> VcxResult<()> {
        trace!("Agent::send_message_anonymously >>> message: {:?}, did_doc: {:?}", message, did_dod);
        let envelope = EncryptionEnvelope::create(&message, None, &did_dod)?;
        httpclient::post_message(&envelope.0, &did_dod.get_endpoint())?;
        Ok(())
    }

    pub fn delete(&self) -> VcxResult<()> {
        trace!("Agent::delete >>>");
        send_delete_connection_message(&self.pw_did, &self.pw_vk, &self.agent_did, &self.agent_vk)
    }
}