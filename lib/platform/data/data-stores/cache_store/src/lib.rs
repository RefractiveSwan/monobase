//! CacheStore abstraction with optional in-memory backend.
//!
//! Keyspace conventions:
//! - Rate limits: `rate_limit:<node>:<window>` counters (integer)
//! - DP budgets: `dp_budget:<node>:<yyyy-mm-dd>` counters scaled by 1_000
//!
//! TTL rules:
//! - Rate limit keys: caller-provided window TTL
//! - DP budgets: 24h TTL aligned to the date component

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use redis::AsyncCommands;
use refractive_swan_configuration::{string_var, u64_var};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

/// Supported cache backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheBackend {
    Redis,
    InMemory,
    Disabled,
}

impl CacheBackend {
    pub fn from_env_value(value: &str) -> Result<Self, CacheConfigError> {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(CacheBackend::Disabled);
        }
        match normalized.as_str() {
            "redis" => Ok(CacheBackend::Redis),
            "inmemory" | "in_memory" => Ok(CacheBackend::InMemory),
            "disabled" => Ok(CacheBackend::Disabled),
            other => Err(CacheConfigError::InvalidBackend(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CacheBackend::Redis => "redis",
            CacheBackend::InMemory => "inmemory",
            CacheBackend::Disabled => "disabled",
        }
    }
}

/// Cache configuration loaded from environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheConfig {
    pub backend: CacheBackend,
    pub url: Option<String>,
    pub pool_max: u32,
    pub default_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::Disabled,
            url: None,
            pool_max: 5,
            default_ttl_secs: 60,
        }
    }
}

impl CacheConfig {
    /// Load cache configuration from environment variables.
    ///
    /// Recognized variables:
    /// - `refractive_swan_CACHE_BACKEND` (redis|inmemory|disabled, default disabled)
    /// - `refractive_swan_CACHE_URL` (required for Redis)
    /// - `refractive_swan_CACHE_POOL_MAX` (default 5)
    /// - `refractive_swan_CACHE_DEFAULT_TTL_SECS` (default 60)
    pub fn from_env() -> Result<Self, CacheConfigError> {
        let mut cfg = CacheConfig::default();
        if let Some(raw_backend) =
            string_var("refractive_swan_CACHE_BACKEND").map_err(CacheConfigError::Env)?
        {
            cfg.backend = CacheBackend::from_env_value(&raw_backend)?;
        }
        cfg.url = string_var("refractive_swan_CACHE_URL").map_err(CacheConfigError::Env)?;
        cfg.pool_max = u64_var("refractive_swan_CACHE_POOL_MAX")
            .map_err(CacheConfigError::Env)?
            .and_then(|v| v.try_into().ok())
            .unwrap_or(cfg.pool_max);
        cfg.default_ttl_secs = u64_var("refractive_swan_CACHE_DEFAULT_TTL_SECS")
            .map_err(CacheConfigError::Env)?
            .unwrap_or(cfg.default_ttl_secs);
        if cfg.backend == CacheBackend::Redis && cfg.url.is_none() {
            return Err(CacheConfigError::MissingUrl);
        }
        Ok(cfg)
    }
}

/// Errors for cache configuration and operations.
#[derive(Debug, Error)]
pub enum CacheConfigError {
    #[error("invalid cache backend: {0}")]
    InvalidBackend(String),
    #[error("cache backend requires url")]
    MissingUrl,
    #[error("env read error: {0}")]
    Env(#[from] refractive_swan_configuration::EnvValueError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("cache disabled")]
    Disabled,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
}

/// Trait for cache operations.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Backend label for observability.
    fn backend(&self) -> CacheBackend;

    /// Get a raw value by key.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Set a value with optional TTL.
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError>;

    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Increment a counter (atomic).
    ///
    /// Implementations should set TTL when the key is first created and `ttl` is provided.
    async fn incr(&self, key: &str, delta: i64, ttl: Option<Duration>) -> Result<i64, CacheError>;

    /// Health check for readiness reporting.
    async fn health_check(&self) -> Result<(), CacheError>;
}

/// Build a cache store from configuration.
pub fn cache_from_config(
    config: &CacheConfig,
) -> Result<Option<Arc<dyn CacheStore + Send + Sync>>, CacheError> {
    match config.backend {
        CacheBackend::Disabled => Ok(None),
        CacheBackend::InMemory => Ok(Some(Arc::new(InMemoryCache::new(config.default_ttl_secs)))),
        CacheBackend::Redis => Ok(Some(Arc::new(RedisCache::new(config)?))),
    }
}

#[derive(Clone, Debug)]
struct ValueEntry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

impl ValueEntry {
    fn expired(&self) -> bool {
        self.expires_at
            .map(|deadline| Instant::now() >= deadline)
            .unwrap_or(false)
    }
}

/// Simple in-memory cache suitable for tests and dev nodes.
pub struct InMemoryCache {
    default_ttl: Option<Duration>,
    inner: Arc<RwLock<HashMap<String, ValueEntry>>>,
}

/// Redis-backed cache (async connection per operation).
pub struct RedisCache {
    client: redis::Client,
    default_ttl: Option<Duration>,
}

impl InMemoryCache {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            default_ttl: if default_ttl_secs == 0 {
                None
            } else {
                Some(Duration::from_secs(default_ttl_secs))
            },
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn ttl(&self, ttl: Option<Duration>) -> Option<Duration> {
        ttl.or(self.default_ttl)
    }
}

#[async_trait]
impl CacheStore for InMemoryCache {
    fn backend(&self) -> CacheBackend {
        CacheBackend::InMemory
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get(key) {
            if entry.expired() {
                guard.remove(key);
                return Ok(None);
            }
            return Ok(Some(entry.value.clone()));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError> {
        let expires_at = self.ttl(ttl).map(|t| Instant::now() + t);
        let mut guard = self.inner.write().await;
        guard.insert(
            key.to_string(),
            ValueEntry {
                value: value.to_vec(),
                expires_at,
            },
        );
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut guard = self.inner.write().await;
        guard.remove(key);
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64, ttl: Option<Duration>) -> Result<i64, CacheError> {
        let expires_at = self.ttl(ttl).map(|t| Instant::now() + t);
        let mut guard = self.inner.write().await;
        let current = guard
            .get(key)
            .filter(|entry| !entry.expired())
            .and_then(|entry| String::from_utf8(entry.value.clone()).ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let next = current.saturating_add(delta);
        guard.insert(
            key.to_string(),
            ValueEntry {
                value: next.to_string().into_bytes(),
                expires_at,
            },
        );
        Ok(next)
    }

    async fn health_check(&self) -> Result<(), CacheError> {
        Ok(())
    }
}

impl RedisCache {
    pub fn new(config: &CacheConfig) -> Result<Self, CacheError> {
        let url = config
            .url
            .as_ref()
            .ok_or_else(|| CacheError::ConnectionFailed("redis url missing".into()))?;
        let client = redis::Client::open(url.as_str())
            .map_err(|err| CacheError::ConnectionFailed(err.to_string()))?;
        let default_ttl = if config.default_ttl_secs == 0 {
            None
        } else {
            Some(Duration::from_secs(config.default_ttl_secs))
        };
        Ok(Self {
            client,
            default_ttl,
        })
    }

    async fn conn(&self) -> Result<redis::aio::MultiplexedConnection, CacheError> {
        self.client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|err| CacheError::ConnectionFailed(err.to_string()))
    }

    fn ttl_secs(&self, ttl: Option<Duration>) -> Option<u64> {
        ttl.or(self.default_ttl).map(|d| d.as_secs())
    }
}

#[async_trait]
impl CacheStore for RedisCache {
    fn backend(&self) -> CacheBackend {
        CacheBackend::Redis
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let mut conn = self.conn().await?;
        let value: Option<Vec<u8>> = conn
            .get(key)
            .await
            .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        Ok(value)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut conn = self.conn().await?;
        if let Some(ttl_secs) = self.ttl_secs(ttl) {
            let _: () = redis::cmd("SET")
                .arg(key)
                .arg(value)
                .arg("EX")
                .arg(ttl_secs as usize)
                .query_async(&mut conn)
                .await
                .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        } else {
            let _: () = conn
                .set(key, value)
                .await
                .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.conn().await?;
        let _: () = conn
            .del(key)
            .await
            .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64, ttl: Option<Duration>) -> Result<i64, CacheError> {
        let mut conn = self.conn().await?;
        let next: i64 = conn
            .incr(key, delta)
            .await
            .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        if next == delta
            && let Some(ttl_secs) = self.ttl_secs(ttl)
        {
            let _: () = redis::cmd("EXPIRE")
                .arg(key)
                .arg(ttl_secs as usize)
                .query_async(&mut conn)
                .await
                .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        }
        Ok(next)
    }

    async fn health_check(&self) -> Result<(), CacheError> {
        let mut conn = self.conn().await?;
        let _: () = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|err| CacheError::OperationFailed(err.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration as StdDuration, SystemTime};
    use tokio::time::sleep;

    #[test]
    fn redis_requires_url() {
        let cfg = CacheConfig {
            backend: CacheBackend::Redis,
            url: None,
            pool_max: 1,
            default_ttl_secs: 1,
        };
        let result = cache_from_config(&cfg);
        assert!(matches!(result, Err(CacheError::ConnectionFailed(_))));
    }

    #[tokio::test]
    async fn incr_sets_and_reads() {
        let cache = InMemoryCache::new(0);
        let value = cache.incr("counter", 1, None).await.unwrap();
        assert_eq!(value, 1);
        let again = cache.incr("counter", 2, None).await.unwrap();
        assert_eq!(again, 3);
        let bytes = cache.get("counter").await.unwrap().unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "3");
    }

    #[tokio::test]
    async fn ttl_expires_entries() {
        let cache = InMemoryCache::new(0);
        cache
            .set("temp", b"hello", Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(cache.get("temp").await.unwrap().is_some());
        sleep(Duration::from_millis(20)).await;
        assert!(cache.get("temp").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn incr_sets_ttl_when_missing() {
        let cache = InMemoryCache::new(0);
        let _ = cache
            .incr("ttl-counter", 1, Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(cache.get("ttl-counter").await.unwrap().is_some());
        sleep(Duration::from_millis(20)).await;
        assert!(cache.get("ttl-counter").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn redis_round_trip_when_url_present() {
        let url = match std::env::var("REFRACTIVE_SWAN_REDIS_URL") {
            Ok(url) => url,
            Err(_) => return, // skip when Redis is not provisioned in dev/CI
        };
        let cfg = CacheConfig {
            backend: CacheBackend::Redis,
            url: Some(url),
            pool_max: 2,
            default_ttl_secs: 2,
        };
        let store = cache_from_config(&cfg)
            .expect("redis config")
            .expect("redis store");
        let key = format!(
            "cache_store_test:{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(StdDuration::from_secs(0))
                .as_nanos()
        );
        let _ = store.delete(&key).await;
        let value = store
            .incr(&key, 1, Some(Duration::from_secs(1)))
            .await
            .unwrap();
        assert_eq!(value, 1);
        let bytes = store.get(&key).await.unwrap().unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "1");
    }
}
