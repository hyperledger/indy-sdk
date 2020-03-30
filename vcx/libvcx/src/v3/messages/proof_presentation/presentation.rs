use v3::messages::a2a::{MessageId, A2AMessage};
use v3::messages::attachment::{Attachments, AttachmentEncoding};
use v3::messages::ack::PleaseAck;
use messages::thread::Thread;
use messages::proofs::proof_message::ProofMessage;
use std::convert::TryInto;

use error::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Presentation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>
}

impl Presentation {
    pub fn create() -> Self {
        Presentation::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_presentations_attach(mut self, presentations: String) -> VcxResult<Presentation> {
        self.presentations_attach.add_json_attachment(::serde_json::Value::String(presentations), AttachmentEncoding::Base64)?;
        Ok(self)
    }
}

please_ack!(Presentation);
threadlike!(Presentation);
a2a_message!(Presentation);

impl TryInto<Presentation> for ProofMessage {
    type Error = VcxError;

    fn try_into(self) -> Result<Presentation, Self::Error> {
        let presentation = Presentation::create()
            .set_presentations_attach(self.libindy_proof)?
            .ask_for_ack();

        Ok(presentation)
    }
}

impl TryInto<ProofMessage> for Presentation {
    type Error = VcxError;

    fn try_into(self) -> Result<ProofMessage, Self::Error> {
        let mut proof = ProofMessage::new();
        proof.libindy_proof = self.presentations_attach.content().unwrap();
        Ok(proof)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use v3::messages::proof_presentation::presentation_request::tests::{thread, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation() -> Presentation {
        let mut attachment = Attachments::new();
        attachment.add_json_attachment(_attachment(), AttachmentEncoding::Base64).unwrap();

        Presentation {
            id: MessageId::id(),
            comment: Some(_comment()),
            presentations_attach: attachment,
            thread: thread(),
            please_ack: Some(PleaseAck {}),
        }
    }

    #[test]
    fn test_presentation_build_works() {
        let presentation: Presentation = Presentation::default()
            .set_comment(_comment())
            .ask_for_ack()
            .set_thread_id(&thread_id())
            .set_presentations_attach(_attachment().to_string()).unwrap();

        assert_eq!(_presentation(), presentation);
    }
}
