# Kanban — Project Alignment and Intentions (001)
**Principles:** FOSS-only (GPLv3), reproducibility first, privacy-by-design.  
**Stakeholders:** _Product (TBD) · Clinical (TBD) · Engineering (TBD) · Security/Privacy (TBD)_  
**Decision records:** ADR stubs to be indexed here once authored.

## Kanban
### ALIGN-01 — Principles
- [ ] Publish core principles: GPLv3/FOSS-only dependencies; deterministic builds; reproducible pipelines; observability-first.
- [ ] Reference in feature Kanbans and README.

### ALIGN-02 — Licensing
- [ ] Document GPLv3 stance, acceptable licenses for dependencies (no closed SaaS).
- [ ] Add checklist for new crates/binaries to validate license posture.

### ALIGN-03 — Data policy
- [ ] Define governance for clinical/FHIR/NCIt/OBO data; PHI handling rules; synthetic vs real data separation.
- [ ] State storage/retention boundaries for artifacts and logs.

### ALIGN-04 — Contribution model
- [ ] Describe contributor roles, review expectations, and branch/commit conventions.
- [ ] Note CLA stance (if any) and alignment with GPLv3.

### ALIGN-05 — Security posture
- [ ] Threat model overview; dependency scanning; secrets handling; minimal env surface.
- [ ] Incident response playbook stub and references.

## Acceptance Criteria
- Principles published and referenced by feature Kanbans.
- ADR index present; stakeholders assigned; licensing and data policy clearly stated.
