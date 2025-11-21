use once_cell::sync::Lazy;
use std::{env, path::PathBuf, sync::Mutex};

#[derive(Default)]
struct ObservabilityEnvState {
    loaded: bool,
}

static OBS_ENV_STATE: Lazy<Mutex<ObservabilityEnvState>> =
    Lazy::new(|| Mutex::new(ObservabilityEnvState::default()));

pub(crate) fn ensure_env() -> Result<(), refractive_swan_configuration::EnvLoadError> {
    let mut state = OBS_ENV_STATE
        .lock()
        .expect("observability env state mutex poisoned");
    if state.loaded {
        return Ok(());
    }
    let explicit_env_file = env::var("refractive_swan_ENV_FILE").ok();
    match refractive_swan_configuration::load_env("platform.observability") {
        Ok(outcome) => {
            if let Some(file) = explicit_env_file {
                let configured_path = PathBuf::from(&file);
                let loaded_match = outcome
                    .files
                    .iter()
                    .any(|loaded| loaded.ends_with(&configured_path));
                if !loaded_match {
                    return Err(refractive_swan_configuration::EnvLoadError::FileMissing {
                        namespace: outcome.namespace,
                        profile: outcome.profile,
                        attempted: vec![configured_path],
                    });
                }
            }
            state.loaded = true;
            Ok(())
        }
        Err(refractive_swan_configuration::EnvLoadError::FileMissing { .. }) => Ok(()),
        Err(err) => Err(err),
    }
}

/// Allow callers to eagerly load environment files.
pub fn init_environment() -> Result<(), refractive_swan_configuration::EnvLoadError> {
    ensure_env()
}

#[cfg(test)]
pub(crate) fn reset_env_state_for_tests() {
    let mut state = OBS_ENV_STATE
        .lock()
        .expect("observability env state mutex poisoned");
    state.loaded = false;
}

#[cfg(test)]
mod tests {
    use super::{ensure_env, init_environment, reset_env_state_for_tests};
    use std::{
        env,
        sync::{Mutex, OnceLock},
    };

    static ENV_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_guard() -> &'static Mutex<()> {
        ENV_GUARD.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn init_environment_succeeds_without_env_file() {
        let _lock = env_guard().lock().unwrap();
        reset_env_state_for_tests();
        unsafe {
            env::remove_var("refractive_swan_ENV_FILE");
        }
        init_environment().expect("observability env loads without panic");
        init_environment().expect("second init succeeds");
    }

    #[test]
    fn ensure_env_errors_for_invalid_env_file() {
        let _lock = env_guard().lock().unwrap();
        reset_env_state_for_tests();
        unsafe {
            env::set_var("refractive_swan_ENV_FILE", "missing.observability.env");
        }
        let err = ensure_env().expect_err("invalid env file should error");
        matches!(
            err,
            refractive_swan_configuration::EnvLoadError::DotEnv { .. }
        );
        unsafe {
            env::remove_var("refractive_swan_ENV_FILE");
        }
    }
}
