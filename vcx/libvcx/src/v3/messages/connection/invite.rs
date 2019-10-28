use v3::messages::{MessageType, A2AMessageKinds, MessageId};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Invitation {
    #[serde(rename = "@type")]
    pub msg_type: MessageType,
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

impl Invitation {
    pub fn create() -> Invitation {
        Invitation::default()
    }

    pub fn set_id(mut self, id: MessageId) -> Invitation {
        self.id = id;
        self
    }

    pub fn set_label(mut self, label: String) -> Invitation {
        self.label = label;
        self
    }

    pub fn set_service_endpoint(mut self, service_endpoint: String) -> Invitation {
        self.service_endpoint = service_endpoint;
        self
    }

    pub fn set_recipient_keys(mut self, recipient_keys: Vec<String>) -> Invitation {
        self.recipient_keys = recipient_keys;
        self
    }

    pub fn set_routing_keys(mut self, routing_keys: Vec<String>) -> Invitation {
        self.routing_keys = routing_keys;
        self
    }
}

impl Default for Invitation {
    fn default() -> Invitation {
        Invitation {
            msg_type: MessageType::build(A2AMessageKinds::Invitation),
            id: MessageId::new(),
            label: String::new(),
            service_endpoint: String::new(),
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use v3::messages::connection::did_doc::tests::*;

    fn _id() -> MessageId {
        MessageId(String::from("testid"))
    }

    pub fn _invitation() -> Invitation {
        Invitation {
            msg_type: MessageType::build(A2AMessageKinds::Invitation),
            id: _id(),
            label: _label(),
            recipient_keys: _recipient_keys(),
            routing_keys: _routing_keys(),
            service_endpoint: _service_endpoint(),
        }
    }

    #[test]
    fn test_request_build_works() {
        let invitation: Invitation = Invitation::default()
            .set_id(_id())
            .set_label(_label())
            .set_service_endpoint(_service_endpoint())
            .set_recipient_keys(_recipient_keys())
            .set_routing_keys(_routing_keys());

        assert_eq!(_invitation(), invitation);
    }
}