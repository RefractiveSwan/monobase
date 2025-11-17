# Kanban - feature/license-compliance-layer (020)

**Theme:** Licensing & compliance - enforcement & policy hooks  
**Branch:** `feature/platform/LIC-020-license-compliance-layer`  
**Goal:** Turn the existing license metadata into enforceable policies that gate mapping, CLIs, and exports according to configured license modes.

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

### LIC-01 – Compliance crate & policy model

- [ ] Add `lib/platform/compliance` crate (`dfps_compliance`) with:

  - [ ] `ComplianceMode` enum:

    - `Internal`, `Partner`, `OpenSource`.

  - [ ] `Policy` struct capturing:

    - Which `LicenseTier` values are allowed per mode (e.g., `Licensed` disallowed in `OpenSource`).
    - Allowed actions: `Ingest`, `Map`, `Export`.

  - [ ] Env-driven config:

    - `DFPS_COMPLIANCE_MODE`
    - `DFPS_COMPLIANCE_POLICY_PATH` (optional JSON/YAML override).

### LIC-02 – License-aware gating

- [ ] Integrate `dfps_compliance` into mapping paths:

  - [ ] Before mapping codes:

    - Check `EnrichedCode.license_tier` and current `ComplianceMode`.
    - If forbidden, produce `MappingResult` with:

      - `state = MappingState::NoMatch`
      - `reason = Some("license_blocked")`.

  - [ ] Ensure this behavior is clearly logged and tagged in metrics.

- [ ] Update `map_bundles` / `map_codes` CLIs to:

  - [ ] Print a summary of license-blocked codes.
  - [ ] Optionally fail-fast in strict modes (`--fail-on-license-block`).

### LIC-03 – Export & docs safeguards

- [ ] Add helper APIs for downstream exporters (warehouse, BI):

  - [ ] `dfps_compliance::assert_export_allowed(license_tiers: &[LicenseTier]) -> Result<(), ComplianceError>`.

- [ ] Document how to:

  - [ ] Run DFPS in `OpenSource` mode (no licensed vocabularies).
  - [ ] Run DFPS in `Internal` mode (full mapping allowed).

### LIC-04 – Tests & audit logging

- [ ] Add tests in `dfps_test_suite` ensuring:

  - [ ] In `OpenSource` mode, CPT/SNOMED codes are blocked from mapping; LOINC/OBO remain allowed.
  - [ ] In `Internal` mode, behavior is unchanged from current mapping.

- [ ] Ensure logs from `dfps_observability` include:

  - [ ] License mode.
  - [ ] Counts of license-blocked codes.

---

## INPROGRESS
- _Empty_

---

## REVIEW
- _Empty_

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
