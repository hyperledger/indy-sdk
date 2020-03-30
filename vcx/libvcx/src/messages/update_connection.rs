use messages::*;
use messages::message_type::MessageTypes;
use settings;
use utils::httpclient;
use error::prelude::*;
use utils::httpclient::AgencyMock;
use utils::constants::DELETE_CONNECTION_RESPONSE;

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConnection {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionStatus {
    AlreadyConnected,
    NotConnected,
    Deleted,
}

impl Serialize for ConnectionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ConnectionStatus::AlreadyConnected => "CS-101",
            ConnectionStatus::NotConnected => "CS-102",
            ConnectionStatus::Deleted => "CS-103",
        };
        serde_json::Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConnectionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("CS-101") => Ok(ConnectionStatus::AlreadyConnected),
            Some("CS-102") => Ok(ConnectionStatus::NotConnected),
            Some("CS-103") => Ok(ConnectionStatus::Deleted),
            _ => Err(de::Error::custom("Unexpected message type."))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateConnectionResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "statusCode")]
    status_code: ConnectionStatus,
}

#[derive(Debug)]
pub struct DeleteConnectionBuilder {
    to_did: String,
    to_vk: String,
    status_code: ConnectionStatus,
    agent_did: String,
    agent_vk: String,
    version: settings::ProtocolTypes,
}

impl DeleteConnectionBuilder {
    pub fn create() -> DeleteConnectionBuilder {
        trace!("DeleteConnection::create_message >>>");

        DeleteConnectionBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            status_code: ConnectionStatus::Deleted,
            agent_did: String::new(),
            agent_vk: String::new(),
            version: settings::get_protocol_type(),
        }
    }

    pub fn version(&mut self, version: &Option<settings::ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("DeleteConnection::send >>>");

        AgencyMock::set_next_response(DELETE_CONNECTION_RESPONSE.to_vec());

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &Vec<u8>) -> VcxResult<()> {
        trace!("parse_create_keys_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::UpdateConnectionResponse(_)) => Ok(()),
            A2AMessage::Version2(A2AMessageV2::UpdateConnectionResponse(_)) => Ok(()),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of UpdateConnectionResponse"))
        }
    }
}

pub fn send_delete_connection_message(pw_did: &str, pw_verkey: &str, agent_did: &str, agent_vk: &str) -> VcxResult<()> {
    delete_connection()
        .to(pw_did)?
        .to_vk(pw_verkey)?
        .agent_did(agent_did)?
        .agent_vk(agent_vk)?
        .send_secure()
        .map_err(|err| err.extend("Cannot delete connection"))
}

//TODO Every GeneralMessage extension, duplicates code
impl GeneralMessage for DeleteConnectionBuilder {
    type Msg = DeleteConnectionBuilder;

    fn set_agent_did(&mut self, did: String) {
        self.agent_did = did;
    }

    fn set_agent_vk(&mut self, vk: String) {
        self.agent_vk = vk;
    }

    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        let message = match self.version {
            settings::ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::UpdateConnection(
                        UpdateConnection {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateConnectionStatus),
                            status_code: self.status_code.clone(),
                        }
                    )
                ),
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 =>
                A2AMessage::Version2(
                    A2AMessageV2::UpdateConnection(
                        UpdateConnection {
                            msg_type: MessageTypes::build(A2AMessageKinds::UpdateConnectionStatus),
                            status_code: self.status_code.clone(),
                        }
                    )
                )
        };

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::devsetup::SetupDefaults;

    #[test]
    fn test_deserialize_delete_connection_payload() {
        let _setup = SetupDefaults::init();

        let payload = vec![130, 165, 64, 116, 121, 112, 101, 130, 164, 110, 97, 109, 101, 179, 67, 79, 78, 78, 95, 83, 84, 65, 84, 85, 83, 95, 85, 80, 68, 65, 84, 69, 68, 163, 118, 101, 114, 163, 49, 46, 48, 170, 115, 116, 97, 116, 117, 115, 67, 111, 100, 101, 166, 67, 83, 45, 49, 48, 51];
        let msg_str = r#"{ "@type": { "name": "CONN_STATUS_UPDATED", "ver": "1.0" }, "statusCode": "CS-103" }"#;
        let delete_connection_payload: UpdateConnectionResponse = serde_json::from_str(&msg_str).unwrap();
        assert_eq!(delete_connection_payload, rmp_serde::from_slice(&payload).unwrap());
    }
}
