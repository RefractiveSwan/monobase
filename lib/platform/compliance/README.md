# dfps_compliance – License tier policy & export gating

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

The crate uses `dfps_configuration::load_env("platform.compliance")` and typed helpers; configure via:

- `DFPS_COMPLIANCE_MODE` – `internal` (default), `partner`, or `open_source`. Controls baseline tier matrix.
- `DFPS_COMPLIANCE_POLICY_PATH` – optional JSON/YAML file with overrides (`mode`, `allowed_actions`, `allowed_tiers`).
- `DFPS_WORKSPACE_ROOT` – optional workspace root override (resolved automatically by default) used to locate relative policy files.

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
let config = dfps_compliance::ComplianceConfig::from_env()?; // namespace platform.compliance
let policy = config.load_policy()?; // apply JSON/YAML overrides if present

// Inject into pipeline/datamart/CLI layers:
let should_map = policy.is_allowed(ComplianceAction::Map, LicenseTier::Licensed);
dfps_compliance::assert_export_allowed(&tiers, &policy)?;
```

Downstream crates (`dfps_mapping`, `dfps_pipeline`, `dfps_datamart`) accept a `Policy` (often built in app/CLI/API code) instead of calling `load_policy_from_env`. This keeps domain logic deterministic and separates transport concerns from compliance rules.
