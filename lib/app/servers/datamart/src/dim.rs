use serde::{Deserialize, Serialize};

use crate::keys::{DimCodeKey, DimEncounterKey, DimNCITKey, DimPatientKey};
use dfps_core::{
    encounter::Encounter,
    mapping::{CodeElement, DimNCITConcept},
    patient::Patient,
    staging::StgSrCodeExploded,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimPatient {
    pub key: DimPatientKey,
    pub patient_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimEncounter {
    pub key: DimEncounterKey,
    pub encounter_id: String,
    pub patient_key: DimPatientKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimCode {
    pub key: DimCodeKey,
    pub sr_id: String,
    pub code_element_id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimNCIT {
    pub key: DimNCITKey,
    pub ncit_id: String,
    pub preferred_name: String,
    pub semantic_group: String,
}

impl DimPatient {
    pub fn from_patient(patient: &Patient) -> Self {
        let patient_id = patient.id.0.clone();
        Self {
            key: DimPatientKey::from_patient_id(&patient_id),
            patient_id,
        }
    }
}

impl DimEncounter {
    pub fn from_encounter(encounter: &Encounter, patient_key: DimPatientKey) -> Self {
        let encounter_id = encounter.id.0.clone();
        Self {
            key: DimEncounterKey::from_encounter_id(&encounter_id),
            encounter_id,
            patient_key,
        }
    }
}

impl DimCode {
    pub fn from_staging(row: &StgSrCodeExploded) -> Self {
        let element = CodeElement::from(row);
        Self::from_code_element(row, &element)
    }

    pub fn from_code_element(row: &StgSrCodeExploded, element: &CodeElement) -> Self {
        Self {
            key: DimCodeKey::from_code_element_id(&element.id),
            sr_id: row.sr_id.clone(),
            code_element_id: element.id.clone(),
            system: element.system.clone(),
            code: element.code.clone(),
            display: element.display.clone(),
        }
    }
}

impl DimNCIT {
    pub fn from_concept(concept: &DimNCITConcept) -> Self {
        Self {
            key: DimNCITKey::from_ncit_id(&concept.ncit_id),
            ncit_id: concept.ncit_id.clone(),
            preferred_name: concept.preferred_name.clone(),
            semantic_group: concept.semantic_group.clone(),
        }
    }

    pub fn no_match() -> Self {
        Self {
            key: DimNCITKey::no_match(),
            ncit_id: "NO_MATCH".into(),
            preferred_name: "Unmapped (NoMatch)".into(),
            semantic_group: "NoMatch".into(),
        }
    }

    pub fn unknown(id: &str) -> Self {
        Self {
            key: DimNCITKey::from_ncit_id(id),
            ncit_id: id.to_string(),
            preferred_name: "Unknown NCIt concept".into(),
            semantic_group: "Unknown".into(),
        }
    }
}
