use once_cell::sync::Lazy;
use std::{env, path::PathBuf, sync::Mutex};

#[derive(Default)]
struct TestSuiteEnvState {
    loaded: bool,
}

static TEST_SUITE_ENV: Lazy<Mutex<TestSuiteEnvState>> =
    Lazy::new(|| Mutex::new(TestSuiteEnvState::default()));

fn ensure_namespace_loaded() -> Result<(), dfps_configuration::EnvLoadError> {
    let mut state = TEST_SUITE_ENV
        .lock()
        .expect("dfps_test_suite env state lock poisoned");
    if state.loaded {
        return Ok(());
    }
    match dfps_configuration::load_env("platform.test_suite") {
        Ok(_) => {}
        Err(dfps_configuration::EnvLoadError::FileMissing { .. }) => {}
        Err(err) => return Err(err),
    }
    state.loaded = true;
    Ok(())
}

/// Load the `platform.test_suite` env namespace (idempotent).
pub fn init_environment() -> Result<(), dfps_configuration::EnvLoadError> {
    ensure_namespace_loaded()
}

/// Ensure `DFPS_EVAL_DATA_ROOT` is set, defaulting to the repo fixtures directory.
pub fn ensure_eval_data_root() -> Result<PathBuf, dfps_configuration::EnvLoadError> {
    ensure_namespace_loaded()?;
    if let Ok(raw) = env::var("DFPS_EVAL_DATA_ROOT")
        && !raw.trim().is_empty()
    {
        return Ok(PathBuf::from(raw));
    }

    let default_root = dfps_configuration::workspace_root()?.join("lib/domain/eval/data/eval");
    unsafe {
        env::set_var("DFPS_EVAL_DATA_ROOT", &default_root);
    }
    Ok(default_root)
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
        let previous = env::var(&key).ok();
        unsafe {
            env::set_var(&key, value.into());
        }
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        unsafe {
            if let Some(prev) = &self.previous {
                env::set_var(&self.key, prev);
            } else {
                env::remove_var(&self.key);
            }
        }
    }
}

pub fn scoped_env_var(key: impl Into<String>, value: impl Into<String>) -> ScopedEnvVar {
    ScopedEnvVar::new(key, value)
}
