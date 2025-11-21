# refractive_swan_compliance – License tier policy & export gating

Home for workspace-wide compliance policy (mapping/export tiers, override files, and env wiring). See:

- `docs/system-design/base/directory-architecture.md`
- `docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-11--platform-compliance--export-gating`
- `docs/runbook/030-mapping-and-terminology/mapping-eval-quickstart.md` (how CLI/CI enforce tiers)

## Concepts

- `ComplianceMode` – coarse policy mode (`Internal`, `Partner`, `OpenSource`).
- `ComplianceAction` – actions a tier may perform (`Map`, `Export`).
- `Policy` – mapping from action → allowed tiers plus helper `is_allowed` / `assert_export_allowed`.
  - Apps inject a `Policy` into mapping/pipeline/datamart surfaces; domain crates never read env directly.
  - `Policy::default_for_mode(mode)` yields the built-in behavior, while `PolicyOverrides` allow JSON/YAML tweaks.

## Environment variables

The crate uses `refractive_swan_configuration::load_env("platform.compliance")` and typed helpers; configure via:

- `refractive_swan_COMPLIANCE_MODE` – `internal` (default), `partner`, or `open_source`. Controls baseline tier matrix.
- `refractive_swan_COMPLIANCE_POLICY_PATH` – optional JSON/YAML file with overrides (`mode`, `allowed_actions`, `allowed_tiers`).
- `refractive_swan_WORKSPACE_ROOT` – optional workspace root override (resolved automatically by default) used to locate relative policy files.

Example override (`compliance.partner.yaml`):

```yaml
mode: partner
allowed_actions: ["map", "export"]
allowed_tiers:
  map:
    - licensed
    - open
  export:
    - open
```

## Usage pattern

```rust
let config = refractive_swan_compliance::ComplianceConfig::from_env()?; // namespace platform.compliance
let policy = config.load_policy()?; // apply JSON/YAML overrides if present

// Inject into pipeline/datamart/CLI layers:
let should_map = policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed);
refractive_swan_compliance::assert_export_allowed(&tiers, &policy)?;
```

## Config & entrypoints

- `ComplianceConfig` bundles the resolved `ComplianceMode`, an optional policy override path (relative to `refractive_swan_WORKSPACE_ROOT` when not absolute), and the detected workspace root. This ensures file lookups behave the same in CI and local dev.
- Apps/servers call `refractive_swan_configuration::load_env("platform.compliance")` once, then construct the policy at startup:
  - `refractive_swan_cli` bins (`map_bundles`, `map_codes`, `load_datamart`) build a policy before streaming results so compliance failures can stop the process deterministically.
  - `refractive_swan_api` wires the policy into `ApiState` and reuses it for every request, sharing the same policy with analytics persistence/export gating.
  - Warehouse loaders (`refractive_swan_datamart`) require the caller to provide a `Policy`, ensuring SQLite export jobs honor the same tier matrix as live API/CLI flows.
- Domain crates (`refractive_swan_mapping`, `refractive_swan_pipeline`, etc.) remain agnostic of env/config parsing—they accept a pre-built `Policy` (defaulting via `Policy::default_for_mode` in tests) so behavior stays deterministic.

Downstream crates (`refractive_swan_mapping`, `refractive_swan_pipeline`, `refractive_swan_datamart`) accept a `Policy` (often built in app/CLI/API code) instead of calling `load_policy_from_env`. This keeps domain logic deterministic and separates transport concerns from compliance rules.
