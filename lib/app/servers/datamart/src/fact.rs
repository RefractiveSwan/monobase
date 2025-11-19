use serde::{Deserialize, Serialize};

use crate::keys::{DimCodeKey, DimEncounterKey, DimNCITKey, DimPatientKey};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactServiceRequest {
    pub sr_id: String,
    pub patient_key: DimPatientKey,
    pub encounter_key: Option<DimEncounterKey>,
    pub code_key: DimCodeKey,
    pub ncit_key: Option<DimNCITKey>,
    pub status: String,
    pub intent: String,
    pub description: String,
    pub ordered_at: Option<String>,
}
