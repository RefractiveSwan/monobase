use std::collections::HashMap;

use refractive_swan_core::fhir;

use crate::profiles::{ElementDefinition as ProfileElement, FhirProfile};

use super::{RequirementRef, ValidationIssue, ValidationSeverity};

/// Map RequirementRefs to specific profile element definitions for traceability.
pub fn profile_requirement_links(profile: &FhirProfile) -> Vec<(RequirementRef, ProfileElement)> {
    [
        (RequirementRef::RSubject, "ServiceRequest.subject"),
        (RequirementRef::RStatus, "ServiceRequest.status"),
        (RequirementRef::RTrace, "ServiceRequest.id"),
    ]
    .iter()
    .filter_map(|(req, path)| profile.element_by_path(path).cloned().map(|el| (*req, el)))
    .collect()
}

/// Profile-driven checks for ServiceRequest cardinalities and required elements.
pub fn validate_sr_profile(
    sr: &fhir::ServiceRequest,
    profile: &FhirProfile,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let requirement_lookup = build_requirement_lookup(profile);

    for element in &profile.elements {
        if !element.path.starts_with("ServiceRequest.") {
            continue;
        }
        let min = element.min.unwrap_or(0);
        if min > 0 && !sr_field_present(sr, &element.path) {
            let requirement = requirement_lookup
                .get(element.path.as_str())
                .copied()
                .unwrap_or(RequirementRef::RTrace);
            let id = profile_issue_id(&element.path, "MISSING");
            let message = format!(
                "{} required by profile {} is missing.",
                element.path, profile.meta.url
            );
            issues.push(ValidationIssue::new(
                id,
                ValidationSeverity::Error,
                message,
                requirement,
            ));
        }
        if let Some(max) = element.max.as_deref()
            && max == "0"
            && sr_field_present(sr, &element.path)
        {
            let requirement = requirement_lookup
                .get(element.path.as_str())
                .copied()
                .unwrap_or(RequirementRef::RTrace);
            let id = profile_issue_id(&element.path, "NOT_ALLOWED");
            let message = format!(
                "{} is not permitted by profile {}.",
                element.path, profile.meta.url
            );
            issues.push(ValidationIssue::new(
                id,
                ValidationSeverity::Error,
                message,
                requirement,
            ));
        }
    }

    issues
}

fn build_requirement_lookup(profile: &FhirProfile) -> HashMap<String, RequirementRef> {
    profile_requirement_links(profile)
        .into_iter()
        .map(|(req, el)| (el.path.clone(), req))
        .collect()
}

fn sr_field_present(sr: &fhir::ServiceRequest, path: &str) -> bool {
    match path {
        "ServiceRequest.id" => sr.id.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.status" => sr.status.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.intent" => sr.intent.as_deref().map(|v| !v.is_empty()).unwrap_or(false),
        "ServiceRequest.subject" => sr
            .subject
            .as_ref()
            .and_then(|r| r.reference.as_deref())
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        "ServiceRequest.encounter" => sr
            .encounter
            .as_ref()
            .and_then(|r| r.reference.as_deref())
            .map(|v| !v.is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

fn profile_issue_id(path: &str, suffix: &str) -> String {
    let key = path
        .trim_start_matches("ServiceRequest.")
        .replace('.', "_")
        .to_ascii_uppercase();
    format!("VAL_SR_PROFILE_{}_{}", key, suffix)
}
