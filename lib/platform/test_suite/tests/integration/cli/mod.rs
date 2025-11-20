//! CLI contract integration tests (REFR-14, REFR-03).

use assert_cmd::Command;
use dfps_contracts::{
    DimNCITConcept, EvalRunResponse, LoadSummary, MappingResult, PipelineMetrics,
};
use dfps_core::mapping::MappingState;
use dfps_test_suite::{ensure_eval_data_root, init_environment, regression};
use serde_json::Value;
use std::io::Write;
use tempfile::NamedTempFile;

fn parse_record(line: &str) -> (String, Value) {
    let mut obj: serde_json::Map<String, Value> =
        serde_json::from_str(line).expect("record should be JSON");
    let kind = obj
        .remove("kind")
        .and_then(|value| value.as_str().map(|s| s.to_string()))
        .expect("record kind present");
    (kind, Value::Object(obj))
}

fn cli_command(bin: &str) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "-p", "dfps_cli", "--bin", bin, "--"]);
    cmd
}

#[test]
fn map_bundles_streams_contract_rows() {
    init_environment().expect("dfps_test_suite env");
    let bundle = regression::baseline_fhir_bundle();
    let mut bundle_file = NamedTempFile::new().expect("temp file");
    serde_json::to_writer(&mut bundle_file, &bundle).expect("write bundle NDJSON");
    writeln!(bundle_file).expect("newline");

    let assert = cli_command("map_bundles")
        .arg(bundle_file.path())
        .arg("--log-level")
        .arg("warn")
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");

    let mut metrics_seen = false;
    let mut mapping_rows = 0;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let (kind, payload) = parse_record(line);
        match kind.as_str() {
            "mapping_result" => {
                let mapping: MappingResult =
                    serde_json::from_value(payload.clone()).expect("mapping_result contract");
                assert!(matches!(
                    mapping.state,
                    MappingState::AutoMapped | MappingState::NeedsReview | MappingState::NoMatch
                ));
                mapping_rows += 1;
            }
            "dim_concept" => {
                let _: DimNCITConcept =
                    serde_json::from_value(payload.clone()).expect("dim_concept contract");
            }
            "metrics_summary" => {
                let metrics: PipelineMetrics =
                    serde_json::from_value(payload.clone()).expect("pipeline metrics contract");
                assert!(metrics.bundle_count >= 1);
                metrics_seen = true;
            }
            _ => {}
        }
    }
    assert!(mapping_rows > 0, "expected at least one mapping_result");
    assert!(metrics_seen, "expected metrics_summary");
}

#[test]
fn load_datamart_emits_contract_summary() {
    init_environment().expect("dfps_test_suite env");
    let bundle = regression::baseline_fhir_bundle();
    let mut bundle_file = NamedTempFile::new().expect("bundle file");
    serde_json::to_writer(&mut bundle_file, &bundle).expect("bundle NDJSON");
    writeln!(bundle_file).expect("newline");

    let warehouse_file = NamedTempFile::new().expect("warehouse file");
    let warehouse_url = format!("sqlite://{}", warehouse_file.path().display());
    let bundle_path = bundle_file.path().display().to_string();

    let assert = cli_command("load_datamart")
        .arg("--input")
        .arg(&bundle_path)
        .arg("--input-kind")
        .arg("bundle")
        .env("DFPS_WAREHOUSE_URL", &warehouse_url)
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let (kind, payload) = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_record)
        .next()
        .expect("load_summary record");
    assert_eq!(kind, "load_summary");
    let summary: LoadSummary =
        serde_json::from_value(payload).expect("load_summary contract deserialization");
    assert!(summary.facts >= 1);
}

#[test]
fn eval_mapping_outputs_contract_summary() {
    init_environment().expect("dfps_test_suite env");
    let eval_root = ensure_eval_data_root().expect("eval data root");
    let dataset = "pet_ct_small";
    let assert = cli_command("eval_mapping")
        .args(["--dataset", dataset])
        .env("DFPS_EVAL_DATA_ROOT", eval_root)
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8 stdout");
    let (kind, payload) = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_record)
        .next()
        .expect("eval_summary record");
    assert_eq!(kind, "eval_summary");
    let response: EvalRunResponse = serde_json::from_value(payload).expect("eval summary contract");
    assert_eq!(response.dataset, dataset);
    assert!(response.summary.total_cases > 0);
}
