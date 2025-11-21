use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use refractive_swan_terminology::codesystem::LicenseTier;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Runtime compliance modes that tune which license tiers are permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceMode {
    Internal,
    Partner,
    OpenSource,
}

impl ComplianceMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            ComplianceMode::Internal => "internal",
            ComplianceMode::Partner => "partner",
            ComplianceMode::OpenSource => "open_source",
        }
    }

    pub fn from_env_value(value: &str) -> Result<Self, ComplianceError> {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(ComplianceMode::Internal);
        }
        Self::from_str(&normalized)
    }
}

impl FromStr for ComplianceMode {
    type Err = ComplianceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "internal" => Ok(ComplianceMode::Internal),
            "partner" => Ok(ComplianceMode::Partner),
            "open_source" | "opensource" => Ok(ComplianceMode::OpenSource),
            other => Err(ComplianceError::InvalidMode {
                value: other.to_string(),
            }),
        }
    }
}

/// Actions that can be gated by compliance policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAction {
    Ingest,
    Map,
    Export,
}

impl ComplianceAction {
    pub const fn all() -> [ComplianceAction; 3] {
        [
            ComplianceAction::Ingest,
            ComplianceAction::Map,
            ComplianceAction::Export,
        ]
    }
}

/// Policy describing which license tiers are permitted for each action under a given mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub mode: ComplianceMode,
    pub allowed_actions: BTreeSet<ComplianceAction>,
    pub allowed_tiers: BTreeMap<ComplianceAction, BTreeSet<LicenseTier>>,
}

impl Policy {
    /// Construct a policy with defaults for the specified mode.
    pub fn default_for_mode(mode: ComplianceMode) -> Self {
        let allowed_actions: BTreeSet<_> = ComplianceAction::all().into_iter().collect();
        let default_tiers = default_allowed_tiers(mode);
        let mut allowed_tiers = BTreeMap::new();
        for action in &allowed_actions {
            allowed_tiers.insert(*action, default_tiers.clone());
        }
        Self {
            mode,
            allowed_actions,
            allowed_tiers,
        }
    }

    /// Merge overrides on top of the current policy.
    pub fn apply_overrides(mut self, overrides: PolicyOverrides) -> Self {
        if let Some(mode) = overrides.mode {
            self = Policy::default_for_mode(mode);
        }

        if let Some(actions) = overrides.allowed_actions {
            self.allowed_actions = actions.into_iter().collect();
        }

        if let Some(tiers) = overrides.allowed_tiers {
            for (action, allowed) in tiers {
                self.allowed_tiers
                    .insert(action, allowed.into_iter().collect());
            }
        }

        self.allowed_tiers
            .retain(|action, _| self.allowed_actions.contains(action));

        let default_tiers = default_allowed_tiers(self.mode);
        for action in &self.allowed_actions {
            self.allowed_tiers
                .entry(*action)
                .or_insert_with(|| default_tiers.clone());
        }

        self
    }

    /// Whether the given action is permitted by the policy.
    pub fn is_action_allowed(&self, action: ComplianceAction) -> bool {
        self.allowed_actions.contains(&action)
    }

    /// Allowed license tiers for a specific action, if configured.
    pub fn allowed_tiers_for(&self, action: ComplianceAction) -> Option<&BTreeSet<LicenseTier>> {
        self.allowed_tiers.get(&action)
    }

    /// Returns true if the license tier is permitted for the given action.
    pub fn is_allowed(&self, action: ComplianceAction, tier: LicenseTier) -> bool {
        self.is_action_allowed(action)
            && self
                .allowed_tiers_for(action)
                .map(|tiers| tiers.contains(&tier))
                .unwrap_or(false)
    }
}

/// Optional overrides when loading a policy from configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOverrides {
    pub mode: Option<ComplianceMode>,
    pub allowed_actions: Option<Vec<ComplianceAction>>,
    pub allowed_tiers: Option<BTreeMap<ComplianceAction, Vec<LicenseTier>>>,
}

impl PolicyOverrides {
    pub fn from_path(path: &Path) -> Result<Self, ComplianceError> {
        let contents =
            fs::read_to_string(path).map_err(|source| ComplianceError::PolicyPathIo {
                path: path.to_path_buf(),
                source,
            })?;
        parse_policy(path, &contents)
    }
}

/// Assert that a set of license tiers is permitted for export under the given policy.
pub fn assert_export_allowed(
    license_tiers: &[LicenseTier],
    policy: &Policy,
) -> Result<(), ComplianceError> {
    for tier in license_tiers {
        if !policy.is_allowed(ComplianceAction::Export, *tier) {
            return Err(ComplianceError::ExportNotAllowed {
                tier: tier.as_str().to_string(),
                mode: policy.mode.as_str().to_string(),
            });
        }
    }
    Ok(())
}

fn default_allowed_tiers(mode: ComplianceMode) -> BTreeSet<LicenseTier> {
    match mode {
        ComplianceMode::Internal => vec![
            LicenseTier::Licensed,
            LicenseTier::Open,
            LicenseTier::InternalOnly,
        ]
        .into_iter()
        .collect(),
        ComplianceMode::Partner => vec![LicenseTier::Licensed, LicenseTier::Open]
            .into_iter()
            .collect(),
        ComplianceMode::OpenSource => vec![LicenseTier::Open].into_iter().collect(),
    }
}

fn parse_policy(path: &Path, contents: &str) -> Result<PolicyOverrides, ComplianceError> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let parse_error = |err: String| ComplianceError::PolicyPathParse {
        path: path.to_path_buf(),
        message: err,
    };

    match ext.as_str() {
        "json" => serde_json::from_str(contents).map_err(|err| parse_error(err.to_string())),
        "yaml" | "yml" => {
            serde_yaml::from_str(contents).map_err(|err| parse_error(err.to_string()))
        }
        _ => {
            if let Ok(value) = serde_json::from_str(contents) {
                Ok(value)
            } else {
                serde_yaml::from_str(contents).map_err(|yaml_err| parse_error(yaml_err.to_string()))
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("invalid compliance mode '{value}', expected internal | partner | open_source")]
    InvalidMode { value: String },
    #[error("failed to load compliance env: {0}")]
    Env(refractive_swan_configuration::EnvLoadError),
    #[error("invalid compliance env value: {0}")]
    EnvValue(#[from] refractive_swan_configuration::EnvValueError),
    #[error("failed to read policy file {path:?}: {source}")]
    PolicyPathIo {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse policy file {path:?}: {message}")]
    PolicyPathParse { path: PathBuf, message: String },
    #[error("export blocked for tier '{tier}' under mode '{mode}'")]
    ExportNotAllowed { tier: String, mode: String },
}
