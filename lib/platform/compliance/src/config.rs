use std::{env, path::PathBuf};

use crate::{ComplianceError, ComplianceMode, Policy, PolicyOverrides};

/// Configuration derived from compliance-related environment variables.
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    pub mode: ComplianceMode,
    pub policy_path: Option<PathBuf>,
}

impl ComplianceConfig {
    /// Load compliance configuration from the `platform.compliance` env namespace.
    pub fn from_env() -> Result<Self, ComplianceError> {
        match dfps_configuration::load_env("platform.compliance") {
            Ok(_) => {}
            Err(dfps_configuration::EnvLoadError::FileMissing { .. }) => {}
            Err(err) => return Err(ComplianceError::Env(err)),
        }

        let mode = env::var("DFPS_COMPLIANCE_MODE")
            .ok()
            .and_then(non_empty_string)
            .map(|value| ComplianceMode::from_env_value(&value))
            .transpose()?
            .unwrap_or(ComplianceMode::Internal);

        let policy_path = env::var("DFPS_COMPLIANCE_POLICY_PATH")
            .ok()
            .and_then(non_empty_string)
            .map(|value| resolve_path(&value))
            .transpose()?;

        Ok(Self { mode, policy_path })
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

fn resolve_path(raw: &str) -> Result<PathBuf, ComplianceError> {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    if let Ok(root) = env::var("DFPS_WORKSPACE_ROOT") {
        return Ok(PathBuf::from(root).join(raw));
    }

    env::current_dir()
        .map(|cwd| cwd.join(raw))
        .map_err(ComplianceError::CurrentDir)
}

fn non_empty_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
