use serde::{Deserialize, Serialize};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Flattened ServiceRequest row (`stg_servicerequest_flat`).
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StgServiceRequestFlat {
    pub sr_id: String,
    pub patient_id: String,
    pub encounter_id: Option<String>,
    pub status: String,
    pub intent: String,
    pub description: String,
    pub ordered_at: Option<String>,
}

/// Exploded coding row (`stg_sr_code_exploded`) linking back to ServiceRequest.
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StgSrCodeExploded {
    pub sr_id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}
