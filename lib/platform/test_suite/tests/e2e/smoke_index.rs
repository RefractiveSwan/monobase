//! Full-stack smoke index covering ingestion → mapping → datamart → eval → vector (REFR-14).

use refractive_swan_datamart::from_pipeline_output;
use refractive_swan_eval::FileDatasetStore;
use refractive_swan_pipeline::bundle_to_mapped_sr;
use refractive_swan_test_suite::{ensure_eval_data_root, regression};
use refractive_swan_vector_store::{MockVectorStore, VectorStore};

#[test]
fn smoke_index_exercises_core_surfaces() {
    refractive_swan_test_suite::init_environment().expect("load test suite env");
    let bundle = regression::baseline_fhir_bundle();
    let output = bundle_to_mapped_sr(&bundle).expect("pipeline maps baseline bundle");
    assert!(
        !output.mapping_results.is_empty(),
        "pipeline should emit mapping results"
    );

    let (_dims, facts) = from_pipeline_output(&output);
    assert!(
        !facts.is_empty(),
        "datamart adapter should emit fact rows for baseline bundle"
    );

    let eval_root = ensure_eval_data_root().expect("ensure refractive_swan_EVAL_DATA_ROOT");
    let store = FileDatasetStore::new(eval_root);
    let datasets = store
        .list_manifests()
        .expect("list eval dataset manifests from store");
    assert!(
        datasets.iter().any(|d| d.name == "pet_ct_small"),
        "eval dataset registry should include pet_ct_small"
    );

    let vector_store = MockVectorStore::new("ncit_dev");
    vector_store
        .health("ncit_dev")
        .expect("mock vector store health check");
}
