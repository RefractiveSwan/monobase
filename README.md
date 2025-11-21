# 🦢 Refractive Swan

> **Precision Clinical Terminology Mapping** • Built with Rust for speed, accuracy, and scientific rigor

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-GPLv3-blue?style=flat-square)](LICENSE)
[![FOSS](https://img.shields.io/badge/FOSS-100%25-green?style=flat-square)](#)

**Refractive Swan** is a high-performance, open-source clinical terminology mapper that transforms FHIR ServiceRequests into standardized NCIt concepts. Built from the ground up in Rust, it delivers millisecond-latency semantic mapping with transparent confidence scoring and full traceability.

---

## ✨ Why Refractive Swan?

### 🚀 **Blazing Fast**
Engineered in Rust for millisecond-latency vector search and terminology resolution. Handle thousands of codes per second without breaking a sweat.

### 🔬 **Clinically Accurate**  
Validated against NCIt ontologies with transparent matching logic, confidence scoring, and multi-strategy ranking (lexical, vector, and rule-based).

### 🌐 **Open Source**  
Fully GPL-3.0 licensed with FOSS-only dependencies. Inspect the code, contribute to the community, and deploy anywhere.

### 🎯 **Production-Ready**
Domain-driven architecture with clear layer boundaries, comprehensive test coverage, and environment-based configuration for dev/test/prod workflows.

---

## 🏗 Architecture

Refractive Swan follows a **clean, layered architecture** with strict dependency hygiene:

```
┌─────────────────────────────────────────────────┐
│  APP LAYER (Entrypoints & Interfaces)          │
│  • Web UI (Actix + HTMX)                        │
│  • REST API (Axum)                              │
│  • CLI Tools                                    │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  DOMAIN LAYER (Business Logic)                  │
│  • Pipeline Orchestration                       │
│  • Semantic Mapping Engine                      │
│  • FHIR Ingestion                               │
│  • Ontology Reasoning (OBO Graph)               │
│  • Evaluation & Metrics                         │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  PLATFORM LAYER (Cross-Cutting Concerns)        │
│  • Observability & Logging                      │
│  • Configuration Management                     │
│  • Test Utilities                               │
└─────────────────────────────────────────────────┘
```

**Key Principles:**
- **Unidirectional dependencies**: App → Domain → Platform (never reversed)
- **Env-free domain**: All configuration lives in app/platform layers
- **Ports & adapters**: Domain exposes traits, app implements HTTP/CLI adapters
- **Test coverage**: Comprehensive unit, integration, and snapshot tests

For detailed architecture documentation, se [`docs/system-design/base/directory-architecture.md`](docs/system-design/base/directory-architecture.md).

---

## 🚀 Quick Start

### Prerequisites

1. **Rust toolchain** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Helper tools**
   ```bash
   cargo install cargo-make
   cargo install mdbook
   ```

3. **Environment configuration**
   ```bash
   # Copy environment templates
   cd code/data/environment
   cp .env.app.web.api.example .env.app.web.api.dev
   cp .env.app.web.frontend.example .env.app.web.frontend.dev
   ```

### Build & Test

```bash
cd code
cargo build
cargo test --all
```

### Run the Full MVP Stack

```bash
cd code
cargo make mvp-up
```

This starts:
- **API Backend** at `http://127.0.0.1:8080`
- **Web Frontend** at `http://127.0.0.1:8090`
- **Documentation** at `http://127.0.0.1:3000`

Visit **[http://127.0.0.1:8090/](http://127.0.0.1:8090/)** to access the workbench!

---

## 🎨 Web Interface

The Refractive Swan workbench provides an intuitive interface for:

- **📥 Bundle Ingestion**: Paste or upload FHIR Bundles
- **🔍 Mapping Review**: Interactive tables with state filtering
- **📊 Metrics Dashboard**: Real-time mapping quality metrics
- **🧪 Evaluation Reports**: Gold-standard dataset comparisons
- **🔬 NoMatch Explorer**: Triage unmapped codes with detailed reasoning

**Design Philosophy**: Academic professionalism meets modern UX with fluorescent accent colors inspired by qPCR and microscopy aesthetics.

---

## 📦 Project Structure

```
code/
├── lib/
│   ├── app/                    # Application surfaces
│   │   ├── frontend/web/       # Web UI (Actix + Maud + HTMX)
│   │   └── servers/api/        # REST API (Axum)
│   ├── domain/                 # Business logic
│   │   ├── core/               # Domain models & kernel
│   │   ├── ontologies/         # Ingestion, mapping, terminology
│   │   ├── pipeline/           # Workflow orchestration
│   │   └── eval/               # Evaluation harness
│   └── platform/               # Cross-cutting concerns
│       ├── configuration/      # Environment management
│       └── observability/      # Logging & metrics
├── docs/
│   ├── runbook/                # Operational guides
│   ├── kanban/                 # Feature tracking
│   └── system-design/          # Architecture docs
└── data/
    ├── clinical/               # Ontology files (NCIt, MONDO)
    ├── eval/                   # Gold-standard datasets
    └── environment/            # .env templates
```

---

## 🔧 Development Workflow

### Running Components Individually

**Backend API:**
```bash
cd code
cargo run -p dfps_api --bin dfps_api
# Runs on http://127.0.0.1:8080
```

**Web Frontend:**
```bash
cd code
DFPS_API_BASE_URL=http://127.0.0.1:8080 \
DFPS_FRONTEND_LISTEN_ADDR=127.0.0.1:8090 \
cargo run -p dfps_web_frontend --bin dfps_web_frontend
# Runs on http://127.0.0.1:8090
```

**CLI Tools:**
```bash
cd code
cargo run -p dfps_cli -- map-codes --help
cargo run -p dfps_cli -- map-bundles --file data/examples/bundle.json
```

### Useful Commands

```bash
# Build documentation
cargo make docs

# Serve documentation locally
cargo make docs-serve

# Run layer dependency checks
cargo make layers-check

# Full CI validation
cargo make ci
```

---

## 📊 Mapping Pipeline

```
FHIR ServiceRequest
       ↓
  [Ingestion]
       ↓
  Staging Tables
  (stg_servicerequest_flat, stg_sr_code_exploded)
       ↓
  [Mapping Engine]
       ├─→ Lexical Ranker (Jaro-Winkler)
       ├─→ Vector Ranker (Semantic similarity)
       └─→ Rule-Based Re-ranker
       ↓
  MappingResult
  {ncit_id, confidence, state}
       ↓
  [State Classification]
       ├─→ AutoMapped (high confidence)
       ├─→ NeedsReview (medium confidence)
       └─→ NoMatch (no viable candidates)
```

**States:**
- 🟢 **AutoMapped**: High-confidence matches (>85%) ready for production
- 🟡 **NeedsReview**: Medium-confidence (60-85%) requiring human review
- 🔴 **NoMatch**: No viable candidates found; flagged with reason codes

---

## 📚 Documentation

- **[System Design](docs/system-design/)**: Architecture, layer boundaries, dependency seams
- **[Runbooks](docs/runbook/)**: Step-by-step operational guides
- **[Kanban](docs/kanban/)**: Feature tracking and roadmaps
- **[mdBook](http://127.0.0.1:3000)**: Searchable documentation (when running `cargo make docs-serve`)

---

## 🧪 Testing & Evaluation

### Test Coverage

```bash
# Run all tests
cargo test --all

# Run specific package tests
cargo test -p dfps_mapping
cargo test -p dfps_web_frontend

# Run with output
cargo test -- --nocapture
```

### Evaluation Datasets

Refractive Swan includes gold-standard evaluation datasets for regression testing:

```bash
cargo run -p dfps_cli -- eval-run --dataset gold_pet_ct_small --top-k 3
```

**Metrics:**
- Precision, Recall, Coverage
- Top-1 / Top-3 / Top-5 accuracy
- AutoMapped precision (production readiness indicator)

---

## 🤝 Contributing

We welcome contributions! This project is licensed under **GPL-3.0** and uses only FOSS dependencies.

**Development Guidelines:**
- Follow the layer architecture (app → domain → platform)
- Write tests for new features
- Update documentation for architectural changes
- Use `cargo make ci` before submitting PRs

---

## 📜 License

**GPL-3.0** - See [LICENSE](LICENSE) for details.

This project uses only Free and Open Source Software (FOSS) dependencies to ensure maximum transparency and auditability.

---

## 🔗 Links

- **GitHub**: [Refractive Swan](https://github.com/refractive-swan)
- **Documentation**: Run `cargo make docs-serve` and visit http://127.0.0.1:3000
- **Issues**: Report bugs and request features on GitHub

---

<div align="center">

**Built with ❤️ and Rust**

*Precision clinical terminology mapping for the modern healthcare era*

</div>
