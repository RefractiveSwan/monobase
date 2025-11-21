use std::path::{Path, PathBuf};

use crate::{ComplianceError, ComplianceMode, Policy, PolicyOverrides};

/// Configuration derived from compliance-related environment variables.
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    pub mode: ComplianceMode,
    pub policy_path: Option<PathBuf>,
    pub workspace_root: PathBuf,
}

impl ComplianceConfig {
    /// Load compliance configuration from the `platform.compliance` env namespace.
    pub fn from_env() -> Result<Self, ComplianceError> {
        match dfps_configuration::load_env("platform.compliance") {
            Ok(_) => {}
            Err(dfps_configuration::EnvLoadError::FileMissing { .. }) => {}
            Err(err) => return Err(ComplianceError::Env(err)),
        }

        let workspace_root = dfps_configuration::workspace_root().map_err(ComplianceError::Env)?;

        let mode = match dfps_configuration::string_var("DFPS_COMPLIANCE_MODE")
            .map_err(ComplianceError::EnvValue)?
        {
            Some(value) => ComplianceMode::from_env_value(&value)?,
            None => ComplianceMode::Internal,
        };

        let policy_path = dfps_configuration::string_var("DFPS_COMPLIANCE_POLICY_PATH")
            .map_err(ComplianceError::EnvValue)?
            .map(|value| resolve_path(&workspace_root, &value));

        Ok(Self {
            mode,
            policy_path,
            workspace_root,
        })
    }

    /// Build a policy from the resolved configuration.
    pub fn load_policy(&self) -> Result<Policy, ComplianceError> {
        let mut policy = Policy::default_for_mode(self.mode);
        if let Some(path) = &self.policy_path {
            let overrides = PolicyOverrides::from_path(path)?;
            let base_mode = overrides.mode.unwrap_or(policy.mode);
            policy = Policy::default_for_mode(base_mode).apply_overrides(overrides);
        }
        Ok(policy)
    }
}

fn resolve_path(root: &Path, raw: &str) -> PathBuf {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return candidate;
    }

    root.join(raw)
}
