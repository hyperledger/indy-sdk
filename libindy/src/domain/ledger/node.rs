use super::constants::NODE;

use indy_api_types::validation::Validatable;
use indy_api_types::domain::crypto::did::ShortDidValue;

#[derive(Serialize, PartialEq, Debug)]
pub struct NodeOperation {
    #[serde(rename = "type")]
    pub _type: String,
    pub dest: ShortDidValue,
    pub data: NodeOperationData
}

impl NodeOperation {
    pub fn new(dest: ShortDidValue, data: NodeOperationData) -> NodeOperation {
        NodeOperation {
            _type: NODE.to_string(),
            dest,
            data
        }
    }
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub enum Services {
    VALIDATOR,
    OBSERVER
}

#[derive(Serialize, PartialEq, Debug, Deserialize)]
pub struct NodeOperationData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_port: Option<i32>,
    pub alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<Services>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blskey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blskey_pop: Option<String>
}

impl Validatable for NodeOperationData{
    fn validate(&self) -> Result<(), String> {
        if self.node_ip.is_none() && self.node_port.is_none()
            && self.client_ip.is_none() && self.client_port.is_none()
            && self.services.is_none() && self.blskey.is_none()
            && self.blskey_pop.is_none() {
            return Err(String::from("Invalid data json: all fields missed at once"));
        }

        if (self.node_ip.is_some() || self.node_port.is_some() || self.client_ip.is_some() || self.client_port.is_some()) &&
            (self.node_ip.is_none() || self.node_port.is_none() || self.client_ip.is_none() || self.client_port.is_none()) {
            return Err(String::from("Invalid data json: Fields node_ip, node_port, client_ip, client_port must be specified together"));
        }

        Ok(())
    }
}
