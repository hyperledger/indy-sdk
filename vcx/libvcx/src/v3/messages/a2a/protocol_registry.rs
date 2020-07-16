use regex::Regex;
use strum::IntoEnumIterator;

use v3::messages::a2a::message_family::MessageFamilies;
use v3::messages::discovery::disclose::ProtocolDescriptor;
use settings::Actors;

pub struct ProtocolRegistry {
    protocols: Vec<ProtocolDescriptor>
}

impl ProtocolRegistry {
    pub fn init() -> ProtocolRegistry {
        let mut registry = ProtocolRegistry { protocols: Vec::new() };
        let actors = ::settings::get_actors();

        for family in MessageFamilies::iter() {
            match family {
                family @ MessageFamilies::Routing |
                family @ MessageFamilies::ReportProblem |
                family @ MessageFamilies::Notification |
                family @ MessageFamilies::Connections |
                family @ MessageFamilies::CredentialIssuance |
                family @ MessageFamilies::PresentProof |
                family @ MessageFamilies::TrustPing |
                family @ MessageFamilies::Basicmessage |
                family @ MessageFamilies::DiscoveryFeatures => registry.add_protocol(&actors, family),
                MessageFamilies::Signature => {}
                MessageFamilies::Unknown(_) => {}
            }
        }

        registry
    }

    pub fn add_protocol(&mut self, actors: &Vec<Actors>, family: MessageFamilies) {
        match family.actors() {
            None => {
                self.protocols.push(ProtocolDescriptor { pid: family.id(), roles: None })
            }
            Some((actor_1, actor_2)) => {
                match (actors.contains(&actor_1), actors.contains(&actor_2)) {
                    (true, true) => {
                        self.protocols.push(ProtocolDescriptor { pid: family.id(), roles: None })
                    }
                    (true, false) => {
                        self.protocols.push(ProtocolDescriptor { pid: family.id(), roles: Some(vec![actor_1]) })
                    }
                    (false, true) => {
                        self.protocols.push(ProtocolDescriptor { pid: family.id(), roles: Some(vec![actor_2]) })
                    }
                    (false, false) => {}
                }
            }
        }
    }

    pub fn get_protocols_for_query(&self, query: Option<&str>) -> Vec<ProtocolDescriptor> {
        match query {
            Some(query_) if query_ == "*" => self.protocols.clone(),
            Some(query_) => {
                match Regex::new(query_) {
                    Ok(re) => self.protocols.iter().filter(|protocol| re.is_match(&protocol.pid)).cloned().collect(),
                    Err(_) => vec![]
                }
            }
            None => self.protocols.clone()
        }
    }

    pub fn protocols(&self) -> Vec<ProtocolDescriptor>{
        self.protocols.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::SetupEmpty;

    fn _protocols() -> Vec<ProtocolDescriptor> {
        vec![
            ProtocolDescriptor { pid: "protocol_1.0_test".to_string(), roles: None },
            ProtocolDescriptor { pid: "protocol_1.0_some".to_string(), roles: None },
            ProtocolDescriptor { pid: "0_test.0_test".to_string(), roles: None },
        ]
    }

    fn _protocol_registry() -> ProtocolRegistry {
        ProtocolRegistry { protocols: _protocols() }
    }

    #[test]
    fn test_protocol_registry_init_works() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = ProtocolRegistry::init();
        assert!(registry.protocols.len() > 0);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_none_query() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = _protocol_registry();
        let protocols = registry.get_protocols_for_query(None);
        assert_eq!(_protocols(), protocols);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_placeholder() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = _protocol_registry();

        let protocols = registry.get_protocols_for_query(Some("*"));
        assert_eq!(_protocols(), protocols);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_partial() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = _protocol_registry();

        let protocols = registry.get_protocols_for_query(Some("protocol_1.0*"));

        let expected_protocols = vec![
            ProtocolDescriptor { pid: "protocol_1.0_test".to_string(), roles: None },
            ProtocolDescriptor { pid: "protocol_1.0_some".to_string(), roles: None },
        ];
        assert_eq!(expected_protocols, protocols);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_exact_protocol() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = _protocol_registry();

        let protocols = registry.get_protocols_for_query(Some("protocol_1.0_test"));

        let expected_protocols = vec![
            ProtocolDescriptor { pid: "protocol_1.0_test".to_string(), roles: None },
        ];
        assert_eq!(expected_protocols, protocols);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_no_matching() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = _protocol_registry();

        let protocols = registry.get_protocols_for_query(Some("test_some_other"));

        assert!(protocols.is_empty());
    }

    #[test]
    fn test_get_protocols_for_query_works_for_real() {
        let _setup = SetupEmpty::init();

        let registry: ProtocolRegistry = ProtocolRegistry::init();

        let protocols = registry.get_protocols_for_query(None);
        assert!(!protocols.is_empty());

        let protocols = registry.get_protocols_for_query(Some("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections"));
        let expected_protocols = vec![
            ProtocolDescriptor { pid: MessageFamilies::Connections.id(), roles: None },
        ];
        assert_eq!(expected_protocols, protocols);

        let protocols = registry.get_protocols_for_query(Some("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0"));
        let expected_protocols = vec![
            ProtocolDescriptor { pid: MessageFamilies::Connections.id(), roles: None },
        ];
        assert_eq!(expected_protocols, protocols);
    }

    #[test]
    fn test_get_protocols_for_query_works_for_limited_actors() {
        let _setup = SetupEmpty::init();

        ::settings::set_config_value(::settings::CONFIG_ACTORS, &json!([Actors::Invitee]).to_string());

        let registry: ProtocolRegistry = ProtocolRegistry::init();

        let protocols = registry.get_protocols_for_query(Some("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0"));

        let expected_protocols = vec![
            ProtocolDescriptor { pid: MessageFamilies::Connections.id(), roles: Some(vec![Actors::Invitee]) },
        ];
        assert_eq!(expected_protocols, protocols);
    }
}
