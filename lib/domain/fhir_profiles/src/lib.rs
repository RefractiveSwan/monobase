//! Embedded FHIR StructureDefinition slices used for profile-aware validation.
//! The goal is a lightweight representation of the DFPS-relevant profiles
//! (Patient, Encounter, ServiceRequest) without pulling in a full FHIR engine.
//! See:
//! - docs/system-design/clinical/fhir/requirements/ingestion-requirements.md
//! - docs/system-design/clinical/fhir/overview.md
//! - docs/runbook/fhir-profiles-quickstart.md

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub const PATIENT_PROFILE_URL: &str = "http://hl7.org/fhir/StructureDefinition/Patient";
pub const ENCOUNTER_PROFILE_URL: &str = "http://hl7.org/fhir/StructureDefinition/Encounter";
pub const SERVICE_REQUEST_PROFILE_URL: &str =
    "http://hl7.org/fhir/StructureDefinition/ServiceRequest";

static PATIENT_RAW: &str = include_str!("../../../../data/fhir/profiles/Patient.json");
static ENCOUNTER_RAW: &str = include_str!("../../../../data/fhir/profiles/Encounter.json");
static SERVICE_REQUEST_RAW: &str =
    include_str!("../../../../data/fhir/profiles/ServiceRequest.json");

static PATIENT_PROFILE: Lazy<FhirProfile> =
    Lazy::new(|| FhirProfile::from_json_str(PATIENT_RAW).expect("Patient profile should parse"));
static ENCOUNTER_PROFILE: Lazy<FhirProfile> = Lazy::new(|| {
    FhirProfile::from_json_str(ENCOUNTER_RAW).expect("Encounter profile should parse")
});
static SERVICE_REQUEST_PROFILE: Lazy<FhirProfile> = Lazy::new(|| {
    FhirProfile::from_json_str(SERVICE_REQUEST_RAW).expect("ServiceRequest profile should parse")
});

/// Minimal metadata extracted from a StructureDefinition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub url: String,
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub base_definition: Option<String>,
}

/// Binding information attached to an ElementDefinition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementBinding {
    pub strength: Option<String>,
    pub value_set: Option<String>,
}

/// Simplified ElementDefinition capturing cardinalities and type hints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementDefinition {
    pub id: String,
    pub path: String,
    pub min: Option<u32>,
    pub max: Option<String>,
    pub type_codes: Vec<String>,
    pub must_support: bool,
    pub binding: Option<ElementBinding>,
}

/// Lightweight profile representation (metadata + snapshot elements).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FhirProfile {
    pub meta: ProfileMeta,
    pub elements: Vec<ElementDefinition>,
}

#[derive(Debug)]
pub enum ProfileError {
    MissingSnapshot,
    Parse(serde_json::Error),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileError::MissingSnapshot => {
                write!(f, "profile snapshot is missing from StructureDefinition")
            }
            ProfileError::Parse(err) => {
                write!(f, "failed to parse StructureDefinition JSON: {err}")
            }
        }
    }
}

impl std::error::Error for ProfileError {}

impl FhirProfile {
    /// Build a profile from a raw StructureDefinition JSON string.
    pub fn from_json_str(raw: &str) -> Result<Self, ProfileError> {
        let def: RawStructureDefinition = serde_json::from_str(raw).map_err(ProfileError::Parse)?;
        let snapshot = def.snapshot.ok_or(ProfileError::MissingSnapshot)?;
        let elements = snapshot
            .element
            .into_iter()
            .map(ElementDefinition::from_raw)
            .collect();

        Ok(Self {
            meta: ProfileMeta {
                url: def.url,
                name: def.name.unwrap_or_else(|| def.resource_type.clone()),
                resource_type: def.resource_type,
                base_definition: def.base_definition,
            },
            elements,
        })
    }

    /// Convenience lookup for an element by its FHIR path (e.g. `ServiceRequest.subject`).
    pub fn element_by_path(&self, path: &str) -> Option<&ElementDefinition> {
        self.elements.iter().find(|el| el.path == path)
    }
}

impl ElementDefinition {
    fn from_raw(raw: RawElementDefinition) -> Self {
        Self {
            id: raw
                .id
                .clone()
                .unwrap_or_else(|| raw.path.clone().unwrap_or_default()),
            path: raw.path.unwrap_or_default(),
            min: raw.min,
            max: raw.max,
            type_codes: raw.r#type.into_iter().filter_map(|ty| ty.code).collect(),
            must_support: raw.must_support.unwrap_or(false),
            binding: raw.binding.map(ElementBinding::from_raw),
        }
    }
}

impl ElementBinding {
    fn from_raw(raw: RawElementBinding) -> Self {
        Self {
            strength: raw.strength,
            value_set: raw.value_set,
        }
    }
}

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

#[derive(Debug, Clone, Deserialize)]
struct RawStructureDefinition {
    pub url: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(rename = "baseDefinition")]
    pub base_definition: Option<String>,
    pub snapshot: Option<RawSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSnapshot {
    #[serde(default)]
    pub element: Vec<RawElementDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawElementDefinition {
    pub id: Option<String>,
    pub path: Option<String>,
    pub min: Option<u32>,
    pub max: Option<String>,
    #[serde(default, rename = "type")]
    pub r#type: Vec<RawElementType>,
    #[serde(rename = "mustSupport")]
    pub must_support: Option<bool>,
    pub binding: Option<RawElementBinding>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawElementType {
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawElementBinding {
    pub strength: Option<String>,
    #[serde(rename = "valueSet")]
    pub value_set: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_known_profiles() {
        assert!(load_profile(PATIENT_PROFILE_URL).is_some());
        assert!(load_profile(ENCOUNTER_PROFILE_URL).is_some());
        assert!(load_profile(SERVICE_REQUEST_PROFILE_URL).is_some());
        assert!(load_profile("Unknown").is_none());
    }

    #[test]
    fn parses_snapshot_elements() {
        let profile = load_profile(SERVICE_REQUEST_PROFILE_URL).expect("profile");
        let subject = profile.element_by_path("ServiceRequest.subject").unwrap();
        assert_eq!(subject.min, Some(1));
        assert!(subject.must_support);

        let intent = profile.element_by_path("ServiceRequest.intent").unwrap();
        assert_eq!(intent.min, Some(1));
        assert_eq!(intent.path, "ServiceRequest.intent");
        assert!(intent.binding.is_some());
    }

    #[test]
    fn profile_meta_preserves_urls() {
        let profile = load_profile(PATIENT_PROFILE_URL).unwrap();
        assert_eq!(profile.meta.url, PATIENT_PROFILE_URL.to_string());
        assert_eq!(profile.meta.resource_type, "Patient".to_string());
    }
}
