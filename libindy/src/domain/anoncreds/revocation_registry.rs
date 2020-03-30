use std::collections::HashMap;

use indy_api_types::validation::Validatable;

use super::revocation_registry_definition::RevocationRegistryId;
pub use indy_vdr::ledger::requests::rev_reg::RevocationRegistryV1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "ver")]
pub enum RevocationRegistry {
    #[serde(rename = "1.0")]
    RevocationRegistryV1(RevocationRegistryV1),
}

impl From<RevocationRegistry> for RevocationRegistryV1 {
    fn from(rev_reg: RevocationRegistry) -> Self {
        match rev_reg {
            RevocationRegistry::RevocationRegistryV1(rev_reg) => rev_reg,
        }
    }
}

pub type RevocationRegistries = HashMap<RevocationRegistryId, HashMap<u64, RevocationRegistry>>;


pub fn rev_regs_map_to_rev_regs_local_map(rev_regs: RevocationRegistries) -> HashMap<RevocationRegistryId, HashMap<u64, RevocationRegistryV1>> {
    rev_regs
        .into_iter()
        .map(|(rev_reg_id, rev_reg_to_timespams)| {
            let val = rev_reg_to_timespams
                .into_iter()
                .map(|(timestamp, rev_reg)| {
                    match rev_reg {
                        RevocationRegistry::RevocationRegistryV1(rev_reg) => (timestamp, rev_reg)
                    }
                })
                .collect();
            (rev_reg_id, val)
        })
        .collect()
}

impl Validatable for RevocationRegistry {}
