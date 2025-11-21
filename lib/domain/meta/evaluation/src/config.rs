//! Dataset configuration helpers for `refractive_swan_eval`.
//!
//! Exposes a typed config loader backed by `refractive_swan_configuration` so app crates can
//! honor `refractive_swan_EVAL_DATA_ROOT` consistently without embedding env logic in the
//! domain layer.

use crate::{FileDatasetStore, default_data_root};
use refractive_swan_configuration::{self, EnvLoadError, EnvValueError};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration describing where evaluation datasets live on disk.
#[derive(Debug, Clone)]
pub struct EvalDatasetConfig {
    pub root: PathBuf,
}

impl EvalDatasetConfig {
    /// Load config from the `domain.eval` namespace (falls back to the default root).
    pub fn from_env() -> Result<Self, EvalConfigError> {
        match refractive_swan_configuration::load_env("domain.eval") {
            Ok(_) => {}
            Err(EnvLoadError::FileMissing { .. }) => {}
            Err(err) => return Err(EvalConfigError::Env(err)),
        }

        let root = match refractive_swan_configuration::string_var("refractive_swan_EVAL_DATA_ROOT")
            .map_err(EvalConfigError::EnvValue)?
        {
            Some(raw) if !raw.trim().is_empty() => resolve_path(raw.trim())?,
            _ => default_data_root(),
        };

        Ok(Self { root })
    }

    /// Convert into a `FileDatasetStore`.
    pub fn dataset_store(&self) -> FileDatasetStore {
        FileDatasetStore::new(self.root.clone())
    }
}

fn resolve_path(raw: &str) -> Result<PathBuf, EvalConfigError> {
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return Ok(candidate);
    }
    let workspace =
        refractive_swan_configuration::workspace_root().map_err(EvalConfigError::Env)?;
    Ok(workspace.join(candidate))
}

/// Errors surfaced when loading dataset configuration.
#[derive(Debug, Error)]
pub enum EvalConfigError {
    #[error("refractive_swan_configuration env error: {0}")]
    Env(#[from] EnvLoadError),
    #[error("invalid refractive_swan_EVAL_DATA_ROOT value: {0}")]
    EnvValue(#[from] EnvValueError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_data_root;
    use std::env;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn unset_root() {
        unsafe {
            env::remove_var("refractive_swan_EVAL_DATA_ROOT");
        }
    }

    #[test]
    fn config_defaults_to_bundled_root() {
        let _guard = ENV_GUARD.lock().unwrap();
        unset_root();
        let cfg = EvalDatasetConfig::from_env().expect("config loads");
        assert_eq!(cfg.root, default_data_root());
    }

    #[test]
    fn config_resolves_relative_paths() {
        let _guard = ENV_GUARD.lock().unwrap();
        let relative = "lib/domain/meta/evaluation/data/eval";
        unsafe {
            env::set_var("refractive_swan_EVAL_DATA_ROOT", relative);
        }
        let cfg = EvalDatasetConfig::from_env().expect("config loads");
        let workspace = refractive_swan_configuration::workspace_root().expect("workspace root");
        assert_eq!(cfg.root, workspace.join(relative));
        unset_root();
    }
}
