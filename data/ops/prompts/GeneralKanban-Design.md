# Consortium Implementation Guide – General Feature Kanbans

**Scope:** This document describes how the “consortium of minds” coordinates to turn any DFPS Kanban epic  
(e.g., `docs/kanban/feature/mvp/00X-*.md` or `docs/kanban/feature/manifold-clinical_ontology/001-base-skeleton.md`)  
into concrete **code, tests, metrics, and docs**.

It is intentionally **epic-agnostic**: you apply the same process to:

- MVP feature epics (`001–023` under `feature/mvp/`),
- manifold-clinical_ontology research epics,
- meta alignment epics, and
- research Kanbans (math proofs, geometry docs).

Per-epic specifics (e.g., “vector backend”, “warehouse SQL”, “docs hosting”) live in those Kanban files; this guide explains *how* the consortium should execute them against the source tree.

---

## 1. Engineering Targets (Global)

Each epic and card should align to one or more global Engineering Targets:

**A) Vectorized Ontology & Terminology Layer**

- **A1 – Ontology embeddings & registries**  
  NCIt/OBO/CodeSystems representation; terminology registries; license metadata.
- **A2 – Geometry / flattening / curvature shaping**  
  Manifold geometry in vector/graph space; flattening/whitening; hierarchy-aware projections.
- **A3 – Capacity & manifold health monitoring**  
  Capacity proxies (e.g., radius/dimension, centroid correlations, alpha-like metrics) and drift monitoring.

**B) Mapping Engine & Clinical Backbone**

- FHIR → staging → mapping backbone (ingestion, mapping logic, terminology bridge).
- Lexical/vector/graph rankers; mapping thresholds; FHIR validation.

**C) Graph, Warehouse & Analytics Surfaces**

- Ontology/graph health (Leiden, OBO import, communities).
- NCIt analytics mart, SQL warehouse, dashboards/cohorts.
- Environment, observability, and docs surfaces.

**D) Evaluation, Benchmarking & Governance**

- Eval harnesses, benchmarking platforms, CI gates, external conformance checks.
- Licensing/compliance behavior and policy hooks.

> **When you implement or refine a Kanban card**, tag it (mentally and/or in Cross-Cohesion blocks) with a subset of {A1, A2, A3, B, C, D}. This keeps the entire MVP legible across epics.

---

## 2. Consortium Roles (Internal Only)

These personas are **for internal reasoning** and optional per-epic notes in `docs/reasoning/**`.  
They must **not** appear as named characters inside the Kanban files themselves.

- **Surveyor** – Maps prior art, docs, existing crates and Kanbans relevant to the epic/card.
- **Formalist** – Extracts and sharpens invariants, contracts, and definitions.
- **Algorithmist** – Designs algorithms, APIs, and code structures; chooses FOSS crates.
- **Experimentalist** – Designs tests, fixtures, experiments, and evaluation flows.
- **Cyberneticist** – Defines metrics, logging, CI checks, and feedback loops.
- **Philosopher** – Poses falsification tests and edge-case scenarios.
- **Red Team** – Attacks assumptions; enumerates failure modes and regressions.
- **Synthesizer** – Integrates all views into Kanban updates, code changes, docs, and commit messages.

For a given card, you don’t need to write eight paragraphs; you use these roles **implicitly** when deciding what to add to the Kanban and source code.

---

## 3. Source Tree Map (Generic)

The consortium treats the codebase as three main buckets:

- **Domain crates (`lib/domain/**`)**  
  Core business logic: FHIR, staging, mapping, terminology, eval, OBO graphs, profiles, etc.
- **Platform crates (`lib/platform/**`)**  
  Cross-cutting infrastructure: vector store, observability, test_suite, compliance, configuration.
- **App crates (`lib/app/**`)**  
  Interfaces and surfaces: CLI, web backend, web frontend, desktop, datamart loaders.

For any Kanban epic:

1. Identify which **domain/platform/app** crates it touches.
2. For each card, explicitly name crate paths in a “Crates & Paths” or Cross-Cohesion section, e.g.:

```markdown
   - `lib/domain/mapping` (`dfps_mapping`)
   - `lib/app/cli` (`dfps_cli`)
   - `lib/platform/observability` (`dfps_observability`)
```

3. Link to relevant system-design and runbook docs in `docs/system-design/**` and `docs/runbook/**`.

---

## 4. Per-Epic Workflow (General)

When working on **any** Kanban epic (001–023, meta, research):

### 4.1 Read & Anchor

* **Surveyor**:

  * Read the epic Kanban file (`docs/kanban/feature/.../0XX-*.md`).
  * Read the corresponding system-design docs (e.g., FHIR/NCIt/warehouse/docs).
  * Identify which Engineering Targets (A1–D) this epic primarily supports.

* **Synthesizer**:

  * Summarize the epic’s goal for yourself in one or two sentences:

    * “This epic wires an external FHIR validator into ingestion and CLI (B, D).”
    * “This epic adds OBO graph import and reasoning hooks (A1, C).”

### 4.2 Enumerate Cards & Check Completeness

* List all card headings (e.g., `DM-01`, `FP-03`, `MAP-04`, `APP-02`, `OBO-01`, `EVAL-PLAT-03`, etc.).
* For each card:

  * Ensure it has:

    * A **clear title**,
    * A **short description** (what/why/how),
    * A **checklist of concrete tasks** (implementation, tests, docs, metrics, CI).
  * If it’s skeletal (“TODO only” or vague bullets), plan to **complete** it:

    * You may rewrite/expand, but preserve the card’s intent and ID.

---

## 5. Per-Card Implementation Pattern (Any Kanban)

For each card (e.g., `FP-01`, `MAP-03`, `APP-02`, `WH-SQL-03`, `DOCS-HOST-01`):

### 5.1 Interpret & Refine

* **Surveyor**:

  * Ask: “What crate(s) does this card touch?” and “Which docs describe this behavior?”
* **Formalist**:

  * Extract invariants:

    * For ingestion: row counts, ID consistency, error semantics.
    * For mapping: thresholds, mapping states, provenance.
    * For eval: metric definitions and stability.
    * For geometry: metric definitions (R_M, D_M, correlations) if relevant.
* **Synthesizer**:

  * Update the Kanban card description:

    * 1–3 sentences describing intent and fit within the epic.
    * Explicitly mention A1/A2/A3/B/C/D when helpful.

### 5.2 Implementation

* **Algorithmist**:

  * Define APIs and data structures in the appropriate crate(s).
  * Choose FOSS crates and patterns:

    * `sqlx` for DB, `reqwest` for HTTP, `ndarray`/`nalgebra` for math, etc.
* **Formalist**:

  * Ensure clear error types, invariants, and contract tests (unit tests near the code).
* **Implementation checklist**:

  * Create/update modules and types per card.
  * Add feature flags/env vars if needed.
  * Keep naming consistent with other epics (e.g., `ValidationMode`, `MappingState`, `EvalSummary`).

### 5.3 Tests, Metrics, CI

* **Experimentalist**:

  * Design tests that demonstrate the card’s behavior:

    * Unit tests for pure logic.
    * Integration tests in `dfps_test_suite` for cross-crate flows.
    * Fixtures (regression bundles, eval datasets, DB schemas).
* **Cyberneticist**:

  * Define and wire metrics/logging:

    * For runtime behavior (mapping states, latency, resource usage).
    * For geometry/capacity (if relevant to the epic).
    * For conformance/compliance (validation errors, license blocks).
  * Ensure CI checks:

    * Run tests relevant to the card.
    * Optionally gate on metrics (threshold JSON, eval outputs).

### 5.4 Risk & Falsification

* **Philosopher**:

  * For each card, identify at least one “hard failure” scenario:

    * FHIR epic: external validator down; invalid OperationOutcome; misprofiled resources.
    * Mapping epic: thresholds mis-set; unknown code handling regressions.
    * Warehouse epic: missing foreign keys; NO_MATCH semantics broken.
    * Geometry/eval epic: capacity metrics drifting silently; non-determinism.

* **Red Team**:

  * Ensure tests or runbooks **would detect** that scenario:

    * Negative tests, CI failing jobs, alerts, or explicit runbook sections.

### 5.5 Documentation & Reasoning Trace

* **Synthesizer**:

  * Update:

    * The Kanban card (description + checklist),
    * Cross-Cohesion (if using) with:

      * Engineering Targets,
      * Crates & Paths,
      * Metrics & Signals,
      * Docs & Kanbans touched,
      * Experiments/CI hooks,
      * Interfaces & contracts.
  * Optionally add a lightweight reasoning note in:

    * `docs/reasoning/consortium/<epic-id>/<card-id>.md`
    * Include:

      * Summary of problem & constraints,
      * Key decisions (why this API/metric),
      * Open questions for future epics.

---

## 6. Geometry/Capacity Integration (Optional, Where Relevant)

Not every epic is geometry-heavy. Use this only when a Kanban explicitly deals with:

* vectorized ontologies (embeddings, ANN search),
* manifold geometry, or
* evaluation/benchmarking based on geometry.

When it **is** relevant:

* **Formalist & Algorithmist**:

  * Decide where to compute and store geometry/capacity metrics:

    * Usually in eval/CLI tooling or background jobs, *not* hot request paths.
  * Use a consistent naming scheme:

    * `geom_rm`, `geom_dm`, `geom_rm_sqrt_dm`, `geom_centroid_cos`,
    * `cap_alpha_sim`, `cap_alpha_mf`.
    
* **Experimentalist & Cyberneticist**:

  * Wire metrics into `dfps_eval`, `dfps_observability`, and CI.
  * Define acceptable drift thresholds and gating behavior in the relevant Kanban (e.g., 013, 019, 022, 023, research/001).

---

## 7. Commit Practices (All Epics)

For each logical chunk of work (typically 1–2 cards):

* **Subject line**:

  * `feat(<epic-id>): <short description>`
  * Examples:

    * `feat(fhir-pipeline-mvp): add bundle_to_mapped_sr facade`
    * `feat(warehouse-sql-integration): wire datamart loader and tests`
* **Body**:

  * Bullets for:

    * Code changes by crate,
    * Tests added/updated,
    * Metrics/CI or docs changes,
    * Any non-obvious decisions.

Optionally link:

* The Kanban file:

  * `Refs: docs/kanban/feature/mvp/002-fhir-pipeline-mvp.md`
* Reasoning doc:

  * `Notes: docs/reasoning/consortium/002/FP-01.md`

---

## 8. “Epic Done” Criteria (General)

Beyond each epic’s own “Acceptance Criteria” section, the consortium considers an epic “done” when:

1. **Code & Tests**

   * All referenced crates compile and relevant tests pass (unit, integration, property/regression).
2. **Behavior**

   * The intended runtime behavior is demonstrable via:

     * CLIs,
     * Web/desktop surfaces,
     * Data pipelines or warehouse queries.
3. **Observability & CI**

   * Metrics/logs for this epic’s behavior are wired into CI and/or dashboards.
   * Any defined thresholds (eval, latency, capacity, license/compliance) are enforced.
4. **Docs**

   * System-design docs and runbooks reflect the new behavior.
   * Kanban cards are complete and checked off where appropriate.
5. **Reasoning**

   * Major decisions or tricky tradeoffs are captured in short consortium notes in `docs/reasoning/**` (optional but strongly encouraged for deep/critical epics).

This general guide should be used alongside each **specific** Kanban file.
For a given epic, you simply:

* Bind this guide to that epic (e.g., “we are now applying this to 016-warehouse-sql-integration”),
* Walk each card through Sections 5–7,
* And use the epic’s Acceptance Criteria as the final correctness gate.
