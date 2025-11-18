use dfps_core::staging::StgSrCodeExploded;

use crate::codesystem::{CodeSystemMeta, LicenseTier, SourceKind};
use crate::registry::lookup_codesystem;

#[derive(Debug, Clone)]
pub struct EnrichedCode {
    pub staging: StgSrCodeExploded,
    pub codesystem: Option<&'static CodeSystemMeta>,
    pub license_tier: Option<LicenseTier>,
    pub source_kind: Option<SourceKind>,
    canonical_system: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeKind {
    KnownLicensedSystem,
    KnownOpenSystem,
    OboBacked,
    UnknownSystem,
    MissingSystemOrCode,
}

impl CodeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            CodeKind::KnownLicensedSystem => "known_licensed_system",
            CodeKind::KnownOpenSystem => "known_open_system",
            CodeKind::OboBacked => "obo_backed",
            CodeKind::UnknownSystem => "unknown_system",
            CodeKind::MissingSystemOrCode => "missing_system_or_code",
        }
    }
}

impl EnrichedCode {
    pub fn from_staging(staging: StgSrCodeExploded) -> Self {
        let canonical_system = canonicalize_system(staging.system.as_deref());
        let codesystem = canonical_system
            .as_deref()
            .and_then(|url| lookup_codesystem(url));
        let (license_tier, source_kind) = codesystem
            .map(|meta| (Some(meta.license_tier), Some(meta.source_kind)))
            .unwrap_or((None, None));

        Self {
            staging,
            codesystem,
            license_tier,
            source_kind,
            canonical_system,
        }
    }

    pub fn code_kind(&self) -> CodeKind {
        if self
            .staging
            .system
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
            || self
                .staging
                .code
                .as_deref()
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
        {
            return CodeKind::MissingSystemOrCode;
        }

        if let Some(meta) = self.codesystem {
            match meta.source_kind {
                SourceKind::OboFoundry => CodeKind::OboBacked,
                _ => match meta.license_tier {
                    LicenseTier::Licensed | LicenseTier::InternalOnly => {
                        CodeKind::KnownLicensedSystem
                    }
                    LicenseTier::Open => CodeKind::KnownOpenSystem,
                },
            }
        } else {
            CodeKind::UnknownSystem
        }
    }

    pub fn canonical_system(&self) -> Option<&str> {
        self.canonical_system.as_deref()
    }

    pub fn license_label(&self) -> Option<&'static str> {
        self.license_tier.map(|tier| tier.as_str())
    }

    pub fn source_label(&self) -> Option<&'static str> {
        self.source_kind.map(|kind| kind.as_str())
    }
}

fn canonicalize_system(value: Option<&str>) -> Option<String> {
    let mut url = value?.trim().to_ascii_lowercase();
    if url.is_empty() {
        return None;
    }
    if url.ends_with('/') {
        url.pop();
    }
    match url.as_str() {
        "urn:oid:2.16.840.1.113883.6.96" => Some("http://snomed.info/sct".into()),
        "urn:oid:2.16.840.1.113883.6.1" => Some("http://loinc.org".into()),
        other => Some(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staging(system: Option<&str>, code: Option<&str>) -> StgSrCodeExploded {
        StgSrCodeExploded {
            sr_id: "SR-1".into(),
            system: system.map(|v| v.to_string()),
            code: code.map(|v| v.to_string()),
            display: None,
        }
    }

    #[test]
    fn classifies_missing_data() {
        let enriched = EnrichedCode::from_staging(staging(None, Some("123")));
        assert_eq!(enriched.code_kind(), CodeKind::MissingSystemOrCode);

        let enriched = EnrichedCode::from_staging(staging(Some("http://loinc.org"), None));
        assert_eq!(enriched.code_kind(), CodeKind::MissingSystemOrCode);
    }

    #[test]
    fn classifies_licensed_system() {
        let enriched =
            EnrichedCode::from_staging(staging(Some("http://snomed.info/sct"), Some("123")));
        assert_eq!(enriched.code_kind(), CodeKind::KnownLicensedSystem);
        assert!(enriched.canonical_system().unwrap().contains("snomed"));
    }

    #[test]
    fn classifies_open_system() {
        let enriched =
            EnrichedCode::from_staging(staging(Some("http://loinc.org"), Some("24606-6")));
        assert_eq!(enriched.code_kind(), CodeKind::KnownOpenSystem);
    }

    #[test]
    fn classifies_obo_system() {
        let enriched = EnrichedCode::from_staging(staging(
            Some("http://purl.obolibrary.org/obo/ncit"),
            Some("C19951"),
        ));
        assert_eq!(enriched.code_kind(), CodeKind::OboBacked);
    }

    #[test]
    fn classifies_unknown_system() {
        let enriched =
            EnrichedCode::from_staging(staging(Some("http://example.org/custom"), Some("ABC")));
        assert_eq!(enriched.code_kind(), CodeKind::UnknownSystem);
    }
}
