# Crate: lib/platform/configuration — `refractive_swan_configuration`

**Purpose**  
Workspace‑wide env loader. Resolves a namespaced `.env` and loads it with `dotenvy`.

**Primary API**
```rust
pub fn load_env(namespace: &str) -> Result<EnvLoadOutcome, EnvLoadError>;
```
- `namespace`: dotted path reflecting crate location (e.g., `app.web.api`).
- `EnvLoadOutcome { namespace, profile, files }`, where `profile` = `refractive_swan_ENV` → `APP_ENV` → `"dev"`.

**Resolution rules**
1. If `refractive_swan_ENV_FILE` is set → resolve relative to workspace root and load that file only.
2. Else search directories (in order):
   - `<workspace>/data/environment`
   - `<workspace>`
3. In each dir, try `.env.<namespace>.<profile>` then fallback `.env.<namespace>.local`.

**Workspace root discovery**
- `refractive_swan_WORKSPACE_ROOT` (if exists) or walk up from `current_dir()` until a `Cargo.lock` is found.

**Strict mode**
- If nothing loads **and** `refractive_swan_ENV_STRICT` or `CI` is truthy, return:
  `EnvLoadError::FileMissing { namespace, profile, attempted }`.

**Error variants**
```
CurrentDir(io::Error)
WorkspaceRootNotFound
DotEnv { path, source }
FileMissing { namespace, profile, attempted: Vec<PathBuf> }
```

**Notes & gotchas**
- Boolean envs: empty value counts as **true**; only `false|0|off` (case‑insensitive) are false.
- Paths are resolved relative to **workspace root**, not process CWD.

**Used by**
- `platform.observability`, `platform.test_suite`
- `app.cli`, `app.web.api`, `app.web.frontend`
