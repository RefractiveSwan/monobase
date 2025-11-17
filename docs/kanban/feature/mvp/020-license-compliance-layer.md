# Kanban - feature/license-compliance-layer (020)

**Theme:** Licensing & compliance - enforcement & policy hooks  
**Branch:** `feature/platform/LIC-020-license-compliance-layer`  
**Branch target version:** `v0.1.0`  
**Status:** INPROGRESS  
**Introduced in:** `v0.1.0`  
**Last updated in:** `v0.1.0`  
**Goal:** Turn the existing license metadata into enforceable policies that gate mapping, CLIs, and exports according to configured license modes.

**Scope guardrails:** Keep compliance behavior purely policy-driven; do not change mapping thresholds or vector behavior from epics 013/014/017/018/019. Reuse existing license metadata from `dfps_terminology` and NCIt/OBO imports; no new licensing inference.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO
- _Empty_

---

## INPROGRESS

### LIC-04 – Tests & audit logging

Codify behavior so regression suites (especially vector_mapping.rs from 013) stay green in Internal mode and gain new coverage for OpenSource/Partner.

  - [x] Add tests in `dfps_test_suite` ensuring:

  - [x] In `OpenSource` mode, CPT/SNOMED codes are blocked from mapping; LOINC/OBO remain allowed.
  - [x] In `Internal` mode, behavior is unchanged from current mapping.

  - [x] Ensure logs from `dfps_observability` include:

    - [x] License mode.
    - [x] Counts of license-blocked codes.

  - [ ] Add audit log wording/fields that align with existing NoMatch reasons to avoid confusing downstream analytics.

#### Cross-Cohesion

- **Engineering Targets:** B, D
- **Crates & Paths:**
  - `lib/platform/compliance` (`dfps_compliance`)
  - `lib/platform/test_suite` (`dfps_test_suite`)
  - `lib/platform/observability` (`dfps_observability`)
- **Shared Metrics & Signals:**
  - `auto_mapped`, `needs_review`, `no_match`, `mapping_precision`, `mapping_recall`, `mapping_f1`
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs` (mode matrix)
  - Compliance-mode log assertions in CI
- **Interfaces & Contracts:**
  - CLIs: `dfps_cli map_bundles`, `dfps_cli map_codes`
  - Env: `DFPS_COMPLIANCE_MODE`, `DFPS_COMPLIANCE_POLICY_PATH`

---

## REVIEW

### LIC-01 – Compliance crate & policy model

Define a reusable policy layer that other epics (014 external terminology APIs, 019 OBO import) can depend on without altering their data flows.

- [x] Add `lib/platform/compliance` crate (`dfps_compliance`) with:

  - [x] `ComplianceMode` enum:

    - `Internal`, `Partner`, `OpenSource`.

  - [x] `Policy` struct capturing:

    - Which `LicenseTier` values are allowed per mode (e.g., `Licensed` disallowed in `OpenSource`).
    - Allowed actions: `Ingest`, `Map`, `Export`.

  - [x] Align policy defaults with existing tier semantics from `docs/system-design/clinical/ncit/requirements/ingestion-requirements.md` and `docs/reference-terminology/semantic-relationships.yaml` (no new tiers).

  - [x] Env-driven config:

    - `DFPS_COMPLIANCE_MODE`
    - `DFPS_COMPLIANCE_POLICY_PATH` (optional JSON/YAML override).

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/platform/compliance` (`dfps_compliance`)
  - `lib/domain/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - `auto_mapped`, `needs_review`, `no_match`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/requirements/ingestion-requirements.md`
  - `docs/reference-terminology/semantic-relationships.yaml`
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
  - `docs/kanban/feature/mvp/019-obo-import-and-reasoning.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite` policy/unit coverage for policy parsing
- **Interfaces & Contracts:**
  - Env: `DFPS_COMPLIANCE_MODE`, `DFPS_COMPLIANCE_POLICY_PATH`
  - Traits/APIs: compliance policy loader (new)

### LIC-02 – License-aware gating

Apply the policy layer to the mapping flow without changing ranking logic from 013/017. Ensure observability lines up with vector metrics already present in `vector_mapping.rs`.

  - [x] Integrate `dfps_compliance` into mapping paths:

  - [x] Before mapping codes:

    - Check `EnrichedCode.license_tier` and current `ComplianceMode`.
    - If forbidden, produce `MappingResult` with:

      - `state = MappingState::NoMatch`
      - `reason = Some("license_blocked")`.

  - [x] Ensure this behavior is clearly logged and tagged in metrics.

  - [x] Define metric/log labels consistent with existing mapping observability (e.g., `license_blocked`, `license_mode`) so dashboards from 017 remain compatible.

  - [x] Update `map_bundles` / `map_codes` CLIs to:

  - [x] Print a summary of license-blocked codes.
  - [x] Optionally fail-fast in strict modes (`--fail-on-license-block`).

  - [x] Keep CLI UX consistent with prior flags from epics 013/014; document interaction with `--explain` and eval flags (no breaking changes to defaults).

#### Cross-Cohesion

- **Engineering Targets:** B, D
- **Crates & Paths:**
  - `lib/domain/mapping` (`dfps_mapping`)
  - `lib/platform/compliance` (`dfps_compliance`)
  - `lib/app/cli` (`dfps_cli`)
  - `lib/platform/observability` (`dfps_observability`)
- **Shared Metrics & Signals:**
  - `auto_mapped`, `needs_review`, `no_match`, `mapping_precision`, `mapping_recall`, `mapping_f1`
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/013-mapping-vector-backend.md`
  - `docs/system-design/clinical/ncit/behavior/state-servicerequest.md`
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs` (add compliance modes)
  - CLI smoke for `map_bundles`/`map_codes` strict mode
- **Interfaces & Contracts:**
  - CLIs: `dfps_cli map_bundles`, `dfps_cli map_codes`
  - Env: `DFPS_COMPLIANCE_MODE`, `DFPS_COMPLIANCE_POLICY_PATH`

### LIC-03 – Export & docs safeguards

Provide guardrails for downstream warehouse/BI work (017) without redefining export schemas. Document operational modes alongside existing runbooks.

  - [x] Add helper APIs for downstream exporters (warehouse, BI):

    - [x] `dfps_compliance::assert_export_allowed(license_tiers: &[LicenseTier]) -> Result<(), ComplianceError>`.

  - [x] Document how to:

  - [x] Run DFPS in `OpenSource` mode (no licensed vocabularies).
  - [x] Run DFPS in `Internal` mode (full mapping allowed).

  - [x] Add pointers to the relevant env templates (e.g., `data/environment/.env.platform.vector_store.dev`, `.env.domain.terminology.dev.example`) explaining compliance settings.

#### Cross-Cohesion

- **Engineering Targets:** B, C, D
- **Crates & Paths:**
  - `lib/platform/compliance` (`dfps_compliance`)
  - `lib/app/web/backend/datamart` (`dfps_datamart`)
  - `lib/app/cli` (`dfps_cli`)
- **Shared Metrics & Signals:**
  - `auto_mapped`, `needs_review`, `no_match`, `mapping_precision`, `mapping_recall`
- **Docs & Kanbans Touched:**
  - `docs/runbook/terminology-apis-quickstart.md`
  - `docs/kanban/feature/mvp/017-analytics-dashboards-cohorts.md`
- **Experiments / CI Hooks:**
  - Export/BI smoke checks in `dfps_test_suite` (new compliance gate tests)
- **Interfaces & Contracts:**
  - Env: `DFPS_COMPLIANCE_MODE`
  - APIs: `dfps_compliance::assert_export_allowed`
  

---

## DONE
- _Empty_

---

## Acceptance Criteria

- License tiers and source kinds no longer just annotate `MappingResult`; they govern whether mapping is allowed based on configured mode.
- CLIs and exporters honor compliance policies and surface clear error messages when policies are violated.
- Existing tests still pass in `Internal` mode, and new tests verify correct behavior in more restrictive modes.

## Out of Scope

- Legal contract management, entitlement tracking, or license renewal workflows.
- User identity / role-based exceptions (all policies are workspace-wide in this epic).
