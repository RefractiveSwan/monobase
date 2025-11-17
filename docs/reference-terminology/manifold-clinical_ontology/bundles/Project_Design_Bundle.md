
# Project Design Bundle - Work Breakdown & Acceptance
**Date:** 2025-11-15

## Deliverables
1. Geometry probes with DFPS integration and dashboards.
2. Vectorized Ontology Layer (graph+text unification; synonym gates).
3. Graph health with Leiden; CI checks.
4. Evaluation harness (synthetic + ontology).

## Work Breakdown Structure
- WBS-1: Probes (data model, compute, report).
- WBS-2: Ontology vectorization (embedding, sampling, gating).
- WBS-3: Graph health (Leiden, audit, CI).
- WBS-4: Harness (datasets, metrics, tests).


**WBS**
```mermaid
flowchart TB
  A[WBS-1 Probes] --> A1[Data Model]
  A --> A2[Compute]
  A --> A3[Report]
  B[WBS-2 Ontology] --> B1[Embeddings]
  B --> B2[Synonym Gate]
  C[WBS-3 Graph Health] --> C1[Leiden]
  C --> C2[CI Checks]
  D[WBS-4 Harness] --> D1[Synth Sets]
  D --> D2[Ontology Slices]
  D --> D3[Stat Tests]
```


## Acceptance criteria
- Metrics available in `EvalSummary` and rendered in reports.
- alpha_sim and alpha_mf within 20% on validation.
- Leiden connectivity pass enforced in CI; disconnected% <= 0.5%.
- Mapping F1 non-degrading after interventions.


## References (seed-first, minimal adjacent)

- Cohen, U., Chung, S., Lee, D. D., & Sompolinsky, H. (2020). *Separability and geometry of object manifolds in deep neural networks.* **Nature Communications**. https://www.nature.com/articles/s41467-020-14578-5
- Dapello, J., et al. (2021). *Neural population geometry reveals the role of stochasticity in robust perception.* arXiv:2111.06979. https://ar5iv.org/html/2111.06979
- Yerxa, T., Kuang, X., Simoncelli, E., & Chung, S. (2023). *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* arXiv:2303.03307. https://arxiv.org/pdf/2303.03307
- Chou, K.-C., et al. (2025). *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* bioRxiv:2024.02.26.582157. https://www.biorxiv.org/content/10.1101/2024.02.26.582157v1
- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). *From Louvain to Leiden: guaranteeing well-connected communities.* arXiv:1810.08473. https://arxiv.org/pdf/1810.08473
- Dominguez-Olmedo, A., et al. (2023). *The geometry of concept manifolds.* JMLR 25(62). https://www.jmlr.org/papers/volume25/23-0615/23-0615.pdf
- Primer (weak evidence): *Functions are Vectors.* https://thenumb.at/Functions-are-Vectors/
