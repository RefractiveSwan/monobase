# dfps_terminology

Domain-level terminology layer that normalizes staging codes, classifies their
license/source metadata, and mediates access to external terminology services.
See:

- `docs/system-design/clinical/ncit/architecture.md`
- `docs/system-design/clinical/fhir/concepts/terminology-layer.md`
- `docs/reference-terminology/semantic-relationships.yaml`

## Modules

- `codesystem` – canonical registry metadata (`LicenseTier`, `SourceKind`,
  `CodeSystemMeta`) for SNOMED, LOINC, NCIt, etc.
- `registry` – lookup helpers (`lookup_codesystem`, `list_code_systems`,
  `is_licensed`, `is_open`) backed by the embedded registry.
- `bridge` – `EnrichedCode`, `CodeKind`, and normalization helpers that take a
  `StgSrCodeExploded` row and attach canonical system URLs, license tiers, and
  observability labels for mapping/compliance.
- `valueset` – metadata and lookups for curated ValueSets used across ingestion
  and compliance policies.
- `client` – traits (`TerminologyClient`) plus mock/HTTP client adapters; the
  optional `TerminologyClientConfig::from_env` convenience reads
  `DFPS_TERMINOLOGY_*` variables for CLI/tests, but platform layers should inject
  typed configs instead of letting domain crates read the environment.
- `obo_graph` – embedded NCIt/MONDO graph parser + cached reasoning helpers
  (ancestors, descendants, synonyms, related concepts) used by the OBO bridge.
- `obo` – high-level bridge that exposes versioned graph contexts, merging the
  cached NCIt/MONDO reasoning outputs for mapping/compliance consumers.

## License & source classification

`LicenseTier` (`licensed`, `open`, `internal_only`) and `SourceKind` (`fhir`,
`umls`, `obo_foundry`, `local`) flow through `EnrichedCode` and into
`dfps_mapping::MappingSummary`. Use the exposed `canonicalize_system_url` helper
to normalize system URLs before looking up metadata, and rely on `CodeKind` to
emit consistent observability buckets (`known_licensed_system`, `obo_backed`,
etc.) across ingestion and mapping.

## Term clients & env seams

`TerminologyClientConfig::from_env` loads a base URL/API key/timeout/mode from
`DFPS_TERMINOLOGY_*` environment variables for legacy CLIs. Production surfaces
should construct the config via their platform configuration layer and pass it
into `HttpTerminologyClient::from_config`, keeping HTTP/env responsibilities out
of domain crates.

## Related crates

- The in-crate `obo_graph` module supplies the embedded NCIt/MONDO mini graphs
  used by the `obo` bridge.
- `lib/domain/ontologies/mapping` (`dfps_mapping`) consumes `EnrichedCode` to
  produce license-aware mapping summaries and compliance signals.
