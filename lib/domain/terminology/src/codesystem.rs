use serde::{Deserialize, Serialize};

/// License classification for vocabularies/code systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTier {
    Licensed,
    Open,
    InternalOnly,
}

impl LicenseTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            LicenseTier::Licensed => "licensed",
            LicenseTier::Open => "open",
            LicenseTier::InternalOnly => "internal_only",
        }
    }
}

/// Origin/source classification for a code system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Fhir,
    UMLS,
    OboFoundry,
    Local,
}

impl SourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            SourceKind::Fhir => "fhir",
            SourceKind::UMLS => "umls",
            SourceKind::OboFoundry => "obo_foundry",
            SourceKind::Local => "local",
        }
    }
}

/// Metadata describing a code system/terminology entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeSystemMeta {
    pub url: &'static str,
    pub name: &'static str,
    pub version: Option<&'static str>,
    pub description: &'static str,
    pub license_tier: LicenseTier,
    pub source_kind: SourceKind,
}

impl CodeSystemMeta {
    pub const fn new(
        url: &'static str,
        name: &'static str,
        version: Option<&'static str>,
        description: &'static str,
        license_tier: LicenseTier,
        source_kind: SourceKind,
    ) -> Self {
        Self {
            url,
            name,
            version,
            description,
            license_tier,
            source_kind,
        }
    }
}
