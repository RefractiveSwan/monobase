# Kanban - feature/terminology-external-apis (014)

**Theme:** External infra & heavy services - UMLS/NCIt APIs  
**Branch:** `feature/terminology-external-apis`  
**Goal:** Introduce networked terminology clients for UMLS/NCIm/NCIt and wire them into `dfps_terminology` + `dfps_mapping` as an optional fallback for unknown or low-confidence codes.  
**Status:** INPROGRESS | **Introduced:** `v0.1.0` | **Last updated:** `Unreleased`

### Columns
* **TODO** – Not started yet  
* **INPROGRESS** – In progress  
* **REVIEW** – Needs code review / refactor / docs polish  
* **DONE** – Completed  

---

## TODO

### TERM-API-02 – HTTP client implementations

- [ ] Implement `UmlsTerminologyClient` (behind feature flag `umls-http`):

  - [ ] Uses `reqwest` to call configured UMLS / NCIm endpoints.
  - [ ] Handles auth headers, rate limits (backoff), and simple pagination.

- [ ] Implement `NcitTerminologyClient` (feature `ncit-http`) if distinct:

  - [ ] Resolve NCIt codes and synonyms.
  - [ ] Align responses with `NcitRecord`.

- [x] Provide a `CompositeTerminologyClient` that:

  - [x] Tries local mock tables / embedded JSON first.
  - [x] Falls back to HTTP clients when configured.

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/domain/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/architecture.md`
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
- **Experiments / CI Hooks:**
  - HTTP client smoke tests (mock server)
  - `dfps_test_suite` mapping integration with external lookups
- **Interfaces & Contracts:**
  - Feature flags: `umls-http`, `ncit-http`
  - `CompositeTerminologyClient`
  - Env: `DFPS_TERMINOLOGY_BASE_URL`, `DFPS_TERMINOLOGY_MODE`

- [ ] Extend `dfps_mapping::map_with_summary` to accept an optional `TerminologyClient`:

  - [x] For `UnknownSystem` or low-scoring internal candidates:
    - [x] Call `TerminologyClient::lookup_cui` / `lookup_ncit`.
    - [x] Promote successful lookups to `MappingResult` with:
      - `strategy = MappingStrategy::Rule` or `Composite`.
      - `reason = Some("external_terminology_lookup")`.
  - [x] Leave behavior unchanged when client is `None`.

- [x] Ensure `MappingSummary` captures counts for:

  - [x] `extern_lookup_success`, `extern_lookup_miss`, `extern_lookup_error`.

- [ ] Respect license metadata from `dfps_terminology::LicenseTier` and future compliance rules
  (epic 020) before making external calls (e.g., skip forbidden systems).

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/domain/mapping` (`dfps_mapping`)
  - `lib/domain/terminology` (`dfps_terminology`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
  - `docs/system-design/clinical/ncit/architecture.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
- **Interfaces & Contracts:**
  - `map_staging_codes_with_summary` optional terminology client
  - `MappingSummary` extern lookup counters

### TERM-API-03 – Mapping integration & policy hooks

- [ ] Extend `dfps_mapping::map_with_summary` to accept an optional `TerminologyClient`:

  - [x] For `UnknownSystem` or low-scoring internal candidates:
    - [x] Call `TerminologyClient::lookup_cui` / `lookup_ncit`.
    - [x] Promote successful lookups to `MappingResult` with:
      - `strategy = MappingStrategy::Rule` or `Composite`.
      - `reason = Some("external_terminology_lookup")`.
  - [x] Leave behavior unchanged when client is `None`.

- [x] Ensure `MappingSummary` captures counts for:

  - [x] `extern_lookup_success`, `extern_lookup_miss`, `extern_lookup_error`.

- [ ] Respect license metadata from `dfps_terminology::LicenseTier` and future compliance rules
  (epic 020) before making external calls (e.g., skip forbidden systems).

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/domain/mapping` (`dfps_mapping`)
  - `lib/domain/terminology` (`dfps_terminology`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
  - `docs/system-design/clinical/ncit/architecture.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
- **Interfaces & Contracts:**
  - `map_staging_codes_with_summary` optional terminology client
  - `MappingSummary` extern lookup counters

### TERM-API-04 – Local test doubles & fixtures

- [x] Add a `MockTerminologyClient` in `dfps_terminology::client::testing`:

  - [x] Hard-code mappings for existing regression fixtures (`CPT 78815`, SNOMED PET, etc.).
  - [ ] Simulate latency and error responses for robustness tests (in progress).

- [x] Add integration tests in `dfps_test_suite`:

  - [x] When client is provided (mock), `map_staging_codes_with_summary` uses it for unknown codes.
  - [x] When client is absent, behavior is identical to current mock-table-only mapping.

#### Cross-Cohesion

- **Engineering Targets:** A1, B
- **Crates & Paths:**
  - `lib/domain/terminology` (`dfps_terminology`)
  - `lib/platform/test_suite` (`dfps_test_suite`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
- **Interfaces & Contracts:**
  - `MockTerminologyClient`
  - `TerminologyClient` trait

---

## REVIEW

### TERM-API-01 – TerminologyClient abstraction

- [x] Add a `client` module to `dfps_terminology`:

  - [x] Define trait `TerminologyClient` with operations such as:
    - [x] `lookup_cui(system, code) -> Result<Option<CuiRecord>>`
    - [x] `lookup_ncit(cui_or_code) -> Result<Option<NcitRecord>>`
    - [x] (Optional) `search_by_text(text) -> Result<Vec<NcitRecord>>`.

  - [x] Define simple structs:

    - `CuiRecord { cui: String, preferred_name: String }`
    - `NcitRecord { ncit_id: String, preferred_name: String, synonyms: Vec<String> }`

- [x] Introduce `TerminologyClientConfig` (env-driven):

  - `DFPS_TERMINOLOGY_BASE_URL`, `DFPS_TERMINOLOGY_API_KEY`, `DFPS_TERMINOLOGY_TIMEOUT_SECS`.

#### Cross-Cohesion

- **Engineering Targets:** A1, B
- **Crates & Paths:**
  - `lib/domain/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/architecture.md`
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
- **Experiments / CI Hooks:**
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
  - Unit tests for terminology client
- **Interfaces & Contracts:**
  - Trait `TerminologyClient`
  - Env: `DFPS_TERMINOLOGY_BASE_URL`, `DFPS_TERMINOLOGY_API_KEY`, `DFPS_TERMINOLOGY_TIMEOUT_SECS`, `DFPS_TERMINOLOGY_MODE`

### TERM-API-05 – Config, env, and docs

- [x] Add `.env.domain.terminology.dev/example` in `data/environment` documenting:

  - `DFPS_TERMINOLOGY_BASE_URL`
  - `DFPS_TERMINOLOGY_API_KEY`
  - `DFPS_TERMINOLOGY_TIMEOUT_SECS`
  - `DFPS_TERMINOLOGY_MODE = "mock_only" | "http_fallback" | "http_only"`

- [x] Extend `docs/system-design/clinical/ncit/architecture.md` with a subsection:

  - “External terminology services” describing how the HTTP clients plug into the existing mapping pipeline.

- [x] Add a short runbook `docs/runbook/terminology-apis-quickstart.md`.

#### Cross-Cohesion

- **Engineering Targets:** A1, B, D
- **Crates & Paths:**
  - `lib/domain/terminology` (`dfps_terminology`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - auto_mapped
  - needs_review
  - no_match
- **Docs & Kanbans Touched:**
  - `docs/kanban/feature/mvp/014-terminology-external-apis.md`
  - `docs/system-design/clinical/ncit/architecture.md`
  - `docs/runbook/terminology-apis-quickstart.md`
- **Experiments / CI Hooks:**
  - Config validation tests for terminology env
- **Interfaces & Contracts:**
  - Env: `DFPS_TERMINOLOGY_BASE_URL`, `DFPS_TERMINOLOGY_API_KEY`, `DFPS_TERMINOLOGY_TIMEOUT_SECS`, `DFPS_TERMINOLOGY_MODE`

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

- `dfps_mapping` can optionally consult external terminology services via `dfps_terminology::TerminologyClient`.
- When external APIs are disabled or unreachable, mapping behavior remains deterministic and uses only embedded mock tables.
- Tests prove that external calls improve coverage for previously `UnknownSystem` / `NoMatch` cases without regressing existing golden tests.
- Env + docs clearly describe how to enable/disable external terminology usage.

## Out of Scope

- Hosting or mirroring full UMLS/NCIm/NCIt databases in-house.
- Complex bulk-sync jobs (those belong in a future ETL/warehouse track if needed).
