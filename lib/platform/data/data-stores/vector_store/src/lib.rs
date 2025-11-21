//! Platform vector-store adapters (Qdrant, PGVector, mocks).
//! Traits + config live in `refractive_swan_vector_port`; this crate wires env/config
//! parsing plus concrete backends.
use std::env;

use refractive_swan_configuration::EnvValueError;
pub use refractive_swan_vector_port::*;

/// Load a `VectorStoreConfig` from environment variables.
pub fn config_from_env() -> Result<VectorStoreConfig, VectorStoreConfigError> {
    const DEFAULT_POOL_MAX: u32 = 5;
    const DEFAULT_HEALTH_TIMEOUT_MS: u64 = 500;

    let _ = refractive_swan_configuration::load_env("platform.vector_store");

    let enabled = refractive_swan_configuration::bool_var("refractive_swan_VECTOR_ENABLED")
        .map_err(env_err)?
        .unwrap_or(false);
    let backend = refractive_swan_configuration::string_var("refractive_swan_VECTOR_BACKEND")
        .map_err(env_err)?
        .and_then(|value| value.parse().ok())
        .unwrap_or(VectorBackend::Mock);
    let url = refractive_swan_configuration::string_var("refractive_swan_VECTOR_URL")
        .map_err(env_err)?
        .filter(|value| !value.is_empty());
    let namespace =
        env::var("refractive_swan_VECTOR_NAMESPACE").unwrap_or_else(|_| "default".into());
    let pool_max = refractive_swan_configuration::u32_var("refractive_swan_VECTOR_POOL_MAX")
        .map_err(env_err)?
        .unwrap_or(DEFAULT_POOL_MAX);
    let health_timeout_ms =
        refractive_swan_configuration::u64_var("refractive_swan_VECTOR_HEALTH_TIMEOUT_MS")
            .map_err(env_err)?
            .unwrap_or(DEFAULT_HEALTH_TIMEOUT_MS);

    let config = VectorStoreConfig {
        backend,
        url,
        namespace,
        pool_max,
        health_timeout_ms,
        enabled,
    };
    config.validate()?;
    Ok(config)
}

fn env_err(err: EnvValueError) -> VectorStoreConfigError {
    VectorStoreConfigError::InvalidEnv(err.to_string())
}

#[cfg(feature = "backend-qdrant")]
mod qdrant;
#[cfg(feature = "backend-qdrant")]
pub use qdrant::QdrantVectorStore;

#[cfg(feature = "backend-pgvector")]
mod pgvector_backend;
#[cfg(feature = "backend-pgvector")]
pub use pgvector_backend::PgVectorStore;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, sync::Mutex};

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    fn reset_env() {
        for key in [
            "refractive_swan_VECTOR_ENABLED",
            "refractive_swan_VECTOR_BACKEND",
            "refractive_swan_VECTOR_URL",
            "refractive_swan_VECTOR_NAMESPACE",
            "refractive_swan_VECTOR_POOL_MAX",
            "refractive_swan_VECTOR_HEALTH_TIMEOUT_MS",
        ] {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    fn config_defaults_to_mock_when_disabled() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_VECTOR_NAMESPACE", "ncit_dev");
        }
        let config = config_from_env().expect("config");
        assert_eq!(config.backend, VectorBackend::Mock);
        assert_eq!(config.namespace, "ncit_dev");
        assert!(!config.enabled);
    }

    #[test]
    fn config_rejects_zero_pool_max() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_VECTOR_ENABLED", "true");
            env::set_var("refractive_swan_VECTOR_NAMESPACE", "ncit_dev");
            env::set_var("refractive_swan_VECTOR_POOL_MAX", "0");
        }
        let err = config_from_env().unwrap_err();
        assert_eq!(err, VectorStoreConfigError::InvalidPoolMax);
    }

    #[test]
    fn config_rejects_zero_health_timeout() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_VECTOR_ENABLED", "true");
            env::set_var("refractive_swan_VECTOR_NAMESPACE", "ncit_dev");
            env::set_var("refractive_swan_VECTOR_HEALTH_TIMEOUT_MS", "0");
        }
        let err = config_from_env().unwrap_err();
        assert_eq!(err, VectorStoreConfigError::InvalidTimeout);
    }

    #[test]
    fn config_rejects_empty_namespace_when_enabled() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_VECTOR_ENABLED", "true");
            env::set_var("refractive_swan_VECTOR_NAMESPACE", "   ");
        }
        let err = config_from_env().unwrap_err();
        assert_eq!(err, VectorStoreConfigError::MissingNamespace);
    }

    #[test]
    fn config_requires_url_for_non_mock_backend() {
        let _guard = ENV_GUARD.lock().unwrap();
        reset_env();
        unsafe {
            env::set_var("refractive_swan_VECTOR_ENABLED", "true");
            env::set_var("refractive_swan_VECTOR_NAMESPACE", "ncit_dev");
            env::set_var("refractive_swan_VECTOR_BACKEND", "qdrant");
        }
        let err = config_from_env().unwrap_err();
        assert_eq!(err, VectorStoreConfigError::MissingUrl);
    }
}
