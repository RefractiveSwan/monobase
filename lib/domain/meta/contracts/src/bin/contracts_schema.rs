use std::{
    collections::BTreeSet,
    env,
    fs::{self, File, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
};

use refractive_swan_contracts::{
    AnalyticsSummaryResponse, CohortResponse, ErrorCode, ErrorKind, EvalRunResponse, LoadSummary,
    MeshJobDescriptor, MeshJobResult,
};
use schemars::{JsonSchema, schema_for};
use serde_json::to_writer_pretty;
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../..")
}

fn contracts_dir() -> PathBuf {
    workspace_root().join("ci/contracts")
}

fn write_schema<T: JsonSchema>(out_dir: &Path, name: &str) -> io::Result<()> {
    let schema = schema_for!(T);
    create_dir_all(out_dir)?;
    let path = out_dir.join(name);
    let mut file = File::create(path)?;
    to_writer_pretty(&mut file, &schema)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn write_enum_values(out_dir: &Path, name: &str, values: &[&str]) -> io::Result<()> {
    create_dir_all(out_dir)?;
    let path = out_dir.join(name);
    let mut file = File::create(path)?;
    to_writer_pretty(&mut file, &values)?;
    file.write_all(b"\n")?;
    Ok(())
}

fn export_all(out_dir: &Path) -> io::Result<()> {
    write_schema::<AnalyticsSummaryResponse>(out_dir, "analytics_summary_response.schema.json")?;
    write_schema::<CohortResponse>(out_dir, "cohort_response.schema.json")?;
    write_schema::<EvalRunResponse>(out_dir, "eval_run_response.schema.json")?;
    write_schema::<LoadSummary>(out_dir, "load_summary.schema.json")?;
    write_schema::<MeshJobDescriptor>(out_dir, "mesh_job_descriptor.schema.json")?;
    write_schema::<MeshJobResult>(out_dir, "mesh_job_result.schema.json")?;
    write_enum_values(out_dir, "error_kinds.json", &ErrorKind::variant_names())?;
    write_enum_values(out_dir, "error_codes.json", &ErrorCode::variant_names())?;
    Ok(())
}

fn check_schemas() -> io::Result<()> {
    let tmp = tempdir()?;
    export_all(tmp.path())?;

    let expected_dir = contracts_dir();
    let generated_dir = tmp.path();
    let expected_files = list_files(&expected_dir)?;
    let generated_files = list_files(generated_dir)?;

    let all_files: BTreeSet<_> = expected_files.union(&generated_files).cloned().collect();
    let mut diffs = Vec::new();
    for name in all_files {
        if !expected_files.contains(&name) {
            diffs.push(format!("unexpected schema generated: {name}"));
            continue;
        }
        if !generated_files.contains(&name) {
            diffs.push(format!("missing schema in generator output: {name}"));
            continue;
        }
        let expected = fs::read_to_string(expected_dir.join(&name))?;
        let actual = fs::read_to_string(generated_dir.join(&name))?;
        if expected != actual {
            diffs.push(format!("schema drift detected: {name}"));
        }
    }

    if diffs.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("contract schemas out of date:\n{}", diffs.join("\n")),
        ))
    }
}

fn list_files(dir: &Path) -> io::Result<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                files.insert(name.to_string());
            }
        }
    }
    Ok(files)
}

fn main() -> io::Result<()> {
    let check_mode = env::args().any(|arg| arg == "--check");
    if check_mode {
        check_schemas()
    } else {
        export_all(&contracts_dir())
    }
}

trait EnumVariants {
    fn variant_names() -> Vec<&'static str>;
}

impl EnumVariants for ErrorKind {
    fn variant_names() -> Vec<&'static str> {
        vec![
            "domain_ingestion",
            "domain_mapping",
            "domain_compliance",
            "domain_vector",
            "domain_warehouse",
            "platform_config",
            "platform_store",
            "platform_terminology",
            "app_http_client",
            "app_http_server",
            "app_cli_usage",
            "app_cli_runtime",
        ]
    }
}

impl EnumVariants for ErrorCode {
    fn variant_names() -> Vec<&'static str> {
        use ErrorCode::*;
        vec![
            InvalidJson.as_str(),
            InvalidFhir.as_str(),
            ComplianceBlocked.as_str(),
            InvalidDataset.as_str(),
            VectorBackendUnavailable.as_str(),
            WarehouseExportBlocked.as_str(),
            InternalError.as_str(),
        ]
    }
}
