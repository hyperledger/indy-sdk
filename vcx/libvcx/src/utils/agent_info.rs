use settings;
use error::{VcxResult, VcxErrorKind, VcxError};
use utils::option_util::get_or_err;
use connection::{get_pw_did, get_pw_verkey, get_their_pw_did, get_their_pw_verkey, get_agent_did, get_agent_verkey, get_version};
use settings::{ProtocolTypes, get_config_value, CONFIG_REMOTE_TO_SDK_DID, CONFIG_REMOTE_TO_SDK_VERKEY, CONFIG_AGENCY_DID, CONFIG_AGENCY_VERKEY};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MyAgentInfo {
    pub connection_handle: Option<u32>,
    pub my_pw_did: Option<String>,
    pub my_pw_vk: Option<String>,
    pub their_pw_did: Option<String>,
    pub their_pw_vk: Option<String>,
    pub pw_agent_did: Option<String>,
    pub pw_agent_vk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<settings::ProtocolTypes>,

    // User Agent
    pub agent_did: String,
    pub agent_vk: String,
    pub agency_did: String,
    pub agency_vk: String,
}

pub fn get_agent_attr(v: &Option<String>) -> VcxResult<String> { get_or_err(v, Some(VcxErrorKind::NoAgentInformation)) }

impl MyAgentInfo {
    pub fn connection_handle(&self) -> VcxResult<u32> {
        self.connection_handle
            .ok_or(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
    }

    fn retrieve(&self,
                value: &Option<String>,
                getter: fn(u32) -> VcxResult<String>) -> VcxResult<String> {
        value
            .as_ref()
            .map(|x| Ok(x.to_string()))
            .unwrap_or(getter(self.connection_handle()?))
    }

    pub fn my_pw_did(&self) -> VcxResult<String> { self.retrieve(&self.my_pw_did, get_pw_did) }

    pub fn my_pw_vk(&self) -> VcxResult<String> {  self.retrieve(&self.my_pw_vk, get_pw_verkey) }

    pub fn their_pw_did(&self) -> VcxResult<String> { self.retrieve(&self.their_pw_did, get_their_pw_did) }

    pub fn their_pw_vk(&self) -> VcxResult<String> { self.retrieve(&self.their_pw_vk, get_their_pw_verkey) }

    pub fn pw_agent_did(&self) -> VcxResult<String> { self.retrieve(&self.pw_agent_did, get_agent_did) }

    pub fn pw_agent_vk(&self) -> VcxResult<String> { self.retrieve(&self.pw_agent_vk, get_agent_verkey) }

    pub fn version(&self) -> VcxResult<Option<ProtocolTypes>> { get_version(self.connection_handle()?) }

    pub fn pw_info(&mut self, handle: u32) -> VcxResult<MyAgentInfo> {
        self.my_pw_did = Some(get_pw_did(handle)?);
        self.my_pw_vk = Some(get_pw_verkey(handle)?);
        self.their_pw_did = Some(get_their_pw_did(handle)?);
        self.their_pw_vk = Some(get_their_pw_verkey(handle)?);
        self.pw_agent_did = Some(get_agent_did(handle)?);
        self.pw_agent_vk = Some(get_agent_verkey(handle)?);
        self.version = get_version(handle)?;
        self.connection_handle = Some(handle);
        self.log();

        Ok(self.clone())
    }

    fn log(&self) {
        debug!("my_pw_did: {:?} -- my_pw_vk: {:?} -- their_pw_did: {:?} -- pw_agent_did: {:?} \
        -- pw_agent_vk: {:?} -- their_pw_vk: {:?}-- agent_did: {} -- agent_vk: {} -- version: {:?}",
               self.my_pw_did,
               self.my_pw_vk,
               self.their_pw_did,
               self.their_pw_vk,
               self.pw_agent_did,
               self.pw_agent_vk,
               self.agent_did,
               self.agent_vk,
               self.version,
        );
    }
}

pub fn get_agent_info() -> VcxResult<MyAgentInfo> {
    Ok(MyAgentInfo {
        connection_handle: None,
        my_pw_did: None,
        my_pw_vk: None,
        their_pw_did: None,
        their_pw_vk: None,
        pw_agent_did: None,
        pw_agent_vk: None,
        version: None,
        agent_did: get_config_value(CONFIG_REMOTE_TO_SDK_DID)?,
        agent_vk: get_config_value(CONFIG_REMOTE_TO_SDK_VERKEY)?,
        agency_did: get_config_value(CONFIG_AGENCY_DID)?,
        agency_vk: get_config_value(CONFIG_AGENCY_VERKEY)?,
    })
}
