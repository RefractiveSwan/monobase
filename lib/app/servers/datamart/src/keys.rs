use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimPatientKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimEncounterKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimCodeKey(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DimNCITKey(pub u64);

impl DimPatientKey {
    pub fn from_patient_id(patient_id: &str) -> Self {
        DimPatientKey(stable_key(&["patient", patient_id]))
    }
}

impl DimEncounterKey {
    pub fn from_encounter_id(encounter_id: &str) -> Self {
        DimEncounterKey(stable_key(&["encounter", encounter_id]))
    }
}

impl DimCodeKey {
    pub fn from_code_element_id(element_id: &str) -> Self {
        DimCodeKey(stable_key(&["code", element_id]))
    }
}

impl DimNCITKey {
    pub fn from_ncit_id(ncit_id: &str) -> Self {
        DimNCITKey(stable_key(&["ncit", ncit_id]))
    }

    pub fn no_match() -> Self {
        DimNCITKey(stable_key(&["ncit", "__no_match__"]))
    }
}

fn stable_key(parts: &[&str]) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}
