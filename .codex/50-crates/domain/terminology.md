# Crate: lib/domain/terminology — `refractive_swan_terminology`

**Path:** `code/lib/domain/terminology`  
**Depends on:** `refractive_swan_core`, `serde`.

## Responsibilities
- Normalize and classify **code systems**; provide lightweight **registry** and **OBO** metadata.
- Bridge staging codes to enriched context (license tier, source kind, canonical system).
- Supply **value set** metadata for groupings used elsewhere.

## Modules & key types
- `registry.rs`
  - `list_code_systems()`, `lookup_codesystem(url)`, `is_licensed(url)`, `is_open(url)`.
  - Includes CPT, SNOMED CT, LOINC, and NCIt (OBO) entries.
- `codesystem.rs`
  - `CodeSystemMeta` + enums `LicenseTier { licensed | open | internal_only }`, `SourceKind { fhir | umls | obo_foundry | local }`.
- `bridge.rs`
  - `EnrichedCode::from_staging(StgSrCodeExploded)` → attaches `codesystem`, `license_tier`, `source_kind`, and a **canonical system URL**.
  - `CodeKind` classification: `KnownLicensedSystem | KnownOpenSystem | OboBacked | UnknownSystem | MissingSystemOrCode`.
  - Internal canonicalizer maps OIDs to URLs (e.g., SNOMED, LOINC).
- `obo.rs`
  - Minimal ontology records (`OboOntology`), list/lookup for NCIt/MONDO.
- `valueset.rs`
  - `ValueSetMeta` records for PET imaging subsets combining CPT/SNOMED, LOINC/NCIt.

## How mapping uses this
- `refractive_swan_mapping` calls `EnrichedCode::from_staging(...)` to:
  - Classify by `CodeKind` for **summary** tallies.
  - Attach `license_tier`/`source_kind` into `MappingResult` for downstream filtering.

## Tests
- Verify known systems resolve with expected license/source attributes.
- Verify OBO lookups and value set presence.
- Keep canonicalization stable for OID → URL normalization.

## Cross‑links
- Terminology semantics & policies: `docs/reference-terminology/semantic-relationships.yaml`
