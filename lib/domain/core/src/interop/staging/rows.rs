use crate::clinical::order::{ServiceRequestIntent, ServiceRequestStatus};
use serde::{Deserialize, Serialize};

#[cfg(feature = "dummy")]
use fake::Dummy;

/// Flattened ServiceRequest row (`stg_servicerequest_flat`).
#[cfg_attr(feature = "dummy", derive(Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StgServiceRequestFlat {
    pub sr_id: String,
    pub patient_id: String,
    pub encounter_id: Option<String>,
    pub status: String,
    pub status_enum: ServiceRequestStatus,
    pub intent: String,
    pub intent_enum: ServiceRequestIntent,
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

impl<'de> Deserialize<'de> for StgServiceRequestFlat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            sr_id: String,
            patient_id: String,
            encounter_id: Option<String>,
            status: String,
            #[serde(default)]
            status_enum: Option<ServiceRequestStatus>,
            intent: String,
            #[serde(default)]
            intent_enum: Option<ServiceRequestIntent>,
            description: String,
            ordered_at: Option<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let status_enum = helper
            .status_enum
            .or_else(|| parse_status_enum(&helper.status))
            .unwrap_or_default();
        let intent_enum = helper
            .intent_enum
            .or_else(|| parse_intent_enum(&helper.intent))
            .unwrap_or_default();

        Ok(StgServiceRequestFlat {
            sr_id: helper.sr_id,
            patient_id: helper.patient_id,
            encounter_id: helper.encounter_id,
            status: helper.status,
            status_enum,
            intent: helper.intent,
            intent_enum,
            description: helper.description,
            ordered_at: helper.ordered_at,
        })
    }
}

fn parse_status_enum(value: &str) -> Option<ServiceRequestStatus> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "draft" => Some(ServiceRequestStatus::Draft),
        "active" => Some(ServiceRequestStatus::Active),
        "on-hold" | "on_hold" => Some(ServiceRequestStatus::OnHold),
        "completed" => Some(ServiceRequestStatus::Completed),
        "cancelled" | "canceled" => Some(ServiceRequestStatus::Cancelled),
        "revoked" => Some(ServiceRequestStatus::Revoked),
        "entered-in-error" | "entered_in_error" => Some(ServiceRequestStatus::EnteredInError),
        _ => None,
    }
}

fn parse_intent_enum(value: &str) -> Option<ServiceRequestIntent> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "proposal" => Some(ServiceRequestIntent::Proposal),
        "plan" => Some(ServiceRequestIntent::Plan),
        "order" => Some(ServiceRequestIntent::Order),
        "original-order" | "original_order" => Some(ServiceRequestIntent::OriginalOrder),
        "reflex-order" | "reflex_order" => Some(ServiceRequestIntent::ReflexOrder),
        "filler-order" | "filler_order" => Some(ServiceRequestIntent::FillerOrder),
        _ => None,
    }
}
