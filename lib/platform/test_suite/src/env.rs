use once_cell::sync::Lazy;
use std::{env as std_env, path::PathBuf, sync::Mutex};

#[derive(Default)]
struct TestSuiteEnvState {
    loaded: bool,
}

static TEST_SUITE_ENV: Lazy<Mutex<TestSuiteEnvState>> =
    Lazy::new(|| Mutex::new(TestSuiteEnvState::default()));

fn ensure_namespace_loaded() -> Result<(), refractive_swan_configuration::EnvLoadError> {
    let mut state = TEST_SUITE_ENV
        .lock()
        .expect("refractive_swan_test_suite env state lock poisoned");
    if state.loaded {
        return Ok(());
    }
    match refractive_swan_configuration::load_env("platform.test_suite") {
        Ok(_) => {}
        Err(refractive_swan_configuration::EnvLoadError::FileMissing { .. }) => {}
        Err(err) => return Err(err),
    }
    state.loaded = true;
    Ok(())
}

/// Load the `platform.test_suite` env namespace (idempotent).
pub fn init_environment() -> Result<(), refractive_swan_configuration::EnvLoadError> {
    ensure_namespace_loaded()
}

/// Ensure `refractive_swan_EVAL_DATA_ROOT` is set, defaulting to the repo fixtures directory.
pub fn ensure_eval_data_root() -> Result<PathBuf, refractive_swan_configuration::EnvLoadError> {
    ensure_namespace_loaded()?;
    if let Ok(raw) = std_env::var("refractive_swan_EVAL_DATA_ROOT")
        && !raw.trim().is_empty()
    {
        if let Some(root) = resolve_eval_data_override(PathBuf::from(raw.trim())) {
            return Ok(root);
        }
    }

    let packaged = refractive_swan_eval::default_data_root();
    if packaged.is_dir() {
        return Ok(packaged);
    }

    Ok(refractive_swan_configuration::workspace_root()?
        .join("lib/domain/meta/evaluation/data/eval"))
}

fn resolve_eval_data_override(candidate: PathBuf) -> Option<PathBuf> {
    let nested = candidate.join("eval");
    if nested.is_dir() {
        return Some(nested);
    }

    if candidate.is_dir() {
        return Some(candidate);
    }

    None
}

/// RAII guard for temporarily overriding environment variables in tests.
#[must_use]
pub struct ScopedEnvVar {
    key: String,
    previous: Option<String>,
}

impl ScopedEnvVar {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let previous = std_env::var(&key).ok();
        unsafe {
            std_env::set_var(&key, value.into());
        }
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        unsafe {
            if let Some(prev) = &self.previous {
                std_env::set_var(&self.key, prev);
            } else {
                std_env::remove_var(&self.key);
            }
        }
    }
}

pub fn scoped_env_var(key: impl Into<String>, value: impl Into<String>) -> ScopedEnvVar {
    ScopedEnvVar::new(key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::{fs, sync::Mutex};
    use tempfile::tempdir;

    static ENV_TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn uses_existing_override_directory() {
        let _guard = ENV_TEST_LOCK.lock().expect("env test lock");
        let temp = tempdir().expect("tempdir");
        let override_dir = temp.path().join("datasets");
        fs::create_dir(&override_dir).expect("create override dir");
        let _env = scoped_env_var(
            "refractive_swan_EVAL_DATA_ROOT",
            override_dir.to_string_lossy().to_string(),
        );
        let resolved = ensure_eval_data_root().expect("resolve override dir");
        assert_eq!(resolved, override_dir);
    }

    #[test]
    fn accepts_data_parent_directory() {
        let _guard = ENV_TEST_LOCK.lock().expect("env test lock");
        let temp = tempdir().expect("tempdir");
        let data_dir = temp.path().join("data");
        let eval_dir = data_dir.join("eval");
        fs::create_dir_all(&eval_dir).expect("create nested eval dir");
        let _env = scoped_env_var(
            "refractive_swan_EVAL_DATA_ROOT",
            data_dir.to_string_lossy().to_string(),
        );
        let resolved = ensure_eval_data_root().expect("resolve nested dir");
        assert_eq!(resolved, eval_dir);
    }

    #[test]
    fn falls_back_when_override_missing() {
        let _guard = ENV_TEST_LOCK.lock().expect("env test lock");
        let missing = "/tmp/refractive_swan_missing_eval_root";
        // ensure the directory truly doesn't exist
        let _ = fs::remove_dir_all(missing);
        let _env = scoped_env_var("refractive_swan_EVAL_DATA_ROOT", missing);
        let resolved = ensure_eval_data_root().expect("resolve fallback dir");
        assert_eq!(resolved, refractive_swan_eval::default_data_root());
    }
}
