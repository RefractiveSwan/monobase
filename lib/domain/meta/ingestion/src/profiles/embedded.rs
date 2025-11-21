#![cfg(feature = "profile_validation")]
use once_cell::sync::Lazy;

use super::{FhirProfile, ProfileError};

pub const PATIENT_PROFILE_URL: &str = "http://hl7.org/fhir/StructureDefinition/Patient";
pub const ENCOUNTER_PROFILE_URL: &str = "http://hl7.org/fhir/StructureDefinition/Encounter";
pub const SERVICE_REQUEST_PROFILE_URL: &str =
    "http://hl7.org/fhir/StructureDefinition/ServiceRequest";

static PATIENT_RAW: &str =
    include_str!("../../../../../../data/clinical/fhir/profiles/Patient.json");
static ENCOUNTER_RAW: &str =
    include_str!("../../../../../../data/clinical/fhir/profiles/Encounter.json");
static SERVICE_REQUEST_RAW: &str =
    include_str!("../../../../../../data/clinical/fhir/profiles/ServiceRequest.json");

static PATIENT_PROFILE: Lazy<FhirProfile> =
    Lazy::new(|| FhirProfile::from_json_str(PATIENT_RAW).expect("Patient profile should parse"));
static ENCOUNTER_PROFILE: Lazy<FhirProfile> = Lazy::new(|| {
    FhirProfile::from_json_str(ENCOUNTER_RAW).expect("Encounter profile should parse")
});
static SERVICE_REQUEST_PROFILE: Lazy<FhirProfile> = Lazy::new(|| {
    FhirProfile::from_json_str(SERVICE_REQUEST_RAW).expect("ServiceRequest profile should parse")
});

/// Return an embedded profile by URL (or shorthand resource type).
pub fn load_profile(url: &str) -> Option<FhirProfile> {
    match normalize_key(url) {
        Some(ProfileKey::Patient) => Some(PATIENT_PROFILE.clone()),
        Some(ProfileKey::Encounter) => Some(ENCOUNTER_PROFILE.clone()),
        Some(ProfileKey::ServiceRequest) => Some(SERVICE_REQUEST_PROFILE.clone()),
        None => None,
    }
}

/// Known profile URLs keyed by resource type.
pub fn known_profile_urls() -> &'static [&'static str] {
    &[
        PATIENT_PROFILE_URL,
        ENCOUNTER_PROFILE_URL,
        SERVICE_REQUEST_PROFILE_URL,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileKey {
    Patient,
    Encounter,
    ServiceRequest,
}

fn normalize_key(url: &str) -> Option<ProfileKey> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "patient" | "http://hl7.org/fhir/structuredefinition/patient" => Some(ProfileKey::Patient),
        "encounter" | "http://hl7.org/fhir/structuredefinition/encounter" => {
            Some(ProfileKey::Encounter)
        }
        "servicerequest" | "http://hl7.org/fhir/structuredefinition/servicerequest" => {
            Some(ProfileKey::ServiceRequest)
        }
        _ => None,
    }
}

/// Helper exposed for testing profile parsing errors.
pub fn parse_profile(raw: &str) -> Result<FhirProfile, ProfileError> {
    FhirProfile::from_json_str(raw)
}
