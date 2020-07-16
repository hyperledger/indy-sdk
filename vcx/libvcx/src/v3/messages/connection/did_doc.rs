use v3::messages::connection::invite::{Invitation, PairwiseInvitation};

use error::prelude::*;
use url::Url;
use messages::validation::validate_verkey;
use utils::libindy::ledger;

pub const CONTEXT: &str = "https://w3id.org/did/v1";
pub const KEY_TYPE: &str = "Ed25519VerificationKey2018";
pub const KEY_AUTHENTICATION_TYPE: &str = "Ed25519SignatureAuthentication2018";
pub const SERVICE_SUFFIX: &str = "indy";
pub const SERVICE_TYPE: &str = "IndyAgent";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DidDoc {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(rename = "publicKey")]
    pub public_key: Vec<Ed25519PublicKey>, // TODO: A DID document MAY include a publicKey property??? (https://w3c.github.io/did-core/#public-keys)
    #[serde(default)]
    pub authentication: Vec<Authentication>,
    pub service: Vec<Service>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Ed25519PublicKey {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String, // all list of types: https://w3c-ccg.github.io/ld-cryptosuite-registry/
    pub controller: String,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Authentication {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    #[serde(rename = "recipientKeys")]
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routingKeys")]
    pub routing_keys: Vec<String>,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

impl Default for DidDoc {
    fn default() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: String::new(),
            public_key: vec![],
            authentication: vec![],
            service: vec![Service::default()],
        }
    }
}

impl DidDoc {
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_service_endpoint(&mut self, service_endpoint: String) {
        self.service.get_mut(0)
            .map(|service| {
                service.service_endpoint = service_endpoint;
                service
            });
    }

    pub fn set_keys(&mut self, recipient_keys: Vec<String>, routing_keys: Vec<String>) {
        let mut id = 0;
        let mut public_key = Vec::new();
        let controller = String::from(&self.id); // TODO: Will not be a valid did if created from pairwise invitation

        recipient_keys
            .iter()
            .for_each(|key| {
                self.service.get_mut(0)
                    .map(|service| {
                        service.recipient_keys.push(key.to_string());
                        public_key.push(
                            Ed25519PublicKey {
                                id: id.to_string(),
                                type_: KEY_TYPE.to_string(),
                                controller: controller.clone(),
                                public_key_base_58: String::from(key)
                            });
                        id += 1;
                        service
                    });
            });
        self.public_key = public_key;

        routing_keys
            .iter()
            .for_each(|key| {
                self.service.get_mut(0)
                    .map(|service| {
                        service.routing_keys.push(key.to_string());
                        service
                    });
            });

    }

    pub fn validate(&self) -> VcxResult<()> {
        if self.context != CONTEXT {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported @context value: {:?}", self.context)));
        }

//        if self.id.is_empty() {
//            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "DIDDoc validation failed: id is empty"));
//        }

        for service in self.service.iter() {
            Url::parse(&service.service_endpoint)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Invalid endpoint \"{:?}\", err: {:?}", service.service_endpoint, err)))?;

            service.recipient_keys
                .iter()
                .map(|key| self.validate_recipient_key(key))
                .collect::<VcxResult<()>>()?;

            service.routing_keys
                .iter()
                .map(|key| self.validate_routing_key(key))
                .collect::<VcxResult<()>>()?;
        }

        Ok(())
    }

    fn validate_recipient_key(&self, key: &str) -> VcxResult<()> {
        let public_key = self.validate_public_key(key)?;
        self.validate_authentication(&public_key.id)
    }

    fn validate_routing_key(&self, key: &str) -> VcxResult<()> {
        if DidDoc::_key_parts(key).len() == 2 {
            self.validate_public_key(key)?;
        } else {
            validate_verkey(key)?;
        }
        Ok(())
    }

    fn validate_public_key(&self, target_key: &str) -> VcxResult<&Ed25519PublicKey> {
        let id = DidDoc::_parse_key_reference(target_key);

        let key = self.public_key.iter().find(|key_| key_.id == id.to_string() || key_.public_key_base_58 == id.to_string())
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Cannot find PublicKey definition for key: {:?}", id)))?;

        if key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported PublicKey type: {:?}", key.type_)));
        }

        validate_verkey(&key.public_key_base_58)?;

        Ok(key)
    }

    fn validate_authentication(&self, target_key: &str) -> VcxResult<()> {
        if self.authentication.is_empty() {
            return Ok(());
        }

        let key = self.authentication.iter().find(|key_|
            key_.public_key == target_key.to_string() ||
                DidDoc::_parse_key_reference(&key_.public_key) == target_key.to_string())
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Cannot find Authentication section for key: {:?}", target_key)))?;

        if key.type_ != KEY_AUTHENTICATION_TYPE && key.type_ != KEY_TYPE {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("DIDDoc validation failed: Unsupported Authentication type: {:?}", key.type_)));
        }

        Ok(())
    }

    pub fn resolve_keys(&self) -> (Vec<String>, Vec<String>) {
        let service: Service = match self.service.get(0).cloned() {
            Some(service) => service,
            None => return (Vec::new(), Vec::new())
        };

        let recipient_keys: Vec<String> =
            service.recipient_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        let routing_keys: Vec<String> =
            service.routing_keys
                .iter()
                .map(|key| self.key_for_reference(key))
                .collect();

        (recipient_keys, routing_keys)
    }

    pub fn recipient_keys(&self) -> Vec<String> {
        let (recipient_keys, _) = self.resolve_keys();
        recipient_keys
    }

    pub fn routing_keys(&self) -> Vec<String> {
        let (_, routing_keys) = self.resolve_keys();
        routing_keys
    }

    pub fn get_endpoint(&self) -> String {
        match self.service.get(0) {
            Some(service) => service.service_endpoint.to_string(),
            None => String::new()
        }
    }

    fn key_for_reference(&self, key_reference: &str) -> String {
        let id = DidDoc::_parse_key_reference(key_reference);

        self.public_key.iter().find(|key_| key_.id == id.to_string() || key_.public_key_base_58 == id.to_string())
            .map(|key| key.public_key_base_58.clone())
            .unwrap_or(id)
    }

    fn _build_key_reference(did: &str, id: &str) -> String {
        format!("{}#{}", did, id)
    }

    fn _key_parts(key: &str) -> Vec<&str> {
        key.split("#").collect()
    }

    fn _parse_key_reference(key_reference: &str) -> String {
        let pars: Vec<&str> = DidDoc::_key_parts(key_reference);
        pars.get(1).or(pars.get(0)).map(|s| s.to_string()).unwrap_or_default()
    }
}

impl Default for Service {
    fn default() -> Service {
        Service {
            // TODO: FIXME Several services????
            id: format!("did:example:123456789abcdefghi;{}", SERVICE_SUFFIX),
            type_: String::from(SERVICE_TYPE),
            priority: 0,
            service_endpoint: String::new(),
            recipient_keys: Vec::new(),
            routing_keys: Vec::new(),
        }
    }
}

impl From<Invitation> for DidDoc {
    fn from(invitation: Invitation) -> DidDoc {
        let mut did_doc: DidDoc = DidDoc::default();
        let (service_endpoint, recipient_keys, routing_keys) = match invitation {
            Invitation::Public(invitation) => {
                did_doc.set_id(invitation.did.clone());
                let service = ledger::get_service(&invitation.did).unwrap_or_else(|err| {
                    error!("Failed to obtain service definition from the ledger: {}", err);
                    Service::default()                
                });
                (service.service_endpoint, service.recipient_keys, service.routing_keys)
            },
            Invitation::Pairwise(invitation) => {
                did_doc.set_id(invitation.id.0.clone());
                (invitation.service_endpoint.clone(), invitation.recipient_keys, invitation.routing_keys)
            }
        };
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_keys(recipient_keys, routing_keys);
        did_doc
    }
}

impl From<DidDoc> for PairwiseInvitation {
    fn from(did_doc: DidDoc) -> PairwiseInvitation {
        let (recipient_keys, routing_keys) = did_doc.resolve_keys();

        PairwiseInvitation::create()
            .set_id(did_doc.id.clone())
            .set_service_endpoint(did_doc.get_endpoint())
            .set_recipient_keys(recipient_keys)
            .set_routing_keys(routing_keys)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use settings;
    use utils::devsetup::*;
    use v3::messages::a2a::MessageId;
    use v3::messages::connection::invite::tests::{ _invitation, _public_invitation };

    pub fn _key_1() -> String {
        String::from("GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL")
    }

    pub fn _key_2() -> String {
        String::from("Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR")
    }

    pub fn _key_3() -> String {
        String::from("3LYuxJBJkngDbvJj4zjx13DBUdZ2P96eNybwd2n9L9AU")
    }

    pub fn _id() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _service_endpoint() -> String {
        String::from("http://localhost:8080")
    }

    pub fn _recipient_keys() -> Vec<String> {
        vec![_key_1()]
    }

    pub fn _routing_keys() -> Vec<String> {
        vec![_key_2(), _key_3()]
    }
    
    pub fn _did() -> String {
        String::from("VsKV7grR1BUE29mG2Fm2kX")
    }

    pub fn _key_reference_1() -> String {
        DidDoc::_build_key_reference(&_id(), "1")
    }

    pub fn _key_reference_2() -> String {
        DidDoc::_build_key_reference(&_id(), "2")
    }

    pub fn _key_reference_3() -> String {
        DidDoc::_build_key_reference(&_id(), "3")
    }

    pub fn _label() -> String {
        String::from("test")
    }

    pub fn _did_doc() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: "1".to_string(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_reference_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_2() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_reference_2(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_reference_3(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_3() }
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_3() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_2() },
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_3() }
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_4() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![],
                ..Default::default()
            }],
        }
    }

    pub fn _did_doc_5() -> DidDoc {
        DidDoc {
            context: String::from(CONTEXT),
            id: _id(),
            public_key: vec![
                Ed25519PublicKey { id: _key_reference_1(), type_: KEY_TYPE.to_string(), controller: _id(), public_key_base_58: _key_1() },
            ],
            authentication: vec![
                Authentication { type_: KEY_AUTHENTICATION_TYPE.to_string(), public_key: _key_reference_1() }
            ],
            service: vec![Service {
                service_endpoint: _service_endpoint(),
                recipient_keys: vec![_key_1()],
                routing_keys: vec![_key_2(), _key_3()],
                ..Default::default()
            }],
        }
    }

    #[test]
    fn test_did_doc_build_works() {
        let mut did_doc: DidDoc = DidDoc::default();
        did_doc.set_id(_id());
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(_did_doc(), did_doc);
    }

    #[test]
    fn test_did_doc_validate_works() {
        _did_doc().validate().unwrap();
        _did_doc_2().validate().unwrap();
        _did_doc_3().validate().unwrap();
        _did_doc_4().validate().unwrap();
        _did_doc_5().validate().unwrap();
    }

    #[test]
    fn test_did_doc_key_for_reference_works() {
        assert_eq!(_key_1(), _did_doc().key_for_reference(&_key_reference_1()));
    }

    #[test]
    fn test_did_doc_resolve_keys_works() {
        let (recipient_keys, routing_keys) = _did_doc().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);

        let (recipient_keys, routing_keys) = _did_doc_2().resolve_keys();
        assert_eq!(_recipient_keys(), recipient_keys);
        assert_eq!(_routing_keys(), routing_keys);
    }

    #[test]
    fn test_did_doc_build_key_reference_works() {
        assert_eq!(_key_reference_1(), DidDoc::_build_key_reference(&_id(), "1"));
    }

    #[test]
    fn test_did_doc_parse_key_reference_works() {
        assert_eq!(String::from("1"), DidDoc::_parse_key_reference(&_key_reference_1()));
        assert_eq!(_key_1(), DidDoc::_parse_key_reference(&_key_1()));
    }

    #[test]
    fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), _routing_keys());

        assert_eq!(did_doc, DidDoc::from(Invitation::Pairwise(_invitation())))
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_did_doc_from_public_invitation_works() {
        // TODO: Either make part of ledger tests, or setup ledger mocks
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_keys(_recipient_keys(), _routing_keys());

        let public_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        ledger::add_service(&public_did, &did_doc.service[0]).unwrap();
        let did_doc_from_invite = DidDoc::from(Invitation::Public(_public_invitation(&public_did)));
        assert_eq!(did_doc, did_doc_from_invite)
    }
}
