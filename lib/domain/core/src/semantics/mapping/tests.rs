use super::*;
use crate::interop::staging::StgSrCodeExploded;

#[test]
fn code_element_from_staging_derives_stable_id() {
    let staging = StgSrCodeExploded {
        sr_id: "SR-1".into(),
        system: Some("http://loinc.org".into()),
        code: Some("24606-6".into()),
        display: Some("FDG uptake PET".into()),
    };

    let element: CodeElement = staging.clone().into();
    assert_eq!(element.id, "SR-1::http://loinc.org::24606-6");
    assert_eq!(element.system, staging.system);
    assert_eq!(element.code, staging.code);
    assert_eq!(element.display, staging.display);
}

#[test]
fn code_element_id_falls_back_to_display_and_unknowns() {
    let with_display = CodeElement::id_for(
        "SR-2",
        Some("http://snomed.info/sct"),
        None,
        Some("PET FDG"),
    );
    assert_eq!(with_display, "SR-2::http://snomed.info/sct::PET FDG");

    let fully_unknown = CodeElement::id_for("SR-3", None, None, None);
    assert_eq!(fully_unknown, "SR-3::unknown-system::unknown-code");
}

#[test]
fn mapping_result_builders_apply_default_reasons() {
    let thresholds = MappingThresholds {
        auto_map_min: 0.9,
        needs_review_min: 0.6,
    };
    let source_version = MappingSourceVersion::new("ncit-20240101", "umls-20240101");

    let auto = MappingResult::auto_mapped(
        "SR-10::system::code",
        "C1234",
        0.98,
        thresholds,
        source_version.clone(),
        MappingStrategy::Lexical,
        None,
        Some("internal".into()),
        Some("ncit".into()),
        Some("CUI-1".into()),
    );
    assert_eq!(auto.state, MappingState::AutoMapped);
    assert_eq!(auto.reason.as_deref(), Some("auto_mapped"));
    assert_eq!(auto.ncit_id.as_deref(), Some("C1234"));
    assert_eq!(auto.license_tier.as_deref(), Some("internal"));
    assert_eq!(auto.source_kind.as_deref(), Some("ncit"));
    assert_eq!(auto.cui.as_deref(), Some("CUI-1"));

    let needs_review = MappingResult::needs_review(
        "SR-11::system::code",
        "C5678",
        0.7,
        thresholds,
        source_version.clone(),
        MappingStrategy::Vector,
        Some("low score".into()),
        None,
        None,
        None,
    );
    assert_eq!(needs_review.state, MappingState::NeedsReview);
    assert_eq!(needs_review.reason.as_deref(), Some("low score"));

    let no_match = MappingResult::no_match(
        "SR-12::system::code",
        0.2,
        thresholds,
        source_version,
        MappingStrategy::Composite,
        None,
        None,
        None,
    );
    assert_eq!(no_match.state, MappingState::NoMatch);
    assert_eq!(no_match.reason.as_deref(), Some("no_match"));
    assert!(no_match.ncit_id.is_none());
}
