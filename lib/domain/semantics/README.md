# Domain Semantics

This directory documents the mapping/analytics semantics layer. The main
crates still live in their historical homes (e.g., `lib/domain/ontologies/mapping`),
but architecture docs now group them under the "semantics" bucket.

- `refractive_swan_mapping` – lexical/vector rankers, mapping engine helpers.
- Future analytics helpers will live next to mapping as they are extracted.

REFR-027 tracks moving the actual crates under this directory in later phases.
