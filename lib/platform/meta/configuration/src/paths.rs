use std::{
    env,
    path::{Path, PathBuf},
};

use crate::loader::EnvLoadError;

/// Paths/roots resolved for configuration lookup.
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub workspace_root: PathBuf,
    pub env_dirs: Vec<PathBuf>,
}

/// Resolve workspace + env search directories.
pub fn config_paths() -> Result<ConfigPaths, EnvLoadError> {
    let workspace_root = workspace_root()?;
    let env_dirs = env_search_dirs(&workspace_root);
    Ok(ConfigPaths {
        workspace_root,
        env_dirs,
    })
}

/// Resolve the workspace root (directory containing `Cargo.lock`).
pub fn workspace_root() -> Result<PathBuf, EnvLoadError> {
    if let Ok(root) = env::var("refractive_swan_WORKSPACE_ROOT") {
        let candidate = PathBuf::from(root);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    discover_workspace_root()
}

pub(crate) fn resolve_relative(root: &Path, filename: &str) -> PathBuf {
    let candidate = Path::new(filename);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    }
}

fn env_search_dirs(workspace_root: &Path) -> Vec<PathBuf> {
    if let Ok(dir) = env::var("refractive_swan_ENV_DIR") {
        vec![resolve_relative(workspace_root, &dir)]
    } else {
        vec![
            workspace_root.join("data").join("environment"),
            workspace_root.to_path_buf(),
        ]
    }
}

fn discover_workspace_root() -> Result<PathBuf, EnvLoadError> {
    let mut dir = env::current_dir().map_err(EnvLoadError::CurrentDir)?;
    loop {
        if dir.join("Cargo.lock").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(EnvLoadError::WorkspaceRootNotFound)
}
