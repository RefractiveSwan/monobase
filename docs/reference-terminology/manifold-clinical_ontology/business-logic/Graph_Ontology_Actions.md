# VI. Graph/Ontology Actions

1. **Prefer Leiden over Louvain** for community detection when defining graph neighborhoods for embedding context. Leiden **guarantees connected communities** and converges under iteration; Louvain can output disconnected/badly connected sets that distort manifold statistics. (S4)
2. **Prune and normalize**: drop dangling/isolated nodes; normalize code‑system URLs to canonical forms before graph construction (see `refractive_swan_terminology::bridge`).
3. **Synonym expansion**: treat synonyms/definitions as samples on each concept manifold; gate low‑quality entries with lexical/semantic filters.
4. **Hierarchy handling**: sample ancestor/child prompts to provide hierarchy‑aware context; measure effect on centroid correlations.
5. **Community‑aware sampling**: when training the graph encoder, sample neighbors within **connected** communities; forbid cross‑community leakage unless edges are explicit “xref/related_to”.
