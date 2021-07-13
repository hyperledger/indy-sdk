use error::prelude::*;
use regex::{Regex, Match};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use v3::messages::a2a::message_family::MessageFamilies;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MessageType {
    pub prefix: String,
    pub family: MessageFamilies,
    pub version: String,
    pub type_: String,
}

impl MessageType {
    pub fn did_prefix() -> String {
        format!("{};spec", MessageFamilies::DID)
    }

    pub fn https_prefix() -> String {
        "https://didcomm.org".to_string()
    }

    pub fn build(family: MessageFamilies, name: &str) -> MessageType {
        MessageType {
            prefix: Self::did_prefix(),
            version: family.version().to_string(),
            family,
            type_: name.to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        match value.as_str() {
            Some(type_) => {
                let (prefix, family, version, type_) = parse_message_type(type_).map_err(de::Error::custom)?;
                Ok(MessageType {
                    prefix,
                    family: MessageFamilies::from(family),
                    version,
                    type_,
                })
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = Value::String(self.to_string());
        value.serialize(serializer)
    }
}

impl ::std::string::ToString for MessageType {
    fn to_string(&self) -> String {
        format!("{}/{}/{}/{}", self.prefix, self.family.to_string(), self.version, self.type_)
    }
}

pub fn parse_message_type(message_type: &str) -> VcxResult<(String, String, String, String)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            (?P<prefix>did:[\d\w:]*;spec|https://didcomm.org)/
            (?P<family>.*)/
            (?P<version>.*)/
            (?P<type>.*)").unwrap();
    }

    RE.captures(message_type)
        .and_then(|cap| {
            let prefix = cap.name("prefix").as_ref().map(Match::as_str);
            let family = cap.name("family").as_ref().map(Match::as_str);
            let version = cap.name("version").as_ref().map(Match::as_str);
            let type_ = cap.name("type").as_ref().map(Match::as_str);

            match (prefix, family, version, type_) {
                (Some(prefix), Some(family), Some(version), Some(type_)) =>
                    Some((prefix.to_string(), family.to_string(), version.to_string(), type_.to_string())),
                _ => None
            }
        }).ok_or(VcxError::from_msg(VcxErrorKind::InvalidOption, "Cannot parse @type"))
}

#[cfg(test)]
mod message_type {
    use super::*;
    use v3::messages::a2a::A2AMessage;

    fn _did_based_type() -> String {
        "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/notification/1.0/ack".to_string()
    }

    fn _https_based_type() -> String {
        "https://didcomm.org/notification/1.0/ack".to_string()
    }

    #[test]
    fn build_message_type_works() {
        let message_type = MessageType::build(MessageFamilies::Notification, A2AMessage::ACK).to_string();
        assert_eq!(message_type, _did_based_type());
    }

    #[test]
    fn parse_message_type_works_for_did_based() {
        let message_type: MessageType = serde_json::from_value(serde_json::Value::String(_did_based_type())).unwrap();

        let expected_message_type = MessageType::build(MessageFamilies::Notification, A2AMessage::ACK);

        assert_eq!(message_type, expected_message_type);
    }

    #[test]
    fn parse_message_type_works_for_https_based() {
        let message_type: MessageType = serde_json::from_value(serde_json::Value::String(_https_based_type())).unwrap();

        let mut expected_message_type = MessageType::build(MessageFamilies::Notification, A2AMessage::ACK);
        expected_message_type.prefix = MessageType::https_prefix();

        assert_eq!(message_type, expected_message_type);
    }

    #[test]
    fn parse_message_type_works_for_invalid_type() {
        let _err = serde_json::from_value::<MessageType>(serde_json::Value::String("invalid;did".to_string())).unwrap_err();
    }
}