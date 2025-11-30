# refractive_swan_cache_store (Platform Store)

**Conceptual location:** `lib/platform/data/data-stores/cache_store`  
**Scope:** Cache abstraction for rate limits, DP budget tracking, and analytics query caching. Backends: in-memory + Redis.

This crate now ships a minimal `CacheStore` trait plus an in-memory backend used by the mesh node for governance hooks and health reporting, with a Redis backend for shared counters.

---

## Surface

- **Backends:** `CacheBackend::{Redis, InMemory, Disabled}` (Redis requires `refractive_swan_CACHE_URL`; InMemory is default).
- **Config:** `CacheConfig::from_env()` reads:
  - `refractive_swan_CACHE_BACKEND` (`redis` | `inmemory` | `disabled`, default `disabled`)
  - `refractive_swan_CACHE_URL` (required for Redis)
  - `refractive_swan_CACHE_POOL_MAX` (default `5`)
  - `refractive_swan_CACHE_DEFAULT_TTL_SECS` (default `60`)
- **Trait:** `CacheStore` (async) with `get/set/delete/incr/health_check` and an explicit `backend()` label.
- **Helper:** `cache_from_config(&CacheConfig) -> Option<Arc<dyn CacheStore>>` returns `None` for `Disabled`, builds `InMemoryCache` or `RedisCache`.

### Keyspace & TTL conventions (P3.3)

- **Rate limits:** `rate_limit:<node>:<window>` → integer counter, caller supplies TTL for the window.
- **DP budget “hot cache”:** `dp_budget:<node>:<yyyy-mm-dd>` → integer counter scaled by `1_000` (epsilon * 1000). TTL = 24h aligned to the date.
- **Health:** `CacheStore::health_check` powers `/mesh/health` and `/mesh/job` mapping health payloads (`cache_backend` + `cache_health`).

### InMemory backend

- `InMemoryCache::new(default_ttl_secs)` keeps values in a `RwLock<HashMap<...>>` with optional TTLs.
- `incr` sets TTL on first write when provided; values are stored as UTF-8 integers so `get` works for counters.

### Redis backend

- `RedisCache` uses `redis::Client` with tokio connections; `incr` sets TTL on first write and `health_check` runs `PING`.
- Requires `refractive_swan_CACHE_URL` (e.g., `redis://localhost:6379`); respects `default_ttl_secs`.
- Integration test (`redis_round_trip_when_url_present`) runs only when `REFRACTIVE_SWAN_REDIS_URL` is set (dev/CI hook).

---

## Node runtime integration (P3.3)

- `NodeDataPlane` loads `CacheConfig` from env, builds an in-memory cache by default, and exposes:
  - `/mesh/health`: reports `cache_backend` + `cache_health`.
  - Mesh job governance: DP budget reads from cache before evaluation; budget consumption writes back via `incr` with a 24h TTL.
- Redis backend enables multi-node rate limits and shared DP budget counters; DP budget snapshots are also durably persisted via `RelationalStore` (sqlite today) into `mesh_dp_budget`. Postgres/DuckDB persistence will be enabled once `refractive_swan_relational_store` exposes backends.

---

## Testing

- Unit tests cover `incr`, TTL expiry, TTL-on-first-incr behavior for `InMemoryCache`, and Redis config validation.
- Mesh-node tests assert DP budget denials surface `policy_denied:dp_budget_exceeded` and export denials use license-aware codes.

---

## References

- Mesh runtime: `docs/system-design/mesh/node-runtime.md`
- Kanban: `docs/kanban/feature/mvp/040-infra-and-docs/030-mesh-node-hub-propagation.md` (P3.3)
- Data plane layout: `docs/system-design/base/mesh-data-plane-layout.md`
