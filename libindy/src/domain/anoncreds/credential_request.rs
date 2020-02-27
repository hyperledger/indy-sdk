use ursa::cl::{
    BlindedCredentialSecrets,
    BlindedCredentialSecretsCorrectnessProof,
    CredentialSecretsBlindingFactors,
    Nonce
};
use indy_api_types::domain::crypto::did::DidValue;

use super::credential_definition::CredentialDefinitionId;

use indy_api_types::validation::Validatable;

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialRequest {
    pub prover_did: DidValue,
    pub cred_def_id: CredentialDefinitionId,
    pub blinded_ms: BlindedCredentialSecrets,
    pub blinded_ms_correctness_proof: BlindedCredentialSecretsCorrectnessProof,
    pub nonce: Nonce,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialRequestMetadata {
    pub master_secret_blinding_data: CredentialSecretsBlindingFactors,
    pub nonce: Nonce,
    pub master_secret_name: String
}

impl CredentialRequest {
    pub fn to_unqualified(self) -> CredentialRequest {
        CredentialRequest {
            prover_did: self.prover_did.to_unqualified(),
            cred_def_id: self.cred_def_id.to_unqualified(),
            blinded_ms: self.blinded_ms,
            blinded_ms_correctness_proof: self.blinded_ms_correctness_proof,
            nonce: self.nonce
        }
    }
}

impl Validatable for CredentialRequest {
    fn validate(&self) -> Result<(), String> {
        self.cred_def_id.validate()?;
        self.prover_did.validate()?;
        Ok(())
    }
}

impl Validatable for CredentialRequestMetadata {}