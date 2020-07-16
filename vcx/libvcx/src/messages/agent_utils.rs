use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use settings;
use messages::{A2AMessage, A2AMessageV1, A2AMessageV2, A2AMessageKinds, prepare_message_for_agency, parse_response_from_agency};
use messages::message_type::MessageTypes;
use utils::{error, httpclient, constants};
use utils::libindy::{wallet, anoncreds};
use utils::libindy::signus::create_and_store_my_did;
use utils::option_util::get_or_default;
use error::prelude::*;
use utils::httpclient::AgencyMock;
use utils::libindy::ledger::add_service;
use utils::libindy::cache;

use v3::handlers::connection::agent::AgentInfo;
use v3::messages::connection::invite::PublicInvitation;
use v3::messages::connection::did_doc::Service;

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "fromDID")]
    from_did: String,
    #[serde(rename = "fromDIDVerKey")]
    from_vk: String,
}

impl Connect {
    fn build(from_did: &str, from_vk: &str) -> Connect {
        Connect {
            msg_type: MessageTypes::build(A2AMessageKinds::Connect),
            from_did: from_did.to_string(),
            from_vk: from_vk.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUp {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl SignUp {
    fn build() -> SignUp {
        SignUp {
            msg_type: MessageTypes::build(A2AMessageKinds::SignUp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgent {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl CreateAgent {
    fn build() -> CreateAgent {
        CreateAgent {
            msg_type: MessageTypes::build(A2AMessageKinds::CreateAgent),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgentResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

#[derive(Debug, PartialEq)]
pub enum ComMethodType {
    A2A,
    Webhook
}

impl Serialize for ComMethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ComMethodType::A2A => "1",
            ComMethodType::Webhook => "2",
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("1") => Ok(ComMethodType::A2A),
            Some("2") => Ok(ComMethodType::Webhook),
            _ => Err(de::Error::custom("Unexpected communication method type."))
        }
    }
}

impl UpdateComMethod {
    fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageTypes::build(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    protocol_type: settings::ProtocolTypes,
    agency_url: String,
    pub agency_did: String,
    agency_verkey: String,
    wallet_name: Option<String>,
    wallet_key: String,
    wallet_type: Option<String>,
    agent_seed: Option<String>,
    enterprise_seed: Option<String>,
    wallet_key_derivation: Option<String>,
    name: Option<String>,
    logo: Option<String>,
    path: Option<String>,
    storage_config: Option<String>,
    storage_credentials: Option<String>,
    pool_config: Option<String>,
    did_method: Option<String>,
    communication_method: Option<String>,
    webhook_url: Option<String>,
    use_latest_protocols: Option<String>,
}

pub fn set_config_values(my_config: &Config) {
    let wallet_name = get_or_default(&my_config.wallet_name, settings::DEFAULT_WALLET_NAME);

    settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, &my_config.protocol_type.to_string());
    settings::set_config_value(settings::CONFIG_AGENCY_ENDPOINT, &my_config.agency_url);
    settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
    settings::set_config_value(settings::CONFIG_AGENCY_DID, &my_config.agency_did);
    settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &my_config.agency_verkey);
    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &my_config.agency_verkey);
    settings::set_config_value(settings::CONFIG_WALLET_KEY, &my_config.wallet_key);

    settings::set_opt_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &my_config.wallet_key_derivation);
    settings::set_opt_config_value(settings::CONFIG_WALLET_TYPE, &my_config.wallet_type);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG, &my_config.storage_config);
    settings::set_opt_config_value(settings::CONFIG_WALLET_STORAGE_CREDS, &my_config.storage_credentials);
    settings::set_opt_config_value(settings::CONFIG_POOL_CONFIG, &my_config.pool_config);
    settings::set_opt_config_value(settings::CONFIG_DID_METHOD, &my_config.did_method);
    settings::set_opt_config_value(settings::COMMUNICATION_METHOD, &my_config.communication_method);
    settings::set_opt_config_value(settings::CONFIG_WEBHOOK_URL, &my_config.webhook_url);
}

fn _create_issuer_keys(my_did: &str, my_vk: &str, my_config: &Config) -> VcxResult<(String, String)> {
    if my_config.enterprise_seed == my_config.agent_seed {
        Ok((my_did.to_string(), my_vk.to_string()))
    } else {
        create_and_store_my_did(
            my_config.enterprise_seed.as_ref().map(String::as_str),
            my_config.did_method.as_ref().map(String::as_str),
        )
    }
}

pub fn configure_wallet(my_config: &Config) -> VcxResult<(String, String, String)> {
    let wallet_name = get_or_default(&my_config.wallet_name, settings::DEFAULT_WALLET_NAME);

    wallet::init_wallet(
        &wallet_name,
        my_config.wallet_type.as_ref().map(String::as_str),
        my_config.storage_config.as_ref().map(String::as_str),
        my_config.storage_credentials.as_ref().map(String::as_str),
    )?;
    trace!("initialized wallet");

    // If MS is already in wallet then just continue
    anoncreds::libindy_prover_create_master_secret(::settings::DEFAULT_LINK_SECRET_ALIAS).ok();

    let (my_did, my_vk) = create_and_store_my_did(
        my_config.agent_seed.as_ref().map(String::as_str),
        my_config.did_method.as_ref().map(String::as_str),
    )?;

    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

    Ok((my_did, my_vk, wallet_name))
}

pub fn get_final_config(my_did: &str,
                        my_vk: &str,
                        agent_did: &str,
                        agent_vk: &str,
                        wallet_name: &str,
                        my_config: &Config) -> VcxResult<String> {
    let (issuer_did, issuer_vk) = _create_issuer_keys(my_did, my_vk, my_config)?;

    let mut final_config = json!({
        "wallet_key": &my_config.wallet_key,
        "wallet_name": wallet_name,
        "agency_endpoint": &my_config.agency_url,
        "agency_did": &my_config.agency_did,
        "agency_verkey": &my_config.agency_verkey,
        "sdk_to_remote_did": my_did,
        "sdk_to_remote_verkey": my_vk,
        "institution_did": issuer_did,
        "institution_verkey": issuer_vk,
        "remote_to_sdk_did": agent_did,
        "remote_to_sdk_verkey": agent_vk,
        "institution_name": get_or_default(&my_config.name, "<CHANGE_ME>"),
        "institution_logo_url": get_or_default(&my_config.logo, "<CHANGE_ME>"),
        "genesis_path": get_or_default(&my_config.path, "<CHANGE_ME>"),
        "protocol_type": &my_config.protocol_type,
    });

    if let Some(key_derivation) = &my_config.wallet_key_derivation {
        final_config["wallet_key_derivation"] = json!(key_derivation);
    }
    if let Some(wallet_type) = &my_config.wallet_type {
        final_config["wallet_type"] = json!(wallet_type);
    }
    if let Some(_storage_config) = &my_config.storage_config {
        final_config["storage_config"] = json!(_storage_config);
    }
    if let Some(_storage_credentials) = &my_config.storage_credentials {
        final_config["storage_credentials"] = json!(_storage_credentials);
    }
    if let Some(_pool_config) = &my_config.pool_config {
        final_config["pool_config"] = json!(_pool_config);
    }
    if let Some(_communication_method) = &my_config.communication_method {
        final_config["communication_method"] = json!(_communication_method);
    }
    if let Some(_webhook_url) = &my_config.webhook_url {
        final_config["webhook_url"] = json!(_webhook_url);
    }
    if let Some(_use_latest_protocols) = &my_config.use_latest_protocols {
        final_config["use_latest_protocols"] = json!(_use_latest_protocols);
    }

    Ok(final_config.to_string())
}

pub fn parse_config(config: &str) -> VcxResult<Config> {
    let my_config: Config = ::serde_json::from_str(&config)
        .map_err(|err|
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Cannot parse config: {}", err),
            )
        )?;
    Ok(my_config)
}

pub fn connect_register_provision(config: &str) -> VcxResult<String> {
    trace!("connect_register_provision >>> config: {:?}", config);
    let my_config = parse_config(config)?;

    trace!("***Configuring Library");
    set_config_values(&my_config);

    trace!("***Configuring Wallet");
    let (my_did, my_vk, wallet_name) = configure_wallet(&my_config)?;

    trace!("Connecting to Agency");
    let (agent_did, agent_vk) = match my_config.protocol_type {
        settings::ProtocolTypes::V1 => onboarding_v1(&my_did, &my_vk, &my_config.agency_did)?,
        settings::ProtocolTypes::V2 |
        settings::ProtocolTypes::V3 |
        settings::ProtocolTypes::V4 => onboarding_v2(&my_did, &my_vk, &my_config.agency_did)?,
    };

    debug!("***Constructing final config");
    let config = get_final_config(&my_did, &my_vk, &agent_did, &agent_vk, &wallet_name, &my_config)?;
    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_DID, &agent_did);

    wallet::close_wallet()?;

    Ok(config)
}

fn onboarding_v1(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    /* STEP 1 - CONNECT */
    AgencyMock::set_next_response(constants::CONNECTED_RESPONSE.to_vec());

    let message = A2AMessage::Version1(
        A2AMessageV1::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did)?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::ConnectResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of ConnectResponse"))
        };

    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);

    /* STEP 2 - REGISTER */
    AgencyMock::set_next_response(constants::REGISTER_RESPONSE.to_vec());

    let message = A2AMessage::Version1(
        A2AMessageV1::SignUp(SignUp::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let _response: SignUpResponse =
        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::SignUpResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    AgencyMock::set_next_response(constants::AGENT_CREATED.to_vec());

    let message = A2AMessage::Version1(
        A2AMessageV1::CreateAgent(CreateAgent::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            A2AMessage::Version1(A2AMessageV1::CreateAgentResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    Ok((response.from_did, response.from_vk))
}

pub fn connect_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    /* STEP 1 - CONNECT */
    let message = A2AMessage::Version2(
        A2AMessageV2::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did)?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::ConnectResponse(resp)) =>
                resp,
            _ => return
                Err(VcxError::from_msg(
                    VcxErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of ConnectResponse")
                )
        };

    settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);
    Ok((agency_pw_did, agency_pw_vk))
}

// it will be changed next
fn onboarding_v2(my_did: &str, my_vk: &str, agency_did: &str) -> VcxResult<(String, String)> {
    let (agency_pw_did, _) = connect_v2(my_did, my_vk, agency_did)?;

    /* STEP 2 - REGISTER */
    let message = A2AMessage::Version2(
        A2AMessageV2::SignUp(SignUp::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let _response: SignUpResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::SignUpResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = A2AMessage::Version2(
        A2AMessageV2::CreateAgent(CreateAgent::build())
    );

    let mut response = send_message_to_agency(&message, &agency_pw_did)?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateAgentResponse(resp)) => resp,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    Ok((response.from_did, response.from_vk))
}

pub fn update_agent_info(id: &str, value: &str) -> VcxResult<()> {
    trace!("update_agent_info >>> id: {}, value: {}", id, value);

    let to_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

    let com_method = ComMethod {
        id: id.to_string(),
        e_type: ComMethodType::A2A,
        value: value.to_string(),
    };

    match settings::get_protocol_type() {
        settings::ProtocolTypes::V1 => {
            update_agent_info_v1(&to_did, com_method)
        }
        settings::ProtocolTypes::V2 |
        settings::ProtocolTypes::V3 |
        settings::ProtocolTypes::V4 => {
            update_agent_info_v2(&to_did, com_method)
        }
    }
}

fn update_agent_info_v1(to_did: &str, com_method: ComMethod) -> VcxResult<()> {
    AgencyMock::set_next_response(constants::REGISTER_RESPONSE.to_vec());

    let message = A2AMessage::Version1(
        A2AMessageV1::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, to_did)?;
    Ok(())
}

fn update_agent_info_v2(to_did: &str, com_method: ComMethod) -> VcxResult<()> {
    let message = A2AMessage::Version2(
        A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, to_did)?;
    Ok(())
}

pub fn update_agent_webhook(webhook_url: &str) -> VcxResult<()> {
    trace!("update_agent_webhook >>> webhook_url: {:?}", webhook_url);

    let com_method: ComMethod = ComMethod {
        id: String::from("123"),
        e_type: ComMethodType::Webhook,
        value: String::from(webhook_url)
    };

    match settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID) {
        Ok(to_did) => {
            match settings::get_protocol_type() {
                settings::ProtocolTypes::V1 => update_agent_webhook_v1(&to_did, com_method)?,
                settings::ProtocolTypes::V2 |
                settings::ProtocolTypes::V3 |
                settings::ProtocolTypes::V4 => update_agent_webhook_v2(&to_did, com_method)?,
            }
        },
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

fn update_agent_webhook_v1(to_did: &str, com_method: ComMethod) -> VcxResult<()> {
    if settings::agency_mocks_enabled() { return Ok(()) }

    let message = A2AMessage::Version1(
        A2AMessageV1::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, &to_did)?;
    Ok(())
}

fn update_agent_webhook_v2(to_did: &str, com_method: ComMethod) -> VcxResult<()> {
    let message = A2AMessage::Version2(
        A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, &to_did)?;
    Ok(())
}

pub fn send_message_to_agency(message: &A2AMessage, did: &str) -> VcxResult<Vec<A2AMessage>> {
    let data = prepare_message_for_agency(message, &did, &settings::get_protocol_type())?;

    let response = httpclient::post_u8(&data)
        .map_err(|err| err.map(VcxErrorKind::InvalidHttpResponse, error::INVALID_HTTP_RESPONSE.message))?;

    parse_response_from_agency(&response, &settings::get_protocol_type())
}

pub fn create_pub_agent() -> VcxResult<()> {
    debug!("***Creating public agent");

    let pub_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
    let agent_info: AgentInfo = AgentInfo::create_pub_agent()?;

    let agency_url = settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)?;

    debug!("***Writing agency endpoint and routing keys to the ledger");
    let service = Service {
        service_endpoint: String::from(format!("{}/agency/msg", agency_url)),
        recipient_keys: agent_info.recipient_keys(),
        routing_keys: agent_info.routing_keys()?,
        ..Default::default()
    };

    debug!("***Writing service to the ledger: {:?}", service);
    add_service(&pub_did, &service)?;

    let label = settings::get_config_value(settings::CONFIG_INSTITUTION_NAME)?;
    let pub_invite = PublicInvitation::create().set_label(label).set_public_did(pub_did.to_string()).to_a2a_message();

    cache::save_to_cache("pub_invite", "0", &pub_invite)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    use utils::devsetup::*;
    use api::vcx::vcx_shutdown;

    #[test]
    fn test_connect_register_provision_config_path() {
        let agency_did = "LTjTWsezEmV4wJYD5Ufxvk";
        let agency_vk = "BcCSmgdfChLqmtBkkA26YotWVFBNnyY45WCnQziF4cqN";
        let host = "https://eas.pdev.evernym.com";
        let wallet_key = "test_key";

        let path = if cfg!(target_os = "android") {
            env::var("EXTERNAL_STORAGE").unwrap() + "/tmp/custom1/"
        } else {
            "/tmp/custom1/".to_owned()
        };

        let config = json!({
            "wallet_name": "test_wallet",
            "storage_config": json!({
                "path": path
            }).to_string(),
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        //Creates wallet at custom location
        connect_register_provision(&config.to_string()).unwrap();
        assert!(std::path::Path::new(&(path + "test_wallet")).exists());
        vcx_shutdown(false);
        let my_config: Config = serde_json::from_str(&config.to_string()).unwrap();

        //Opens already created wallet at custom location
        configure_wallet(&my_config).unwrap();
    }

    #[test]
    fn test_connect_register_provision() {
        let _setup = SetupMocks::init();

        let agency_did = "Ab8TvZa3Q19VNkQVzAWVL7";
        let agency_vk = "5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf";
        let host = "http://www.whocares.org";
        let wallet_key = "test_key";
        let config = json!({
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        let result = connect_register_provision(&config.to_string()).unwrap();

        let expected = json!({
            "agency_did":"Ab8TvZa3Q19VNkQVzAWVL7",
            "agency_endpoint":"http://www.whocares.org",
            "agency_verkey":"5LXaR43B1aQyeh94VBP8LG1Sgvjk7aNfqiksBCSjwqbf",
            "genesis_path":"<CHANGE_ME>",
            "institution_did":"FhrSrYtQcw3p9xwf7NYemf",
            "institution_logo_url":"<CHANGE_ME>",
            "institution_name":"<CHANGE_ME>",
            "institution_verkey":"91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "protocol_type":"1.0",
            "remote_to_sdk_did":"A4a69qafqZHPLPPu5JFQrc",
            "remote_to_sdk_verkey":"5wTKXrdfUiTQ7f3sZJzvHpcS7XHHxiBkFtPCsynZtv4k",
            "sdk_to_remote_did":"FhrSrYtQcw3p9xwf7NYemf",
            "sdk_to_remote_verkey":"91qMFrZjXDoi2Vc8Mm14Ys112tEZdDegBZZoembFEATE",
            "wallet_key":"test_key",
            "wallet_name":"LIBVCX_SDK_WALLET"
        });

        assert_eq!(expected, ::serde_json::from_str::<serde_json::Value>(&result).unwrap());
    }

    #[test]
    fn test_method_type_serialization() {
        assert_eq!("\"1\"", serde_json::to_string::<ComMethodType>(&ComMethodType::A2A).unwrap());
        assert_eq!("\"2\"", serde_json::to_string::<ComMethodType>(&ComMethodType::Webhook).unwrap());
    }

    #[test]
    fn test_method_type_deserialization() {
        assert_eq!(ComMethodType::A2A, serde_json::from_str::<ComMethodType>("\"1\"").unwrap());
        assert_eq!(ComMethodType::Webhook, serde_json::from_str::<ComMethodType>("\"2\"").unwrap());
    }

    #[ignore]
    #[test]
    fn test_real_connect_register_provision() {
        let _setup = SetupDefaults::init();

        let agency_did = "VsKV7grR1BUE29mG2Fm2kX";
        let agency_vk = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";
        let host = "http://localhost:8080";
        let wallet_key = "test_key";
        let config = json!({
            "agency_url": host.to_string(),
            "agency_did": agency_did.to_string(),
            "agency_verkey": agency_vk.to_string(),
            "wallet_key": wallet_key.to_string(),
        });

        let result = connect_register_provision(&config.to_string()).unwrap();
        assert!(result.len() > 0);
    }

    #[test]
    fn test_update_agent_info() {
        let _setup = SetupMocks::init();

        update_agent_info("123", "value").unwrap();
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_update_agent_info_real() {
        let _setup = SetupLibraryAgencyV1::init();

        ::utils::devsetup::set_consumer();
        update_agent_info("7b7f97f2", "FCM:Value").unwrap();
    }
}
