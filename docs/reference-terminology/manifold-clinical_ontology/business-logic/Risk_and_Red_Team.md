\
# VII. Risk & Red‑Team Findings

- **Mean‑field mismatch.** If \(\alpha_{\text{mf}}\) diverges from \(\alpha_{\text{sim}}\) by >30%, assumptions (random labels, Gaussian correlations) fail. *Mitigation:* default to \(\alpha_{\text{sim}}\); project out low‑rank centroid structure; use robust anchors. (A2)
- **Synonym noise.** Noisy paraphrases inflate \(R_M\). *Mitigation:* synonym quality scoring; roll back if \(R_M\sqrt{D_M}\) ↑ by >20% after update.
- **Graph fragmentation.** Louvain partitions with disconnected communities inflate \(D_M\)/\(R_M\). *Mitigation:* enforce Leiden; audit disconnectedness.
- **Over‑flattening.** Excess flattening collapses class topology. *Mitigation:* add reconstruction/topology terms (as in S7); monitor triplet‑loss violations.
- **License constraints.** Licensed code systems (e.g., CPT) restrict redistribution. *Mitigation:* track license tiers via `refractive_swan_terminology` and mask artifacts accordingly.
