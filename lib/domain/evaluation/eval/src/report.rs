use crate::{EvalSummary, dataset_root};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Write,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

/// Render a Markdown report for dashboards or CLI consumers.
pub fn render_markdown(summary: &EvalSummary) -> String {
    render_markdown_with_baseline(summary, None)
}

/// Render a Markdown report with an optional baseline comparison.
pub fn render_markdown_with_baseline(
    summary: &EvalSummary,
    baseline: Option<&EvalSummary>,
) -> String {
    let mut out = String::new();
    out.push_str("# Mapping Evaluation Report\n\n");
    out.push_str(&format!(
        "*Total cases:* {}\\n\n*Precision:* {:.4}\\n\\n*Recall:* {:.4}\\n\\n*Accuracy:* {:.4}\\n\\n*F1:* {:.4}\\n\n*AutoMapped precision:* {:.4} ({} of {})\\n\n",
        summary.total_cases,
        summary.precision,
        summary.recall,
        summary.accuracy,
        summary.f1,
        summary.auto_mapped_precision,
        summary.auto_mapped_correct,
        summary.auto_mapped_total
    ));

    if let Some(base) = baseline {
        out.push_str("## Baseline comparison\n\n");
        out.push_str("| Metric | Current | Baseline | Delta |\\n| --- | ---: | ---: | ---: |\\n");
        for (label, current, previous) in comparison_rows(summary, base) {
            out.push_str(&format!(
                "| {} | {:.3} | {:.3} | {} |\\n",
                label,
                current,
                previous,
                delta_string(current, previous)
            ));
        }

        let changes = changelog(summary, base);
        if changes.is_empty() {
            out.push_str("\n_No metric changes vs. baseline._\n");
        } else {
            out.push_str("\n### Metric deltas\n\n");
            for line in changes {
                out.push_str(&format!("- {line}\n"));
            }
        }
    }

    out.push_str("\n## Calibration buckets\n\n");
    out.push_str(
        "_Only MappingResults with an NCIt prediction contribute to calibration buckets._\n\n",
    );
    out.push_str("| Bucket | Range | Predictions | Correct | Accuracy |\\n| --- | --- | ---: | ---: | ---: |\\n");
    for bucket in &summary.score_buckets {
        let range = match (bucket.lower_bound, bucket.upper_bound) {
            (Some(lower), Some(upper)) => format!("{lower:.1}–{upper:.1}"),
            _ => "n/a".to_string(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.3} |\\n",
            bucket.bucket, range, bucket.total, bucket.correct, bucket.accuracy
        ));
    }

    out.push_str("\n## Reason counts\n\n| Reason | Count |\\n| --- | ---: |\\n");
    for (reason, count) in &summary.reason_counts {
        out.push_str(&format!("| {} | {} |\\n", reason, count));
    }

    if let Some(advanced) = &summary.advanced {
        out.push_str("\n## Confidence intervals (95%)\\n\n");
        out.push_str(&format!(
            "Precision: {:.3}–{:.3}\\n\\nRecall: {:.3}–{:.3}\\n\\nF1: {:.3}–{:.3}\\n\\nIterations: {}\\n",
            advanced.precision_ci.0,
            advanced.precision_ci.1,
            advanced.recall_ci.0,
            advanced.recall_ci.1,
            advanced.f1_ci.0,
            advanced.f1_ci.1,
            advanced.bootstrap_iterations
        ));
    }

    out
}

/// Render an HTMX-friendly HTML fragment summarizing the evaluation.
pub fn render_html(summary: &EvalSummary, baseline: Option<&EvalSummary>) -> String {
    let mut out = String::new();
    out.push_str(
        r#"<div class="space-y-6">
  <div class="rounded-xl border border-slate-200 bg-white p-5 space-y-3">
    <div class="flex items-center justify-between">
      <h3 class="text-lg font-semibold text-slate-900">Snapshot</h3>
      <span class="text-sm text-slate-500">Precision/recall/accuracy</span>
    </div>
    <div class="grid gap-4 md:grid-cols-4">
"#,
    );

    for (label, value) in [
        ("Precision", summary.precision),
        ("Recall", summary.recall),
        ("Accuracy", summary.accuracy),
        ("AutoMapped precision", summary.auto_mapped_precision),
    ] {
        let _ = write!(
            out,
            r#"<div class="rounded-lg border border-slate-200 p-4">
  <p class="text-sm text-slate-500">{label}</p>
  <p class="text-2xl font-semibold text-slate-900">{value:.3}</p>
</div>"#
        );
    }

    out.push_str("</div>");

    if let Some(base) = baseline {
        out.push_str(r#"<div class="mt-4 rounded-lg border border-indigo-100 bg-indigo-50 p-4 text-sm text-indigo-900">"#);
        let changes = changelog(summary, base);
        if changes.is_empty() {
            out.push_str("No changes vs. baseline dataset.");
        } else {
            out.push_str("<ul class=\"list-disc space-y-1 pl-5\">");
            for line in changes {
                out.push_str(&format!("<li>{line}</li>"));
            }
            out.push_str("</ul>");
        }
        out.push_str("</div>");
    }

    out.push_str("</div>");

    out.push_str(
        r#"<div class="grid gap-4 md:grid-cols-2">
  <div class="rounded-xl border border-slate-200 bg-white p-5">
    <h3 class="text-base font-semibold text-slate-900 mb-3">State counts</h3>
    <table class="min-w-full text-sm">
      <thead class="text-left text-slate-500">
        <tr><th>State</th><th class="text-right">Count</th></tr>
      </thead>
      <tbody>"#,
    );

    for (state, count) in &summary.state_counts {
        out.push_str(&format!(
            "<tr><td class=\"py-1 text-slate-700\">{state}</td><td class=\"py-1 text-right font-semibold\">{count}</td></tr>"
        ));
    }
    out.push_str("</tbody></table></div>");

    out.push_str(
        r#"<div class="rounded-xl border border-slate-200 bg-white p-5">
    <h3 class="text-base font-semibold text-slate-900 mb-3">Reason counts</h3>
    <table class="min-w-full text-sm">
      <thead class="text-left text-slate-500">
        <tr><th>Reason</th><th class="text-right">Count</th></tr>
      </thead>
      <tbody>"#,
    );
    for (reason, count) in &summary.reason_counts {
        out.push_str(&format!(
            "<tr><td class=\"py-1 text-slate-700\">{reason}</td><td class=\"py-1 text-right font-semibold\">{count}</td></tr>"
        ));
    }
    out.push_str("</tbody></table></div></div>");

    out.push_str(
        r#"<div class="rounded-xl border border-slate-200 bg-white p-5 space-y-3">
    <h3 class="text-base font-semibold text-slate-900">Calibration buckets</h3>
    <table class="min-w-full text-sm">
      <thead class="text-left text-slate-500">
        <tr>
          <th>Bucket</th>
          <th>Range</th>
          <th class="text-right">Predictions</th>
          <th class="text-right">Correct</th>
          <th class="text-right">Accuracy</th>
        </tr>
      </thead>
      <tbody>"#,
    );

    for bucket in &summary.score_buckets {
        let range = match (bucket.lower_bound, bucket.upper_bound) {
            (Some(lower), Some(upper)) => format!("{lower:.1}–{upper:.1}"),
            _ => "n/a".to_string(),
        };
        out.push_str(&format!(
            "<tr>
  <td class=\"py-1 text-slate-700\">{}</td>
  <td class=\"py-1 text-slate-700\">{}</td>
  <td class=\"py-1 text-right font-semibold\">{}</td>
  <td class=\"py-1 text-right font-semibold\">{}</td>
  <td class=\"py-1 text-right font-semibold\">{:.3}</td>
</tr>",
            bucket.bucket, range, bucket.total, bucket.correct, bucket.accuracy
        ));
    }

    out.push_str("</tbody></table></div></div></div>");
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    pub dataset: String,
    pub recorded_at: String,
    pub summary: EvalSummary,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug)]
pub enum BaselineError {
    Io {
        source: io::Error,
        path: PathBuf,
    },
    Parse {
        source: serde_json::Error,
        path: PathBuf,
    },
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaselineError::Io { path, source } => {
                write!(f, "failed to read baseline {}: {}", path.display(), source)
            }
            BaselineError::Parse { path, source } => {
                write!(f, "failed to parse baseline {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for BaselineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BaselineError::Io { source, .. } => Some(source),
            BaselineError::Parse { source, .. } => Some(source),
        }
    }
}

pub fn baseline_path(dataset: &str) -> PathBuf {
    dataset_root().join(format!("{dataset}.baseline.json"))
}

pub fn load_baseline_snapshot(dataset: &str) -> Result<BaselineSnapshot, BaselineError> {
    let path = baseline_path(dataset);
    let mut file = File::open(&path).map_err(|source| BaselineError::Io {
        source,
        path: path.clone(),
    })?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|source| BaselineError::Io {
            source,
            path: path.clone(),
        })?;
    serde_json::from_str(&buf).map_err(|source| BaselineError::Parse { source, path })
}

fn comparison_rows<'a>(
    summary: &'a EvalSummary,
    baseline: &'a EvalSummary,
) -> [(&'static str, f32, f32); 4] {
    [
        ("Precision", summary.precision, baseline.precision),
        ("Recall", summary.recall, baseline.recall),
        ("Accuracy", summary.accuracy, baseline.accuracy),
        (
            "AutoMapped precision",
            summary.auto_mapped_precision,
            baseline.auto_mapped_precision,
        ),
    ]
}

fn delta_string(current: f32, previous: f32) -> String {
    let delta = current - previous;
    if delta.abs() < f32::EPSILON {
        "0.000".to_string()
    } else {
        format!("{delta:+.3}")
    }
}

fn changelog(current: &EvalSummary, baseline: &EvalSummary) -> Vec<String> {
    comparison_rows(current, baseline)
        .iter()
        .filter_map(|(label, now, prev)| {
            let delta = now - prev;
            if delta.abs() < 0.0005 {
                None
            } else if delta.is_sign_positive() {
                Some(format!(
                    "{label} improved by {:+.3} (baseline {:.3})",
                    delta, prev
                ))
            } else {
                Some(format!(
                    "{label} slipped by {:+.3} (baseline {:.3})",
                    delta, prev
                ))
            }
        })
        .collect()
}
