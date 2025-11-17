
# Linguist Bundle - Semantics, Polysemy, and Ontologies
**Date:** 2025-11-15

## Perspective
- Distributional semantics provides manifold samples (synonyms, paraphrases, definitions).
- Polysemy inflates R_M; sense-specific embeddings or context windows reduce width.
- Ontology edges (is-a, part-of) refine centroids and reduce rho_CC.


**Sense/term graph**
```mermaid
graph TD
  Term["mass"] -->|sense 1| S1["tumor mass"]
  Term -->|sense 2| S2["body mass"]
  S1 --> Ont1[oncology node]
  S2 --> Ont2[anthropometry node]
```


## Practices
- Sense inventory for high-polysemy terms.
- Synonym sources curated; verify W_M before/after.


## References (seed-first, minimal adjacent)

- Cohen, U., Chung, S., Lee, D. D., & Sompolinsky, H. (2020). *Separability and geometry of object manifolds in deep neural networks.* **Nature Communications**. https://www.nature.com/articles/s41467-020-14578-5
- Dapello, J., et al. (2021). *Neural population geometry reveals the role of stochasticity in robust perception.* arXiv:2111.06979. https://ar5iv.org/html/2111.06979
- Yerxa, T., Kuang, X., Simoncelli, E., & Chung, S. (2023). *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* arXiv:2303.03307. https://arxiv.org/pdf/2303.03307
- Chou, K.-C., et al. (2025). *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* bioRxiv:2024.02.26.582157. https://www.biorxiv.org/content/10.1101/2024.02.26.582157v1
- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). *From Louvain to Leiden: guaranteeing well-connected communities.* arXiv:1810.08473. https://arxiv.org/pdf/1810.08473
- Dominguez-Olmedo, A., et al. (2023). *The geometry of concept manifolds.* JMLR 25(62). https://www.jmlr.org/papers/volume25/23-0615/23-0615.pdf
- Primer (weak evidence): *Functions are Vectors.* https://thenumb.at/Functions-are-Vectors/
