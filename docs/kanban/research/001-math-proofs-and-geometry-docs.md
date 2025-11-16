# Kanban — Research: Math Proofs and Geometry Docs
**Status:** DOING  
**Owners:** _TBD_  
**Last updated:** <!-- YYYY-MM-DD -->

## Executive Summary
- Geometry of vectorized ontologies (NCIt/OBO + embeddings) drives mapping accuracy, stability of candidate recall, and visualization clarity.
- Effective radius/dimension and capacity estimators indicate when vector rankers generalize or overfit; centroid/axis correlations reveal ontology topology.
- Documentation + proofs ensure reproducibility under GPLv3/FOSS tooling for reviewers and mapping developers.

## Work Items

### GEO-01 — Formal definitions & notation
- [ ] Define effective radius `R_M`, effective dimension `D_M`, manifold centroid `c`, anchor points, axis correlations, and pairwise centroid correlations.
- [ ] Specify capacity estimators: simulation-based `α_sim` and mean-field `α_mf`; note assumptions on embedding normalization.
- [ ] Fix notation for embedding matrices, projection operators, and statistical moments used across docs.

### GEO-02 — Proof sketches & estimator conditions
- [ ] Provide proof sketches for `D_M`, `R_M` concentration under sphere/ellipsoid models.
- [ ] State conditions where `α_mf` approximates `α_sim` (mean-field, isotropy, large-N).
- [ ] Explicitly list failure modes: heavy tails, strong anisotropy, heterogeneous cluster sizes.

### GEO-03 — Algorithms & pseudocode
- [ ] Pseudocode for computing `R_M`, `D_M` via SVD/principal radii; anchor-point selection; centroid correlation matrix.
- [ ] Simulation estimator `α_sim` via linear SVM capacity on synthetic manifolds; mean-field estimator `α_mf` formula and inputs.
- [ ] Complexity notes and FOSS-only dependencies (NumPy/SciPy/ndarray/rust-ndarray).

### GEO-04 — Validation plan & falsification
- [ ] Synthetic benchmarks where `α_mf` vs `α_sim` diverge; thresholds to flag geometry drift.
- [ ] Cross-validate on NCIt/UMLS subsets; permutation tests for centroid correlations.
- [ ] Define CI checks: deterministic seeds, tolerance bands, and fail-fast when anisotropy > threshold.

### GEO-05 — Documentation targets
- [ ] Geometry cheat-sheet (definitions, symbols, units).
- [ ] “How to measure” guide with runnable examples on synthetic data.
- [ ] Glossary aligning terminology with mapping docs and reviewers’ expectations.

### GEO-06 — Visualization specs
- [ ] PCA (2D/3D) for anchor points + centroids; MDS/t-SNE/UMAP visual guidelines (FOSS libs only).
- [ ] Standard color/shape encodings for ontology slices and capacity regimes.
- [ ] Guidance on reproducible seeds and downsampling for reviewer artifacts.

## Acceptance Criteria
- All geometry docs compile; examples runnable on synthetic data with FOSS/GPLv3-compatible libs.
- CI includes geometry checks (capacity deltas, anisotropy thresholds); deterministic seeds.
- Cross-links established to mapping/system design; glossary aligned with ontology semantics.

## Cross-Links
- NCIt architecture: `../system-design/clinical/ncit/architecture/system-architecture.md`
- Future algorithms crate Kanban (placeholder): `./algorithms-crate.md` (to be added)

## Appendix (stub)
- References and derivations (internal sources); to be expanded with formal proofs and numeric examples.
