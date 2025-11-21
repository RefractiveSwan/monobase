use refractive_swan_compliance::{ComplianceMode, Policy};
use refractive_swan_core::mapping::{MappingSourceVersion, MappingThresholds};

use crate::data::{NCIT_DATA_VERSION, UMLS_DATA_VERSION};

#[derive(Debug, Clone)]
pub struct MappingConfig {
    pub thresholds: MappingThresholds,
    pub source_version: MappingSourceVersion,
    pub policy: Policy,
}

impl Default for MappingConfig {
    fn default() -> Self {
        Self {
            thresholds: MappingThresholds::default(),
            source_version: MappingSourceVersion::new(NCIT_DATA_VERSION, UMLS_DATA_VERSION),
            policy: Policy::default_for_mode(ComplianceMode::Internal),
        }
    }
}

impl MappingConfig {
    pub fn with_policy(policy: Policy) -> Self {
        Self {
            policy,
            ..Self::default()
        }
    }
}
