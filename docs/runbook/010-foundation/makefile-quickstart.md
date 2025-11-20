# Makefile Quickstart - DFPS

This quickstart lists the most useful `cargo make` commands organized by workflow area and complexity level.

**Complexity Levels**:
- 🟢 **Beginner**: Single-command flows, just works
- 🟡 **Intermediate**: Granular control, some configuration needed
- 🔴 **Advanced**: Parameterized tasks, expert usage

---

## Foundation & Setup

```bash
# 🟢 Environment
cargo make setup-env        # Interactive wizard to create .env files
cargo make validate-env     # Check all required .env files exist

# 🟡 Tooling
cargo make setup-tooling    # Install cargo-make, mdbook, cargo-watch

# 🔴 Custom templates
NAMESPACE=app.new.service cargo make env-template
```

---

## Core Development

```bash
# 🟢 Basic workflows
cargo make build            # Build workspace
cargo make check            # Type-check workspace
cargo make fmt              # Format all crates
cargo make clippy           # Lint with clippy (deny warnings)
cargo make test             # Run all tests
cargo make clean            # Clean target dir

# 🟡 Advanced
cargo make ci               # Complete CI: fmt + clippy + contracts + test
cargo make test-libs        # Test only library crates
cargo make test-suite       # Test only integration suite
```

---

## MVP Workflows

### 🟢 Beginner - Get Started Fast

```bash
cargo make mvp-up           # Start web frontend + API backend (parallel)
cargo make mvp-health       # Check health of running services
cargo make mvp-api          # Start only API backend
cargo make mvp-web          # Start only web frontend
```

### 🟡 Intermediate - Full Stack

```bash
cargo make mvp-full         # Start docs + API + web (complete stack)
cargo make mvp-validate     # Complete validation pipeline
cargo make mvp-e2e          # End-to-end: validate → build → ready
cargo make mvp-quick-check  # Quick: fmt + check (faster iteration)
```

### 🔴 Advanced - Complex Pipelines

```bash
cargo make mvp-pipeline-full  # eval → warehouse → start services
cargo make mvp-ci-full        # Complete CI with eval checks
cargo make mvp-dev-loop       # Watch mode: auto-restart on changes
cargo make mvp-pre-commit     # Pre-commit checks simulation
```

---

## FHIR Validation

```bash
# 🟢 Basic validation
INPUT=path/to/bundle.json cargo make fhir-validate

# 🟡 Testing
cargo make fhir-test          # Run all FHIR validation tests
cargo make fhir-profile-update  # Test after editing profiles

# 🔴 External validator
DFPS_FHIR_VALIDATOR_BASE_URL=http://validator:8080 \
DFPS_FHIR_VALIDATOR_PROFILE=http://example.com/profile \
INPUT=bundle.json cargo make fhir-validate-external
```

---

## Mapping & Evaluation

```bash
# 🟢 Quick evaluation
cargo make eval-quick         # Small dataset (pet_ct_small)

# 🟡 Comprehensive evaluation
cargo make eval-full          # All datasets (bronze/silver/gold)
cargo make eval-report        # Generate report with baseline comparison

# 🔴 Advanced evaluation
cargo make eval-ci            # CI with thresholds + deterministic checks
INPUT=custom.ndjson cargo make eval-custom
DATASET=my_dataset cargo make eval-custom-dataset
cargo make eval-advanced      # With bootstrap confidence intervals
```

---

## Warehouse & BI

```bash
# 🟢 Setup
cargo make warehouse-setup    # Run migrations

# 🟡 Loading data
INPUT=pipeline.ndjson cargo make warehouse-load
cargo make warehouse-test     # Run integration tests

# 🔴 Advanced flows
INPUT=bundles.ndjson cargo make warehouse-load-bundles  # Map + load
cargo make warehouse-inspect  # Show inspection commands
```

---

## Documentation

```bash
# 🟢 Basic
cargo make docs               # Build mdBook
cargo make docs-serve         # Live-reload server (opens browser)

# 🟡 Maintenance
cargo make docs-sync          # Sync runbooks & kanban into book
cargo make graphviz-render    # Render .dot diagrams to .svg
cargo make graphviz-clean     # Remove generated .svg files
cargo make graphviz-watch     # Watch and auto-render diagrams
```

---

## CLI Tools

```bash
# 🟡 Data processing
INPUT=bundles.ndjson cargo make map-bundles
INPUT=staging_codes.ndjson cargo make map-codes
```

---

## Notes

- **Active profile**: Defaults to `dev`. Override with `DFPS_ENV=test cargo make <task>`
- **Prerequisites**:
  - Install `cargo-make`: `cargo install cargo-make`
  - Install `mdbook`: `cargo install mdbook` (for docs tasks)
  - Install `cargo-watch`: `cargo install cargo-watch` (for watch tasks)
- **Environment files**: See [env-quickstart.md](env-quickstart.md) for setup
- **Runbook alignment**: Each category aligns with `docs/runbook/` sections
- **Help**: Run `cargo make --list-all-steps` to see all available tasks

---

## Quick Start for New Contributors

1. **Setup environment**:
   ```bash
   cargo make setup-env
   cargo make validate-env
   ```

2. **Validate setup**:
   ```bash
   cargo make mvp-validate
   ```

3. **Start MVP**:
   ```bash
   cargo make mvp-up
   ```

4. **Check health**:
   ```bash
   cargo make mvp-health
   ```

Visit `http://127.0.0.1:8090` for the web UI and `http://127.0.0.1:8080/health` for API health.

