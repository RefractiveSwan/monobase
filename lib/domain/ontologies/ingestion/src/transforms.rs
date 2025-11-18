use dfps_core::{
    fhir,
    order::{self, ServiceRequestIntent, ServiceRequestStatus},
    staging::{StgServiceRequestFlat, StgSrCodeExploded},
    value::{EncounterId, PatientId, ServiceRequestId},
};
use serde_json::Error as SerdeError;

use crate::{
    reference,
    validation::{
        ExternalValidationContext, Validated, ValidationIssue, ValidationMode, validate_bundle,
        validate_bundle_with_external,
    },
};

/// Errors surfaced while normalizing raw FHIR payloads.
#[derive(Debug)]
pub enum IngestionError {
    MissingField(&'static str),
    InvalidReference(&'static str),
    InvalidResourceType {
        expected: &'static str,
        found: String,
    },
    InvalidStatus(String),
    InvalidIntent(String),
    Decode(SerdeError),
    ValidationFailed(Vec<ValidationIssue>),
}

impl std::fmt::Display for IngestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing required field: {field}"),
            Self::InvalidReference(field) => write!(f, "invalid reference format for {field}"),
            Self::InvalidResourceType { expected, found } => {
                write!(f, "invalid resourceType '{found}', expected '{expected}'")
            }
            Self::InvalidStatus(value) => write!(f, "invalid status value '{value}'"),
            Self::InvalidIntent(value) => write!(f, "invalid intent value '{value}'"),
            Self::Decode(err) => write!(f, "failed to decode resource: {err}"),
            Self::ValidationFailed(issues) => {
                write!(f, "validation failed with {} issue(s)", issues.len())
            }
        }
    }
}

impl std::error::Error for IngestionError {}

impl From<SerdeError> for IngestionError {
    fn from(value: SerdeError) -> Self {
        Self::Decode(value)
    }
}

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
        Some(reference) => Some(EncounterId(reference::reference_id(reference).ok_or(
            IngestionError::InvalidReference("ServiceRequest.encounter.reference"),
        )?)),
        None => None,
    };

    let (_, status) = parse_status(sr.status.as_deref())?;
    let (_, intent) = parse_intent(sr.intent.as_deref(), status)?;
    let description = description_from_sr(sr);

    Ok(order::ServiceRequest::new(
        ServiceRequestId(sr_id.to_string()),
        PatientId(patient_id),
        encounter_id,
        status,
        intent,
        description,
    ))
}

/// Convert a bundle into staging row collections.
pub fn bundle_to_staging(
    bundle: &fhir::Bundle,
) -> Result<(Vec<StgServiceRequestFlat>, Vec<StgSrCodeExploded>), IngestionError> {
    bundle_to_staging_with_validation(
        bundle,
        ValidationMode::default(),
        ExternalValidationContext::default(),
    )
    .map(|validated| validated.value)
}

/// Convert a bundle into staging rows, returning validation metadata.
pub fn bundle_to_staging_with_validation(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    external: ExternalValidationContext<'_>,
) -> Result<Validated<(Vec<StgServiceRequestFlat>, Vec<StgSrCodeExploded>)>, IngestionError> {
    let report = match mode {
        ValidationMode::Strict | ValidationMode::Lenient => validate_bundle(bundle),
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict => {
            validate_bundle_with_external(bundle, mode, external)
        }
    };
    if matches!(
        mode,
        ValidationMode::Strict | ValidationMode::ExternalStrict
    ) && report.has_errors()
    {
        return Err(IngestionError::ValidationFailed(report.issues.clone()));
    }
    let (flats, exploded) = bundle_to_staging_inner(bundle)?;
    Ok(Validated::new((flats, exploded), report))
}

fn bundle_to_staging_inner(
    bundle: &fhir::Bundle,
) -> Result<(Vec<StgServiceRequestFlat>, Vec<StgSrCodeExploded>), IngestionError> {
    let mut flats = Vec::new();
    let mut exploded = Vec::new();

    for entry in bundle.iter_servicerequests() {
        let sr = entry?;
        let (flat, codes) = sr_to_staging(&sr)?;
        flats.push(flat);
        exploded.extend(codes);
    }

    Ok((flats, exploded))
}

/// Convert a bundle into domain ServiceRequest aggregates.
pub fn bundle_to_domain(
    bundle: &fhir::Bundle,
) -> Result<Vec<order::ServiceRequest>, IngestionError> {
    bundle_to_domain_with_validation(
        bundle,
        ValidationMode::default(),
        ExternalValidationContext::default(),
    )
    .map(|validated| validated.value)
}

/// Convert a bundle into domain ServiceRequest aggregates, returning validation metadata.
pub fn bundle_to_domain_with_validation(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    external: ExternalValidationContext<'_>,
) -> Result<Validated<Vec<order::ServiceRequest>>, IngestionError> {
    let report = match mode {
        ValidationMode::Strict | ValidationMode::Lenient => validate_bundle(bundle),
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict => {
            validate_bundle_with_external(bundle, mode, external)
        }
    };
    if matches!(
        mode,
        ValidationMode::Strict | ValidationMode::ExternalStrict
    ) && report.has_errors()
    {
        return Err(IngestionError::ValidationFailed(report.issues.clone()));
    }
    let output = bundle_to_domain_inner(bundle)?;
    Ok(Validated::new(output, report))
}

fn bundle_to_domain_inner(
    bundle: &fhir::Bundle,
) -> Result<Vec<order::ServiceRequest>, IngestionError> {
    let mut output = Vec::new();
    for entry in bundle.iter_servicerequests() {
        let sr = entry?;
        output.push(sr_to_domain(&sr)?);
    }
    Ok(output)
}

fn description_from_sr(sr: &fhir::ServiceRequest) -> String {
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

fn parse_status(value: Option<&str>) -> Result<(String, ServiceRequestStatus), IngestionError> {
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

fn parse_intent(
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

fn ensure_resource_type(actual: &str, expected: &'static str) -> Result<(), IngestionError> {
    if actual != expected {
        return Err(IngestionError::InvalidResourceType {
            expected,
            found: actual.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_prefers_sr_field() {
        let sr = fhir::ServiceRequest {
            resource_type: "ServiceRequest".into(),
            id: Some("sr".into()),
            status: Some("active".into()),
            intent: Some("order".into()),
            subject: Some(fhir::Reference {
                reference: Some("Patient/p1".into()),
                display: None,
            }),
            encounter: None,
            requester: None,
            supporting_info: vec![],
            code: Some(fhir::CodeableConcept {
                coding: vec![fhir::Coding {
                    system: Some("http://snomed.info/sct".into()),
                    code: Some("123".into()),
                    display: Some("PET".into()),
                }],
                text: Some("PET CT".into()),
            }),
            category: vec![],
            description: Some("Preferred".into()),
            authored_on: Some("2024-05-01T12:00:00Z".into()),
        };

        assert_eq!(description_from_sr(&sr), "Preferred");
    }

    #[test]
    fn invalid_resource_type_errors() {
        let mut sr = minimal_sr();
        sr.resource_type = "Observation".into();
        let err = sr_to_staging(&sr).expect_err("expected resourceType error");
        matches!(
            err,
            IngestionError::InvalidResourceType {
                expected: "ServiceRequest",
                ..
            }
        );
    }

    #[test]
    fn invalid_status_errors() {
        let mut sr = minimal_sr();
        sr.status = Some("bogus".into());
        let err = sr_to_domain(&sr).expect_err("expected invalid status");
        matches!(err, IngestionError::InvalidStatus(value) if value == "bogus");
    }

    #[test]
    fn invalid_intent_errors() {
        let mut sr = minimal_sr();
        sr.intent = Some("weird".into());
        let err = sr_to_domain(&sr).expect_err("expected invalid intent");
        matches!(err, IngestionError::InvalidIntent(value) if value == "weird");
    }

    #[test]
    fn strict_validation_blocks_bundle_to_staging() {
        let bundle = bundle_missing_patient_resource();
        let err = bundle_to_staging_with_validation(
            &bundle,
            ValidationMode::Strict,
            ExternalValidationContext::default(),
        )
        .expect_err("strict validation should fail");
        matches!(err, IngestionError::ValidationFailed(issues) if !issues.is_empty());

        let validated = bundle_to_staging_with_validation(
            &bundle,
            ValidationMode::Lenient,
            ExternalValidationContext::default(),
        )
        .expect("lenient mode");
        assert!(validated.report.has_errors());
    }

    #[test]
    fn strict_validation_blocks_bundle_to_domain() {
        let bundle = bundle_missing_patient_resource();
        let err = bundle_to_domain_with_validation(
            &bundle,
            ValidationMode::Strict,
            ExternalValidationContext::default(),
        )
        .expect_err("strict validation should fail");
        matches!(err, IngestionError::ValidationFailed(issues) if !issues.is_empty());
    }

    fn minimal_sr() -> fhir::ServiceRequest {
        fhir::ServiceRequest {
            resource_type: "ServiceRequest".into(),
            id: Some("sr-1".into()),
            status: Some("active".into()),
            intent: Some("order".into()),
            subject: Some(fhir::Reference {
                reference: Some("Patient/p1".into()),
                display: None,
            }),
            encounter: None,
            requester: None,
            supporting_info: vec![],
            code: None,
            category: vec![],
            description: None,
            authored_on: Some("2024-05-01T12:00:00Z".into()),
        }
    }

    fn bundle_missing_patient_resource() -> fhir::Bundle {
        fhir::Bundle {
            resource_type: "Bundle".into(),
            bundle_type: Some("collection".into()),
            entry: vec![fhir::BundleEntry {
                full_url: None,
                resource: Some(serde_json::json!({
                    "resourceType": "ServiceRequest",
                    "id": "SR-MISSING-PAT",
                    "status": "active",
                    "intent": "order",
                    "subject": { "reference": "Patient/PAT-404" }
                })),
            }],
        }
    }
}
