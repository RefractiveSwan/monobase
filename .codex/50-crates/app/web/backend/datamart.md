# Crate: lib/app/servers/datamart — `dfps_datamart`

**Purpose**  
Build a small star schema from `PipelineOutput` for analytics/UI rendering.

**Key types**
- `Dims { patients, encounters, codes, ncit }` (all deduped via `BTreeMap`)
- `DimPatient`, `DimEncounter`, `DimCode`, `DimNCIT`
- `FactServiceRequest { sr_id, patient_key, encounter_key, code_key, ncit_key, status, intent, description, ordered_at }`

**Keys**
- `DimPatientKey::from_patient_id`
- `DimEncounterKey::from_encounter_id`
- `DimCodeKey::from_code_element_id`
- `DimNCITKey::from_ncit_id` / `DimNCITKey::no_match()`

**Behavior**
- Code dims derive from `CodeElement::from(StgSrCodeExploded)`.
- Missing or `NoMatch` → `ncit_key = NO_MATCH` sentinel with `ncit_id="NO_MATCH"`.
- Returns `(Dims, Vec<FactServiceRequest>)`.

**Tests**
- Integrity + NO_MATCH sentinel coverage included.
