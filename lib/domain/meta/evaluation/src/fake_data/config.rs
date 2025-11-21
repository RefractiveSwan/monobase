use refractive_swan_configuration::{self, EnvLoadError, EnvValueError};
use thiserror::Error;

/// Configuration for fake-data generators (default counts, seeds).
#[derive(Debug, Clone, Default)]
pub struct FakeDataConfig {
    pub default_count: Option<usize>,
    pub default_seed: Option<u64>,
}

impl FakeDataConfig {
    pub fn from_env() -> Result<Self, FakeDataConfigError> {
        match refractive_swan_configuration::load_env("domain.fake_data") {
            Ok(_) => {}
            Err(EnvLoadError::FileMissing { .. }) => {}
            Err(err) => return Err(FakeDataConfigError::Env(err)),
        }
        let default_count =
            refractive_swan_configuration::u64_var("refractive_swan_FAKE_DATA_DEFAULT_COUNT")
                .map_err(FakeDataConfigError::EnvValue)?
                .map(|value| value as usize);
        let default_seed = refractive_swan_configuration::u64_var("refractive_swan_FAKE_DATA_SEED")
            .map_err(FakeDataConfigError::EnvValue)?;
        Ok(Self {
            default_count,
            default_seed,
        })
    }
}

#[derive(Debug, Error)]
pub enum FakeDataConfigError {
    #[error("refractive_swan_configuration env error: {0}")]
    Env(#[from] EnvLoadError),
    #[error("invalid fake data env value: {0}")]
    EnvValue(#[from] EnvValueError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Mutex};

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn reset_env() {
        unsafe {
            env::remove_var("refractive_swan_FAKE_DATA_DEFAULT_COUNT");
            env::remove_var("refractive_swan_FAKE_DATA_SEED");
        }
    }

    #[test]
    fn config_defaults_when_env_missing() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        let cfg = FakeDataConfig::from_env().expect("config loads");
        assert!(cfg.default_count.is_none());
        assert!(cfg.default_seed.is_none());
    }

    #[test]
    fn config_reads_env_values() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_FAKE_DATA_DEFAULT_COUNT", "5");
            env::set_var("refractive_swan_FAKE_DATA_SEED", "42");
        }
        let cfg = FakeDataConfig::from_env().expect("config loads");
        assert_eq!(cfg.default_count, Some(5));
        assert_eq!(cfg.default_seed, Some(42));
        reset_env();
    }
}
