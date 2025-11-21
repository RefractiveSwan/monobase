\
# VIII. Roadmap & Integration (DFPS crates)

**30 days**
- Implement estimators (`dfps_mapping` or a new `dfps_geometry` crate):
  - Anchor solver, \(R_M\), \(D_M\), \(\alpha_{\text{mf}}\); \(\alpha_{\text{sim}}\) via subspace‑search + SVM.
  - Expose metrics to `dfps_observability` and render via `dfps_eval::report`.
- Graph health: add Leiden pre‑processing before training embeddings.

**60 days**
- Add MMCR option and hierarchy‑aware flattening preprocessor; integrate with mapping engine’s vector ranker; monitor deltas in \(R_M\sqrt{D_M}\), \(\rho_{CC}\), α.

**90 days**
- Full evaluation harness across ontology snapshots; pre‑registered criteria; CI checks that gate deployments on capacity drift/correlation spikes/community disconnectedness.

**Touch points**
- `lib/domain/meta/evaluation`: extend `EvalSummary` to log geometry stats and α values.
- `lib/domain/mapping`: plug capacity metrics into error analysis; add reasons like `low_capacity_region`.
- `lib/domain/meta/pipeline`: pass geometry telemetry through pipeline output.
- `lib/platform/observability`: dashboard panels for \(R_M\), \(D_M\), \(\rho_{CC}\), α, and alerts.

**CI checks (fail conditions)**
- mean(R_M√D_M) ↑ > 15% vs. baseline; or ρ_CC ↑ > 0.10 absolute; or α_sim ↓ > 10%.
