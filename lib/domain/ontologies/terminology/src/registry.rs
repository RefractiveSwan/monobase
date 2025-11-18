use crate::codesystem::{CodeSystemMeta, LicenseTier, SourceKind};

static CODE_SYSTEMS: [CodeSystemMeta; 4] = [
    CodeSystemMeta::new(
        "http://www.ama-assn.org/go/cpt",
        "CPT",
        None,
        "Current Procedural Terminology (AMA).",
        LicenseTier::Licensed,
        SourceKind::Fhir,
    ),
    CodeSystemMeta::new(
        "http://snomed.info/sct",
        "SNOMED CT",
        None,
        "Systematized Nomenclature of Medicine -- Clinical Terms.",
        LicenseTier::Licensed,
        SourceKind::Fhir,
    ),
    CodeSystemMeta::new(
        "http://loinc.org",
        "LOINC",
        None,
        "Logical Observation Identifiers Names and Codes.",
        LicenseTier::Open,
        SourceKind::Fhir,
    ),
    CodeSystemMeta::new(
        "http://purl.obolibrary.org/obo/NCIT",
        "NCIt OBO",
        None,
        "NCI Thesaurus (OBO Foundry distribution).",
        LicenseTier::Open,
        SourceKind::OboFoundry,
    ),
];

/// Iterate all registered code systems.
pub fn list_code_systems() -> &'static [CodeSystemMeta] {
    &CODE_SYSTEMS
}

/// Lookup a `CodeSystemMeta` by URL/identifier.
pub fn lookup_codesystem(url: &str) -> Option<&'static CodeSystemMeta> {
    CODE_SYSTEMS
        .iter()
        .find(|meta| meta.url.eq_ignore_ascii_case(url))
}

/// Returns `true` if the code system requires a license.
pub fn is_licensed(url: &str) -> bool {
    lookup_codesystem(url)
        .map(|meta| matches!(meta.license_tier, LicenseTier::Licensed))
        .unwrap_or(false)
}

/// Returns `true` if the code system is open/unlicensed.
pub fn is_open(url: &str) -> bool {
    lookup_codesystem(url)
        .map(|meta| matches!(meta.license_tier, LicenseTier::Open))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_system() {
        let meta =
            lookup_codesystem("http://www.ama-assn.org/go/cpt").expect("CPT should be registered");
        assert!(is_licensed(meta.url));
        assert!(!is_open(meta.url));
    }

    #[test]
    fn lookup_unknown_system() {
        assert!(lookup_codesystem("http://example.com/unknown").is_none());
        assert!(!is_licensed("http://example.com/unknown"));
    }

    #[test]
    fn lookup_open_system() {
        let meta = lookup_codesystem("http://loinc.org").expect("LOINC should be registered");
        assert!(is_open(meta.url));
        assert!(!is_licensed(meta.url));
        assert_eq!(meta.license_tier.as_str(), "open");
    }
}
