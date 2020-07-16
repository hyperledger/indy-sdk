use v3::messages::a2a::{A2AMessage, MessageId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Invitation {
    Pairwise(PairwiseInvitation),
    Public(PublicInvitation)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct PairwiseInvitation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub label: String,
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct PublicInvitation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    pub label: String,
    #[serde(rename = "did")]
    pub did: String
}

impl PairwiseInvitation {
    pub fn create() -> PairwiseInvitation {
        PairwiseInvitation::default()
    }

    pub fn set_label(mut self, label: String) -> PairwiseInvitation {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> PairwiseInvitation {
        self.id = MessageId(id);
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> PairwiseInvitation {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> PairwiseInvitation {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> PairwiseInvitation {
        self.routing_keys = routing_keys;
        self
    }
}

impl PublicInvitation {
    pub fn create() -> PublicInvitation {
        PublicInvitation::default()
    }

    pub fn set_label(mut self, label: String) -> PublicInvitation {
        self.label = label;
        self
    }

    pub fn set_id(mut self, id: String) -> PublicInvitation {
        self.id = MessageId(id);
        self
    }

    pub fn set_public_did(mut self, public_did: String) -> PublicInvitation {
        self.did = public_did;
        self
    }
}

a2a_message!(PairwiseInvitation, ConnectionInvitationPairwise);
a2a_message!(PublicInvitation, ConnectionInvitationPublic);

#[cfg(test)]
pub mod tests {
    use super::*;
    use v3::messages::connection::did_doc::tests::*;

    pub fn _invitation() -> PairwiseInvitation {
        PairwiseInvitation {
            id: MessageId::id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
        }
    }

    pub fn _public_invitation(public_did: &str) -> PublicInvitation {
        PublicInvitation {
            id: MessageId::id(),
            label: _label(),
            did: public_did.to_string()
        }
    }

    pub fn _invitation_json() -> String {
        ::serde_json::to_string(&_invitation()).unwrap()
    }

    #[test]
    fn test_request_build_works() {
        let invitation: PairwiseInvitation = PairwiseInvitation::default()
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_invitation(), invitation);
    }
}
