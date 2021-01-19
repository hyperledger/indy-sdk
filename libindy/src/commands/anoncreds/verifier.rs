use std::collections::HashMap;
use std::rc::Rc;

use crate::domain::anoncreds::credential_definition::{cred_defs_map_to_cred_defs_v1_map, CredentialDefinitionV1, CredentialDefinitionId, CredentialDefinitions};
use crate::domain::anoncreds::proof::Proof;
use crate::domain::anoncreds::proof_request::{ProofRequest, ProofRequestPayload};
use crate::domain::anoncreds::revocation_registry::{rev_regs_map_to_rev_regs_local_map, RevocationRegistryV1, RevocationRegistries};
use crate::domain::anoncreds::revocation_registry_definition::{rev_reg_defs_map_to_rev_reg_defs_v1_map, RevocationRegistryDefinitionV1, RevocationRegistryId, RevocationRegistryDefinitions};
use crate::domain::anoncreds::schema::{schemas_map_to_schemas_v1_map, SchemaV1, SchemaId, Schemas};
use indy_api_types::errors::prelude::*;
use crate::services::anoncreds::AnoncredsService;
use crate::services::metrics::MetricsService;

pub enum VerifierCommand {
    VerifyProof(
        ProofRequest, // proof request
        Proof, // proof
        Schemas, // credential schemas
        CredentialDefinitions, // credential defs
        RevocationRegistryDefinitions, // rev reg defs
        RevocationRegistries, // rev reg entries
        Box<dyn Fn(IndyResult<bool>, Rc<MetricsService>) + Send>),
    GenerateNonce(
        Box<dyn Fn(IndyResult<String>, Rc<MetricsService>) + Send>)
}

pub struct VerifierCommandExecutor {
    anoncreds_service: Rc<AnoncredsService>,
    metrics_service: Rc<MetricsService>,
}

impl VerifierCommandExecutor {
    pub fn new(anoncreds_service: Rc<AnoncredsService>, metrics_service: Rc<MetricsService>) -> VerifierCommandExecutor {
        VerifierCommandExecutor {
            anoncreds_service,
            metrics_service,
        }
    }

    pub fn execute(&self, command: VerifierCommand) {
        match command {
            VerifierCommand::VerifyProof(proof_request, proof, schemas, credential_defs, rev_reg_defs, rev_regs, cb) => {
                debug!(target: "verifier_command_executor", "VerifyProof command received");
                cb(self.verify_proof(&proof_request.value(), proof,
                                     &schemas_map_to_schemas_v1_map(schemas),
                                     &cred_defs_map_to_cred_defs_v1_map(credential_defs),
                                     &rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs),
                                     &rev_regs_map_to_rev_regs_local_map(rev_regs)),
                    self.metrics_service.clone());
            }
            VerifierCommand::GenerateNonce(cb) => {
                debug!(target: "verifier_command_executor", "GenerateNonce command received");
                cb(self.generate_nonce(), self.metrics_service.clone());
            }
        };
    }

    fn verify_proof(&self,
                    proof_req: &ProofRequestPayload,
                    proof: Proof,
                    schemas: &HashMap<SchemaId, SchemaV1>,
                    cred_defs: &HashMap<CredentialDefinitionId, CredentialDefinitionV1>,
                    rev_reg_defs: &HashMap<RevocationRegistryId, RevocationRegistryDefinitionV1>,
                    rev_regs: &HashMap<RevocationRegistryId, HashMap<u64, RevocationRegistryV1>>) -> IndyResult<bool> {
        debug!("verify_proof >>> proof_req: {:?}, proof: {:?}, schemas: {:?}, cred_defs: {:?},  \
               rev_reg_defs: {:?}, rev_regs: {:?}",
               proof_req, proof, schemas, cred_defs, rev_reg_defs, rev_regs);

        let result = self.anoncreds_service.verifier.verify(&proof,
                                                            &proof_req,
                                                            schemas,
                                                            cred_defs,
                                                            rev_reg_defs,
                                                            rev_regs)?;

        debug!("verify_proof <<< result: {:?}", result);

        Ok(result)
    }

    fn generate_nonce(&self) -> IndyResult<String> {
        debug!("generate_nonce >>> ");

        let nonce = self.anoncreds_service.verifier.generate_nonce()?;

        let result = nonce.to_dec()
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Nonce")?;

        debug!("generate_nonce <<< result: {:?}", result);

        Ok(result)
    }
}
