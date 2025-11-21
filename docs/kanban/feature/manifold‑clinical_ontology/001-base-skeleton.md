# 🗂️ Kanban — *manifold‑clinical_ontology* (vectorized ontologies + capacity)

> Scope: build, measure, and govern the **manifold geometry** of NCIt/OBO concept embeddings across the audience bundles you listed (Academic, Clinical, Data, Doctoral, Engineering, Leadership, Linguist, Mathematician, Philosopher, Physicist, Project‑Design, Research, UX/UI).

**Columns** use GitHub‑style checkboxes and IDs (TERM/GEO/MAP/GRAPH/EVAL) you can lift into issues.

### Backlog

* [ ] **TERM‑01 | Vocabulary curation plan** — nominate concept slices (oncology imaging, labs, histology) and sources (NCIt/OBO, UMLS). Output: `/docs/reference-terminology/manifold-clinical_ontology/domain/*_Bundle.md` checklists per bundle.
* [ ] **TERM‑02 | Synonym gate** — define allow‑list/ban‑list and provenance fields; wire to “synonym gate” in docs. Output: policy YAML + code hooks.
* [ ] **GEO‑01 | Capacity metrics** — implement per‑concept **R_M**, **D_M (PR)**, centroid correlations **ρ_CC**, **α_mf** and **α_sim** + bootstrap CIs; JSON logging. (Specs in Report §IV‑A.) ([PMC][1])
* [ ] **GEO‑02 | Hierarchy‑aware flattening** — prototype centroid low‑rank projection + within‑class whitening; target ↓R_M, ↓D_M, ↓ρ_CC without degrading mapping. (Report §IV‑B.) ([PMC][1])
* [ ] **GRAPH‑01 | Leiden cleaning** — pre‑embedding community detection with Leiden; reject badly connected partitions; γ‑sweep and stability curve. (Report §VI.)
* [ ] **GRAPH‑02 | Resolution policy** — codify γ defaults per ontology slice (fine vs coarse granularity) + regression tests.
* [ ] **MAP‑01 | Re‑ranker hooks** — expose geometry flags (`capacity_low`, `centroid_corr_high`) in `MappingResult.reason`; connect to Auto/Review/NoMatch thresholds. (Docs already show state machines.)
* [ ] **EVAL‑01 | Counterexamples** — add generators where **capacity stays constant but mapping accuracy changes** (and vice versa). (Report §V, §VII.)
* [ ] **ALG‑01 | Algebraic probes** — implement low‑degree vanishing‑ideal detector for collapsed manifolds; alert when cross‑class residual separation < ε. (Report §IV‑D.)
* [ ] **OPS‑01 | Dashboards & SLAs** — drift alerts when |α_sim−α_mf|/α_sim > 0.2, ρ_CC ↑ > 0.05 abs, or Mean(R_M√D_M) ↑ > 20% vs. baseline. (Report §V, §VIII.) ([PMC][1])

### Doing

* [ ] **GEO‑01‑A | refractive_swan capacity evaluator** — add `geometry_probe()` producing `(R_M, D_M, ρ_CC, α_mf, α_sim)`; integrate with `EvalSummary` and pipeline reports. (Wire to `lib/domain/meta/evaluation` and `lib/domain/mapping`.)
* [ ] **GRAPH‑01‑A | Leiden in pipeline** — CLI to run γ‑grid, fail build on disconnected/badly connected communities ≠ 0.

### Review

* [ ] **TERM‑02‑A | Synonym gate A/B** — show effect on R_M, D_M, ρ_CC and mapping F1 before/after; rollback automatically if Mean(R_M√D_M) ↑ > 20%.
* [ ] **ALG‑01‑A | Vanishing‑ideal alerts** — validate against synthetic collapsed classes and real NCIt slices.

### Done

* [ ] **Docs skeletons** — audience bundle pages + FHIR/NCIt system design (provided).

---

# A Consortium‑of‑Minds Inquiry into Manifold Capacity, Geometry, and Vectorized Ontologies

## I. Executive Summary (≤300 words)

We connect **manifold capacity theory** to **vectorized ontologies** (NCIt/OBO) and deliver refractive_swan‑ready methods to **measure**, **shape**, and **govern** representation geometry. Capacity ( \alpha ) is the critical load (P/N) enabling linear separability of (P) manifolds in (N) dimensions. Mean‑field theory expresses ( \alpha^{-1} ) as an expectation of a support‑function optimization; **anchor points** induced by KKT conditions define **effective radius** (R_M) and **effective dimension** (D_M); the combined scale (R_M\sqrt{D_M}) (width) limits capacity. Empirical and theoretical work show layerwise improvements in separability track decreases in (D_M), (R_M), and centroid correlations ( \rho_{CC} ). We operationalize this with:
**(A)** capacity estimators ((R_M, D_M, \rho_{CC}, \alpha_{mf}, \alpha_{sim})) with bootstrap CIs and JSON logs;
**(B)** hierarchy‑aware flattening (centroid low‑rank removal + within‑class whitening; MMCR‑style objective) to reduce (R_M, D_M, \rho_{CC});
**(C)** **Leiden** community detection to guarantee well‑connected ontology graph communities pre‑embedding; and
**(D)** an evaluation harness with **counterexamples** and **algebraic probes** (vanishing ideals) to catch collapsed class manifolds.
These mechanisms plug directly into refractive_swan crates (`refractive_swan_mapping`, `refractive_swan_eval`, `refractive_swan_pipeline`, `refractive_swan_terminology`) and enforce safety with alerts when (|\alpha_{sim}-\alpha_{mf}|/\alpha_{sim} > 0.2), ( \rho_{CC}) spikes, or (R_M\sqrt{D_M}) inflates. ([PMC][1])

---

## II. Claims Matrix

| Paper                       | Claim (short)                                                                                                                               | Formal statement / locus                                                                                                       | Evidence                                                           | Relation                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------- |
| Dapello et al. 2021 (MFTMA) | Capacity via mean‑field; anchors define (R_M, D_M); width (R_M\sqrt{D_M}) controls (\alpha).                                                | Inverse capacity ( \alpha^{-1} = \mathbb{E}_T[F(T)]); anchors from KKT; (R_M, D_M) from anchor statistics; ball approximation. | arXiv PDF (defs & derivations). Strong.                            | Extends earlier theory; basis for our estimators. |
| Yerxa et al. 2023 (MMCR)    | Loss aligns with capacity; support‑function/KKT framing of (\alpha).                                                                        | Presents ( \alpha^{-1} ) via support function; MMCR objective (L \approx -|GZ|_*) aligns embeddings.                           | PMC article; equations & proof sketches. Medium‑strong. ([PMC][1]) | Extends: suggests capacity‑aware training.        |
| Chou et al. 2025 (GLUE)     | Dual estimators (\alpha_{sim}) & (\alpha_{mf}); anchor/axis metrics; empirical validation.                                                  | Methods & Supplement S1 list geometric measures and capacity constructs, contrasting (\alpha_{sim}) vs (\alpha_{mf}).          | PMC full text. Medium. ([PMC][1])                                  | Extends: practical probes & gap analysis.         |
| Traag et al. 2019 (Leiden)  | Leiden yields **connected** communities; converges to partitions without badly connected communities; Louvain can output disconnected ones. | Theory + experiments; guarantees after iterations.                                                                             | arXiv (ar5iv) article. Strong.                                     | Contradicts reliance on Louvain for graph health. |
| Bartlett (RKHS notes)       | Functions‑as‑vectors via RKHS; evaluation functionals bounded; kernel view unifies feature maps.                                            | Lecture notes defining RKHS and kernels; function space Hilbert structure.                                                     | Berkeley notes. Medium (tutorial). ([People @ EECS][2])            |                                                   |

---

## III. Formal Notes (definitions, lemmas, sketches)

**Definition 1 (Capacity).** For (P) manifolds ({\mathcal{M}*\mu}*{\mu=1}^P \subset \mathbb{R}^N) with random labels (y_\mu\in{\pm1}), the **linear classification capacity** is (\alpha_c = P_c/N), where (P_c) is the largest (P) such that a separating hyperplane exists w.h.p. **Mean‑field** inverse capacity:
[
\alpha^{-1}=\mathbb{E}*{T}!\left[\min*{V}|V-T|*2^2\ \text{s.t.}\ \min*{S\in \mathrm{conv}(\mathcal{M})} V!\cdot! S \ge 0\right],
]
with **anchors** (\tilde S(T)\in \mathrm{conv}(\mathcal{M}_\mu)) emerging from KKT conditions. ([PMC][1])

**Definition 2 (Effective radius/dimension).** Let anchor statistics define: (R_M:=\mathbb{E}|\tilde S|). Let (C) be the covariance of anchor‑projections; **effective dimension** (D_M := \frac{(\mathrm{tr},C)^2}{\mathrm{tr}(C^2)}) (participation ratio). **Width** (W_M:=R_M\sqrt{D_M}). Capacity decreases monotonically with (W_M) under ball approximation.

**Definition 3 (Centroid correlations).** With centroids (c_\mu = \mathbb{E}*{x\in\mathcal{M}*\mu}[x]), define (\rho_{CC} = \mathbb{E}*{\mu\ne\nu}[\cos(c*\mu, c_\nu)]). Low‑rank shared components in ({c_\mu}) reduce capacity; projecting out this subspace increases effective (\alpha).

**Lemma 1 (Ball approximation).** For ensembles with random centers/orientations, replacing each (\mathcal{M}_\mu) by a (D_M)‑ball of radius (R_M) preserves (\alpha) to first order; thus (W_M) controls separability. *Sketch:* match first‑ and second‑order anchor statistics; invoke mean‑field self‑consistency.

**Definition 4 (Extrinsic vs. intrinsic curvature).** Intrinsic curvature derives from the manifold’s Riemannian metric; extrinsic curvature from the second fundamental form of its embedding in (\mathbb{R}^N). Flattening seeks transforms that reduce extrinsic curvature (and thus (D_M)) without destroying class topology. (Operationalized in §IV‑B.) ([PMC][1])

**Definition 5 (Functions‑as‑vectors, RKHS).** A Reproducing Kernel Hilbert Space (RKHS) (\mathcal{H}) embeds functions as vectors with inner product induced by a positive‑definite kernel (k). Embeddings via feature maps (\phi(x)) realize geometric analysis in (\mathcal{H}). ([People @ EECS][2])

**Proposition 1 (Correlation projection).** If ({c_\mu}) have rank‑(K\ll P) structure, projecting (x\mapsto x - UU^\top x) onto the orthogonal complement of top‑(K) centroid PCs leaves within‑manifold geometry unchanged while increasing (\alpha). *Sketch:* Remove shared bias limiting margin; mean‑field capacity depends on projected centers.

---

## IV. Algorithms (pseudocode, complexity, expected effect)

### A) Capacity estimation from embeddings (refractive_swan evaluator)

**Goal.** Estimate (R_M, D_M, \rho_{CC}, \alpha_{mf}, \alpha_{sim}) per concept and globally; emit JSON.

**Pseudocode (vector/matrix primitives):**

```python
def capacity_metrics(X: np.ndarray, y: np.ndarray, m_list=[32,128,512], B=1000):
    # X: (N,D), y: concept IDs
    C = np.unique(y)
    mu = {c: X[y==c].mean(0) for c in C}
    # within-class covariance eigenspectrum
    eig = {}
    R = {}
    for c in C:
        Xc = X[y==c]; Z = Xc - mu[c]
        # covariance via thin SVD
        U, s, _ = np.linalg.svd(Z/np.sqrt(max(1,len(Z)-1)), full_matrices=False)
        lam = s**2; eig[c] = lam
        PR = (lam.sum()**2) / (lam**2).sum()
        R[c] = np.sqrt((Z**2).sum(1).mean())
    # centroid-correlation matrix (sparse kNN optional)
    rho = centroid_cosines(list(mu.values()))
    # α_mf via ball approximation
    Rbar = np.mean(list(R.values())); Dbar = np.median([pr(eig[c]) for c in C])
    alpha_mf = alpha_ball(Rbar, Dbar)        # lookup/closed form
    # α_sim curve: project to m dims then linear OVR classifier
    alpha_sim = {}
    for m in m_list:
        Xproj = pca_or_rproj(X, m)
        alpha_sim[m] = one_vs_rest_linear_acc(Xproj, y)    # balanced Acc/F1
    # bootstrap CIs for Rbar, Dbar, rho
    CIs = bootstrap_metrics(...)
    return dump_json(mu, R, eig, rho, alpha_mf, alpha_sim, CIs)
```

**Cost.** Per class SVD on ((n_c\times D)) (use truncated when (D) large): (O(\min(n_c D^2, D n_c^2))). Linear classifiers: (O(\sum_m N m)).
**Expected effect.** Interventions that **lower (R_M, D_M, \rho_{CC})** increase (\alpha); deviations between (\alpha_{sim}) and (\alpha_{mf}) indicate mean‑field mismatch. ([PMC][1])

---

### B) Hierarchy‑aware flattening / linearization

**Idea.** Emulate known layerwise effects: reduce (D_M) (pool/aggregate), reduce (R_M) (nonlinearity + normalization), reduce (\rho_{CC}) (remove shared modes).

**Recipe.**

1. **Centroid projection:** (x \leftarrow x - U_KU_K^\top x) where (U_K) spans top‑(K) centroid PCs.
2. **Whiten within‑class:** (x \leftarrow \Sigma_c^{-1/2}(x-\mu_c)) with shrinkage.
3. **MMCR‑style loss:** train a small adapter (g_\theta) to minimize (L=-|GZ|** + \lambda\cdot \text{topology_penalty}) (preserve nearest‑centroid ordering). Expect (R_M\downarrow, D_M\downarrow, \rho*{CC}\downarrow). ([PMC][1])

---

### C) Graph‑aware embedding updates (vectorized ontology layer)

**Inputs.** Ontology graph (G=(V,E)) with synonym/ancestor edges; text/definition corpora.
**Algorithm.**

* Build concept samples by **synonym sets** and **definition paraphrases**; gate low‑quality synonyms (provenance + overlap tests).
* Add **graph context pooling**: (x_c \leftarrow \mathrm{Agg}{x_c, x_{\text{syn}(c)}, x_{\text{anc}(c)}}) (mean/max attention with learned weights).
* Recompute geometry; rollback if Mean((R_M\sqrt{D_M})) ↑ > 20% or (\rho_{CC}) ↑ > 0.05 absolute.

---

### D) Capacity‑preserving community detection (Graph Health)

**Pipeline.**

1. Run **Leiden** (modularity or CPM) over (G) with γ grid.
2. **Reject** partitions with any **disconnected** or **badly connected** communities; iterate until guarantee holds; store γ→partition curve.
3. Use communities for negative sampling / curriculum, then embed.
   **Rationale.** Leiden guarantees connected communities after iterations; Louvain may return disconnected ones, distorting geometry estimates.

---

### E) Algebraic probes (vanishing ideals; high‑dim approximation)

**Goal.** Detect **collapsed** class manifolds (algebraic overlap) beyond metric cues.
**Approximation.** Fit low‑degree polynomial features (\phi_d(x)) with Lasso to find non‑trivial (p(x)) that nearly vanish on class (c): minimize (|p(\phi_d(X_c))|_2^2 + \lambda|p|_1) while maximizing residual on negatives. Alert when many classes share the same vanishing polynomials or when class‑specific vanishing error (\ll) off‑class error margin.

---

## V. Experiments (design matrix, metrics, thresholds)

**Design factors.**

* **Geometry:** (R_M \in {0.1,0.2,0.4}), (D_M \in {4,8,16,32}), (\rho_{CC}\in{0,0.05,0.1,0.2}).
* **Curvature:** extrinsic curvature parameter (\kappa\in{0,\kappa_{\text{mild}},\kappa_{\text{high}}}) via curved synthetic manifolds (swiss‑roll → linear map → noise).
* **Graph:** Louvain vs Leiden; γ sweep.
* **Interventions:** centroid projection on/off; whitening on/off; MMCR adapter on/off.

**Datasets.** (a) NCIt slices (imaging, histology); (b) synthetic manifolds matched to empirical (R_M, D_M); (c) graph variants pre/post Leiden cleaning.

**Metrics.**

* Geometry: (R_M, D_M, \rho_{CC}, W_M).
* Capacity: (\alpha_{mf}) (ball approx), (\alpha_{sim}(m)) curve via projected linear OVR accuracy.
* Downstream: mapping **F1 / AutoMapped / NeedsReview rates**.

**Acceptance thresholds (gate deployments).**

* (|\alpha_{sim}-\alpha_{mf}|/\alpha_{sim} \le 0.2) (else investigate model assumptions).
* **No** increase in Mean (\rho_{CC}) > 0.05 abs; **no** increase in Mean (W_M) > 20% without compensating (\alpha_{sim})↑ ≥ 10%.
* **Leiden** partition selected when disconnected fraction = 0 and stability across γ is high. ([PMC][1])

**Counterexamples (implement in eval).**

1. **Same (\alpha), worse mapping:** hold (R_M, D_M) fixed; rotate a subset of centroids to increase nearest‑centroid confusability → mapping F1↓ at similar (\alpha_{mf}).
2. **Similar (\alpha), better mapping:** project out top‑K centroid modes (reduce (\rho_{CC})), add mild within‑class noise to keep (W_M) ≈ const → accuracy↑.

**Statistical testing.** 1,000 bootstraps per run; report 95% CIs; permutation tests for class‑label invariance. ([PMC][1])

---

## VI. Graph / Ontology Actions

1. **Prefer Leiden** to guarantee connected communities and iterative convergence to partitions without badly connected subsets. Store γ→cluster count & stability; reject any partition failing the connectivity guarantee.
2. **Prune** dangling/low‑degree nodes; **collapse** near‑duplicate synonym cliques (cosine > τ) post‑Leiden.
3. **Hierarchy handling:** ancestor‑aware pooling with decays (e.g., (w_{\text{ancestor}}=\lambda^{\text{depth}})); re‑measure (R_M, D_M, \rho_{CC}) and roll back if thresholds fail.

---

## VII. Risks & Red‑Team Findings (with mitigations)

* **Mean‑field mismatch.** Non‑Gaussian centers or structured labels break (\alpha_{mf}). *Mitigation:* compare to (\alpha_{sim}); alert if gap > 0.2; inspect low‑rank center structure; project it out. ([PMC][1])
* **Synonym noise.** Inflates (R_M) and can raise (\rho_{CC}). *Mitigation:* synonym gate + revert if Mean (W_M) ↑ > 20%.
* **Community artifacts.** Louvain fragmentation distorts sampling and geometry. *Mitigation:* Leiden + connectivity tests in CI.
* **Collapsed manifolds (algebraic).** Metric probes miss it. *Mitigation:* vanishing‑ideal detector (low‑degree) to flag shared polynomials across classes.
* **Over‑flattening.** Excess whitening can erase clinically meaningful variation. *Mitigation:* topology penalty (nearest‑centroid order preservation) and per‑bundle clinical review.

---

## VIII. Roadmap & Integration (refractive_swan crates/docs; metrics; CI)

**30 days (A & C).**

* Implement `geometry_probe()` (Rust) producing `{r_m, d_m, rho_cc, alpha_mf, alpha_sim}`; attach to `MappingResult.reason` flags.
* Add **Leiden** pipeline (γ grid + connectivity checks) and fail CI on bad partitions.
* **Logs/JSON**: global + per‑class metrics with bootstrap CIs; dashboards plot (W_M), (\rho_{CC}), (\alpha_{sim}(m)).

**60 days (A2 & B).**

* Ship **flattening adapter** (centroid projection + whitening + MMCR‑style loss) as optional pre‑ranker stage; A/B on NCIt slices.
* Wire **counterexample** generators into `refractive_swan_eval` to validate gating rules. ([PMC][1])

**90 days (D).**

* **Algebraic probes** (vanishing‑ideal alarms) in `refractive_swan_eval`; policy to block deployments on collapse alerts.

**Crate touch‑points.**

* `lib/domain/mapping` — add `geometry_probe()` & flags; flattening pre‑ranker hook.
* `lib/domain/meta/evaluation` — metrics structs + bootstrap; counterexample generators.
* `lib/domain/meta/pipeline` — surface metrics to observability.
* `lib/domain/ontologies/terminology` — synonym gate + provenance; Leiden outputs for graph health.

**CI checks (defaults).**

* `|α_sim−α_mf|/α_sim ≤ 0.2`, `Mean ρ_CC ≤ baseline+0.05`, `Mean(W_M) ≤ baseline×1.2`, **Leiden connectivity = pass**. ([PMC][1])

---

## IX. References (seed‑biased; URLs via citations)

* Dapello, J. et al. *Neural population geometry reveals the role of stochasticity in robust perception.* (MFTMA; anchors, (R_M, D_M), width). arXiv PDF.
* Yerxa, T. E. et al. *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* (Support‑function capacity; MMCR objective.) PMC article. ([PMC][1])
* Chou, C.‑N. et al. *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* (Dual (\alpha_{sim})/(\alpha_{mf}); geometry metrics; empirics.) PMC article. ([PMC][1])
* Traag, V. A. et al. *From Louvain to Leiden: guaranteeing well‑connected communities.* (Connectivity guarantees; convergence.) arXiv (ar5iv) article.
* Bartlett, P. *Reproducing Kernel Hilbert Spaces.* (Functions‑as‑vectors; RKHS.) Lecture notes. ([People @ EECS][2])
* **Vanishing ideals**: *Approximating Latent Manifolds in Neural Networks via Vanishing Ideals.* (Algebraic probes; practical approximations.) arXiv PDF.

---

### Appendix—refractive_swan Capacity Estimation Plan (concise cut‑sheet)

* **Inputs:** embeddings (X\in\mathbb{R}^{N\times D}), labels (concept IDs).
* **Compute:** per‑class ( \mu_c, \Sigma_c\Rightarrow R_c, PR_c\ (D_{M,c}),) pairwise centroid cosines; global (R_M=\mathrm{mean},R_c), (D_M=\mathrm{median},PR_c), ( \rho_{CC}=\mathrm{mean,cos}).
* **Capacity:** ( \alpha_{mf}=\alpha_{\text{ball}}(R_M,D_M)); ( \alpha_{sim}(m)) from projected linear separability curves.
* **CIs:** bootstrap per‑class and global metrics (B=1000).
* **Emit JSON:** keyed by concept + global summary; wire to `refractive_swan_eval` and dashboards. ([PMC][1])

### Appendix—Graph Community Tuning (reproducible steps)

1. Run Leiden with γ∈Γ; iterate until **no badly connected** communities; store γ→partition + metrics.
2. Fail CI if any disconnected community exists; pin γ that maximizes stability and downstream ( \alpha_{sim}).

### Appendix—Counterexample Construction (generators)

* **Hold (R_M, D_M), vary (\rho_{CC}):** simulate equal‑covariance Gaussians; rotate centroids to reduce inter‑class angles → F1↓ at constant (\alpha_{mf}).
* **Reduce (\rho_{CC}), keep (W_M)≈const:** project out top‑K centroid PCs; add isotropic noise to keep (R_M) stable → F1↑ with (\alpha_{mf}) unchanged.

### Appendix—Algebraic Probes (tractable approximation)

* Build degree‑(d) polynomial features; L1‑regularized regression to find low‑norm (p(x)) s.t. (p(x)\approx 0) on class, (> \epsilon) off‑class; alert on shared vanishing polynomials across classes.
