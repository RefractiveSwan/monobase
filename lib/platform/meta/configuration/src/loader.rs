use std::{
    env,
    path::{Path, PathBuf},
};

use dotenvy::Error as DotEnvError;
use thiserror::Error;

use crate::paths::{config_paths, resolve_relative};

/// Outcome of loading an environment file for a specific namespace/profile.
#[derive(Debug, Clone)]
pub struct EnvLoadOutcome {
    pub namespace: String,
    pub profile: String,
    pub files: Vec<PathBuf>,
}

/// Attempt to load the `.env.<namespace>.<profile>` file for the current crate.
///
/// * `namespace` follows the directory structure (e.g., `app.web.api`).
/// * `profile` is resolved from `DFPS_ENV` / `APP_ENV`, defaulting to `dev`.
/// * `DFPS_ENV_FILE`, if set, overrides the filename entirely (relative to workspace).
pub fn load_env(namespace: &str) -> Result<EnvLoadOutcome, EnvLoadError> {
    let profile = env_profile();
    let paths = config_paths()?;
    let explicit_file = env::var("DFPS_ENV_FILE").ok();

    let mut loaded_files = Vec::new();
    let mut attempted_paths = Vec::new();

    if let Some(filename) = explicit_file {
        let resolved = resolve_relative(&paths.workspace_root, &filename);
        attempted_paths.push(resolved.clone());
        load_file(&resolved)?;
        loaded_files.push(resolved);
    } else {
        'outer: for dir in &paths.env_dirs {
            let primary = dir.join(format!(".env.{namespace}.{profile}"));
            attempted_paths.push(primary.clone());
            if primary.exists() {
                load_file(&primary)?;
                loaded_files.push(primary);
                break 'outer;
            }

            let fallback = dir.join(format!(".env.{namespace}.local"));
            attempted_paths.push(fallback.clone());
            if fallback.exists() {
                load_file(&fallback)?;
                loaded_files.push(fallback);
                break 'outer;
            }
        }
    }

    if loaded_files.is_empty() && strict_mode() {
        return Err(EnvLoadError::FileMissing {
            namespace: namespace.to_string(),
            profile: profile.clone(),
            attempted: attempted_paths,
        });
    }

    Ok(EnvLoadOutcome {
        namespace: namespace.to_string(),
        profile,
        files: loaded_files,
    })
}

fn load_file(path: &Path) -> Result<(), EnvLoadError> {
    dotenvy::from_filename(path)
        .map(|_| ())
        .map_err(|source| EnvLoadError::DotEnv {
            path: path.to_path_buf(),
            source,
        })
}

fn env_profile() -> String {
    env::var("DFPS_ENV")
        .or_else(|_| env::var("APP_ENV"))
        .unwrap_or_else(|_| "dev".to_string())
}

fn strict_mode() -> bool {
    env_flag("DFPS_ENV_STRICT") || env_flag("CI")
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                true
            } else {
                !matches!(trimmed.to_ascii_lowercase().as_str(), "false" | "0" | "off")
            }
        })
        .unwrap_or(false)
}

#[derive(Debug, Error)]
pub enum EnvLoadError {
    #[error("failed to determine current directory: {0}")]
    CurrentDir(std::io::Error),
    #[error("workspace root (containing Cargo.lock) not found")]
    WorkspaceRootNotFound,
    #[error("failed to load dotenv file {path:?}: {source}")]
    DotEnv { path: PathBuf, source: DotEnvError },
    #[error(
        "no env file found for namespace '{namespace}' profile '{profile}'. Looked for: {attempted:?}"
    )]
    FileMissing {
        namespace: String,
        profile: String,
        attempted: Vec<PathBuf>,
    },
}
