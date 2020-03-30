use settings;

use serde::{de, Deserializer, Deserialize, Serializer, Serialize};
use serde_json::Value;
use regex::{Regex, Match};
use messages::A2AMessageKinds;
use error::prelude::*;

pub const MESSAGE_VERSION_V1: &str = "1.0";
pub const DID: &str = "did:sov:123456789abcdefghi1234";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum MessageTypes {
    MessageTypeV1(MessageTypeV1),
    MessageTypeV2(MessageTypeV2),
}

impl MessageTypes {
    pub fn build_v1(kind: A2AMessageKinds) -> MessageTypeV1 {
        MessageTypeV1 {
            name: kind.name(),
            ver: MESSAGE_VERSION_V1.to_string(),
        }
    }

    pub fn build_v2(kind: A2AMessageKinds) -> MessageTypeV2 {
        MessageTypeV2 {
            did: DID.to_string(),
            family: kind.family(),
            version: kind.family().version().to_string(),
            type_: kind.name(),
        }
    }

    pub fn build(kind: A2AMessageKinds) -> MessageTypes {
        match settings::get_protocol_type() {
            settings::ProtocolTypes::V1 => MessageTypes::MessageTypeV1(MessageTypes::build_v1(kind)),
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 => MessageTypes::MessageTypeV2(MessageTypes::build_v2(kind))
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        match self {
            MessageTypes::MessageTypeV1(type_) => type_.name.as_str(),
            MessageTypes::MessageTypeV2(type_) => type_.type_.as_str(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct MessageTypeV1 {
    pub name: String,
    pub ver: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageTypeV2 {
    pub did: String,
    pub family: MessageFamilies,
    pub version: String,
    pub type_: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum MessageFamilies {
    Routing,
    Onboarding,
    Pairwise,
    Configs,
    CredentialExchange,
    Unknown(String),
}

impl MessageFamilies {
    pub fn version(&self) -> &'static str {
        match self {
            MessageFamilies::Routing => "1.0",
            MessageFamilies::Onboarding => "1.0",
            MessageFamilies::Pairwise => "1.0",
            MessageFamilies::Configs => "1.0",
            MessageFamilies::CredentialExchange => "1.0",
            _ => "1.0"
        }
    }
}

impl From<String> for MessageFamilies {
    fn from(family: String) -> Self {
        match family.as_str() {
            "routing" => MessageFamilies::Routing,
            "onboarding" => MessageFamilies::Onboarding,
            "pairwise" => MessageFamilies::Pairwise,
            "configs" => MessageFamilies::Configs,
            "credential-exchange" => MessageFamilies::CredentialExchange,
            family @ _ => MessageFamilies::Unknown(family.to_string())
        }
    }
}

impl ::std::string::ToString for MessageFamilies {
    fn to_string(&self) -> String {
        match self {
            MessageFamilies::Routing => "routing".to_string(),
            MessageFamilies::Onboarding => "onboarding".to_string(),
            MessageFamilies::Pairwise => "pairwise".to_string(),
            MessageFamilies::CredentialExchange => "credential_exchange".to_string(),
            MessageFamilies::Configs => "configs".to_string(),
            MessageFamilies::Unknown(family) => family.to_string()
        }
    }
}


pub fn parse_message_type(message_type: &str) -> VcxResult<(String, String, String, String)> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            (?P<did>[\d\w:]*);
            (?P<spec>.*)/
            (?P<family>.*)/
            (?P<version>.*)/
            (?P<type>.*)").unwrap();
    }

    RE.captures(message_type)
        .and_then(|cap| {
            let did = cap.name("did").as_ref().map(Match::as_str);
            let family = cap.name("family").as_ref().map(Match::as_str);
            let version = cap.name("version").as_ref().map(Match::as_str);
            let type_ = cap.name("type").as_ref().map(Match::as_str);

            match (did, family, version, type_) {
                (Some(did), Some(family), Some(version), Some(type_)) =>
                    Some((did.to_string(), family.to_string(), version.to_string(), type_.to_string())),
                _ => None
            }
        }).ok_or(VcxError::from_msg(VcxErrorKind::InvalidOption, "Cannot parse @type"))
}

impl<'de> Deserialize<'de> for MessageTypeV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        match value.as_str() {
            Some(type_) => {
                let (did, family, version, type_) = parse_message_type(type_).map_err(de::Error::custom)?;
                Ok(MessageTypeV2 {
                    did,
                    family: MessageFamilies::from(family),
                    version,
                    type_,
                })
            }
            _ => Err(de::Error::custom("Unexpected @type field structure."))
        }
    }
}

impl Serialize for MessageTypeV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = Value::String(format!("{};spec/{}/{}/{}", self.did, self.family.to_string(), self.version, self.type_));
        value.serialize(serializer)
    }
}