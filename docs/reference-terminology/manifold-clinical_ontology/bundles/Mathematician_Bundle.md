
# Mathematician Bundle - Definitions & Problems
**Date:** 2025-11-15

## Objects
- Manifolds {M_mu} subset R^D; centroids c_mu; anchors x_tilde_mu.
- Geometry: R_M, D_M (participation ratio), centroid correlation rho_CC.

## Problems
1. Prove risk bounds for linear classifiers trained on manifold data in terms of W_M and rho_CC.
2. Characterize stability of alpha under low-rank projections of centroid space.


**Structure diagram**
```mermaid
classDiagram
  class Manifold {
    +samples : R^D
    +centroid : R^D
    +anchors : R^D
  }
  class Geometry {
    +R_M : float
    +D_M : float
    +rho_CC : float
  }
  Manifold --> Geometry
```

## References (seed-first, minimal adjacent)

- Cohen, U., Chung, S., Lee, D. D., & Sompolinsky, H. (2020). *Separability and geometry of object manifolds in deep neural networks.* **Nature Communications**. https://www.nature.com/articles/s41467-020-14578-5
- Dapello, J., et al. (2021). *Neural population geometry reveals the role of stochasticity in robust perception.* arXiv:2111.06979. https://ar5iv.org/html/2111.06979
- Yerxa, T., Kuang, X., Simoncelli, E., & Chung, S. (2023). *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* arXiv:2303.03307. https://arxiv.org/pdf/2303.03307
- Chou, K.-C., et al. (2025). *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* bioRxiv:2024.02.26.582157. https://www.biorxiv.org/content/10.1101/2024.02.26.582157v1
- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). *From Louvain to Leiden: guaranteeing well-connected communities.* arXiv:1810.08473. https://arxiv.org/pdf/1810.08473
- Dominguez-Olmedo, A., et al. (2023). *The geometry of concept manifolds.* JMLR 25(62). https://www.jmlr.org/papers/volume25/23-0615/23-0615.pdf
- Primer (weak evidence): *Functions are Vectors.* https://thenumb.at/Functions-are-Vectors/
