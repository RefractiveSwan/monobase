## Kanban – Data Model, Fake Data, Test Suite

### Columns

* **TODO** – Not started yet
* **INPROGRESS** – In progress
* **REVIEW** – Needs code review / refactor / docs polish
* **DONE** – Completed (you’ll move things here)

---

## TODO

---

## DONE

### Domain model crate (`lib/core`)

* [x] **DM-05 – Documentation for domain model**

  * Module-level docs now describe the domain model’s intent and reference the FHIR/NCIt diagrams.

* [x] **DM-06 – Domain examples + doctests**

  * Added doctest snippets showing how to construct a `ServiceRequest` aligned with the ingestion diagrams.

### Observability & ergonomics

* [x] **OBS-01 – Structured logging hooks**

  * Added `dfps_observability` crate plus `map_bundles` logging/metrics so CLI runs emit counts per mapping state.

* [x] **OBS-02 – Metrics snapshot test**

  * Added an end-to-end test (`observability_metrics.rs`) that asserts metrics from a seeded bundle match the pipeline output counts.

* [x] **OBS-03 – CLI ergonomics & docs**

  * The consolidated CLI crate exposes `--help`, log-level selection, and documented sample commands (including deterministic seeds for `generate_fhir_bundle`).

---

## INPROGRESS

---

## REVIEW

### Test suite crate (`lib/test_suite`)

Central place for **shared test utilities, property tests, integration-style tests** across the workspace.

* [x] **TS-01 – Test harness layout**

  * `dfps_test_suite` now exposes `fixtures`, `assertions`, and `regression` modules plus workspace integration tests.
  * All workspace tests now live under `dfps_test_suite/tests/{unit,integration,e2e}` for clear separation by type.

  * In `lib/test_suite/src/lib.rs`, expose helpers:

    ```rust
    pub mod fixtures;
    pub mod assertions;
    ```
  * In the root workspace, create `tests/` integration tests that depend on `test_suite` to avoid duplication.

* [x] **TS-02 – Basic happy-path tests**

  * Integration test exercises fake-data scenario generation and serde round-trips.

  * Tests that:

    * Generate fake domain objects via `fake_data`.
    * Serialize/deserialize via serde.
    * Verify invariants hold (e.g. `status` and `intent` combos, non-empty IDs).

* [x] **TS-03 – Property-based tests (if using proptest)**

  * Added proptest suite that feeds seeded scenarios through invariant + JSON assertions.

  * Use `proptest` or similar to:

    * Generate random domain values (possibly via your fake_data crate).
    * Assert round-trip properties (`json -> dom -> json`, mapping to NCIt, etc.).
  * Ensures your functional domain model behaves well over a large input surface.

* [x] **TS-04 – Regression fixtures**

  * Added baseline `ServiceRequest` JSON fixture with regression tests/loader.

  * When bugs are found, add fixtures (e.g. JSON samples) and tests in `test_suite` as non-regression guards.

* [x] **TS-05 – CI integration**

  * Introduced GitHub Actions workflow running fmt, clippy, and tests workspace-wide.

  * Add a workspace-wide CI script / GitHub Action:

    * `cargo fmt --all`
    * `cargo clippy --all-targets -- -D warnings`
    * `cargo test --all`

### Fake data generator crate (`lib/fake_data`)

Using `fake` crate for convenient generators against your domain types.

* [x] **FD-01 – Wire `fake` + `Dummy` derives**
  * Core exposes a `dummy` feature powered by `fake::Dummy`, enabled automatically from `dfps_fake_data`.

  * Add `fake` + `rand` deps (already in WS-02).
  * For simpler types, derive `Dummy` directly in `core` (behind a cfg or feature if you want), or in `fake_data` via wrapper types.

* [x] **FD-02 – Implement generators for value objects**

  * Added helpers for IDs, status/intent, and descriptions backed by `fake` + `rand`.

  * Provide helpers like:

    ```rust
    pub fn fake_patient_id() -> PatientId { /* ... */ }
    pub fn fake_service_request_id() -> ServiceRequestId { /* ... */ }
    ```
  * Use `Faker` or fine-grained fakers (names, dates, codes) to approximate realistic distributions (e.g. PET/CT vs other orders).

* [x] **FD-03 – Implement coherent aggregate generators**

  * Added `ServiceRequestScenario` with consistent Patient/Encounter/Order wiring.

  * Functions that create full, internally consistent aggregates:

    ```rust
    pub fn fake_service_request_scenario() -> ServiceRequestScenario {
        // patient + encounter + SR with consistent IDs & timestamps
    }
    ```
  * Ensure generated data respects domain invariants (e.g. status transitions allowed by your state machine).

* [x] **FD-04 – Seeded fake data for reproducible tests**

  * All generators accept deterministic seeds (e.g. scenario + ID helpers).

  * Add helpers that take an RNG seed:

    ```rust
    pub fn fake_with_seed(seed: u64) -> ServiceRequest { /* ... */ }
    ```
  * This lets your test_suite crate reproduce failing cases deterministically, per best practices for property-based testing.

* [x] **FD-05 – CLI / dev helper (optional)**

  * Added `dfps_fake_data` binary `generate_sample` that emits NDJSON scenarios.

  * Add a small binary target in `fake_data` or a separate bin crate (e.g. `bin/generate_sample.rs`) that dumps fake domain objects as NDJSON for quick eyeballing.

### Domain model crate (`lib/core`)

Functional domain modeling + `serde` via ADTs and newtypes, roughly along the lines of the Xebia / fmodel-style posts.

* [x] **DM-01 – Define core bounded contexts / modules**

  * Decide module layout in `lib/core/src/lib.rs`, e.g.:

    ```rust
    pub mod patient;
    pub mod encounter;
    pub mod order;        // ServiceRequest, etc.
    pub mod value_objects;
    ```
  * Each module should reflect a clear domain concept, not just “tables”.

* [x] **DM-02 – Introduce value objects & newtypes**

  * Define strongly typed IDs & primitives, e.g.:

    ```rust
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PatientId(pub String);

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ServiceRequestId(pub String);
    ```
  * Use `struct` and `enum` ADTs to encode valid states and invariants (e.g. `OrderStatus`, `Intent`). 

* [x] **DM-03 – Model core entities using ADTs + serde**

  * For each main domain type (Patient, Encounter, ServiceRequest):

    * Use `struct`/`enum` combos to represent allowed shapes.
    * Derive `Serialize`, `Deserialize` via Serde macros:

      ````rust
      use serde::{Serialize, Deserialize};

      #[derive(Clone, Debug, Serialize, Deserialize)]
      pub struct ServiceRequest { /* fields */ }
      ``` :contentReference[oaicite:5]{index=5}  
      ````
  * Only expose invariants via constructors / smart constructors where needed (e.g. `ServiceRequest::new(...)` ensuring status+intent combos are valid).

* [x] **DM-04 – JSON round-trip tests (unit tests)**

  * In `lib/core/src/lib.rs` or `tests/serde_roundtrip.rs`:

    * Add tests that:

      * Build a sample domain struct.
      * Serialize to JSON with `serde_json`.
      * Deserialize back and assert equality.
  * Confirms serde attributes & domain types line up.

### Workspace wiring (Cargo + dirs)

* [x] **WS-01 – Create workspace + lib/ structure**

  * Create `lib/core`, `lib/fake_data`, `lib/test_suite`.
  * In root `Cargo.toml`, add a `[workspace]` with members:
    `["lib/core", "lib/fake_data", "lib/test_suite"]`.

* [x] **WS-02 – Add per-crate Cargo manifests**

  * `lib/core/Cargo.toml`: library crate, no `main`, with:

    ```toml
    [package]
    name = "dfps_core"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    serde = { version = "1", features = ["derive"] }
    serde_json = "1"
    ```

    (Serde derive setup per official docs).
  * `lib/fake_data/Cargo.toml`:

    ```toml
    [package]
    name = "dfps_fake_data"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    core = { path = "../core" }
    fake = { version = "4", features = ["derive"] }
    rand = "0.8"
    ```

    (Using the `fake` crate’s `Dummy`/`Fake` traits).
  * `lib/test_suite/Cargo.toml`:

    ```toml
    [package]
    name = "dfps_test_suite"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    core = { path = "../core" }
    fake_data = { path = "../fake_data" }
    serde_json = "1"
    proptest = "1" # optional, if you want property-based tests
    ```

* [x] **WS-03 – Basic compilation sanity check**

  * Add minimal `src/lib.rs` files in all three crates (even empty `pub mod` stub).
  * Run `cargo check` at workspace root and make sure all three crates compile.

## DONE

