# CROSS-COHESION REQUIREMENTS (per Kanban card)

For every Kanban card section you touch (e.g., headings) like:

### VEC-01 – VectorStore abstraction & wiring
### VEC-02 – First concrete backend (pgvector or Qdrant)
...

you MUST append a **standard cross-cohesion block** at the end of that section with the exact heading:

#### Cross-Cohesion

The block MUST follow this structure:

```markdown
#### Cross-Cohesion

- **Engineering Targets:** A1, A3, B
- **Crates & Paths:**
  - `lib/platform/vector_store` (`dfps_vector_store`)
  - `lib/domain/mapping` (`dfps_mapping`)
- **Shared Metrics & Signals:**
  - Geometry: `geom_rm`, `geom_dm`, `geom_rm_sqrt_dm`, `geom_centroid_cos`
  - Mapping: `auto_mapped`, `needs_review`, `no_match`
  - Vector infra: `vector_queries`, `vector_hits`, `vector_fallbacks`, `vector_latency_ms_p50`, `vector_latency_ms_p95`
  - Graph health: `graph_communities_count`, `graph_leiden_bad_communities`, `graph_modularity`, `graph_conductance_mean`
- **Docs & Kanbans Touched:**
  - `docs/system-design/clinical/ncit/architecture/system-architecture.md`
  - `docs/system-design/clinical/ncit/concepts/vector-layer.md`
  - `docs/kanban/research/math-proofs-and-geometry-docs.md`
- **Experiments / CI Hooks:**
  - `dfps_eval` capacity/geometry snapshot job
  - `dfps_test_suite/tests/integration/vector_mapping.rs`
- **Interfaces & Contracts:**
  - Traits: `VectorStore`
  - CLIs: `dfps_cli build-vector-index`, `dfps_cli map-codes`
  - Env: `DFPS_VECTOR_ENABLED`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_NAMESPACE`
````

IMPORTANT RULES:

1. **Do NOT remove or change any existing content above this block** in the section. You are only appending the `#### Cross-Cohesion` block at the end of each VEC-XX section you modify.

2. You MUST use the same headings and field names in every block:

   * `#### Cross-Cohesion`
   * `Engineering Targets`
   * `Crates & Paths`
   * `Shared Metrics & Signals`
   * `Docs & Kanbans Touched`
   * `Experiments / CI Hooks`
   * `Interfaces & Contracts`

3. For **Engineering Targets**, choose a subset of this fixed vocabulary:

   * `A1` – Ontology embedding construction
   * `A2` – Geometry/flattening/curvature shaping
   * `A3` – Capacity estimation & monitoring
   * `B`  – Mapping Engine behavior / ranking
   * `C`  – Graph health / community structure
   * `D`  – Evaluation harness / CI
     Example: `- **Engineering Targets:** A1, A3, B`

4. For **Shared Metrics & Signals**, you may only use metric names from this fixed set. Select the ones relevant to that card:

   Geometry / capacity metrics:

   * `geom_rm`              (effective radius R_M)
   * `geom_dm`              (effective dimension D_M)
   * `geom_rm_sqrt_dm`      (combined width R_M * sqrt(D_M))
   * `geom_centroid_cos`    (mean NN centroid cosine)
   * `geom_axis_overlap`    (mean subspace overlap)
   * `cap_alpha_sim`        (empirical capacity estimator)
   * `cap_alpha_mf`         (mean-field capacity proxy)

   Mapping metrics:

   * `auto_mapped`
   * `needs_review`
   * `no_match`
   * `mapping_precision`
   * `mapping_recall`
   * `mapping_f1`

   Vector infra metrics:

   * `vector_queries`
   * `vector_hits`
   * `vector_fallbacks`
   * `vector_latency_ms_p50`
   * `vector_latency_ms_p95`

   Graph health metrics:

   * `graph_communities_count`
   * `graph_leiden_bad_communities`
   * `graph_modularity`
   * `graph_conductance_mean`

   You do NOT need to list all metrics in every card; list only the ones that card will log or consume.

5. For **Crates & Paths**, list the specific Rust crates and paths this card touches, using the form:

   * `` `lib/platform/vector_store` (`dfps_vector_store`) ``
   * `` `lib/domain/mapping` (`dfps_mapping`) ``
   * `` `lib/app/cli` (`dfps_cli`) ``
   * `` `lib/domain/eval` (`dfps_eval`) ``
     etc.

6. For **Docs & Kanbans Touched**, include the most relevant docs/kanban files this card interacts with. Use relative paths like:

   * `docs/system-design/clinical/fhir/overview.md`
   * `docs/system-design/clinical/ncit/architecture/system-architecture.md`
   * `docs/system-design/clinical/ncit/concepts/vector-layer.md`
   * `docs/runbook/vector-store-quickstart.md`
   * `docs/kanban/research/math-proofs-and-geometry-docs.md`

7. For **Experiments / CI Hooks**, briefly name:

   * Which `dfps_eval` jobs or evaluation routines will be updated or created.
   * Which `dfps_test_suite` tests will exercise this card (e.g., `tests/integration/vector_mapping.rs`).
   * Any specific CI job names if they are known, otherwise describe them generically.

8. For **Interfaces & Contracts**, list:

   * Relevant traits (e.g., `VectorStore`, `CandidateRanker`).
   * Relevant CLIs (e.g., `dfps_cli build-vector-index`, `dfps_cli map-codes`, `dfps_cli eval-mapping`).
   * Relevant environment variables (e.g., `DFPS_VECTOR_ENABLED`, `DFPS_VECTOR_BACKEND`, `DFPS_VECTOR_URL`, `DFPS_VECTOR_NAMESPACE`).

9. The cross-cohesion block MUST be present for each VEC-XX section you edit. Sections you do not touch can remain without this block.

10. Keep the block concise (5–12 lines of bullets). It is a coordination “handshake”, not a long narrative.

Remember:

* Do NOT rename the metrics or targets.
* Do NOT move or remove existing Kanban content.
* ONLY append this standardized `#### Cross-Cohesion` block to each Kanban card section you modify.