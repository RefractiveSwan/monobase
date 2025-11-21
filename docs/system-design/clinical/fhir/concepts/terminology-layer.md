# Terminology Layer

refractive_swan keeps an explicit terminology layer between FHIR staging and NCIt mapping. It tracks the provenance and license tier for every code system we touch and records which OBO Foundry ontologies back our open vocabularies.

## Components

- `refractive_swan_terminology::codesystem`
  - Registry of FHIR CodeSystems with license tier (`licensed`, `open`, `internal_only`) and source kind (`fhir`, `umls`, `obo_foundry`, `local`).
- `refractive_swan_terminology::obo`
  - Metadata for NCIt OBO, MONDO, and other OBO Foundry ontologies we rely on, including bridges into the in-crate `obo_graph` module when the `obo-graph` feature flag is enabled.
- `refractive_swan_terminology::valueset`
  - ValueSet descriptors that group code systems for refractive_swan workflows.
- `refractive_swan_terminology::bridge::EnrichedCode`
  - Decorates `StgSrCodeExploded` rows with canonical system URLs, license/source metadata, and `CodeKind` classification (licensed, open, OBO, unknown, missing).

## How it fits

1. FHIR staging (`stg_sr_code_exploded`) flows into the terminology bridge.
2. The bridge normalises URLs, records license/source metadata, and classifies each code.
3. Mapping uses this metadata to:
   - short-circuit missing/unknown systems (`reason = "missing_system_or_code"` or `"unknown_code_system"`), and
   - surface license context on every `MappingResult` for downstream policy or observability.
4. OBO-backed concepts (e.g., NCIt OBO, MONDO) are always treated as open.

## Compliance policy layer

- `lib/platform/compliance` (`refractive_swan_compliance`) owns `ComplianceMode` (`internal`, `partner`, `open_source`) and `Policy` objects that describe which `LicenseTier` values are allowed for `ingest`, `map`, and `export` actions.
- Defaults: `internal` allows `licensed`/`open`/`internal_only`; `partner` allows `licensed`/`open`; `open_source` allows `open` only. All actions are enabled unless overridden.
- Configuration: `refractive_swan_COMPLIANCE_MODE` (default `internal`) and optional `refractive_swan_COMPLIANCE_POLICY_PATH` (JSON/YAML) to override actions or tier allowances per mode.
- Mapping/export surfaces consume these policies in epic 020 while keeping ranking/vector behavior unchanged (see `docs/kanban/feature/mvp/020-license-compliance-layer.md`).

> When updating the terminology layer, ensure the registries, helper enums, and bridge logic stay consistent with the kanban (TERM-01 � TERM-07) and that `MappingResult` metadata stays in sync with docs.

## Graph context & reasoning

- Terminology can optionally load graph-backed views (NCIt + companion slices) via `refractive_swan_terminology::obo_graph::load_ontology_graph` when `obo-graph` is enabled.
- `CachedOntologyGraph` memoizes query results for `ancestors`, `descendants`, `synonym_set`, and hop-bounded `related_concepts`, keyed by normalized NCIt IDs and graph version.
- Mapping consumers should treat the graph surface as an enrichment layer: synonym expansion and graph-distance-aware tweaks must keep deterministic ordering when the feature is disabled.
- Graph version metadata (`graph_versions`) surfaces provenance (e.g., `ncit-mini-0.1.1`) for cache invalidation and observability tags.
## License-aware mapping outputs

- `refractive_swan_core::mapping::MappingResult` now carries `license_tier` and `source_kind` strings for every emitted row.
- `refractive_swan_mapping::map_staging_codes` and `map_staging_codes_with_summary` attach those labels using `EnrichedCode::license_label` / `source_label`.
- `MappingResult.reason` explicitly reports `"missing_system_or_code"`, `"unknown_code_system"`, and `"license_blocked"` (when compliance mode forbids mapping the license tier) when the terminology layer short-circuits a mapping attempt.

## Observability hooks

- `refractive_swan_mapping::MappingSummary` tallies counts by `CodeKind` and license tier (`unknown` bucket included).  Use `map_staging_codes_with_summary` to retrieve `(results, dims, summary)` without re-implementing tally logic.
- `map_codes` prints the summary to stderr so local runs immediately reveal how many codes were missing identifiers, from licensed systems, or unknown systems.
- `map_bundles` (via `refractive_swan_pipeline`) can combine `PipelineMetrics` and `MappingSummary` to log observability counters alongside ingestion validation events.
