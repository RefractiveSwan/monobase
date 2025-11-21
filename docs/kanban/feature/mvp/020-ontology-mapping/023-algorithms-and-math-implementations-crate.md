# Kanban — feature/algorithms-and-math-implementations-crate (023)
**Summary:** Plan a Rust `refractive_swan_math` / `refractive_swan_algorithms` crate delivering FOSS/GPLv3-compatible primitives for embeddings, dimensionality reduction, geometry metrics, and capacity estimators that support mapping/vector workflows.

## Scope & Non-Goals
- In scope: crate scaffolding; linear algebra via FOSS crates; PCA/MDS/random projections; manifold metrics (`R_M`, `D_M`, centroid/axis correlations); capacity estimators (`α_sim` via linear SVM/perceptron proxy, `α_mf` mean-field); CSV export helpers; optional UMAP via FOSS.
- Out of scope: training deep models; proprietary toolchains; GPU-specific backends beyond FOSS crates; non-FOSS dependencies.

## Kanban
### ALG-01 — Crate scaffolding (`refractive_swan_math`/`refractive_swan_algorithms`)
- [ ] Create crate with GPLv3-compatible license, feature flags for optional algos (e.g., `umap`).
- [ ] Module layout: `linalg`, `dimred`, `geometry`, `capacity`, `viz`, `bench`.
- [ ] Documentation header linking vector layer and geometry Kanban.

### ALG-02 — Linear algebra ops
- [ ] Choose `ndarray`/`nalgebra` (FOSS) or pure Rust; add BLAS optional feature.
- [ ] Helpers for centering, normalization, covariance, SVD (wrapping `ndarray-linalg` if available).
- [ ] Deterministic RNG seeding utilities for reproducible tests/benchmarks.

### ALG-03 — Dimensionality reduction
- [ ] PCA (eigen/SVD paths) with explained-variance outputs.
- [ ] MDS (classical) with distance matrix input; random projection helper for speed baselines.
- [ ] Optional UMAP (FOSS crate) behind feature flag; consistent seeds.

### ALG-04 — Geometry metrics
- [ ] Effective radius `R_M` and effective dimension `D_M` via participation ratio/principal radii.
- [ ] Centroid correlations and axis correlations; anchor-point selection helper.
- [ ] API to accept embedding matrices and emit scalar metrics + diagnostics.

### ALG-05 — Capacity estimators
- [ ] Simulation-based `α_sim` using linear SVM/perceptron wrappers on synthetic manifolds.
- [ ] Mean-field `α_mf` proxy; document assumptions and inputs (`R_M`, `D_M`).
- [ ] Configurable seeds, trial counts, and tolerance thresholds for CI.

### ALG-06 — Visualization helpers
- [ ] CSV/JSON export of embeddings, centroids, projections for plotting (PCA/MDS/UMAP).
- [ ] Small color/label schema suggestions for downstream plotting (kept data-only here).
- [ ] No bundled GUI; ensure outputs are consumable by FOSS notebooks/scripts.

### ALG-07 — Benchmarks + property tests
- [ ] Microbenchmarks for PCA/MDS and capacity estimators on small fixtures.
- [ ] Property tests: invariance under translation/rotation for `R_M`/`D_M`; reproducibility under fixed seeds.
- [ ] Note any `no_std` constraints (likely `std` only; document limitations).

## Acceptance Criteria
- Reproducible outputs on small synthetic fixtures; benchmarks runnable in CI (behind feature flag if slow).
- All dependencies FOSS and GPLv3-compatible; feature flags documented.
- Cross-links present to geometry Kanban, vector layer doc, and NCIt architecture.

## Cross-Links
- Geometry Kanban: `../../research/math-proofs-and-geometry-docs.md`
- Vector layer doc: `../../system-design/clinical/ncit/concepts/vector-layer.md`
- NCIt architecture: `../../system-design/clinical/ncit/architecture/system-architecture.md`

## Example Code (pseudocode)
```rust
// PCA
let x: Array2<f64> = load_embeddings();
let centered = center(&x);
let (components, explained_var) = pca(&centered, k)?;

// Geometry metrics
let metrics = manifold_metrics(&x); // returns { r_m, d_m, centroid_corr, axis_corr }

// Capacity simulation
let alpha_sim = capacity_simulation(&x, trials=200, seed=42)?;
let alpha_mf = capacity_mean_field(&metrics);
```
