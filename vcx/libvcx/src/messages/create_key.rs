use settings;
use messages::*;
use messages::message_type::MessageTypes;
use utils::{httpclient, constants};
use error::prelude::*;
use settings::ProtocolTypes;
use utils::httpclient::AgencyMock;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateKey {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "forDID")]
    for_did: String,
    #[serde(rename = "forDIDVerKey")]
    for_verkey: String,
}

impl CreateKey {
    fn build(for_did: &str, for_verkey: &str) -> CreateKey {
        CreateKey {
            msg_type: MessageTypes::build(A2AMessageKinds::CreateKey),
            for_did: for_did.to_string(),
            for_verkey: for_verkey.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    for_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug)]
pub struct CreateKeyBuilder {
    for_did: String,
    for_verkey: String,
    version: ProtocolTypes,
}

impl CreateKeyBuilder {
    pub fn create() -> CreateKeyBuilder {
        trace!("CreateKeyBuilder::create_message >>>");

        CreateKeyBuilder {
            for_did: String::new(),
            for_verkey: String::new(),
            version: settings::get_protocol_type(),
        }
    }

    pub fn for_did(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.for_did = did.to_string();
        Ok(self)
    }

    pub fn for_verkey(&mut self, verkey: &str) -> VcxResult<&mut Self> {
        validation::validate_verkey(verkey)?;
        self.for_verkey = verkey.to_string();
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&self) -> VcxResult<(String, String)> {
        trace!("CreateKeyMsg::send >>>");

        if settings::agency_mocks_enabled() {
            match self.version {
                settings::ProtocolTypes::V1 => AgencyMock::set_next_response(constants::CREATE_KEYS_RESPONSE.to_vec()),
                settings::ProtocolTypes::V2 |
                settings::ProtocolTypes::V3 => AgencyMock::set_next_response(constants::CREATE_KEYS_V2_RESPONSE.to_vec()),
            }
        }

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        let message = match self.version {
            settings::ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::CreateKey(CreateKey::build(&self.for_did, &self.for_verkey))
                ),
            settings::ProtocolTypes::V2 |
            settings::ProtocolTypes::V3 =>
                A2AMessage::Version2(
                    A2AMessageV2::CreateKey(CreateKey::build(&self.for_did, &self.for_verkey))
                )
        };

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: &Vec<u8>) -> VcxResult<(String, String)> {
        trace!("parse_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            A2AMessage::Version2(A2AMessageV2::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            _ => Err(VcxError::from(VcxErrorKind::InvalidHttpResponse))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use utils::constants::{MY1_SEED, MY2_SEED, MY3_SEED, CREATE_KEYS_V2_RESPONSE};
    use utils::constants::CREATE_KEYS_RESPONSE;
    use utils::libindy::signus::create_and_store_my_did;
    use messages::create_keys;
    use utils::devsetup::*;

    #[test]
    fn test_create_key_set_values() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";

        create_keys()
            .for_did(for_did).unwrap()
            .for_verkey(for_verkey).unwrap();
    }

    #[test]
    fn test_create_key_set_values_and_serialize() {
        let _setup = SetupLibraryWallet::init();

        let (_agent_did, agent_vk) = create_and_store_my_did(Some(MY2_SEED), None).unwrap();
        let (my_did, my_vk) = create_and_store_my_did(Some(MY1_SEED), None).unwrap();
        let (_agency_did, agency_vk) = create_and_store_my_did(Some(MY3_SEED), None).unwrap();

        settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        let bytes = create_keys()
            .for_did(&my_did).unwrap()
            .for_verkey(&my_vk).unwrap()
            .prepare_request().unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_parse_create_keys_v1_response() {
        let _setup = SetupMocks::init();

        let mut builder = create_keys();

        let (for_did, for_verkey) = builder.version(&Some(ProtocolTypes::V1)).unwrap().parse_response(&CREATE_KEYS_RESPONSE.to_vec()).unwrap();

        assert_eq!(for_did, "U5LXs4U7P9msh647kToezy");
        assert_eq!(for_verkey, "FktSZg8idAVzyQZrdUppK6FTrfAzW3wWVzAjJAfdUvJq");
    }

    #[test]
    fn test_parse_create_keys_v2_response() {
        let _setup = SetupMocks::init();

        let mut builder = create_keys();

        let (for_did, for_verkey) = builder.version(&Some(ProtocolTypes::V2)).unwrap().parse_response(&CREATE_KEYS_V2_RESPONSE.to_vec()).unwrap();

        assert_eq!(for_did, "MNepeSWtGfhnv8jLB1sFZC");
        assert_eq!(for_verkey, "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9");
    }

    #[test]
    fn test_create_key_set_invalid_did_errors() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoT";
        let res = create_keys()
            .for_did(for_did)
            .unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidDid);
    }
}

