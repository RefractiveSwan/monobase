# Roadmap — Post-MVP
**Goal:** Outline post-MVP initiatives: scaling namespaces, advanced ANN indexes, ontology visualization, community detection, geometry-aware ranking, warehouse integrations, and teaching expansion—all FOSS-only.

## Epics (Thematic)
- **Namespace Scaling:** Multi-namespace isolation, quotas, and schema migrations; composite PK enforcement; metrics per namespace.
  - Risks: cross-namespace contamination; migration downtime.
  - Gating: precision/recall floors hold per namespace; migration dry-run passes.
- **Advanced ANN Indexes:** HNSW/IVF tuning, PQ/OPQ for larger corpora; fallback safety.
  - Risks: recall regressions; resource spikes.
  - Gating: capacity/recall above floor; latency SLO met; rollback plan ready.
- **Ontology Visualization Dashboards:** PCA/MDS/UMAP overlays for NCIt/OBO; capacity heatmaps.
  - Risks: misinterpretation of clusters; privacy leaks from labels.
  - Gating: anonymized views; reviewer sign-off; deterministic seeds.
- **Community Detection (Leiden):** Graph builds from ontology edges and embeddings; Leiden clustering via FOSS libs.
  - Risks: unstable partitions; compute cost.
  - Gating: stability across seeds; cluster quality metrics above threshold.
- **Geometry-Aware Ranking:** Adjust scores using `R_M`, `D_M`, centroid/axis correlations; integrate ALG-014 outputs.
  - Risks: overfitting to geometry artifacts.
  - Gating: mapping precision/recall floors hold; capacity health metrics stable.
- **Data Warehouse Integrations:** Expand datamart to warehouse pipelines; snapshotting; SQL analytics.
  - Risks: schema drift; load performance.
  - Gating: migration playbooks; warehouse SLOs; regression tests.
- **Teaching Expansion:** More modules (vector search advanced, geometry deep dives); contributor guides with runnable notebooks.
  - Risks: stale content; non-reproducible examples.
  - Gating: each module reviewed; examples runnable with FOSS tooling.

## Risks & Gating Criteria
- Maintain precision/recall floors per epic; monitor capacity health (`α`, `R_M`, `D_M`) to detect drift.
- Enforce GPLv3/FOSS-only dependencies; no closed SaaS.
- Require rollback plans for ANN/index changes; deterministic seeds in tests/visuals.

## Acceptance Criteria
- Each epic lists FOSS technology choices and gating (precision/recall, capacity metrics, SLOs).
- Dashboards/visuals reproducible with FOSS libs; namespaces isolated and observable.
- Teaching content expanded with objectives, prerequisites, runnable examples, and review checklists.
