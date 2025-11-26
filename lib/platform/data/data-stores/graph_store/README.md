# refractive_swan_cache_store (Platform Store)

**Conceptual location:** `lib/platform/store/cache_store`  
**Current physical location:** Not yet implemented  
**Scope:** Key-value cache abstraction with Redis (and optional in-memory) backends

This directory represents the **planned home** for caching infrastructure. The initial implementation will target Redis, with an optional in-memory backend for testing and standalone deployments.

---

## Purpose

`refractive_swan_cache_store` provides:

1. **Cache traits**: `CacheStore` with get/set/delete/incr operations
2. **Backend enum**: `CacheBackend` (Redis, InMemory)
3. **Configuration**: `CacheConfig` with URL, pool size, TTL defaults
4. **Error mapping**: Unified `CacheError` wrapping Redis/backend errors

**Goal**: Decouple caching logic from domain/warehouse code, enabling per-node cache backend selection.

---

## Trait Design (Sketch)

### CacheBackend

```rust
/// Supported cache backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheBackend {
    /// Redis (production-grade distributed cache)
    Redis,
    
    /// In-memory (local hashmap, no persistence)
    InMemory,
    
    /// Disabled (no-op cache for minimal deployments)
    Disabled,
}
```

### CacheConfig

```rust
/// Configuration for cache store connection.
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Backend type
    pub backend: CacheBackend,
    
    /// Connection URL (e.g., `redis://localhost:6379`)
    pub url: String,
    
    /// Maximum pool size
    pub pool_max: u32,
    
    /// Default TTL for cached items (seconds)
    pub default_ttl_secs: u64,
}
```

### CacheStore Trait

```rust
/// Trait for cache operations.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Get a value by key.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;
    
    /// Set a value with optional TTL.
    async fn set(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<(), CacheError>;
    
    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    
    /// Increment a counter (atomic).
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, CacheError>;
    
    /// Check if the cache is healthy.
    async fn health_check(&self) -> Result<(), CacheError>;
}
```

### CacheError

```rust
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("cache disabled")]
    Disabled,
    
    #[error("connection failed: {source}")]
    ConnectionFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("operation failed: {0}")]
    OperationFailed(String),
    
    #[error("serialization error: {0}")]
    SerializationError(String),
}
```

---

## Use Cases

### Analytics Query Caching

Cache expensive OLAP queries (NCIt summary, cohort results):

```rust
// In refractive_swan_datamart or refractive_swan_datawarehouse
async fn ncit_summary_cached(
    &self,
    filters: &Filters,
    cache: &dyn CacheStore,
) -> Result<Vec<SummaryRow>,DatamartError> {
    let cache_key = format!("ncit_summary:{}", filters.hash());
    
    // Check cache
    if let Some(cached) = cache.get(&cache_key).await? {
        return Ok(bincode::deserialize(&cached)?);
    }
    
    // Miss: run query
    let rows = self.ncit_summary_uncached(filters).await?;
    
    // Store in cache (1 hour TTL)
    let serialized = bincode::serialize(&rows)?;
    cache.set(&cache_key, &serialized, Some(3600)).await?;
    
    Ok(rows)
}
```

### Rate Limiting / Counters

Track request counts for governance:

```rust
// In refractive_swan_mesh_governance or refractive_swan_mesh_node
async fn check_rate_limit(
    &self,
    node_id: &MeshNodeId,
    cache: &dyn CacheStore,
) -> Result<bool, GovernanceError> {
    let key = format!("rate_limit:{}:hourly", node_id);
    let count = cache.incr(&key, 1).await?;
    
    // Set TTL on first increment
    if count == 1 {
        cache.set(&key, &[], Some(3600)).await?;
    }
    
    Ok(count <= 1000) // max 1000 requests/hour
}
```

### Session / Auth Tokens (Future)

If hub-node communication requires auth:

```rust
// In refractive_swan_mesh_hub
async fn validate_token(
    &self,
    token: &str,
    cache: &dyn CacheStore,
) -> Result<MeshNodeId, AuthError> {
    let key = format!("auth:token:{}", token);
    
    match cache.get(&key).await? {
        Some(node_id_bytes) => Ok(MeshNodeId(String::from_utf8(node_id_bytes)?)),
        None => Err(AuthError::InvalidToken),
    }
}
```

---

## Backends

### Redis

- **Production-grade**: Distributed, persistent, atomic operations
- **Use case**: Multi-node deployments, hub coordination
- **Driver**: `redis-rs` or `fred` crate

### InMemory

- **Simple hashmap**: `Arc<RwLock<HashMap<String, (Vec<u8>, Option<Instant>)>>>`
- **Use case**: Standalone nodes, testing
- **Limitations**: Not distributed, lost on restart

### Disabled

- **No-op**: All operations succeed immediately, no caching
- **Use case**: Minimal nodes with no performance concerns

---

## Per-Node Backend Selection

Different nodes can use different cache backends:

- **Node A** (production, small): InMemory (sufficient for single-instance)
- **Node B** (production, clustered): Redis (shared cache across replicas)
- **Node C** (dev/test): Disabled (no overhead)

---

## Migration Plan (MESH-025)

### Phase 1: Design Only (current)

- This README documents the planned abstraction
- No implementation yet

### Phase 2: Implement Trait + Redis Backend

- Create `lib/platform/store/cache_store/src/traits.rs`
- Create `lib/platform/store/cache_store/src/backends/redis.rs`
- Create `lib/platform/store/cache_store/src/backends/inmemory.rs`

### Phase 3: Wire into Datamart/Governance

- `refractive_swan_datamart` accepts optional `Arc<dyn CacheStore>`
- Analytics queries check cache before running SQL
- `refractive_swan_mesh_governance` uses cache for rate limits/counters

### Phase 4: Hub Coordination

- `refractive_swan_mesh_hub` uses Redis cache for:
  - Node registration/discovery
  - Job queue metadata
  - Auth tokens (if required)

---

## Testing

### Unit Tests

- `CacheConfig` validation
- In-memory backend correctness (get/set/delete/incr)
- TTL expiration logic

### Integration Tests

- Real Redis connectivity (requires Docker/env)
- Concurrent operations (race conditions)
- Connection pool behavior

### Domain Tests

- Analytics query caching (hit/miss scenarios)
- Rate limiting enforcement
- Cache-aside pattern correctness

---

## References

- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
- **Governance Model**: `lib/platform/mesh/governance/README.md` (to be created)
- **Node Runtime**: `docs/system-design/mesh/node-runtime.md` (to be created)
