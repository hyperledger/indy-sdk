use serde_json;

use std::collections::HashMap;
use api::VcxStateType;
use messages;
use settings;
use messages::{RemoteMessageType, MessageStatusCode, GeneralMessage};
use messages::payload::{Payloads, PayloadKinds};
use messages::thread::Thread;
use messages::get_message::get_ref_msg;
use connection;
use credential_request::CredentialRequest;
use utils::error;
use utils::libindy::{payments, anoncreds};
use utils::constants::CRED_MSG;
use utils::openssl::encode;
use utils::libindy::payments::PaymentTxn;
use utils::qualifier;
use object_cache::ObjectCache;
use error::prelude::*;

use v3::handlers::issuance::Issuer;
use utils::agent_info::{get_agent_info, MyAgentInfo, get_agent_attr};

lazy_static! {
    static ref ISSUER_CREDENTIAL_MAP: ObjectCache < IssuerCredentials > = Default::default();
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum IssuerCredentials {
    #[serde(rename = "3.0")]
    Pending(IssuerCredential),
    #[serde(rename = "1.0")]
    V1(IssuerCredential),
    #[serde(rename = "2.0")]
    V3(Issuer),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct IssuerCredential {
    source_id: String,
    credential_attributes: String,
    msg_uid: String,
    schema_seq_no: u32,
    issuer_did: String,
    state: VcxStateType,
    pub credential_request: Option<CredentialRequest>,
    pub credential_offer: Option<CredentialOffer>,
    credential_name: String,
    pub credential_id: String,
    pub cred_def_id: String,
    pub cred_def_handle: u32,
    ref_msg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev_reg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tails_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev_reg_def_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cred_rev_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev_cred_payment_txn: Option<PaymentTxn>,
    price: u64,
    payment_address: Option<String>,
    #[serde(rename = "issued_did")]
    my_did: Option<String>,
    #[serde(rename = "issued_vk")]
    my_vk: Option<String>,
    #[serde(rename = "remote_did")]
    their_did: Option<String>,
    #[serde(rename = "remote_vk")]
    their_vk: Option<String>,
    agent_did: Option<String>,
    agent_vk: Option<String>,
    thread: Option<Thread>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CredentialOffer {
    pub msg_type: String,
    pub version: String,
    //vcx version of cred_offer
    pub to_did: String,
    //their_pw_did for this relationship
    pub from_did: String,
    //my_pw_did for this relationship
    pub libindy_offer: String,
    pub cred_def_id: String,
    pub credential_attrs: serde_json::Map<String, serde_json::Value>,
    //promised attributes revealed in credential
    pub schema_seq_no: u32,
    pub claim_name: String,
    pub claim_id: String,
    pub msg_ref_id: Option<String>,
    pub thread_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CredentialMessage {
    pub libindy_cred: String,
    pub rev_reg_def_json: String,
    pub cred_def_id: String,
    pub msg_type: String,
    pub claim_offer_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_revoc_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoc_reg_delta_json: Option<String>,
    pub version: String,
    pub from_did: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PaymentInfo {
    pub payment_required: String,
    pub payment_addr: String,
    pub price: u64,
}

impl PaymentInfo {
    pub fn get_address(&self) -> String {
        self.payment_addr.to_string()
    }

    pub fn get_price(&self) -> u64 {
        self.price
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize payment info: {:?}", err)))
    }
}

impl IssuerCredential {
    pub fn create(cred_def_handle: u32,
                  source_id: String,
                  issuer_did: String,
                  credential_name: String,
                  credential_data: String,
                  price: u64) -> VcxResult<IssuerCredential> {
        trace!("create >>> cred_def_handle: {}, source_id: {}, issuer_did: {}, credential_name: {}, credential_data: {}, price: {}",
               cred_def_handle, source_id, issuer_did, credential_name, secret!( & credential_data), price);

        let cred_def_id = ::credential_def::get_cred_def_id(cred_def_handle)?;
        let rev_reg_id = ::credential_def::get_rev_reg_id(cred_def_handle)?;
        let tails_file = ::credential_def::get_tails_file(cred_def_handle)?;
        let rev_reg_def_json = ::credential_def::get_rev_reg_def(cred_def_handle)?;

        let mut issuer_credential = IssuerCredential {
            credential_id: source_id.to_string(),
            source_id,
            msg_uid: String::new(),
            credential_attributes: credential_data,
            issuer_did,
            state: VcxStateType::VcxStateNone,
            //Todo: Take out schema
            schema_seq_no: 0,
            credential_request: None,
            credential_offer: None,
            credential_name,
            ref_msg_id: None,
            rev_reg_id,
            rev_reg_def_json,
            cred_rev_id: None,
            rev_cred_payment_txn: None,
            tails_file,
            price,
            payment_address: None,
            cred_def_id,
            cred_def_handle,
            thread: Some(Thread::new()),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        };
        apply_agent_info(&mut issuer_credential, &get_agent_info()?);

        issuer_credential.validate_credential_offer()?;

        issuer_credential.state = VcxStateType::VcxStateInitialized;

        Ok(issuer_credential)
    }

    fn validate_credential_offer(&self) -> VcxResult<u32> {
        //TODO: validate credential_attributes against credential_def
        debug!("successfully validated issuer_credential {}", self.source_id);
        Ok(error::SUCCESS.code_num)
    }

    pub fn generate_credential_offer_msg(&mut self) -> VcxResult<(String, String)> {
        let mut payload = Vec::new();

        let connection_name = settings::get_config_value(settings::CONFIG_INSTITUTION_NAME)?;

        let payment = self.generate_payment_info()?;

        let title = if let Some(x) = payment {
            payload.push(json!(x));
            format!("{} wants you to pay tokens for: {}", connection_name, self.credential_name)
        } else {
            format!("{} is offering you a credential: {}", connection_name, self.credential_name)
        };

        let credential_offer = self.generate_credential_offer()?;
        let cred_json = json!(credential_offer);

        payload.push(cred_json);

        let payload = serde_json::to_string(&payload)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot serialize payload: {}", err)))?;

        self.credential_offer = Some(credential_offer);

        Ok((payload, title))
    }

    fn send_credential_offer(&mut self, connection_handle: u32) -> VcxResult<u32> {
        trace!("IssuerCredential::send_credential_offer >>> connection_handle: {}", connection_handle);

        debug!("sending credential offer for issuer_credential {} to connection {}", self.source_id, connection::get_source_id(connection_handle).unwrap_or_default());
        if self.state != VcxStateType::VcxStateInitialized {
            warn!("credential {} has invalid state {} for sending credentialOffer", self.source_id, self.state as u32);
            return Err(VcxError::from_msg(VcxErrorKind::NotReady, format!("credential {} has invalid state {} for sending credentialOffer", self.source_id, self.state as u32)));
        }

        if !connection::is_valid_handle(connection_handle) {
            warn!("invalid connection handle ({})", connection_handle);
            return Err(VcxError::from_msg(VcxErrorKind::InvalidConnectionHandle, format!("invalid connection handle ({})", connection_handle)));
        }

        let agent_info = get_agent_info()?.pw_info(connection_handle)?;

        let (payload, title) = self.generate_credential_offer_msg()?;

        debug!("credential offer data: {}", secret!(&payload));

        let response =
            messages::send_message()
                .to(&agent_info.my_pw_did()?)?
                .to_vk(&agent_info.my_pw_vk()?)?
                .msg_type(&RemoteMessageType::CredOffer)?
                .edge_agent_payload(&agent_info.my_pw_vk()?,
                                    &agent_info.their_pw_vk()?,
                                    &payload,
                                    PayloadKinds::CredOffer,
                                    self.thread.clone(),
                )?
                .agent_did(&agent_info.pw_agent_did()?)?
                .agent_vk(&agent_info.pw_agent_vk()?)?
                .set_title(&title)?
                .set_detail(&title)?
                .status_code(&MessageStatusCode::Accepted)?
                .send_secure()
                .map_err(|err| err.extend("could not send credential offer"))?;

        apply_agent_info(self, &agent_info);
        self.msg_uid = response.get_msg_uid()?;
        self.state = VcxStateType::VcxStateOfferSent;

        debug!("sent credential offer for: {}", self.source_id);
        Ok(error::SUCCESS.code_num)
    }

    fn generate_credential_msg(&mut self, my_pw_did: &str) -> VcxResult<String> {
        let attrs_with_encodings = self.create_attributes_encodings()?;

        let data = if settings::indy_mocks_enabled() {
            CRED_MSG.to_string()
        } else {
            let cred = self.generate_credential(&attrs_with_encodings, my_pw_did)?;
            serde_json::to_string(&cred)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredential, format!("Cannot serialize credential: {}", err)))?
        };

        Ok(data)
    }

    fn send_credential(&mut self, connection_handle: u32) -> VcxResult<u32> {
        trace!("IssuerCredential::send_credential >>> connection_handle: {}", connection_handle);

        debug!("sending credential for issuer_credential {} to connection {}", self.source_id, connection::get_source_id(connection_handle).unwrap_or_default());
        if self.state != VcxStateType::VcxStateRequestReceived {
            warn!("credential {} has invalid state {} for sending credential", self.source_id, self.state as u32);
            return Err(VcxError::from_msg(VcxErrorKind::NotReady, format!("credential {} has invalid state {} for sending credential", self.source_id, self.state as u32)));
        }

        if connection::is_valid_handle(connection_handle) == false {
            warn!("invalid connection handle ({}) in send_credential_offer", connection_handle);
            return Err(VcxError::from_msg(VcxErrorKind::InvalidCredentialHandle, format!("invalid connection handle ({}) in send_credential_offer", connection_handle)));
        }

        self.verify_payment()?;

        let agent_info = get_agent_info()?.pw_info(connection_handle)?;

        let data = self.generate_credential_msg(&agent_info.my_pw_did()?)?;

        debug!("credential data: {}", secret!(&data));

        let cred_req_msg_id = self.credential_request
            .as_ref()
            .and_then(|cred_req| cred_req.msg_ref_id.as_ref())
            .ok_or(VcxError::from(VcxErrorKind::InvalidCredentialRequest))?;

        self.thread.as_mut().map(|thread| thread.sender_order += 1);

        let response = messages::send_message()
            .to(&agent_info.my_pw_did()?)?
            .to_vk(&agent_info.my_pw_vk()?)?
            .msg_type(&RemoteMessageType::Cred)?
            .status_code(&MessageStatusCode::Accepted)?
            .edge_agent_payload(&agent_info.my_pw_vk()?,
                                &agent_info.their_pw_vk()?,
                                &data,
                                PayloadKinds::Cred,
                                self.thread.clone(),
            )?
            .agent_did(&agent_info.pw_agent_did()?)?
            .agent_vk(&agent_info.pw_agent_vk()?)?
            .ref_msg_id(Some(cred_req_msg_id.to_string()))?
            .send_secure()
            .map_err(|err| err.extend("could not send credential offer"))?;

        apply_agent_info(self, &agent_info);
        self.msg_uid = response.get_msg_uid()?;
        self.state = VcxStateType::VcxStateAccepted;

        debug!("issued credential: {}", self.source_id);
        Ok(error::SUCCESS.code_num)
    }

    pub fn create_attributes_encodings(&self) -> VcxResult<String> {
        encode_attributes(&self.credential_attributes)
    }

    // TODO: The error arm of this Result is never used in any calling functions.
    // So currently there is no way to test the error status.
    fn get_credential_offer_status(&mut self, message: Option<String>) -> VcxResult<u32> {
        debug!("updating state for credential offer: {} msg_uid: {:?}", self.source_id, self.msg_uid);
        if self.state == VcxStateType::VcxStateRequestReceived {
            return Ok(self.get_state());
        }

        if message.is_none() && (self.state != VcxStateType::VcxStateOfferSent
            || self.msg_uid.is_empty()
            || self.my_did.is_none()) { return Ok(self.get_state()); }

        let (payload, offer_uid) = match message {
            None => {
                // Check cloud agent for pending messages
                let (msg_id, message) = get_ref_msg(&self.msg_uid,
                                                    &get_agent_attr(&self.my_did)?,
                                                    &get_agent_attr(&self.my_vk)?,
                                                    &get_agent_attr(&self.agent_did)?,
                                                    &get_agent_attr(&self.agent_vk)?)?;

                let (payload, thread) = Payloads::decrypt(&get_agent_attr(&self.my_vk)?, &message)
                    .map_err(|err| VcxError::from_msg(
                        VcxErrorKind::Common(err.into()),
                        "Cannot decrypt CredentialOffer payload")
                    )?;

                if let Some(_) = thread {
                    let remote_did = get_agent_attr(&self.their_did)?;
                    self.thread.as_mut().map(|thread| thread.increment_receiver(&remote_did));
                }

                (payload, Some(msg_id))
            }
            Some(ref payload) => (payload.clone(), None)
        };

        let mut cred_req: CredentialRequest = serde_json::from_str(&payload)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize CredentialRequest: {}", err),
            ))?;

        cred_req.msg_ref_id = offer_uid;

        self.credential_request = Some(cred_req);
        debug!("received credential request for credential offer: {}", self.source_id);
        self.state = VcxStateType::VcxStateRequestReceived;
        Ok(self.get_state())
    }

    fn update_state(&mut self, message: Option<String>) -> VcxResult<u32> {
        trace!("IssuerCredential::update_state >>>");
        let result = self.get_credential_offer_status(message);
        debug!("result: {:?}", result);
        result
        //There will probably be more things here once we do other things with the credential
    }

    fn get_state(&self) -> u32 {
        trace!("IssuerCredential::get_state >>>");
        self.state as u32
    }
    fn get_offer_uid(&self) -> &String { &self.msg_uid }

    fn get_credential_attributes(&self) -> &String { &self.credential_attributes }
    fn get_source_id(&self) -> &String { &self.source_id }

    fn generate_credential(&mut self, credential_data: &str, did: &str) -> VcxResult<CredentialMessage> {
        let indy_cred_offer = self.credential_offer
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidCredential, "Invalid Credential: `credential_offer` field not found"))?;

        let indy_cred_req = self.credential_request
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidCredentialRequest, "Invalid Credential: `credential_request` field not found"))?;

        let (cred, cred_revoc_id, revoc_reg_delta_json) =
            anoncreds::libindy_issuer_create_credential(&indy_cred_offer.libindy_offer,
                                                        &indy_cred_req.libindy_cred_req,
                                                        &credential_data,
                                                        self.rev_reg_id.clone(),
                                                        self.tails_file.clone())?;

        self.cred_rev_id = cred_revoc_id.clone();

        let their_pw_did = get_agent_attr(&self.their_did)?;

        let cred_def_id =
            if !qualifier::is_fully_qualified(&their_pw_did) {
                anoncreds::libindy_to_unqualified(&self.cred_def_id)?
            } else {
                self.cred_def_id.clone()
            };

        Ok(CredentialMessage {
            claim_offer_id: self.msg_uid.clone(),
            from_did: String::from(did),
            version: String::from("0.1"),
            msg_type: PayloadKinds::Cred.name().to_string(),
            libindy_cred: cred,
            rev_reg_def_json: self.rev_reg_def_json.clone().unwrap_or(String::new()),
            cred_def_id,
            cred_revoc_id,
            revoc_reg_delta_json,
        })
    }

    fn generate_credential_offer(&self) -> VcxResult<CredentialOffer> {
        let attr_map = convert_to_map(&self.credential_attributes)?;
        let libindy_offer = anoncreds::libindy_issuer_create_credential_offer(&self.cred_def_id)?;

        let my_did = self.my_did.clone().unwrap_or_default();
        let their_did = self.their_did.clone().unwrap_or_default();

        debug!("generate_credential_offer remote_did {}", their_did);

        let (libindy_offer, cred_def_id) =
            if !qualifier::is_fully_qualified(&their_did) {
                (anoncreds::libindy_to_unqualified(&libindy_offer)?,
                 anoncreds::libindy_to_unqualified(&self.cred_def_id)?)
            } else {
                (libindy_offer, self.cred_def_id.clone())
            };

        Ok(CredentialOffer {
            msg_type: PayloadKinds::CredOffer.name().to_string(),
            version: String::from("0.1"),
            to_did: their_did,
            from_did: my_did,
            credential_attrs: attr_map,
            schema_seq_no: self.schema_seq_no.clone(),
            claim_name: String::from(self.credential_name.clone()),
            claim_id: String::from(self.credential_id.clone()),
            msg_ref_id: None,
            cred_def_id,
            libindy_offer,
            thread_id: None,
        })
    }

    fn revoke_cred(&mut self) -> VcxResult<()> {
        let tails_file = self.tails_file
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationInfo: `tails_file` field not found"))?;

        let rev_reg_id = self.rev_reg_id
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationInfo: `rev_reg_id` field not found"))?;

        let cred_rev_id = self.cred_rev_id
            .as_ref()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationInfo: `cred_rev_id` field not found"))?;

        let (payment, _) = anoncreds::revoke_credential(tails_file, rev_reg_id, cred_rev_id)?;

        self.rev_cred_payment_txn = payment;
        Ok(())
    }

    fn generate_payment_info(&mut self) -> VcxResult<Option<PaymentInfo>> {
        if self.price > 0 {
            let address: String = ::utils::libindy::payments::create_address(None)?;
            self.payment_address = Some(address.clone());
            Ok(Some(PaymentInfo {
                payment_required: "one-time".to_string(),
                payment_addr: address,
                price: self.price,
            }))
        } else {
            Ok(None)
        }
    }

    fn verify_payment(&mut self) -> VcxResult<()> {
        if self.price > 0 {
            let invoice_address = self.payment_address.as_ref()
                .ok_or(VcxError::from(VcxErrorKind::InvalidPaymentAddress))?;

            let address = payments::get_address_info(&invoice_address)?;

            if address.balance < self.price { return Err(VcxError::from(VcxErrorKind::InsufficientTokenAmount)); }
        }
        Ok(())
    }

    fn get_payment_txn(&self) -> VcxResult<PaymentTxn> {
        trace!("IssuerCredential::get_payment_txn >>>");

        match self.payment_address {
            Some(ref payment_address) if self.price > 0 => {
                Ok(PaymentTxn {
                    amount: self.price,
                    credit: true,
                    inputs: vec![payment_address.to_string()],
                    outputs: Vec::new(),
                })
            }
            _ => Err(VcxError::from(VcxErrorKind::NoPaymentInformation))
        }
    }
}

fn apply_agent_info(cred: &mut IssuerCredential, agent_info: &MyAgentInfo) {
    cred.my_did = agent_info.my_pw_did.clone();
    cred.my_vk = agent_info.my_pw_vk.clone();
    cred.their_did = agent_info.their_pw_did.clone();
    cred.their_vk = agent_info.their_pw_vk.clone();
    cred.agent_did = agent_info.pw_agent_did.clone();
    cred.agent_vk = agent_info.pw_agent_vk.clone();
}

/**
    Input: supporting two formats:
    eg:
    perferred format: json, property/values
    {"address2":"101 Wilson Lane"}
    or
    deprecated format: json, key/array (of one item)
    {"address2":["101 Wilson Lane"]}


    Output: json: dictionary with key, object of raw and encoded values
    eg:
    {
      "address2": {
        "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
        "raw": "101 Wilson Lane"
      }
    }
*/

pub fn encode_attributes(attributes: &str) -> VcxResult<String> {
    let mut attributes: HashMap<String, serde_json::Value> = serde_json::from_str(attributes)
        .map_err(|err| {
            warn!("Invalid Json for Attribute data");
            VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize credential attributes: {}", err))
        })?;

    let mut dictionary = HashMap::new();

    for (attr, attr_data) in attributes.iter_mut() {
        let first_attr: &str = match attr_data {
            // old style input such as {"address2":["101 Wilson Lane"]}
            serde_json::Value::Array(array_type) => {
                let attrib_value: &str = match array_type.get(0).and_then(serde_json::Value::as_str) {
                    Some(x) => x,
                    None => {
                        warn!("Cannot encode attribute: {}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
                        return Err(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, "Attribute value not found"));
                    }
                };

                warn!("Old attribute format detected. See vcx_issuer_create_credential api for additional information.");
                attrib_value
            }

            // new style input such as {"address2":"101 Wilson Lane"}
            serde_json::Value::String(str_type) => str_type,
            // anything else is an error
            _ => {
                warn!("Invalid Json for Attribute data");
                return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Invalid Json for Attribute data"));
            }
        };

        let encoded = encode(&first_attr)?;
        let attrib_values = json!({
            "raw": first_attr,
            "encoded": encoded
        });

        dictionary.insert(attr, attrib_values);
    }

    serde_json::to_string_pretty(&dictionary)
        .map_err(|err| {
            warn!("Invalid Json for Attribute data");
            VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid Json for Attribute data: {}", err))
        })
}

pub fn get_encoded_attributes(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => obj.create_attributes_encodings(),
            IssuerCredentials::V1(ref obj) => obj.create_attributes_encodings(),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle))
        }
    })
}

pub fn get_offer_uid(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => Ok(obj.get_offer_uid().to_string()),
            IssuerCredentials::V1(ref obj) => Ok(obj.get_offer_uid().to_string()),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle))
        }
    })
}

pub fn get_payment_txn(handle: u32) -> VcxResult<PaymentTxn> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => obj.get_payment_txn(),
            IssuerCredentials::V1(ref obj) => obj.get_payment_txn(),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::NoPaymentInformation))
        }
    })
}

pub fn issuer_credential_create(cred_def_handle: u32,
                                source_id: String,
                                issuer_did: String,
                                credential_name: String,
                                credential_data: String,
                                price: u64) -> VcxResult<u32> {
    trace!("issuer_credential_create >>> cred_def_handle: {}, source_id: {}, issuer_did: {}, credential_name: {}, credential_data: {}, price: {}",
           cred_def_handle, source_id, issuer_did, credential_name, secret!(&credential_data), price);

//    // Initiate connection of new format -- redirect to v3 folder
//    if settings::is_aries_protocol_set() {
//        let issuer = v3::handlers::issuance::Issuer::create(cred_def_handle, &credential_data, &source_id)?;
//        return ISSUER_CREDENTIAL_MAP.add(IssuerCredentials::V3(issuer));
//    }

    let issuer_credential = IssuerCredential::create(cred_def_handle, source_id, issuer_did, credential_name, credential_data, price)?;

    let handle = ISSUER_CREDENTIAL_MAP.add(IssuerCredentials::Pending(issuer_credential))?;
    debug!("creating issuer_credential {} with handle {}", get_source_id(handle).unwrap_or_default(), handle);

    Ok(handle)
}

pub fn update_state(handle: u32, message: Option<String>) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref mut obj) => {
                obj.update_state(message.clone())
                    .or_else(|_| Ok(obj.get_state()))
            }
            IssuerCredentials::V1(ref mut obj) => {
                obj.update_state(message.clone())
                    .or_else(|_| Ok(obj.get_state()))
            }
            IssuerCredentials::V3(ref mut obj) => {
                obj.update_status(message.clone())?;
                obj.get_state()
            }
        }
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => Ok(obj.get_state()),
            IssuerCredentials::V1(ref obj) => Ok(obj.get_state()),
            IssuerCredentials::V3(ref obj) => obj.get_state(),
        }
    })
}

pub fn get_credential_status(handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidIssuerCredentialHandle, "Cannot get credential status for V1 Credential object")),
            IssuerCredentials::V1(_) => Err(VcxError::from_msg(VcxErrorKind::InvalidIssuerCredentialHandle, "Cannot get credential status for V1 Credential object")),
            IssuerCredentials::V3(ref obj) => obj.get_credential_status(),
        }
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle)))
}

pub fn release_all() {
    ISSUER_CREDENTIAL_MAP.drain().ok();
}

pub fn is_valid_handle(handle: u32) -> bool {
    ISSUER_CREDENTIAL_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        serde_json::to_string(obj)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize IssuerCredential object: {:?}", err)))
    })
}

pub fn from_string(credential_data: &str) -> VcxResult<u32> {
    let issuer_credential: IssuerCredentials = serde_json::from_str(credential_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize IssuerCredential: {:?}", err)))?;

    ISSUER_CREDENTIAL_MAP.add(issuer_credential)
}

pub fn generate_credential_offer_msg(handle: u32) -> VcxResult<(String, String)> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref mut obj) => obj.generate_credential_offer_msg(),
            IssuerCredentials::V1(ref mut obj) => obj.generate_credential_offer_msg(),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle)), // TODO: implement
        }
    })
}

pub fn send_credential_offer(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        let new_credential = match credential {
            IssuerCredentials::Pending(ref mut obj) => {
                // if Aries connection is established --> Convert Pending object to Aries credential
                if ::connection::is_v3_connection(connection_handle)? {
                    let mut issuer = Issuer::create(obj.cred_def_handle, &obj.credential_attributes, &obj.source_id)?;
                    issuer.send_credential_offer(connection_handle)?;

                    IssuerCredentials::V3(issuer)
                } else { // else - Convert Pending object to Proprietary credential
                    obj.send_credential_offer(connection_handle)?;
                    IssuerCredentials::V1(obj.clone())
                }
            }
            IssuerCredentials::V1(ref mut obj) => {
                obj.send_credential_offer(connection_handle)?;
                IssuerCredentials::V1(obj.clone())
            }
            IssuerCredentials::V3(ref mut obj) => {
                obj.send_credential_offer(connection_handle)?;
                IssuerCredentials::V3(obj.clone())
            }
        };
        *credential = new_credential;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_credential_msg(handle: u32, my_pw_did: &str) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref mut obj) => obj.generate_credential_msg(my_pw_did),
            IssuerCredentials::V1(ref mut obj) => obj.generate_credential_msg(my_pw_did),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle)), // TODO: implement
        }
    })
}

pub fn send_credential(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref mut obj) => {
                obj.send_credential(connection_handle)
            }
            IssuerCredentials::V1(ref mut obj) => {
                obj.send_credential(connection_handle)
            }
            IssuerCredentials::V3(ref mut obj) => {
                obj.send_credential(connection_handle)?;
                Ok(error::SUCCESS.code_num)
            }
        }
    })
}

pub fn revoke_credential(handle: u32) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref mut obj) => obj.revoke_cred(),
            IssuerCredentials::V1(ref mut obj) => obj.revoke_cred(),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::NotReady)), // TODO: implement
        }
    })
}

pub fn convert_to_map(s: &str) -> VcxResult<serde_json::Map<String, serde_json::Value>> {
    serde_json::from_str(s)
        .map_err(|_| {
            warn!("{}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
            VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, error::INVALID_ATTRIBUTES_STRUCTURE.message)
        })
}

pub fn get_credential_attributes(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => Ok(obj.get_credential_attributes().to_string()),
            IssuerCredentials::V1(ref obj) => Ok(obj.get_credential_attributes().to_string()),
            IssuerCredentials::V3(_) => Err(VcxError::from(VcxErrorKind::NotReady)), // TODO: implement
        }
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |obj| {
        match obj {
            IssuerCredentials::Pending(ref obj) => Ok(obj.get_source_id().to_string()),
            IssuerCredentials::V1(ref obj) => Ok(obj.get_source_id().to_string()),
            IssuerCredentials::V3(ref obj) => obj.get_source_id()
        }
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::Value;
    use settings;
    use connection::tests::build_test_connection;
    use credential_request::CredentialRequest;
    #[allow(unused_imports)]
    use utils::{constants::*,
                libindy::{LibindyMock,
                          anoncreds::{libindy_create_and_store_credential_def,
                                      libindy_issuer_create_credential_offer,
                                      libindy_prover_create_credential_req},
                          wallet::get_wallet_handle, wallet},
                get_temp_dir_path,
    };
    use utils::devsetup::*;
    use utils::httpclient::AgencyMock;
    use credential_def::tests::create_cred_def_fake;

    static DEFAULT_CREDENTIAL_NAME: &str = "Credential";
    static DEFAULT_CREDENTIAL_ID: &str = "defaultCredentialId";

    static CREDENTIAL_DATA: &str =
        r#"{"address2":["101 Wilson Lane"],
        "zip":["87121"],
        "state":["UT"],
        "city":["SLC"],
        "address1":["101 Tela Lane"]
        }"#;

    pub fn util_put_credential_def_in_issuer_wallet(_schema_seq_num: u32, _wallet_handle: i32) {
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let tag = "test_tag";
        let config = "{support_revocation: false}";

        libindy_create_and_store_credential_def(&issuer_did, SCHEMAS_JSON, tag, None, config).unwrap();
    }

    fn default_agent_info(connection_handle: Option<u32>) -> MyAgentInfo {
        MyAgentInfo {
            my_pw_did: Some("8XFh8yBzrpJQmNyZzgoTqB".to_string()),
            my_pw_vk: Some(VERKEY.to_string()),
            their_pw_did: Some(DID.to_string()),
            their_pw_vk: Some(VERKEY.to_string()),
            pw_agent_did: Some(DID.to_string()),
            pw_agent_vk: Some(VERKEY.to_string()),
            agent_did: DID.to_string(),
            agent_vk: VERKEY.to_string(),
            agency_did: DID.to_string(),
            agency_vk: VERKEY.to_string(),
            version: None,
            connection_handle,
        }
    }

    pub fn create_standard_issuer_credential(connection_handle: Option<u32>) -> IssuerCredential {
        let credential_req: CredentialRequest = serde_json::from_str(CREDENTIAL_REQ_STRING).unwrap();
        let (credential_offer, _) = ::credential::parse_json_offer(CREDENTIAL_OFFER_JSON).unwrap();
        let mut issuer_credential = IssuerCredential {
            source_id: "standard_credential".to_owned(),
            schema_seq_no: 32,
            msg_uid: "1234".to_owned(),
            credential_attributes: CREDENTIAL_DATA.to_owned(),
            issuer_did: "QTrbV4raAcND4DWWzBmdsh".to_owned(),
            state: VcxStateType::VcxStateInitialized,
            credential_name: DEFAULT_CREDENTIAL_NAME.to_owned(),
            credential_request: Some(credential_req.to_owned()),
            credential_offer: Some(credential_offer.to_owned()),
            credential_id: String::from(DEFAULT_CREDENTIAL_ID),
            price: 1,
            payment_address: Some(payments::build_test_address("9UFgyjuJxi1i1HD")),
            ref_msg_id: None,
            rev_reg_id: None,
            tails_file: None,
            cred_rev_id: None,
            rev_cred_payment_txn: None,
            rev_reg_def_json: None,
            cred_def_id: CRED_DEF_ID.to_string(),
            cred_def_handle: 0,
            thread: Some(Thread::new()),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        };
        apply_agent_info(&mut issuer_credential, &default_agent_info(connection_handle));
        issuer_credential
    }

    pub fn create_standard_issuer_credential_json(connection_handle: Option<u32>) -> String {
        let issuer_credential = create_standard_issuer_credential(connection_handle);
        serde_json::to_string(&IssuerCredentials::V1(issuer_credential)).unwrap()
    }

    pub fn create_pending_issuer_credential() -> IssuerCredential {
        let credential_req: CredentialRequest = serde_json::from_str(CREDENTIAL_REQ_STRING).unwrap();
        let (credential_offer, _) = ::credential::parse_json_offer(CREDENTIAL_OFFER_JSON).unwrap();
        let connection_handle = Some(::connection::tests::build_test_connection());
        let mut credential: IssuerCredential = IssuerCredential {
            source_id: "test_has_pending_credential_request".to_owned(),
            schema_seq_no: 32,
            msg_uid: "1234".to_owned(),
            credential_attributes: "nothing".to_owned(),
            issuer_did: "QTrbV4raAcND4DWWzBmdsh".to_owned(),
            state: VcxStateType::VcxStateOfferSent,
            credential_request: Some(credential_req.to_owned()),
            credential_offer: Some(credential_offer.to_owned()),
            credential_name: DEFAULT_CREDENTIAL_NAME.to_owned(),
            credential_id: String::from(DEFAULT_CREDENTIAL_ID),
            cred_def_id: CRED_DEF_ID.to_string(),
            cred_def_handle: 1,
            ref_msg_id: None,
            rev_reg_id: None,
            cred_rev_id: None,
            rev_cred_payment_txn: None,
            rev_reg_def_json: None,
            tails_file: None,
            price: 0,
            payment_address: None,
            thread: Some(Thread::new()),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        };

        apply_agent_info(&mut credential, &default_agent_info(connection_handle));
        credential
    }

    pub fn create_full_issuer_credential() -> (IssuerCredential, ::credential::Credential) {
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (_, cred_def_handle) = ::credential_def::tests::create_cred_def_real(true);
        let cred_def_id = ::credential_def::get_cred_def_id(cred_def_handle).unwrap();
        let rev_reg_id = ::credential_def::get_rev_reg_id(cred_def_handle).unwrap();
        let tails_file = ::credential_def::get_tails_file(cred_def_handle).unwrap();
        let rev_reg_def_json = ::credential_def::get_rev_reg_def(cred_def_handle).unwrap();
        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;

        let mut issuer_credential = IssuerCredential {
            source_id: "source_id".to_string(),
            msg_uid: String::new(),
            credential_attributes: credential_data.to_string(),
            issuer_did: issuer_did.to_string(),
            state: VcxStateType::VcxStateNone,
            //Todo: Take out schema
            schema_seq_no: 0,
            credential_request: None,
            credential_offer: None,
            credential_name: "cred_name".to_string(),
            credential_id: String::new(),
            ref_msg_id: None,
            rev_reg_id,
            rev_reg_def_json,
            cred_rev_id: None,
            rev_cred_payment_txn: None,
            tails_file,
            price: 1,
            payment_address: None,
            cred_def_id,
            cred_def_handle,
            thread: Some(Thread::new()),
            my_did: None,
            my_vk: None,
            their_did: None,
            their_vk: None,
            agent_did: None,
            agent_vk: None,
        };

        apply_agent_info(&mut issuer_credential, &get_agent_info().unwrap());

        let payment = issuer_credential.generate_payment_info().unwrap();
        let their_did = &issuer_credential.their_did.clone().unwrap_or_default();
        let credential_offer = issuer_credential.generate_credential_offer().unwrap();
        let cred_json = json!(credential_offer);
        let mut payload = Vec::new();

        if payment.is_some() { payload.push(json!(payment.unwrap())); }
        payload.push(cred_json);
        let payload = serde_json::to_string(&payload).unwrap();

        issuer_credential.credential_offer = Some(issuer_credential.generate_credential_offer().unwrap());
        let credential = ::credential::tests::create_credential(&payload);
        issuer_credential.credential_request = Some(credential.build_request(&issuer_credential.issuer_did, &their_did).unwrap());
        (issuer_credential, credential)
    }

    fn _issuer_credential_create() -> u32 {
        issuer_credential_create(create_cred_def_fake(),
                                 "1".to_string(),
                                 "8XFh8yBzrpJQmNyZzgoTqB".to_owned(),
                                 "credential_name".to_string(),
                                 "{\"attr\":\"value\"}".to_owned(),
                                 1).unwrap()
    }

    #[test]
    fn test_issuer_credential_create_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        assert!(handle > 0);
    }

    #[test]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        let string = to_string(handle).unwrap();
        assert!(!string.is_empty());
    }

    #[test]
    fn test_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();

        let handle = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle, connection_handle).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle).unwrap(), VcxStateType::VcxStateOfferSent as u32);
        assert_eq!(get_offer_uid(handle).unwrap(), "ntc2ytb");
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_generate_cred_offer() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let _issuer = create_full_issuer_credential().0
            .generate_credential_offer().unwrap();
    }

    #[test]
    fn test_retry_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection();

        let handle = _issuer_credential_create();
        assert_eq!(get_state(handle).unwrap(), VcxStateType::VcxStateInitialized as u32);

        LibindyMock::set_next_result(error::TIMEOUT_LIBINDY_ERROR.code_num);

        let res = send_credential_offer(handle, connection_handle).unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidState);
        assert_eq!(get_state(handle).unwrap(), VcxStateType::VcxStateInitialized as u32);
        assert_eq!(get_offer_uid(handle).unwrap(), "");

        // Can retry after initial failure
        assert_eq!(send_credential_offer(handle, connection_handle).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle).unwrap(), VcxStateType::VcxStateOfferSent as u32);
        assert_eq!(get_offer_uid(handle).unwrap(), "ntc2ytb");
    }

    #[test]
    fn test_credential_can_be_resent_after_failure() {
        let _setup = SetupMocks::init();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, "QTrbV4raAcND4DWWzBmdsh");

        let mut credential = create_standard_issuer_credential(None);
        credential.state = VcxStateType::VcxStateRequestReceived;

        let connection_handle = build_test_connection();

        LibindyMock::set_next_result(error::TIMEOUT_LIBINDY_ERROR.code_num);

        assert_eq!(credential.send_credential(connection_handle).unwrap_err().kind(), VcxErrorKind::Common(1038));
        assert_eq!(credential.msg_uid, "1234");
        assert_eq!(credential.state, VcxStateType::VcxStateRequestReceived);

        // Retry sending the credential, use the mocked http. Show that you can retry sending the credential
        credential.send_credential(connection_handle).unwrap();
        assert_eq!(credential.msg_uid, "ntc2ytb");
        assert_eq!(credential.state, VcxStateType::VcxStateAccepted);
    }

    #[test]
    fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();

        let string = to_string(handle).unwrap();

        let value: serde_json::Value = serde_json::from_str(&string).unwrap();
        assert_eq!(value["version"], PENDING_OBJECT_SERIALIZE_VERSION);

        release(handle).unwrap();

        let new_handle = from_string(&string).unwrap();

        let new_string = to_string(new_handle).unwrap();
        assert_eq!(new_string, string);
    }

    #[test]
    fn test_update_state_with_pending_credential_request() {
        let _setup = SetupMocks::init();

        let mut credential = create_pending_issuer_credential();

        AgencyMock::set_next_response(CREDENTIAL_REQ_RESPONSE.to_vec());
        AgencyMock::set_next_response(UPDATE_CREDENTIAL_RESPONSE.to_vec());

        credential.update_state(None).unwrap();
        assert_eq!(credential.get_state(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_update_state_with_message() {
        let _setup = SetupMocks::init();

        let mut credential = create_pending_issuer_credential();

        credential.update_state(Some(CREDENTIAL_REQ_RESPONSE_STR.to_string())).unwrap();
        assert_eq!(credential.get_state(), VcxStateType::VcxStateRequestReceived as u32);
    }

    #[test]
    fn test_update_state_with_bad_message() {
        let _setup = SetupMocks::init();

        let mut credential = create_pending_issuer_credential();

        let err = credential.update_state(Some(INVITE_ACCEPTED_RESPONSE.to_string())).unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::InvalidJson);

        assert_eq!(credential.get_state(), VcxStateType::VcxStateOfferSent as u32);
    }

    #[test]
    fn basic_add_attribute_encoding() {
        let _setup = SetupDefaults::init();

        // FIXME Make this a real test and add additional test for create_attributes_encodings
        let issuer_credential = create_standard_issuer_credential(None);
        issuer_credential.create_attributes_encodings().unwrap();

        let mut issuer_credential = create_standard_issuer_credential(None);
        assert_eq!(issuer_credential.credential_attributes, CREDENTIAL_DATA);

        issuer_credential.credential_attributes = String::from("attr");

        let res = issuer_credential.create_attributes_encodings().unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidJson);
    }

    #[test]
    fn test_that_test_mode_enabled_bypasses_libindy_create_credential() {
        let _setup = SetupMocks::init();

        let mut credential = create_standard_issuer_credential(None);
        credential.state = VcxStateType::VcxStateRequestReceived;

        let connection_handle = build_test_connection();

        credential.send_credential(connection_handle).unwrap();
        assert_eq!(credential.state, VcxStateType::VcxStateAccepted);
    }

    #[test]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = _issuer_credential_create();
        let h2 = _issuer_credential_create();
        let h3 = _issuer_credential_create();
        let h4 = _issuer_credential_create();
        let h5 = _issuer_credential_create();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
    }

    #[test]
    fn test_errors() {
        let _setup = SetupLibraryWallet::init();

        assert_eq!(to_string(0).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(release(0).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
    }

    #[test]
    fn test_encoding() {
        let _setup = SetupMocks::init();

        let issuer_credential_handle = issuer_credential_create(::credential_def::tests::create_cred_def_fake(),
                                                                "IssuerCredentialName".to_string(),
                                                                "000000000000000000000000Issuer02".to_string(),
                                                                "CredentialNameHere".to_string(),
                                                                r#"["name","gpa"]"#.to_string(),
                                                                1).unwrap();
        get_encoded_attributes(issuer_credential_handle).unwrap_err();

        let issuer_credential_handle = issuer_credential_create(::credential_def::tests::create_cred_def_fake(),
                                                                "IssuerCredentialName".to_string(),
                                                                "000000000000000000000000Issuer02".to_string(),
                                                                "CredentialNameHere".to_string(),
                                                                r#"{"name":["frank"],"gpa":["4.0"]}"#.to_string(),
                                                                1).unwrap();

        let _encoded_attributes = self::get_encoded_attributes(issuer_credential_handle).unwrap();
    }

    #[test]
    fn test_verify_payment() {
        let _setup = SetupMocks::init();

        let mut credential = create_standard_issuer_credential(None);

        // Success
        credential.price = 3;
        credential.payment_address = Some(payments::build_test_address("9UFgyjuJxi1i1HD"));
        assert!(credential.verify_payment().is_ok());

        // Err - Wrong payment amount
        credential.price = 200;
        assert_eq!(credential.verify_payment().unwrap_err().kind(), VcxErrorKind::InsufficientTokenAmount);

        // Err - address not set
        credential.payment_address = None;
        assert_eq!(credential.verify_payment().unwrap_err().kind(), VcxErrorKind::InvalidPaymentAddress);
    }

    #[test]
    fn test_send_credential_with_payments() {
        let _setup = SetupMocks::init();

        let mut credential = create_standard_issuer_credential(None);
        credential.state = VcxStateType::VcxStateRequestReceived;
        credential.price = 3;
        credential.payment_address = Some(payments::build_test_address("9UFgyjuJxi1i1HD"));

        let connection_handle = build_test_connection();

        // Success
        credential.send_credential(connection_handle).unwrap();
        assert_eq!(credential.msg_uid, "ntc2ytb");
        assert_eq!(credential.state, VcxStateType::VcxStateAccepted);

        // Amount wrong
        credential.state = VcxStateType::VcxStateRequestReceived;
        credential.price = 200;
        assert!(credential.send_credential(connection_handle).is_err());
        let payment = serde_json::to_string(&credential.get_payment_txn().unwrap()).unwrap();
        assert!(payment.len() > 20);
    }

    #[test]
    fn test_revoke_credential() {
        let _setup = SetupMocks::init();

        let mut credential = create_standard_issuer_credential(None);

        credential.tails_file = Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string());
        credential.cred_rev_id = None;
        credential.rev_reg_id = None;
        assert_eq!(credential.revoke_cred().unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);
        credential.tails_file = None;
        credential.cred_rev_id = Some(CRED_REV_ID.to_string());
        credential.rev_reg_id = None;
        assert_eq!(credential.revoke_cred().unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);
        credential.tails_file = None;
        credential.cred_rev_id = None;
        credential.rev_reg_id = Some(REV_REG_ID.to_string());
        assert_eq!(credential.revoke_cred().unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);

        credential.tails_file = Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string());
        credential.cred_rev_id = Some(CRED_REV_ID.to_string());
        credential.rev_reg_id = Some(REV_REG_ID.to_string());
        credential.rev_cred_payment_txn = None;

        credential.revoke_cred().unwrap();
        assert!(credential.rev_cred_payment_txn.is_some());
    }


    #[test]
    fn test_encode_with_several_attributes_success() {
        let _setup = SetupDefaults::init();

        //        for reference....expectation is encode_attributes returns this:

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });


        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"],
            "zip":["87121"],
            "state":["UT"],
            "city":["SLC"],
            "address1":["101 Tela Lane"]
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_with_one_attribute_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"]}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(expected_json, results_json, "encode_attributes failed to return expected results");
    }

    #[test]
    fn test_encode_with_new_format_several_attributes_success() {
        let _setup = SetupDefaults::init();

        //        for reference....expectation is encode_attributes returns this:

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":"101 Wilson Lane",
            "zip":"87121",
            "state":"UT",
            "city":"SLC",
            "address1":"101 Tela Lane"
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_with_new_format_one_attribute_success() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            }
        });

        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2": "101 Wilson Lane"}"#;

        let expected_json = serde_json::to_string_pretty(&expected).unwrap();

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        assert_eq!(expected_json, results_json, "encode_attributes failed to return expected results");
    }

    #[test]
    fn test_encode_with_mixed_format_several_attributes_success() {
        let _setup = SetupDefaults::init();

        //        for reference....expectation is encode_attributes returns this:

        let expected = json!({
            "address2": {
                "encoded": "68086943237164982734333428280784300550565381723532936263016368251445461241953",
                "raw": "101 Wilson Lane"
            },
            "zip": {
                "encoded": "87121",
                "raw": "87121"
            },
            "city": {
                "encoded": "101327353979588246869873249766058188995681113722618593621043638294296500696424",
                "raw": "SLC"
            },
            "address1": {
                "encoded": "63690509275174663089934667471948380740244018358024875547775652380902762701972",
                "raw": "101 Tela Lane"
            },
            "state": {
                "encoded": "93856629670657830351991220989031130499313559332549427637940645777813964461231",
                "raw": "UT"
            }
        });


        static TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":["101 Wilson Lane"],
            "zip":"87121",
            "state":"UT",
            "city":["SLC"],
            "address1":"101 Tela Lane"
            }"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }

    #[test]
    fn test_encode_bad_format_returns_error() {
        let _setup = SetupDefaults::init();

        static BAD_TEST_CREDENTIAL_DATA: &str =
            r#"{"format doesnt make sense"}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
    }

    #[test]
    fn test_encode_old_format_empty_array_error() {
        let _setup = SetupDefaults::init();

        static BAD_TEST_CREDENTIAL_DATA: &str =
            r#"{"address2":[]}"#;

        assert!(encode_attributes(BAD_TEST_CREDENTIAL_DATA).is_err())
    }

    #[test]
    fn test_encode_empty_field() {
        let _setup = SetupDefaults::init();

        let expected = json!({
            "empty_field": {
                "encoded": "102987336249554097029535212322581322789799900648198034993379397001115665086549",
                "raw": ""
            }
        });

        static TEST_CREDENTIAL_DATA: &str = r#"{"empty_field": ""}"#;

        let results_json = encode_attributes(TEST_CREDENTIAL_DATA).unwrap();

        let results: Value = serde_json::from_str(&results_json).unwrap();
        assert_eq!(expected, results);
    }
}
