pub mod dim;
pub mod fact;
pub mod keys;
pub mod port;
pub mod sql;

use std::collections::{BTreeMap, HashMap};

use dfps_contracts::{MappingState, PipelineOutput, StgServiceRequestFlat};
use dfps_core::{
    encounter::Encounter,
    mapping::CodeElement,
    patient::Patient,
    value::{EncounterId, PatientId},
};

pub use dfps_contracts::LoadSummary;
pub use dim::*;
pub use fact::*;
pub use keys::*;
pub use port::{DatamartError, DatamartSink, SqliteDatamart};
pub use sql::{
    CohortFilters, LoadError, WarehouseConfig, cohort, connect_sqlite, ddl_statements,
    load_from_pipeline_output, load_streaming_iter, migrate, ncit_summary,
};

#[derive(Debug, Default, Clone)]
pub struct Dims {
    pub patients: Vec<DimPatient>,
    pub encounters: Vec<DimEncounter>,
    pub codes: Vec<DimCode>,
    pub ncit: Vec<DimNCIT>,
}

pub fn from_pipeline_output(output: &PipelineOutput) -> (Dims, Vec<FactServiceRequest>) {
    let mut patient_lookup: HashMap<String, DimPatientKey> = HashMap::new();
    let mut encounter_lookup: HashMap<String, DimEncounterKey> = HashMap::new();
    let mut code_lookup: HashMap<String, (DimCodeKey, String)> = HashMap::new();
    let mut ncit_lookup: HashMap<String, DimNCITKey> = HashMap::new();
    let mut sr_lookup: HashMap<String, &StgServiceRequestFlat> = HashMap::new();

    let mut patient_dims: BTreeMap<u64, DimPatient> = BTreeMap::new();
    let mut encounter_dims: BTreeMap<u64, DimEncounter> = BTreeMap::new();
    let mut code_dims: BTreeMap<u64, DimCode> = BTreeMap::new();
    let mut ncit_dims: BTreeMap<u64, DimNCIT> = BTreeMap::new();

    for flat in &output.flats {
        sr_lookup.insert(flat.sr_id.clone(), flat);
        let patient_key = DimPatientKey::from_patient_id(&flat.patient_id);
        patient_lookup.insert(flat.patient_id.clone(), patient_key);
        patient_dims.entry(patient_key.0).or_insert_with(|| {
            let patient = Patient::new(PatientId::new(flat.patient_id.clone()));
            DimPatient::from_patient(&patient)
        });

        if let Some(encounter_id) = &flat.encounter_id {
            let encounter_key = DimEncounterKey::from_encounter_id(encounter_id);
            encounter_lookup.insert(encounter_id.clone(), encounter_key);
            encounter_dims.entry(encounter_key.0).or_insert_with(|| {
                let encounter = Encounter::new(
                    EncounterId::new(encounter_id.clone()),
                    PatientId::new(flat.patient_id.clone()),
                );
                DimEncounter::from_encounter(&encounter, patient_key)
            });
        }
    }

    for exploded in &output.exploded_codes {
        let element = CodeElement::from(exploded);
        code_lookup.entry(element.id.clone()).or_insert_with(|| {
            let dim = DimCode::from_code_element(exploded, &element);
            let key = dim.key;
            code_dims.entry(key.0).or_insert(dim);
            (key, exploded.sr_id.clone())
        });
    }

    for concept in &output.dim_concepts {
        let key = DimNCITKey::from_ncit_id(&concept.ncit_id);
        ncit_lookup.insert(concept.ncit_id.clone(), key);
        ncit_dims
            .entry(key.0)
            .or_insert_with(|| DimNCIT::from_concept(concept));
    }

    let no_match_key = DimNCITKey::no_match();
    let mut facts = Vec::new();

    for result in &output.mapping_results {
        if let Some((code_key, sr_id)) = code_lookup.get(&result.code_element_id)
            && let Some(flat) = sr_lookup.get(sr_id)
        {
            let patient_key = patient_lookup[&flat.patient_id];
            let encounter_key = flat
                .encounter_id
                .as_ref()
                .and_then(|id| encounter_lookup.get(id).copied());

            let ncit_key = match (result.state, result.ncit_id.as_ref()) {
                (MappingState::NoMatch, _) | (_, None) => {
                    ncit_dims
                        .entry(no_match_key.0)
                        .or_insert_with(DimNCIT::no_match);
                    Some(no_match_key)
                }
                (_, Some(id)) => {
                    let entry = ncit_lookup.entry(id.clone()).or_insert_with(|| {
                        let key = DimNCITKey::from_ncit_id(id);
                        ncit_dims
                            .entry(key.0)
                            .or_insert_with(|| DimNCIT::unknown(id));
                        key
                    });
                    Some(*entry)
                }
            };

            facts.push(FactServiceRequest {
                sr_id: flat.sr_id.clone(),
                patient_key,
                encounter_key,
                code_key: *code_key,
                ncit_key,
                mapping_state: mapping_state_label(result.state).to_string(),
                status: flat.status.clone(),
                intent: flat.intent.clone(),
                description: flat.description.clone(),
                ordered_at: flat.ordered_at.clone(),
            });
        }
    }

    let dims = Dims {
        patients: patient_dims.into_values().collect(),
        encounters: encounter_dims.into_values().collect(),
        codes: code_dims.into_values().collect(),
        ncit: ncit_dims.into_values().collect(),
    };

    (dims, facts)
}

fn mapping_state_label(state: MappingState) -> &'static str {
    match state {
        MappingState::AutoMapped => "auto_mapped",
        MappingState::NeedsReview => "needs_review",
        MappingState::NoMatch => "no_match",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dfps_contracts::{DimNCITConcept, MappingResult, MappingState, StgSrCodeExploded};
    use dfps_core::{
        clinical::order::{ServiceRequestIntent, ServiceRequestStatus},
        mapping::{MappingSourceVersion, MappingStrategy, MappingThresholds},
        staging::StgServiceRequestFlat,
    };

    fn sample_output() -> PipelineOutput {
        PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-1".into(),
                patient_id: "PAT-1".into(),
                encounter_id: Some("ENC-1".into()),
                status: "active".into(),
                status_enum: ServiceRequestStatus::Active,
                intent: "order".into(),
                intent_enum: ServiceRequestIntent::Order,
                description: "PET-CT".into(),
                ordered_at: Some("2024-05-01T12:00:00Z".into()),
            }],
            exploded_codes: vec![StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://loinc.org".into()),
                code: Some("24606-6".into()),
                display: Some("FDG uptake".into()),
            }],
            mapping_results: vec![MappingResult {
                code_element_id: "SR-1::http://loinc.org::24606-6".into(),
                cui: Some("C0001".into()),
                ncit_id: Some("C1234".into()),
                score: 0.98,
                strategy: MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("ncit", "umls"),
                reason: None,
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: vec![DimNCITConcept {
                ncit_id: "C1234".into(),
                preferred_name: "FDG Uptake".into(),
                semantic_group: "Procedure".into(),
            }],
            vector_usage: None,
        }
    }

    #[test]
    fn builds_dims_and_facts() {
        let output = sample_output();
        let (dims, facts) = from_pipeline_output(&output);
        assert_eq!(dims.patients.len(), 1);
        assert_eq!(dims.encounters.len(), 1);
        assert_eq!(dims.codes.len(), 1);
        assert_eq!(dims.ncit.len(), 1);
        assert_eq!(facts.len(), 1);
        let fact = &facts[0];
        assert!(fact.ncit_key.is_some());
        assert_eq!(fact.status, "active");
        assert_eq!(fact.mapping_state, "auto_mapped");
    }

    fn sample_no_match_output() -> PipelineOutput {
        PipelineOutput {
            flats: vec![StgServiceRequestFlat {
                sr_id: "SR-2".into(),
                patient_id: "PAT-1".into(),
                encounter_id: None,
                status: "active".into(),
                status_enum: ServiceRequestStatus::Active,
                intent: "order".into(),
                intent_enum: ServiceRequestIntent::Order,
                description: "Unknown code".into(),
                ordered_at: None,
            }],
            exploded_codes: vec![StgSrCodeExploded {
                sr_id: "SR-2".into(),
                system: Some("http://example.test/system".into()),
                code: Some("UNKNOWN".into()),
                display: Some("Unknown code".into()),
            }],
            mapping_results: vec![MappingResult {
                code_element_id: "SR-2::http://example.test/system::UNKNOWN".into(),
                cui: None,
                ncit_id: None,
                score: 0.0,
                strategy: MappingStrategy::Lexical,
                state: MappingState::NoMatch,
                thresholds: MappingThresholds::default(),
                source_version: MappingSourceVersion::new("ncit", "umls"),
                reason: Some("unknown_code_system".into()),
                license_tier: None,
                source_kind: None,
            }],
            dim_concepts: vec![],
            vector_usage: None,
        }
    }

    #[test]
    fn no_match_results_create_shared_dim() {
        let output = sample_no_match_output();
        let (dims, facts) = from_pipeline_output(&output);
        assert_eq!(facts.len(), 1);
        let fact = &facts[0];
        let ncit_key = fact.ncit_key.expect("no-match key should exist");
        let sentinel = dims
            .ncit
            .iter()
            .find(|dim| dim.key == ncit_key)
            .expect("no-match dim present");
        assert_eq!(sentinel.ncit_id, "NO_MATCH");
        assert_eq!(fact.mapping_state, "no_match");
    }
}
