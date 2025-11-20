use dfps_core::{
    fhir,
    order::{self, ServiceRequestIntent, ServiceRequestStatus},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
    value::{EncounterId, PatientId, ServiceRequestId},
};

use crate::reference;

use super::errors::IngestionError;

/// Convert a FHIR ServiceRequest into staging rows (flat + exploded coding rows).
pub fn sr_to_staging(
    sr: &fhir::ServiceRequest,
) -> Result<(StgServiceRequestFlat, Vec<StgSrCodeExploded>), IngestionError> {
    ensure_resource_type(&sr.resource_type, "ServiceRequest")?;
    let sr_id = sr
        .id
        .clone()
        .ok_or(IngestionError::MissingField("ServiceRequest.id"))?;

    let patient_id = sr
        .subject
        .as_ref()
        .ok_or(IngestionError::MissingField("ServiceRequest.subject"))?;
    let patient_id = reference::reference_id(patient_id).ok_or(
        IngestionError::InvalidReference("ServiceRequest.subject.reference"),
    )?;

    let encounter_id = match sr.encounter.as_ref() {
        Some(reference) => Some(reference::reference_id(reference).ok_or(
            IngestionError::InvalidReference("ServiceRequest.encounter.reference"),
        )?),
        None => None,
    };

    let (status, status_enum) = parse_status(sr.status.as_deref())?;
    let (intent, _) = parse_intent(sr.intent.as_deref(), status_enum)?;
    let description = description_from_sr(sr);

    let flat = StgServiceRequestFlat {
        sr_id: sr_id.clone(),
        patient_id,
        encounter_id,
        status,
        intent,
        description,
        ordered_at: sr.authored_on.clone(),
    };

    let exploded = sr
        .code
        .as_ref()
        .map(|code| code.coding.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|coding| StgSrCodeExploded {
            sr_id: sr_id.clone(),
            system: coding.system,
            code: coding.code,
            display: coding.display,
        })
        .collect();

    Ok((flat, exploded))
}

/// Normalize a FHIR ServiceRequest into the domain aggregate.
pub fn sr_to_domain(sr: &fhir::ServiceRequest) -> Result<order::ServiceRequest, IngestionError> {
    ensure_resource_type(&sr.resource_type, "ServiceRequest")?;
    let sr_id = sr
        .id
        .as_deref()
        .ok_or(IngestionError::MissingField("ServiceRequest.id"))?;
    let patient_reference = sr
        .subject
        .as_ref()
        .ok_or(IngestionError::MissingField("ServiceRequest.subject"))?;
    let patient_id = reference::reference_id(patient_reference).ok_or(
        IngestionError::InvalidReference("ServiceRequest.subject.reference"),
    )?;

    let encounter_id = match sr.encounter.as_ref() {
        Some(reference) => Some(EncounterId::new(reference::reference_id(reference).ok_or(
            IngestionError::InvalidReference("ServiceRequest.encounter.reference"),
        )?)),
        None => None,
    };

    let (_, status) = parse_status(sr.status.as_deref())?;
    let (_, intent) = parse_intent(sr.intent.as_deref(), status)?;
    let description = description_from_sr(sr);

    Ok(order::ServiceRequest::new(
        ServiceRequestId::new(sr_id.to_string()),
        PatientId::new(patient_id),
        encounter_id,
        status,
        intent,
        description,
    ))
}

pub fn description_from_sr(sr: &fhir::ServiceRequest) -> String {
    sr.description
        .clone()
        .or_else(|| sr.code.as_ref().and_then(|code| code.text.clone()))
        .or_else(|| {
            sr.code
                .as_ref()
                .and_then(|code| code.coding.iter().find_map(|coding| coding.display.clone()))
        })
        .unwrap_or_else(|| "unspecified service request".to_string())
}

pub fn parse_status(value: Option<&str>) -> Result<(String, ServiceRequestStatus), IngestionError> {
    let raw = value.ok_or(IngestionError::MissingField("ServiceRequest.status"))?;
    let normalized = raw.to_ascii_lowercase();
    let status = match normalized.as_str() {
        "draft" => ServiceRequestStatus::Draft,
        "on-hold" | "on_hold" => ServiceRequestStatus::OnHold,
        "completed" => ServiceRequestStatus::Completed,
        "cancelled" | "canceled" => ServiceRequestStatus::Cancelled,
        "revoked" => ServiceRequestStatus::Revoked,
        "entered-in-error" | "entered_in_error" => ServiceRequestStatus::EnteredInError,
        "active" => ServiceRequestStatus::Active,
        other => return Err(IngestionError::InvalidStatus(other.to_string())),
    };
    Ok((normalized, status))
}

pub fn parse_intent(
    value: Option<&str>,
    status: ServiceRequestStatus,
) -> Result<(String, ServiceRequestIntent), IngestionError> {
    let raw = value.ok_or(IngestionError::MissingField("ServiceRequest.intent"))?;
    let normalized = raw.to_ascii_lowercase();
    let intent = match normalized.as_str() {
        "proposal" => ServiceRequestIntent::Proposal,
        "plan" => ServiceRequestIntent::Plan,
        "order" => ServiceRequestIntent::Order,
        "original-order" | "original_order" => ServiceRequestIntent::OriginalOrder,
        "reflex-order" | "reflex_order" => ServiceRequestIntent::ReflexOrder,
        "filler-order" | "filler_order" => ServiceRequestIntent::FillerOrder,
        other => return Err(IngestionError::InvalidIntent(other.to_string())),
    };

    let coerced = match status {
        ServiceRequestStatus::Draft => match intent {
            ServiceRequestIntent::Plan | ServiceRequestIntent::Proposal => intent,
            _ => ServiceRequestIntent::Plan,
        },
        ServiceRequestStatus::Completed | ServiceRequestStatus::Cancelled => {
            ServiceRequestIntent::Order
        }
        _ => intent,
    };
    Ok((normalized, coerced))
}

pub fn ensure_resource_type(actual: &str, expected: &'static str) -> Result<(), IngestionError> {
    if actual != expected {
        return Err(IngestionError::InvalidResourceType {
            expected,
            found: actual.to_string(),
        });
    }
    Ok(())
}
