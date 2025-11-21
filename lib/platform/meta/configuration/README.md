# refractive_swan_configuration – Workspace env + config helpers

Centralized entrypoint for resolving refractive_swan environment namespaces/profiles and parsing
typed values used by app/domain/platform crates.

See:
- `docs/runbook/010-foundation/env-quickstart.md`
- `docs/system-design/base/directory-architecture.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-10--platform-configuration--env-loading`

## Usage

- Call `load_env("<namespace>")` from each crate (`app.web.api`, `app.cli`, `platform.vector_store`, etc.). It loads `.env.<namespace>.<profile>` (profile comes from `refractive_swan_ENV`/`APP_ENV`, default `dev`).
- Search strategy:
  1. Respect `refractive_swan_ENV_FILE` (absolute or relative to workspace root).
  2. Else, iterate directories from `config_paths().env_dirs`: `refractive_swan_ENV_DIR` (if set) followed by `<workspace>/data/environment` and the workspace root.
  3. Strict mode (`refractive_swan_ENV_STRICT=1` or `CI` truthy) raises `EnvLoadError::FileMissing` when no file is found; non-strict mode succeeds with an empty file list.
  4. Workspace root detection honors `refractive_swan_WORKSPACE_ROOT` or walks up from `cwd` until it finds `Cargo.lock`.

## Typed helpers

Crate layout:

- `loader` – `load_env`, `EnvLoadOutcome`, and `EnvLoadError`.
- `paths` – `workspace_root`, `config_paths`, and the resolved env directories.
- `values` – typed parsers (`bool_var`, `u32_var`, `u64_var`, `port_var`) plus `EnvValueError`.

To avoid bespoke parsing throughout the repo:

- `bool_var("NAME")` → `Option<bool>` accepting `true/false/1/0/on/off` (empty string coerces to `true`).
- `u32_var`, `u64_var`, `port_var` → `Option<T>` with descriptive `EnvValueError`s on invalid input (`port_var` enforces 1–65535).
- `workspace_root()` / `config_paths()` expose the resolved root + env dirs so crates can map relative paths into the workspace without reimplementing discovery.

Crates like `refractive_swan_vector_store`, `refractive_swan_compliance`, `refractive_swan_api`, and `refractive_swan_observability` should call these helpers instead of `std::env::var` + ad-hoc parsing.
