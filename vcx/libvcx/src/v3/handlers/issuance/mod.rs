pub mod issuer;
pub mod states;
pub mod messages;
pub mod holder;

use std::collections::HashMap;
use error::prelude::*;
use messages::get_message::Message;
use v3::messages::a2a::{A2AMessage, MessageId};
use v3::handlers::issuance::issuer::IssuerSM;
use v3::handlers::issuance::messages::CredentialIssuanceMessage;
use v3::handlers::issuance::holder::HolderSM;
use v3::messages::issuance::credential::Credential;
use v3::messages::issuance::credential_offer::CredentialOffer;
use connection;

// Issuer

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issuer {
    issuer_sm: IssuerSM
}

impl Issuer {
    pub fn create(cred_def_handle: u32, credential_data: &str, source_id: &str) -> VcxResult<Issuer> {
        trace!("Issuer::issuer_create_credential >>> cred_def_handle: {:?}, credential_data: {:?}, source_id: {:?}", cred_def_handle, credential_data, source_id);

        let cred_def_id = ::credential_def::get_cred_def_id(cred_def_handle)?;
        let rev_reg_id = ::credential_def::get_rev_reg_id(cred_def_handle)?;
        let tails_file = ::credential_def::get_tails_file(cred_def_handle)?;
        let issuer_sm = IssuerSM::new(&cred_def_id, credential_data, rev_reg_id, tails_file, source_id);
        Ok(Issuer { issuer_sm })
    }

    pub fn send_credential_offer(&mut self, connection_handle: u32) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialInit(connection_handle))
    }

    pub fn send_credential(&mut self, _connection_handle: u32) -> VcxResult<()> { // TODO: should use connection_handle
        self.step(CredentialIssuanceMessage::CredentialSend())
    }

    pub fn get_state(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.state())
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn update_status(&mut self, msg: Option<String>) -> VcxResult<()> {
        match msg {
            Some(msg) => {
                let message: Message = ::serde_json::from_str(&msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot deserialize Message: {:?}", err)))?;

                let message = connection::decode_message(self.issuer_sm.get_connection_handle(), message)?;
                self.step(message.into())
            }
            None => {
                self.issuer_sm = self.issuer_sm.clone().update_state()?;
                Ok(())
            }
        }
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().handle_message(message)?;
        Ok(())
    }
}

// Holder

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Holder {
    holder_sm: HolderSM
}

impl Holder {
    pub fn create(credential_offer: CredentialOffer, source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::holder_create_credential >>> credential_offer: {:?}, source_id: {:?}", credential_offer, source_id);

        let holder_sm = HolderSM::new(credential_offer, source_id.to_string());

        Ok(Holder { holder_sm })
    }

    pub fn send_request(&mut self, connection_handle: u32) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialRequestSend(connection_handle))
    }

    pub fn update_state(&mut self, msg: Option<String>) -> VcxResult<()> {
        match msg {
            Some(msg) => {
                let message: Message = ::serde_json::from_str(&msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state: Message deserialization failed: {:?}", err)))?;

                let message = connection::decode_message(self.holder_sm.get_connection_handle(), message)?;
                self.step(message.into())
            }
            None => {
                self.holder_sm = self.holder_sm.clone().update_state()?;
                Ok(())
            }
        }
    }

    pub fn get_status(&self) -> u32 {
        self.holder_sm.state()
    }

    pub fn get_source_id(&self) -> String {
        self.holder_sm.get_source_id()
    }

    pub fn get_credential(&self) -> VcxResult<(String, Credential)> {
        self.holder_sm.get_credential()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().handle_message(message)?;
        Ok(())
    }

    pub fn get_credential_offer_message(connection_handle: u32, msg_id: &str) -> VcxResult<CredentialOffer> {
        let message = connection::get_message_by_id(connection_handle, msg_id.to_string())?;

        let (id, credential_offer): (MessageId, CredentialOffer) = match message {
            A2AMessage::CredentialOffer(credential_offer) => (credential_offer.id.clone(), credential_offer),
            msg => return Err(VcxError::from_msg(VcxErrorKind::InvalidMessages, format!("Message of different type was received: {:?}", msg)))
        };

        connection::add_pending_messages(connection_handle, map! { id => msg_id.to_string() })?;

        Ok(credential_offer)
    }

    pub fn get_credential_offer_messages(conn_handle: u32) -> VcxResult<Vec<CredentialOffer>> {
        let messages = connection::get_messages(conn_handle)?;
        let (uids, msgs): (HashMap<MessageId, String>, Vec<CredentialOffer>) = messages
            .into_iter()
            .filter_map(|(uid, a2a_message)| {
                match a2a_message {
                    A2AMessage::CredentialOffer(credential_offer) => {
                        Some((uid, credential_offer.id.clone(), credential_offer))
                    }
                    _ => None
                }
            }).fold((HashMap::new(), vec![]), |(mut uids, mut msgs), (uid, id, msg)| {
            uids.insert(id, uid);
            msgs.push(msg);
            (uids, msgs)
        });

        connection::add_pending_messages(conn_handle, uids)?;

        Ok(msgs)
    }
}
