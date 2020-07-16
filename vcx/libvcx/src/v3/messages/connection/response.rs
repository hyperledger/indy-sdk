use error::prelude::*;
use utils::libindy::crypto;
use base64;
use time;

use messages::thread::Thread;
use v3::messages::connection::did_doc::*;
use v3::messages::a2a::{A2AMessage, MessageId};
use v3::messages::a2a::message_family::MessageFamilies;
use v3::messages::a2a::message_type::MessageType;
use v3::messages::ack::PleaseAck;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Response {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    pub connection: ConnectionData,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: DidDoc,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SignedResponse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "connection~sig")]
    pub connection_sig: ConnectionSignature,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionSignature {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
    pub signature: String,
    pub sig_data: String,
    pub signer: String,
}

impl Response {
    pub fn create() -> Response {
        Response::default()
    }

    pub fn set_did(mut self, did: String) -> Response {
        self.connection.did = did.clone();
        self.connection.did_doc.set_id(did);
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Response {
        self.connection.did_doc.set_service_endpoint(service_endpoint);
        self
    }

    pub fn set_keys(mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) -> Response {
        self.connection.did_doc.set_keys(recipient_keys, routing_keys);
        self
    }

    pub fn encode(&self, key: &str) -> VcxResult<SignedResponse> {
        let connection_data = json!(self.connection).to_string();

        let now: u64 = time::get_time().sec as u64;

        let mut sig_data = now.to_be_bytes().to_vec();

        sig_data.extend(connection_data.as_bytes());

        let signature = crypto::sign(key, &sig_data)?;

        let sig_data = base64::encode_config(&sig_data, base64::URL_SAFE);

        let signature = base64::encode_config(&signature, base64::URL_SAFE);

        let connection_sig = ConnectionSignature {
            signature,
            sig_data,
            signer: key.to_string(),
            ..Default::default()
        };

        let signed_response = SignedResponse {
            id: self.id.clone(),
            thread: self.thread.clone(),
            connection_sig,
            please_ack: self.please_ack.clone(),
        };

        Ok(signed_response)
    }
}

please_ack!(Response);
threadlike!(Response);

impl SignedResponse {
    pub fn decode(self, key: &str) -> VcxResult<Response> {
        let signature = base64::decode_config(&self.connection_sig.signature.as_bytes(), base64::URL_SAFE)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot decode ConnectionResponse: {:?}", err)))?;

        let sig_data = base64::decode_config(&self.connection_sig.sig_data.as_bytes(), base64::URL_SAFE)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot decode ConnectionResponse: {:?}", err)))?;

        if !crypto::verify(&key, &sig_data, &signature)? {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "ConnectionResponse signature is invalid for original Invite recipient key"));
        }

        let sig_data = &sig_data[8..];

        let connection: ConnectionData = ::serde_json::from_slice(&sig_data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, err.to_string()))?;

        Ok(Response {
            id: self.id,
            thread: self.thread,
            connection,
            please_ack: self.please_ack,
        })
    }
}

a2a_message!(SignedResponse, ConnectionResponse);

impl Default for ConnectionSignature {
    fn default() -> ConnectionSignature {
        ConnectionSignature {
            msg_type: MessageType::build(MessageFamilies::Signature, "ed25519Sha512_single"),
            signature: String::new(),
            sig_data: String::new(),
            signer: String::new(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use v3::messages::connection::did_doc::tests::*;
    use utils::libindy::tests::test_setup;

    fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    fn _key() -> String {
        String::from("CnEDk9HrMnmiHXEV1WFgbVCRteYnPqsJwrTdcZaNhFVW")
    }

    pub fn _thread() -> Thread {
        Thread::new().set_thid(String::from("test_id"))
    }

    pub fn _thread_id() -> String {
        _thread().thid.unwrap()
    }

    pub fn _response() -> Response {
        Response {
            id: MessageId::id(),
            thread: _thread(),
            connection: ConnectionData {
                did: _did(),
                did_doc: _did_doc()
            },
            please_ack: None,
        }
    }

    pub fn _signed_response() -> SignedResponse {
        SignedResponse {
            id: MessageId::id(),
            thread: _thread(),
            connection_sig: ConnectionSignature {
                signature: String::from("yeadfeBWKn09j5XU3ITUE3gPbUDmPNeblviyjrOIDdVMT5WZ8wxMCxQ3OpAnmq1o-Gz0kWib9zr0PLsbGc2jCA=="),
                sig_data: String::from("MTU3MTg0NzQwM3siZGlkIjoiVnNLVjdnclIxQlVFMjltRzJGbTJrWCIsImRpZF9kb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImF1dGhlbnRpY2F0aW9uIjpbeyJwdWJsaWNLZXkiOiJWc0tWN2dyUjFCVUUyOW1HMkZtMmtYIzEiLCJ0eXBlIjoiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCJ9XSwiaWQiOiJWc0tWN2dyUjFCVUUyOW1HMkZtMmtYIiwicHVibGljS2V5IjpbeyJpZCI6IjEiLCJvd25lciI6IlZzS1Y3Z3JSMUJVRTI5bUcyRm0ya1giLCJwdWJsaWNLZXlCYXNlNTgiOiI3SjNYczhLUVV0U2ZNenB0ZVVLcThiNDg5bzdENFB4QVkxSjFKQUxDNDF6ayIsInR5cGUiOiJFZDI1NTE5VmVyaWZpY2F0aW9uS2V5MjAxOCJ9LHsiaWQiOiIyIiwib3duZXIiOiJWc0tWN2dyUjFCVUUyOW1HMkZtMmtYIiwicHVibGljS2V5QmFzZTU4IjoiSGV6Y2UyVVdNWjN3VWhWa2gyTGZLU3M4bkR6V3d6czJXaW43RXpOTjNZYVIiLCJ0eXBlIjoiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgifSx7ImlkIjoiMyIsIm93bmVyIjoiVnNLVjdnclIxQlVFMjltRzJGbTJrWCIsInB1YmxpY0tleUJhc2U1OCI6IjNMWXV4SkJKa25nRGJ2Smo0emp4MTNEQlVkWjJQOTZlTnlid2QybjlMOUFVIiwidHlwZSI6IkVkMjU1MTlWZXJpZmljYXRpb25LZXkyMDE4In1dLCJzZXJ2aWNlIjpbeyJpZCI6ImRpZDpleGFtcGxlOjEyMzQ1Njc4OWFiY2RlZmdoaTtkaWQtY29tbXVuaWNhdGlvbiIsInByaW9yaXR5IjowLCJyZWNpcGllbnRLZXlzIjpbIlZzS1Y3Z3JSMUJVRTI5bUcyRm0ya1gjMSJdLCJyb3V0aW5nS2V5cyI6WyJWc0tWN2dyUjFCVUUyOW1HMkZtMmtYIzIiLCJWc0tWN2dyUjFCVUUyOW1HMkZtMmtYIzMiXSwic2VydmljZUVuZHBvaW50IjoiaHR0cDovL2xvY2FsaG9zdDo4MDgwIiwidHlwZSI6ImRpZC1jb21tdW5pY2F0aW9uIn1dfX0="),
                signer: _key(),
                ..Default::default()
            },
            please_ack: None,
        }
    }

    #[test]
    fn test_response_build_works() {
        let response: Response = Response::default()
            .set_did(_did())
            .set_thread_id(&_thread_id())
            .set_service_endpoint(_service_endpoint())
            .set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_response(), response);
    }

    #[test]
    fn test_response_encode_works() {
        let setup = test_setup::key();
        let signed_response: SignedResponse = _response().encode(&setup.key).unwrap();
        assert_eq!(_response(), signed_response.decode(&setup.key).unwrap());
    }
}
