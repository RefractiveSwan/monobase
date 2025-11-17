# Env Quickstart - DFPS Workspace

This runbook explains how environment profiles are organized across the DFPS workspace, how the `dfps_configuration` loader discovers `.env` files, and how to create local overrides for dev/test/prod.

---

## 1. Loader overview

Every crate calls `dfps_configuration::load_env(<namespace>)` during startup. The loader:

1. Resolves the active profile from `DFPS_ENV` (falls back to `APP_ENV`, defaults to `dev`).
2. Determines the workspace root (walks up until it finds `Cargo.lock` unless `DFPS_WORKSPACE_ROOT` is set).
3. Searches `data/environment/` (or `DFPS_ENV_DIR`) for `.env.<namespace>.<profile>`. If missing, it also checks `.env.<namespace>.local` in the same directory.
4. Loads the first file it finds; if none exist and `DFPS_ENV_STRICT=1` (or `CI` is set), the process aborts with a helpful error listing every path that was attempted.
5. `DFPS_ENV_FILE` overrides the filename entirely-useful for pointing at deployment-specific secrets.

## 2. Namespace map

| Namespace | Crate(s) | Typical file |
| --- | --- | --- |
| `app.web.api` | `dfps_api` (`lib/app/web/backend/api`) | `data/environment/.env.app.web.api.dev` |
| `app.web.frontend` | `dfps_web_frontend` (`lib/app/web/frontend`) | `data/environment/.env.app.web.frontend.dev` |
| `app.cli` | `dfps_cli` binaries (`map_bundles`, `map_codes`) | `data/environment/.env.app.cli.dev` |
| `domain.fake_data` | `dfps_fake_data` sample generator | `data/environment/.env.domain.fake_data.dev` |
| `platform.observability` | `dfps_observability` logging helpers | `data/environment/.env.platform.observability.dev` |
| `platform.test_suite` | `dfps_test_suite` fixtures + integration tests | `data/environment/.env.platform.test_suite.test` (when `DFPS_ENV=test`) |

To bootstrap a new namespace, drop a `.env.<namespace>.example` file in `data/environment/` and follow the same naming convention.

## 3. Required variables

| Variable | Scope | Description |
| --- | --- | --- |
| `DFPS_ENV` / `APP_ENV` | Loader | Selects the active profile (`dev`, `test`, `prod`). |
| `DFPS_ENV_FILE` | Loader | Absolute/relative path to a specific env file. Overrides namespace logic when set. |
| `DFPS_ENV_DIR` | Loader | Directory containing env files (default `data/environment`). |
| `DFPS_WORKSPACE_ROOT` | Loader | Explicit workspace root path (optional). |
| `DFPS_ENV_STRICT` | Loader | When truthy, fail if no env file is found (automatically true in CI). |
| `DFPS_API_HOST` / `DFPS_API_PORT` | Backend | Overrides `ApiServerConfig` bind address (optional). |
| `DFPS_FRONTEND_LISTEN_ADDR` | Frontend | Bind address for `dfps_web_frontend`. |
| `DFPS_API_BASE_URL` | Frontend | URL that the frontend uses to reach the backend. |
| `DFPS_API_CLIENT_TIMEOUT_SECS` | Frontend | Reqwest timeout (seconds). |
| `DFPS_DOCS_URL` | Frontend | Optional URL that `/docs` should redirect to (e.g., `http://127.0.0.1:3000`). |

Each namespace-specific `.env` file can also hold logging directives (`RUST_LOG`), telemetry endpoints, or other secrets that surfaces consume.

## 4. Managing `.env` files

1. Navigate to `code/data/environment/`.
2. Copy the relevant `.env.<namespace>.example` file to `.env.<namespace>.<profile>` (e.g., `.env.app.web.api.dev`).
3. Fill in any secrets or overrides.
4. Keep real `.env.*` files untracked-only the `.example` templates live in git.

Example (`.env.app.web.frontend.dev`):

```bash
DFPS_FRONTEND_LISTEN_ADDR=127.0.0.1:8090
DFPS_API_BASE_URL=http://127.0.0.1:8080
DFPS_API_CLIENT_TIMEOUT_SECS=15
DFPS_DOCS_URL=http://127.0.0.1:3000
RUST_LOG=dfps_web_frontend=info
```

## 5. CI & production guidance

- Set `DFPS_ENV_STRICT=1` (or let `CI` do it) so missing env files break fast with a detailed error.
- In CI, copy production-safe templates into `data/environment/` before running binaries, or set `DFPS_ENV_FILE` to point at a secure secret mount.
- For deployments, provide the env files via your orchestration layer (Kubernetes ConfigMap/Secret, GitHub Actions `env` file, etc.) and set `DFPS_ENV_DIR` accordingly.
- Avoid storing credentials in the repo-use the `.example` files purely as documentation of required keys.

For web-specific startup instructions, see `docs/runbook/web-quickstart.md`.
