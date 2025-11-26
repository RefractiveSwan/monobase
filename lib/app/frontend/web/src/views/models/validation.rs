use refractive_swan_contracts::pipeline::{ValidationReport, ValidationSeverity};

use super::types::{ValidationIssueView, ValidationSummaryView};

pub fn summary_from_reports(reports: &[ValidationReport]) -> Option<ValidationSummaryView> {
    if reports.is_empty() {
        return None;
    }
    let mut summary = ValidationSummaryView {
        total: 0,
        errors: 0,
        warnings: 0,
        info: 0,
        has_errors: false,
        issues: Vec::new(),
    };

    for (idx, report) in reports.iter().enumerate() {
        let label = if reports.len() > 1 {
            format!("Bundle {}", idx + 1)
        } else {
            "Bundle".to_string()
        };
        for issue in &report.issues {
            summary.total += 1;
            match issue.severity {
                ValidationSeverity::Error => summary.errors += 1,
                ValidationSeverity::Warning => summary.warnings += 1,
                ValidationSeverity::Info => summary.info += 1,
            }
            summary.issues.push(ValidationIssueView {
                id: issue.id.clone(),
                bundle_label: label.clone(),
                severity: issue.severity,
                message: issue.message.clone(),
                requirement: issue.requirement_ref().to_string(),
            });
        }
    }

    summary.has_errors = summary.errors > 0;
    Some(summary)
}
