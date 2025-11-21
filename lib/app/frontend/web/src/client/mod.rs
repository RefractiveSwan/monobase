mod backend;
mod error;
mod types;

pub use backend::*;
pub use error::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use serde_json::json;

    #[test]
    fn pipeline_output_contract_round_trips() {
        let payload = json!({
            "flats": [{
                "sr_id": "SR-1",
                "patient_id": "P1",
                "encounter_id": null,
                "status": "active",
                "intent": "order",
                "description": "PET-CT",
                "ordered_at": "2024-05-01T12:00:00Z"
            }],
            "exploded_codes": [{
                "sr_id": "SR-1",
                "system": "http://loinc.org",
                "code": "24606-6",
                "display": "FDG uptake"
            }],
            "mapping_results": [{
                "code_element_id": "SR-1::http://loinc.org::24606-6",
                "cui": "C0001",
                "ncit_id": "C1234",
                "score": 0.99,
                "strategy": "lexical",
                "state": "auto_mapped",
                "thresholds": { "auto_map_min": 0.9, "needs_review_min": 0.7 },
                "source_version": { "ncit": "ncit-2024", "umls": "umls-2024" },
                "reason": null,
                "license_tier": null,
                "source_kind": null
            }],
            "dim_concepts": [{
                "ncit_id": "C1234",
                "preferred_name": "FDG Uptake",
                "semantic_group": "Test"
            }],
            "vector_usage": null
        });
        let contract: MapBundlesResponse = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(contract.flats.len(), 1);
        let round_trip = serde_json::to_value(&contract).unwrap();
        assert!(round_trip.get("mapping_results").is_some());
        assert!(round_trip.get("vector_usage").is_some());
    }

    #[test]
    fn analytics_contract_deserializes_backend_payloads() {
        let payload = json!({
            "rows": [{
                "ncit_id": "C1234",
                "preferred_name": "FDG Uptake",
                "mapping_state": "auto_mapped",
                "time_bucket": "2024-05-01",
                "count": 3
            }]
        });
        let summary: AnalyticsSummaryResponse = serde_json::from_value(payload).unwrap();
        assert_eq!(summary.rows[0].ncit_id, "C1234");

        let cohort_json = json!({
            "total": 1,
            "rows": [{
                "sr_id": "SR-1",
                "patient_id": "P1",
                "encounter_id": "E1",
                "ncit_id": "C1234",
                "status": "active",
                "intent": "order",
                "description": "PET",
                "ordered_at": "2024-05-01T12:00:00Z",
                "mapping_state": "auto_mapped"
            }]
        });
        let cohort: CohortResponse = serde_json::from_value(cohort_json).unwrap();
        assert_eq!(cohort.total, 1);
        assert_eq!(cohort.rows[0].ncit_id.as_deref(), Some("C1234"));
    }

    #[test]
    fn backend_error_user_messages_include_code() {
        let body = json!({
            "code": "invalid_fhir",
            "kind": "domain_ingestion",
            "message": "bundle missing entries",
            "request_id": "4ed499d6-bafd-45d5-b3c6-d2da7a5e0a10"
        })
        .to_string();
        let err = error::BackendError::from_http(StatusCode::UNPROCESSABLE_ENTITY, body);
        assert!(err.user_message().contains("invalid_fhir"));
        assert!(
            err.to_string()
                .contains("4ed499d6-bafd-45d5-b3c6-d2da7a5e0a10")
        );
    }
}
