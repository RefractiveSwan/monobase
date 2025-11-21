# Crate: lib/domain/mapping — `refractive_swan_mapping`

**Path:** `code/lib/domain/mapping`  
**Depends on:** `refractive_swan_core`, `refractive_swan_terminology`, `serde(_json)`.

## Responsibilities
- Map staging codes to **NCIt** concepts; keep logic **deterministic and local**.
- Combine lexical + vector mock rankers with a rule re‑ranker; use **UMLS cross‑refs** where available.
- Attach **license/source** metadata using `refractive_swan_terminology`.

## Modules & data
- `data.rs`
  - `load_ncit_concepts()` → `Vec<(NCItConcept, DimNCITConcept)>` (embedded JSON).
  - `load_umls_xrefs()` → `HashMap<(system, code), UmlsXref>` (embedded JSON).
  - Version constants: `NCIT_DATA_VERSION`, `UMLS_DATA_VERSION`.
- `lib.rs`
  - Rankers: `LexicalRanker`, `VectorRankerMock`, `RuleReranker`.
  - Engine: `MappingEngine<L,V>` with `ranked_candidates()` and `explain()`.
  - API: `map_staging_codes(...)`, `map_staging_codes_with_summary(...)`, `explain_staging_code(...)`.
  - Summary: `MappingSummary { total, by_code_kind, by_license_tier }`.
  - Classification helpers: `classify(score, thresholds)` → `MappingState`.
  - Result assembly: `build_result_with_score(...)`, `source_versions()`.

## Behavior
- For (system, code) present in `umls_xrefs.json` → emit **rule‑based** high‑score mapping (`0.99`) with `reason = "umls_direct_xref"`.
- Else → combine lexical/vector candidates; `RuleReranker` nudges **NCIT** upward slightly.
- Final `MappingResult` includes `state` by threshold, `source_version`, and, via `terminology::EnrichedCode`, `license_tier` and `source_kind`.

## Tests
- Determinism checks for engine outputs.
- Data loaders parse and include expected rows.
- Summary tallies by `CodeKind` (`known_licensed_system`, `unknown_system`, etc.) and license tiers.

## Cross‑links
- NCIt architecture & states: `docs/system-design/ncit/**`
- Terminology/registry semantics: `docs/reference-terminology/semantic-relationships.yaml`
