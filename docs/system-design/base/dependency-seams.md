# Dependency Seams & DTO Ownership

**Goal:** make it explicit which crates own the data transfer objects (DTOs) and
ports that cross the application ↔ domain ↔ platform layers. See also
`docs/system-design/base/directory-architecture.md` for the high-level layout.

## Ownership map

| Flow / DTO category | Owning crates | Notes |
| --- | --- | --- |
| **FHIR / staging DTOs** | `refractive_swan_core` (`interop::fhir`, `interop::staging`), `refractive_swan_ingestion` (validation/report types) | App crates (CLI/API) treat these as *read-only* contracts. All env/config decisions stay outside the domain crates. |
| **Mapping DTOs** | `refractive_swan_core::semantics::mapping`, `refractive_swan_mapping` (ranker summaries) | Domain exposes pure mapping results; app/platform crates inject compliance policies, vector adapters, and terminology clients. |
| **Pipeline output** | `refractive_swan_pipeline` (re-exported via `refractive_swan_contracts`) | Primary cross-surface payload feeding CLI/API/warehouse. Remains env-free so platform/app layers can reuse it wholesale. |
| **Analytics / datamart DTOs** | `refractive_swan_datamart` (dim/fact models), `refractive_swan_contracts` (shared analytics responses), `refractive_swan_web_dto` / `refractive_swan_cli_dto` veneers | Warehouse loaders persist `PipelineOutput` via `refractive_swan_datamart`; app surfaces re-use the contracts via DTO veneers. |
| **Eval DTOs** | `refractive_swan_eval` (`EvalSummary`, `DatasetManifest`), `refractive_swan_contracts` (re-exports + `EvalRunResponse`) | CLI/API surfaces deserialize directly into these types for reporting and CI gating. |
| **UI view models** | `refractive_swan_web_frontend` (Maud view models + reqwest DTO wrappers) | Frontend consumes `refractive_swan_contracts` payloads via `client.rs`; HTMX views remain adapter-only (no domain imports). |
| **Config / env seams** | `refractive_swan_configuration` (app/platform adapters) | Domain crates accept typed configs (e.g., `TerminologyClientConfig`) constructed in the app/platform layer (see Terminology + external validator seams below). |

## Port & adapter highlights

### Terminology clients (domain ↔ platform)

* Domain: `lib/domain/ports/terminology` (re-exporting `refractive_swan_terminology::client::{TerminologyClient, TerminologyClientConfig}`) exposes trait + config struct only.
* Platform/App: load env via `refractive_swan_configuration` (or CLI flags), build a config,
  and pass it into `HttpTerminologyClient::from_config`. Domain code never reads
  env variables directly.

### External FHIR validator

* Domain: `lib/domain/ports/validation` (source: `refractive_swan_ingestion::validation::{ExternalValidator, ExternalValidationContext}`) – trait + context only.
* Platform/App: implement `ExternalValidator` (HTTP, mock, noop) and supply it
  through the context so ingestion stays deterministic and env-free.

### Vector store

* Domain: `lib/domain/ports/data/data-store/vector` (`refractive_swan_vector_port`) exposes `VectorStore`, `VectorStoreConfig`, `VectorUsageSnapshot`, and `VectorPipelineContext`.
* Platform: `refractive_swan_vector_store` implements adapters for Qdrant/PGVector/Mock and
  exposes `config_from_env` so CLI/API crates can inject a concrete store without
  leaking env parsing into domain code.

### Compliance policies

* Domain/App code call `refractive_swan_compliance::Policy` (pure data) + helpers.
* Platform/App load policy from env (or files) via `refractive_swan_compliance::ComplianceConfig`
  and inject it into CLI/API orchestrators.

### Datamart sink (analytics adapter)

* Domain: `lib/domain/ports/data/data-plane/datamart` defines the sink trait consumed by `refractive_swan_pipeline` when emitting analytics rows.
* Platform/App: `refractive_swan_datamart` implements the sink (SQLite today) and is injected into `refractive_swan_api`, CLI loaders, and future mesh nodes.

### DTO veneers

* Domain contracts live in `refractive_swan_contracts`, but surfaces consume them via `lib/dto/*` veneers.
* `lib/dto/web` (`refractive_swan_web_dto`) serves the Axum API + Actix frontend.
* `lib/dto/cli` (`refractive_swan_cli_dto`) powers NDJSON/CLI payloads.
* `lib/dto/mesh` (`refractive_swan_mesh_dto`) exposes the mesh/node control-plane DTO surface.

### Node data plane (runtime orchestrator)

* `refractive_swan_mesh_node::NodeDataPlane` now lives under `lib/platform/mesh/node`.
* App servers (`refractive_swan_api`) and future mesh runtimes construct this struct with
  their injected ports (pipeline, datamart sink, dataset store, vector context)
  and then expose HTTP/gRPC handlers as thin adapters.

### Eval dataset store

* Domain: `refractive_swan_eval::DatasetStore` abstracts dataset/case access. The default
  `FileDatasetStore` is an implementation that reads from the workspace data root.
* App: CLI, API, and web frontends hold `Arc<dyn DatasetStore>` values so tests
  and future remote stores can be swapped in without touching domain logic.

### CLI pipeline command

* Domain: `refractive_swan_pipeline::{PipelinePort, DefaultPipeline}` exposes an inbound port
  (`map_bundle`) that orchestrates ingestion + mapping without touching IO/env.
* App: `refractive_swan_cli` bins construct `DefaultPipeline` and treat each bin as a command
  adapter (parse args/NDJSON, call the port, emit DTOs).

## How to use this doc

* When defining a new DTO or cross-layer payload, decide which crate owns it and
  update the table above.
* If a domain crate needs config/env, add a trait or config struct and have the
  caller supply values (do **not** read env from domain).
* Update `docs/system-design/base/directory-architecture.md` if a new bucket or
  adapter category is introduced.
