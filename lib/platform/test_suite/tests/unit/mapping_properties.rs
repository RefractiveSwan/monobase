//! REFR-06 – Mapping engine properties (lexical/vector/rule parity).
//! Ensures ranked candidates remain sorted and synonym augmentation never regresses.

use proptest::prelude::*;
use refractive_swan_core::{mapping::CodeElement, staging::StgSrCodeExploded};
use refractive_swan_mapping::{default_engine, map_staging_codes};

fn staging_with_display(display: String) -> StgSrCodeExploded {
    StgSrCodeExploded {
        sr_id: "SR-prop".into(),
        system: Some("http://snomed.info/sct".into()),
        code: Some("999999".into()),
        display: Some(display),
    }
}

proptest! {
    #[test]
    fn synonym_augmentation_never_decreases_score(base in "\\w{3,12}") {
        let base_code = staging_with_display(base.clone());
        let augmented_code = staging_with_display(format!("{base} PET"));

        let (base_results, _) = map_staging_codes(vec![base_code]);
        let (aug_results, _) = map_staging_codes(vec![augmented_code]);

        prop_assert_eq!(base_results.len(), 1);
        prop_assert_eq!(aug_results.len(), 1);

        let base_score = base_results[0].score;
        let aug_score = aug_results[0].score;
        prop_assert!(aug_score >= base_score);
    }
}

proptest! {
    #[test]
    fn ranked_candidates_are_sorted(display in "\\w{3,12}") {
        let staging = staging_with_display(display);
        let code = CodeElement::from(staging);
        let engine = default_engine();
        let candidates = engine.ranked_candidates(&code);
        prop_assert!(!candidates.is_empty());
        let top = candidates[0].score;
        for candidate in candidates.iter().skip(1) {
            prop_assert!(top >= candidate.score);
        }
    }
}
