use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use dfps_cli::cli_core::{
    CliError, CliResult, init_cli_env, input_reader, json_stream, run_bin, write_record,
};
use dfps_core::fhir::Bundle;
use dfps_ingestion::validation::{
    ExternalValidationContext, ValidationMode, ValidationSeverity,
    external::{
        ExternalValidationError, ExternalValidationReport, ExternalValidator, OperationOutcome,
    },
};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "validate_fhir",
    about = "Validate FHIR Bundles using internal rules plus optional external $validate"
)]
struct Args {
    /// Bundle JSON/NDJSON file. If omitted, reads from stdin.
    #[arg(long, value_name = "PATH")]
    input: Option<PathBuf>,
    /// Validation mode (default: external_preferred).
    #[arg(long, value_enum, default_value = "external_preferred")]
    mode: Mode,
    /// Optional profile URL to pass to the external validator.
    #[arg(long, value_name = "URL")]
    profile: Option<String>,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
enum Mode {
    Lenient,
    Strict,
    ExternalPreferred,
    ExternalStrict,
}

impl From<Mode> for ValidationMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Lenient => ValidationMode::Lenient,
            Mode::Strict => ValidationMode::Strict,
            Mode::ExternalPreferred => ValidationMode::ExternalPreferred,
            Mode::ExternalStrict => ValidationMode::ExternalStrict,
        }
    }
}

#[derive(Serialize)]
struct SummaryRow {
    total_bundles: usize,
    total_issues: usize,
    errors: usize,
    warnings: usize,
    infos: usize,
}

fn main() {
    run_bin("validate_fhir", run);
}

fn run() -> CliResult<()> {
    init_cli_env()?;
    let args = Args::parse();
    let mode: ValidationMode = args.mode.into();
    let validator = match mode {
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict => {
            Some(BlockingHttpValidator::try_new()?)
        }
        _ => None,
    };

    let reader = input_reader(args.input.as_ref())?;
    let mut bundles = json_stream::<Bundle>(reader);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    let mut total_bundles = 0usize;
    let mut total_issues = 0usize;
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut total_infos = 0usize;

    while let Some(bundle) = bundles.next() {
        let bundle = bundle?;
        total_bundles += 1;
        let ctx = ExternalValidationContext {
            validator: validator.as_ref().map(|v| v as &dyn ExternalValidator),
            profile_url: args.profile.as_deref(),
        };
        let report =
            dfps_ingestion::validation::validate_bundle_with_external_profile(&bundle, mode, ctx);
        for issue in &report.issues {
            total_issues += 1;
            match issue.severity {
                ValidationSeverity::Error => total_errors += 1,
                ValidationSeverity::Warning => total_warnings += 1,
                ValidationSeverity::Info => total_infos += 1,
            }
            write_record(&mut handle, "validation_issue", issue)?;
        }
    }

    if total_bundles == 0 {
        eprintln!("warning: no Bundle payloads detected in input");
    }

    let summary = SummaryRow {
        total_bundles,
        total_issues,
        errors: total_errors,
        warnings: total_warnings,
        infos: total_infos,
    };
    write_record(&mut handle, "validation_summary", &summary)?;

    if matches!(
        mode,
        ValidationMode::Strict | ValidationMode::ExternalStrict
    ) && total_errors > 0
    {
        return Err(CliError::invalid(format!(
            "validation detected {total_errors} error(s)"
        )));
    }

    Ok(())
}

#[derive(Clone)]
struct BlockingHttpValidator {
    client: reqwest::blocking::Client,
    base_url: String,
    default_profile: Option<String>,
}

impl BlockingHttpValidator {
    fn try_new() -> CliResult<Self> {
        let base_url = std::env::var("DFPS_FHIR_VALIDATOR_BASE_URL").map_err(|_| {
            CliError::config("DFPS_FHIR_VALIDATOR_BASE_URL must be set for external validation")
        })?;
        let timeout_secs = std::env::var("DFPS_FHIR_VALIDATOR_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(10);
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|err| CliError::external(err.to_string()))?;
        let default_profile = std::env::var("DFPS_FHIR_VALIDATOR_PROFILE").ok();
        Ok(Self {
            client,
            base_url,
            default_profile,
        })
    }

    fn profile<'a>(&'a self, override_url: Option<&'a str>) -> Option<&'a str> {
        override_url.or(self.default_profile.as_deref())
    }
}

impl ExternalValidator for BlockingHttpValidator {
    fn validate_bundle(
        &self,
        bundle: &Bundle,
        profile_url: Option<&str>,
    ) -> Result<ExternalValidationReport, ExternalValidationError> {
        let mut req = self
            .client
            .post(format!("{}/$validate", self.base_url))
            .json(bundle);
        if let Some(profile) = self.profile(profile_url) {
            req = req.query(&[("profile", profile)]);
        }
        let response = req
            .send()
            .map_err(|err| ExternalValidationError::Failed(err.to_string()))?;
        let outcome = if response.status().is_success() {
            Some(
                response
                    .json::<OperationOutcome>()
                    .map_err(|err| ExternalValidationError::Failed(err.to_string()))?,
            )
        } else {
            return Err(ExternalValidationError::Failed(format!(
                "validator returned status {}",
                response.status()
            )));
        };
        Ok(ExternalValidationReport::from_operation_outcome(outcome))
    }
}
