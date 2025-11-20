# Dependency Seams & DTO Ownership

**Goal:** make it explicit which crates own the data transfer objects (DTOs) and
ports that cross the application ↔ domain ↔ platform layers. See also
`docs/system-design/base/directory-architecture.md` for the high-level layout.

## Ownership map

| Flow / DTO category | Owning crates | Notes |
| --- | --- | --- |
| **FHIR / staging DTOs** | `dfps_core` (`interop::fhir`, `interop::staging`), `dfps_ingestion` (validation/report types) | App crates (CLI/API) treat these as *read-only* contracts. All env/config decisions stay outside the domain crates. |
| **Mapping DTOs** | `dfps_core::semantics::mapping`, `dfps_mapping` (ranker summaries) | Domain exposes pure mapping results; app/platform crates inject compliance policies, vector adapters, and terminology clients. |
| **Pipeline output** | `dfps_pipeline` (re-exported via `dfps_contracts`) | Primary cross-surface payload feeding CLI/API/warehouse. Remains env-free so platform/app layers can reuse it wholesale. |
| **Analytics / datamart DTOs** | `dfps_datamart` (dim/fact models), `dfps_api` (HTTP wrappers), `dfps_contracts` (shared analytics responses) | Warehouse loaders persist `PipelineOutput` via `dfps_datamart`; API/CLI surfaces re-use the contracts without introducing bespoke DTOs. |
| **Eval DTOs** | `dfps_eval` (`EvalSummary`, `DatasetManifest`), `dfps_contracts` (re-exports + `EvalRunResponse`) | CLI/API surfaces deserialize directly into these types for reporting and CI gating. |
| **UI view models** | `dfps_web_frontend` (Maud view models + reqwest DTO wrappers) | Frontend consumes `dfps_contracts` payloads via `client.rs`; HTMX views remain adapter-only (no domain imports). |
| **Config / env seams** | `dfps_configuration` (app/platform adapters) | Domain crates accept typed configs (e.g., `TerminologyClientConfig`) constructed in the app/platform layer (see Terminology + external validator seams below). |

## Port & adapter highlights

### Terminology clients (domain ↔ platform)

* Domain: `dfps_terminology::client::{TerminologyClient, TerminologyClientConfig}`
  exposes trait + config struct only.
* Platform/App: load env via `dfps_configuration` (or CLI flags), build a config,
  and pass it into `HttpTerminologyClient::from_config`. Domain code never reads
  env variables directly.

### External FHIR validator

* Domain: `dfps_ingestion::validation::{ExternalValidator, ExternalValidationContext}`
  – trait + context only.
* Platform/App: implement `ExternalValidator` (HTTP, mock, noop) and supply it
  through the context so ingestion stays deterministic and env-free.

### Vector store

* Domain: `dfps_mapping` and `dfps_pipeline` consume the `dfps_vector_port` traits
  (`VectorStore`, `VectorStoreConfig`, `VectorUsageSnapshot`) and
  `VectorPipelineContext` (constructed in app/platform layers).
* Platform: `dfps_vector_store` implements adapters for Qdrant/PGVector/Mock and
  exposes `config_from_env` so CLI/API crates can inject a concrete store without
  leaking env parsing into domain code.

### Compliance policies

* Domain/App code call `dfps_compliance::Policy` (pure data) + helpers.
* Platform/App load policy from env (or files) via `dfps_compliance::ComplianceConfig`
  and inject it into CLI/API orchestrators.

### Datamart sink (analytics adapter)

* Domain: `dfps_pipeline::PipelinePort` emits `PipelineOutput`/mapping rows without
  caring about storage.
* App: `dfps_datamart::DatamartSink` is the outbound port that persists/query
  analytics. `SqliteDatamart` lives in the `dfps_datamart` crate and is injected
  into `dfps_api` and CLI loaders, keeping HTTP/CLI code free of SQL details.

### Eval dataset store

* Domain: `dfps_eval::DatasetStore` abstracts dataset/case access. The default
  `FileDatasetStore` is an implementation that reads from the workspace data root.
* App: CLI, API, and web frontends hold `Arc<dyn DatasetStore>` values so tests
  and future remote stores can be swapped in without touching domain logic.

### CLI pipeline command

* Domain: `dfps_pipeline::{PipelinePort, DefaultPipeline}` exposes an inbound port
  (`map_bundle`) that orchestrates ingestion + mapping without touching IO/env.
* App: `dfps_cli` bins construct `DefaultPipeline` and treat each bin as a command
  adapter (parse args/NDJSON, call the port, emit DTOs).

## How to use this doc

* When defining a new DTO or cross-layer payload, decide which crate owns it and
  update the table above.
* If a domain crate needs config/env, add a trait or config struct and have the
  caller supply values (do **not** read env from domain).
* Update `docs/system-design/base/directory-architecture.md` if a new bucket or
  adapter category is introduced.
