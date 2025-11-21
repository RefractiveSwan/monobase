# I. Executive Summary

**Title:** *A Consortium-of-Minds Inquiry into Manifold Capacity, Geometry, and Vectorized Ontologies*

We connect mean‑field manifold capacity theory to algorithms and controls for a **Vectorized Ontology Layer** that treats NCIt/OBO code systems as joint graph‑and‑vector objects. Classification capacity **α = P/N** depends on **effective manifold radius (R_M)**, **effective dimension (D_M)**, and **centroid correlations (ρ_CC)** measured from **anchor points**; reductions in D_M and R_M and decorrelation of centroids increase α. These quantities admit practical estimators and have been observed to improve across network hierarchies and under explicit **flattening** objectives. [A1, A2, S7, S3]

Operationally, we: (A) build hybrid text+graph embeddings and estimate α via simulation (α_sim) and mean‑field (α_mf) using anchor statistics; (B) use these geometry metrics to predict mapping accuracy and error modes (**AutoMapped / NeedsReview / NoMatch**); (C) enforce graph health with **Leiden** communities to avoid spurious, disconnected clusters that distort geometry; and (D) deliver a repeatable evaluation harness and CI checks. [S4]

**Key levers** (and expected measurable effects): (1) hierarchy‑aware **flattening** (R_M↓, D_M↓ → α↑); (2) removal of low‑rank centroid components (ρ_CC↓ → α↑); (3) community quality via Leiden (connected communities → more stable manifold statistics); and (4) MMCR‑style pretraining that aligns embeddings with task geometry (R_M√D_M ↓). [S7, A2, A1]

We provide formal notes, algorithms with complexity, experiment plans, risk analyses, and a 30/60/90‑day roadmap wired to the refractive_swan codebase (eval/mapping/pipeline crates). All recommendations include **measurable, testable** criteria and alerts for **capacity drift**, **correlation spikes**, and **community fragmentation**.
