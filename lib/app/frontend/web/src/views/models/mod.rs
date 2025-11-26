mod analytics;
mod helpers;
mod mapping;
mod types;
mod validation;

pub use types::*;
pub use validation::summary_from_reports;

// Tests from original file
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MapBundlesResponse;
    use refractive_swan_contracts::pipeline::{
        DimNCITConcept, MappingResult, MappingSourceVersion, MappingState, MappingStrategy,
        MappingThresholds, StgServiceRequestFlat, StgSrCodeExploded,
    };
    use refractive_swan_core::order::{ServiceRequestIntent, ServiceRequestStatus};

    fn sample_response() -> MapBundlesResponse {
        let flats = vec![StgServiceRequestFlat {
            sr_id: "SR-1".into(),
            patient_id: "P1".into(),
            encounter_id: None,
            status: "active".into(),
            status_enum: ServiceRequestStatus::Active,
            intent: "order".into(),
            intent_enum: ServiceRequestIntent::Order,
            description: "test".into(),
            ordered_at: Some("2024-01-01".into()),
        }];

        let exploded_codes = vec![
            StgSrCodeExploded {
                sr_id: "SR-1".into(),
                system: Some("http://loinc.org".into()),
                code: Some("24606-6".into()),
                display: Some("FDG Uptake".into()),
            },
            StgSrCodeExploded {
                sr_id: "SR-2".into(),
                system: Some("http://loinc.org".into()),
                code: Some("99999-9".into()),
                display: Some("Unknown code".into()),
            },
        ];

        let dim_concepts = vec![DimNCITConcept {
            ncit_id: "C1234".into(),
            preferred_name: "FDG Uptake".into(),
            semantic_group: "Procedure".into(),
        }];

        let thresholds = MappingThresholds::default();
        let source_version = MappingSourceVersion::new("ncit-mini-0.1.1", "umls-2024AA");

        let mapping_results = vec![
            MappingResult {
                code_element_id: "SR-1::http://loinc.org::24606-6".into(),
                cui: Some("CUI-1234".into()),
                ncit_id: Some("C1234".into()),
                score: 0.99,
                strategy: MappingStrategy::Lexical,
                state: MappingState::AutoMapped,
                thresholds,
                source_version: source_version.clone(),
                reason: Some("auto_mapped".into()),
                license_tier: Some("open".into()),
                source_kind: Some("fhir".into()),
            },
            MappingResult {
                code_element_id: "SR-2::http://loinc.org::99999-9".into(),
                cui: None,
                ncit_id: None,
                score: 0.10,
                strategy: MappingStrategy::Lexical,
                state: MappingState::NoMatch,
                thresholds,
                source_version,
                reason: Some("missing_system_or_code".into()),
                license_tier: None,
                source_kind: None,
            },
        ];

        MapBundlesResponse {
            flats,
            exploded_codes,
            mapping_results,
            dim_concepts,
            vector_usage: None,
            validation_reports: Vec::new(),
        }
    }

    #[test]
    fn derives_no_match_rows_from_mapping_results() {
        let response = sample_response();
        let view = MappingResultsView::from_response(&response);
        assert_eq!(view.no_matches.len(), 1);
        assert_eq!(view.no_matches[0].sr_id, "SR-2");
        assert_eq!(view.no_matches[0].code, "99999-9");
        assert_eq!(
            view.no_matches[0].reason,
            Some("missing_system_or_code".into())
        );
    }
}
