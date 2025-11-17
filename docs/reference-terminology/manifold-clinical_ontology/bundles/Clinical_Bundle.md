
# Clinical Bundle - Safety, Review, and Examples
**Date:** 2025-11-15

## What clinicians see
- Clear mapping state (AutoMapped / NeedsReview / NoMatch) with rationale (score + geometry flags).
- Review UI: highlight evidence strings; show nearby concepts; one-click approve/reject.

## Safety rules
- Never auto-map when geometry drift alerts fire (alpha gap > 20%).
- High-risk specialties (oncology) default to NeedsReview unless gold-standard evidence present.


**Clinician review flow**
```mermaid
flowchart TD
  Start[New Code] --> Score[System score + geometry flags]
  Score -->|>= tau_auto & no alerts| Auto[AutoMapped]
  Score -->|otherwise| Review[NeedsReview]
  Review --> Approve[Approve] --> Auto
  Review --> Reject[Reject] --> NoMatch[NoMatch]
```


## Examples
- Synonym pitfall: "Mass" (tumor vs weight) - synonym gate avoids wrong expansion.
- Hierarchy clue: parent/child context clarifies ambiguous short labels.

## KPIs
- Reviewer workload down 20% at same precision.
- Near-miss analysis closed within 48h.


## References (seed-first, minimal adjacent)

- Cohen, U., Chung, S., Lee, D. D., & Sompolinsky, H. (2020). *Separability and geometry of object manifolds in deep neural networks.* **Nature Communications**. https://www.nature.com/articles/s41467-020-14578-5
- Dapello, J., et al. (2021). *Neural population geometry reveals the role of stochasticity in robust perception.* arXiv:2111.06979. https://ar5iv.org/html/2111.06979
- Yerxa, T., Kuang, X., Simoncelli, E., & Chung, S. (2023). *Learning Efficient Coding of Natural Images with Maximum Manifold Capacity Representations.* arXiv:2303.03307. https://arxiv.org/pdf/2303.03307
- Chou, K.-C., et al. (2025). *Geometry Linked to Untangling Efficiency Reveals Structure and Computation in Neural Populations.* bioRxiv:2024.02.26.582157. https://www.biorxiv.org/content/10.1101/2024.02.26.582157v1
- Traag, V. A., Waltman, L., & van Eck, N. J. (2019). *From Louvain to Leiden: guaranteeing well-connected communities.* arXiv:1810.08473. https://arxiv.org/pdf/1810.08473
- Dominguez-Olmedo, A., et al. (2023). *The geometry of concept manifolds.* JMLR 25(62). https://www.jmlr.org/papers/volume25/23-0615/23-0615.pdf
- Primer (weak evidence): *Functions are Vectors.* https://thenumb.at/Functions-are-Vectors/
