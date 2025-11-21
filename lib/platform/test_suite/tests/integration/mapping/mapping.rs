//! Mapping integration tests covering lexical/rule paths (REFR-06).

use refractive_swan_core::mapping::MappingState;
use refractive_swan_mapping::map_staging_codes;
use refractive_swan_test_suite::fixtures;

#[test]
fn cpt_code_maps_to_ncit() {
    let code = fixtures::mapping_cpt_code();
    let (results, dims) = map_staging_codes(vec![code]);

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.ncit_id.as_deref(), Some("NCIT:C19951"));
    assert!(result.score >= 0.95);
    assert!(!dims.is_empty());
}

#[test]
fn snomed_pet_maps_to_same_ncit() {
    let code = fixtures::mapping_snomed_code();
    let (results, _) = map_staging_codes(vec![code]);

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.ncit_id.as_deref(), Some("NCIT:C19951"));
    assert!(result.score >= 0.95);
}

#[test]
fn unknown_codes_report_no_match() {
    let code = fixtures::mapping_unknown_code();
    let (results, _) = map_staging_codes(vec![code]);

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.state, MappingState::NoMatch);
    assert!(result.ncit_id.is_none());
    assert_eq!(result.reason.as_deref(), Some("missing_system_or_code"));
}

#[test]
fn unknown_systems_surface_reason() {
    let code = fixtures::mapping_unknown_system_code();
    let (results, _) = map_staging_codes(vec![code]);

    assert_eq!(results.len(), 1);
    let result = &results[0];
    assert_eq!(result.state, MappingState::NoMatch);
    assert_eq!(result.reason.as_deref(), Some("unknown_code_system"));
    assert!(result.license_tier.is_none());
    assert!(result.source_kind.is_none());
}

#[test]
fn licensed_systems_expose_metadata() {
    let code = fixtures::mapping_cpt_code();
    let (results, _) = map_staging_codes(vec![code]);

    let result = &results[0];
    assert_eq!(result.license_tier.as_deref(), Some("licensed"));
    assert_eq!(result.source_kind.as_deref(), Some("fhir"));
}

#[test]
fn obo_systems_marked_open() {
    let code = fixtures::mapping_ncit_obo_code();
    let (results, _) = map_staging_codes(vec![code]);

    let result = &results[0];
    assert_eq!(result.source_kind.as_deref(), Some("obo_foundry"));
    assert_eq!(result.license_tier.as_deref(), Some("open"));
}
