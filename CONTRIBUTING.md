# Contributing Guide

Thank you for your interest in contributing! This document explains how to set up a development environment, our expectations for code and documentation, and how to propose changes. It is written for an academic/research audience but applies to all contributors.

- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Security Policy](./SECURITY.md)

---

## 1) Project Overview

This repository is a Rust workspace containing:

- **Backend API (Axum):** `lib/app/servers/api`
- **Frontend UI (Actix + Maud + HTMX/Tailwind):** `lib/app/frontend/web`
- **CLI tools:** `lib/app/frontend/cli` (`map_bundles`, `map_codes`)
- **Domain/Platform crates:** `lib/domain/*`, `lib/platform/*` (ingestion, mapping, pipeline, configuration, observability, test suite)

See the runbook: `docs/runbook/web-quickstart.md`.

---

## 2) Prerequisites

- Rust (pinned via `.rust-toolchain.toml`)
- `cargo` (bundled with Rust)
- Optional: `jq` for JSON inspection
- A modern browser for the frontend

Clone the repository and work from the workspace root (often `code/`):

```bash
git clone <your-fork>
cd code
````

---

## 3) Environment Configuration

The configuration loader automatically reads namespace‑specific dotenv files from `data/environment/` based on `refractive_swan_ENV` (default: `dev`). Do **not** commit real secrets-only the `.example` templates.

1. Copy templates:

   ```bash
   cp data/environment/.env.app.web.api.example        data/environment/.env.app.web.api.dev
   cp data/environment/.env.app.web.frontend.example   data/environment/.env.app.web.frontend.dev
   cp data/environment/.env.platform.test_suite.test.example data/environment/.env.platform.test_suite.test
   # (and others as needed)
   ```
2. Adjust ports, URLs, and log levels as needed.

**Tip:** The loader finds the workspace root by walking up to the directory that contains `Cargo.lock`. Run `cargo build` once from `code/` to generate it, or set `refractive_swan_WORKSPACE_ROOT` explicitly.

---

## 4) Build, Run, and Test

### Build

```bash
cargo build
```

### Run the Backend

```bash
cargo run -p refractive_swan_api --bin refractive_swan_api
# Expected: server listening on 127.0.0.1:8080
```

### Run the Frontend

```bash
refractive_swan_API_BASE_URL=http://127.0.0.1:8080 \
refractive_swan_FRONTEND_LISTEN_ADDR=127.0.0.1:8090 \
cargo run -p refractive_swan_web_frontend --bin refractive_swan_web_frontend
# Visit http://127.0.0.1:8090
```

### CLI Examples

```bash
# Map a Bundle from file or stdin
cargo run -p refractive_swan_cli --bin map_bundles -- docs/samples/sample-bundle.json

# Map codes with optional explanations
cargo run -p refractive_swan_cli --bin map_codes -- --explain --explain-top 5 < codes.ndjson
```

### Tests (unit, integration, property, and e2e)

```bash
# Ensure test profile env is set; the test suite expects it
export refractive_swan_ENV=test
cargo test --workspace
```

Representative locations:

* Unit/property tests: `lib/platform/test_suite/tests/unit`
* Integration tests: `lib/platform/test_suite/tests/integration`
* E2E tests: `lib/platform/test_suite/tests/e2e`

---

## 5) Coding Standards

### Style & Lints

* Format: `cargo fmt --all`
* Lint: `cargo clippy --all-targets -- -D warnings`

### Error Handling & Logging

* Prefer `thiserror` for typed errors and `?` for propagation.
* Use `log` macros (`info!`, `warn!`, `error!`) and initialize logging with `env_logger` where appropriate.
* Surface actionable messages in HTTP errors (avoid leaking secrets or stack traces).

### Workspace Dependencies

Crates reference shared versions via `*.workspace = true`. If you add a dependency commonly used across crates, add it under `[workspace.dependencies]` in the workspace `Cargo.toml`.

### Testing

* For new pipeline features, add:

  * **Unit tests** for pure functions,
  * **Property tests** (when it makes sense),
  * **Integration tests** that exercise crate boundaries,
  * **E2E tests** for API/CLI flows (update fixtures under `lib/platform/test_suite/fixtures` when needed).
* Prefer deterministic seeds for generators in tests.

### Documentation

* Public items should have doc comments (`///`) describing intent, invariants, and examples.
* Update `docs/system-design/*` when you change data flow, schemas, or key assumptions.
* If you change backend routes or payloads, update `docs/api/openapi.yaml` and any curl examples.

---

## 6) Proposing Changes

1. **Discuss**: Open an issue for significant changes (architecture, public API, data schemas).
2. **Branch**: Use descriptive names, e.g., `feature/mapping-thresholds` or `fix/ndjson-parser`.
3. **Commit messages**: Use imperative mood, reference issues when relevant.
   Example: `fix(api): normalize uppercase intent values`
4. **Pull Request**:

   * Explain motivation, approach, and trade‑offs.
   * Include tests and docs updates.
   * Note any migration steps.
5. **Review**:

   * Be responsive to review comments.
   * Prefer small, incremental PRs over large, all‑at‑once changes.

We use squash‑merge or rebase‑merge to keep history readable.

---

## 7) Authorship, Citation, and Academic Norms

* Credit contributors in `CHANGELOG.md`/release notes and, where appropriate, in scholarly outputs.
* If your contribution is substantial and leads to a publication, follow community norms for authorship (e.g., ICMJE/discipline‑specific guidance).
* Cite prior work and clearly label adapted code/data with upstream links.
* Add or update `CITATION.cff` if relevant (optional but encouraged).

---

## 8) Data & Privacy

* Do **not** commit real PHI/PII or credentials. Use synthetic or de‑identified data.
* Keep real `.env.*.<profile>` files untracked; only commit `.example` templates.
* If you encounter a potential security/privacy issue, follow the [Security Policy](./SECURITY.md).

---

## 9) Licensing

Unless stated otherwise in the workspace `Cargo.toml` or `LICENSE`, contributions are accepted under the project’s license. Confirm before contributing if you require a different license grant.

---

## 10) Contact

* General questions: **<JDea280@wgu.edu>**
* Conduct concerns: **<JDea280@wgu.edu>**
* Security reports: **<JDea280@wgu.edu>**

*Thank you for helping us build reproducible, responsible research software.*