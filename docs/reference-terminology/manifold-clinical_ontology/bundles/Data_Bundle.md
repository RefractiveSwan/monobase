
# Data Bundle - Schemas, Lineage, and Quality
**Date:** 2025-11-15

## In/Out Schemas (simplified)
- Input: staged code (system, code, display), ontology nodes/edges, synonym tables, embeddings.
- Output: mapping results + geometry stats.


**Data Flow**
```mermaid
flowchart LR
  Raw[Raw FHIR/HL7] --> Stage[Staging]
  Stage --> Explode[Explode Codes]
  Explode --> Embed[Text/Graph Embeddings]
  Embed --> Probe[geometry_probe]
  Probe --> Report[Eval/Report + Dashboards]
```

**ER diagram**
```mermaid
erDiagram
  ONT_NODE ||--o{ ONT_EDGE : links
  ONT_NODE {
    string id
    string label
    string[] synonyms
  }
  ONT_EDGE {
    string src
    string dst
    string rel
  }
  EMBEDDING ||--o{ SAMPLE : provides
  EMBEDDING {
    string id
    float[] vec
  }
  SAMPLE {
    string concept_id
    string source
    string text
  }
```


## Data quality gates
- Null/duplicate codes blocked.
- Synonym source allowlist; rollback if W_M increases > 20%.
- Connectivity check: disconnected% <= 0.5% (Leiden).


## References (seed-first, minimal adjacent)

- Cohen, U., Chung, S., Lee, D. D., & Sompolinsky, H. (2020). *Separability and geometry of object manifolds in deep neural networks.* **Nature Communications**. https://www.nature.com/articles/s41467-020-14578-5
- Dapello, J., et al. (2021). *Neural population geometry reveals the role of stochasticity in robust perception.* arXiv:2111.06979. https://ar5iv.org/html/2111.06979
- Yerxa, T., Kuang, X., Simoncelli, E., & Chung, S. (2023). *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* arXiv:2303.03307. https://arxiv.org/pdf/2303.03307
- Chou, K.-C., et al. (2025). *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* bioRxiv:2024.02.26.582157. https://www.biorxiv.org/content/10.1101/2024.02.26.582157v1
- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). *From Louvain to Leiden: guaranteeing well-connected communities.* arXiv:1810.08473. https://arxiv.org/pdf/1810.08473
- Dominguez-Olmedo, A., et al. (2023). *The geometry of concept manifolds.* JMLR 25(62). https://www.jmlr.org/papers/volume25/23-0615/23-0615.pdf
- Primer (weak evidence): *Functions are Vectors.* https://thenumb.at/Functions-are-Vectors/
