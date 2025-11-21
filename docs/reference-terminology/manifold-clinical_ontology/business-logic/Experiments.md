# V. Experiments (design matrix, metrics, thresholds)

## Factors × Levels
- **Curvature / Flattening:** {none, FlatNet, hierarchy‑aware block} (S7, S3)
- **Centroid correlation:** {none, low‑rank projection K=1..5}
- **Graph communities:** {Leiden, Louvain}
- **Embedding regime:** {random init, trained text only, text+graph}
- **Dimensionality \(d\):** {128, 256, 384, 768}
- **Noise:** {clean synonyms, noisy synonyms (p%)}

## Metrics
- \(R_M\), \(D_M\), \(R_M\sqrt{D_M}\), \(\rho_{CC}\); capacities \(\alpha_{\text{mf}}\), \(\alpha_{\text{sim}}\).
- Mapping outcomes: precision/recall/F1; state distribution (AutoMapped/NeedsReview/NoMatch).

## Acceptance thresholds
- **MFT consistency:** \(|\alpha_{\text{sim}}-\alpha_{\text{mf}}|/\alpha_{\text{sim}} \le 0.2\).
- **Capacity gain:** ≥ 10% α increase for new encoders vs. baseline without degradation in precision.
- **Graph health:** disconnected communities < 0.5%; otherwise switch to Leiden.

## Protocols
1) **Synthetic manifolds.** Generate \(D_M\)‑dimensional ellipsoids with controllable \(R_M\), \(D_M\), and centroid correlations; validate \(\alpha_{\text{mf}}\) vs. \(\alpha_{\text{sim}}\).  
2) **Ontology embeddings.** Build per‑concept sample sets from synonyms/definitions; compute metrics pre/post flattening and low‑rank centroid projection.  
3) **Graph health A/B.** Run Louvain vs. Leiden; measure disconnectedness and downstream \(R_M,D_M,\alpha\).  
4) **Ablations.** Remove synonyms or ancestor prompts to quantify their contributions to \(R_M\), \(D_M\), \(\rho_{CC}\).

## Statistical tests
- Paired t‑tests (or Wilcoxon) on \(R_M\), \(D_M\), \(\alpha\) deltas per concept.
- Bootstrap 95% CIs for mapping precision/recall (already available via `refractive_swan_eval` advanced feature).
