#![cfg(feature = "profile_validation")]
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
pub struct RawStructureDefinition {
    pub url: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(rename = "baseDefinition")]
    pub base_definition: Option<String>,
    pub snapshot: Option<RawSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSnapshot {
    #[serde(default)]
    pub element: Vec<RawElementDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawElementDefinition {
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
pub struct RawElementType {
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawElementBinding {
    pub strength: Option<String>,
    #[serde(rename = "valueSet")]
    pub value_set: Option<String>,
}
