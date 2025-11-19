//! End-to-end flow tests across ingestion → mapping → analytics (REFR-14).

pub mod fhir_ingest_flow;
pub mod mapping_pipeline_flow;
pub mod observability_metrics_flow;
pub mod service_request_flow;
pub mod smoke_index;
