use v3::messages::a2a::{MessageId, A2AMessage};
use v3::messages::issuance::CredentialPreviewData;
use v3::messages::attachment::{Attachments, AttachmentId};
use v3::messages::mime_type::MimeType;
use error::{VcxError, VcxResult, VcxErrorKind};
use messages::thread::Thread;
use issuer_credential::CredentialOffer as CredentialOfferV1;
use messages::payload::PayloadKinds;
use std::convert::TryInto;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialOffer {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreviewData,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>
}

impl CredentialOffer {
    pub fn create() -> Self {
        CredentialOffer::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_offers_attach(mut self, credential_offer: &str) -> VcxResult<CredentialOffer> {
        self.offers_attach.add_base64_encoded_json_attachment(AttachmentId::CredentialOffer, ::serde_json::Value::String(credential_offer.to_string()))?;
        Ok(self)
    }

    pub fn set_credential_preview_data(mut self, credential_preview: CredentialPreviewData) -> VcxResult<CredentialOffer> {
        self.credential_preview = credential_preview;
        Ok(self)
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> VcxResult<CredentialOffer> {
        self.credential_preview = self.credential_preview.add_value(name, value, mime_type)?;
        Ok(self)
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }
}

a2a_message!(CredentialOffer);

impl TryInto<CredentialOffer> for CredentialOfferV1 {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialOffer, Self::Error> {
        let mut credential_preview = CredentialPreviewData::new();

        for (key, value) in self.credential_attrs {
            credential_preview = credential_preview.add_value(&key, &value.as_str().unwrap_or_default(), MimeType::Plain)?;
        }

        CredentialOffer::create()
            .set_id(self.thread_id.unwrap_or_default())
            .set_credential_preview_data(credential_preview)?
            .set_offers_attach(&self.libindy_offer)
    }
}

impl TryInto<CredentialOfferV1> for CredentialOffer {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialOfferV1, Self::Error> {
        let indy_cred_offer_json = self.offers_attach.content()?;
        let indy_cred_offer: ::serde_json::Value = ::serde_json::from_str(&indy_cred_offer_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Indy Offer: {:?}", err)))?;

        let mut credential_attrs: ::serde_json::Map<String, ::serde_json::Value> = ::serde_json::Map::new();

        for attr in self.credential_preview.attributes {
            credential_attrs.insert(attr.name.clone(), ::serde_json::Value::String(attr.value.clone()));
        }

        Ok(CredentialOfferV1 {
            msg_type: PayloadKinds::CredOffer.name().to_string(),
            version: String::from("0.1"),
            to_did: String::new(),
            from_did: String::new(),
            credential_attrs,
            schema_seq_no: 0,
            claim_name: self.comment,
            claim_id: String::new(),
            msg_ref_id: None,
            cred_def_id: indy_cred_offer["cred_def_id"].as_str().map(String::from).unwrap_or_default(),
            libindy_offer: indy_cred_offer_json,
            thread_id: Some(self.id.0.clone()),
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use v3::messages::connection::response::tests::*;

    fn _attachment() -> ::serde_json::Value {
        json!({
            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
        })
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _value() -> (&'static str, &'static str) {
        ("attribute", "value")
    }

    pub fn _preview_data() -> CredentialPreviewData {
        let (name, value) = _value();
        CredentialPreviewData::new()
            .add_value(name, value, MimeType::Plain).unwrap()
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_credential_offer().id.0)
    }

    pub fn thread_id() -> String {
        thread().thid.unwrap()
    }

    pub fn _credential_offer() -> CredentialOffer {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::CredentialOffer, _attachment()).unwrap();

        CredentialOffer {
            id: MessageId::id(),
            comment: Some(_comment()),
            credential_preview: _preview_data(),
            offers_attach: attachment,
            thread: Some(_thread()),
        }
    }

    #[test]
    fn test_credential_offer_build_works() {
        let credential_offer: CredentialOffer = CredentialOffer::create()
            .set_comment(_comment())
            .set_thread_id(&_thread_id())
            .set_credential_preview_data(_preview_data()).unwrap()
            .set_offers_attach(&_attachment().to_string()).unwrap();

        assert_eq!(_credential_offer(), credential_offer);
    }
}