use refractive_swan_compliance::{ComplianceConfig, Policy};
use refractive_swan_cli_dto::PipelineMetrics;

use super::{CliError, CliResult};

pub fn load_policy() -> CliResult<Policy> {
    let config = ComplianceConfig::from_env()
        .map_err(|err| CliError::config(format!("compliance config error: {err}")))?;
    config
        .load_policy()
        .map_err(|err| CliError::config(format!("compliance policy error: {err}")))
}

pub fn tag_metrics(metrics: &mut PipelineMetrics, policy: &Policy) {
    metrics.compliance_mode = Some(policy.mode.as_str().to_string());
}

pub fn enforce_metrics_gate(
    policy: &Policy,
    metrics: &PipelineMetrics,
    fail_on_license_block: bool,
) -> CliResult<()> {
    if fail_on_license_block && metrics.license_blocked > 0 {
        return Err(CliError::compliance(format!(
            "{} mapping result(s) blocked by compliance mode {}; rerun without --fail-on-license-block or adjust refractive_swan_COMPLIANCE_MODE",
            metrics.license_blocked,
            policy.mode.as_str()
        )));
    }
    Ok(())
}

pub fn enforce_license_blocks(
    policy: &Policy,
    blocked: usize,
    fail_on_license_block: bool,
) -> CliResult<()> {
    if fail_on_license_block && blocked > 0 {
        Err(CliError::compliance(format!(
            "{blocked} code(s) blocked due to compliance mode {}; rerun with --fail-on-license-block disabled or adjust refractive_swan_COMPLIANCE_MODE",
            policy.mode.as_str()
        )))
    } else {
        Ok(())
    }
}
