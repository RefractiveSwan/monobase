use maud::{Markup, html};
use refractive_swan_contracts::eval::{DatasetTier, EvalSummary};

use crate::views::components::{badge::*, button::*, card::*, table::*, typography::*};
use crate::views::layout::{Breadcrumb, PageCallout, PageShellProps, page_shell};
use crate::views::models::{
    EvalJobStatusView, EvalJobView, EvalSummaryViewData, MappingResultsView, PageContext,
};

pub fn render_eval_page(ctx: &PageContext) -> String {
    page_shell(PageShellProps {
        title: "Evaluation Control Center",
        chrome: &ctx.chrome,
        content: html! {
            (render_dataset_tiers(ctx))
            div class="grid gap-6 xl:grid-cols-3" {
                div class="space-y-6 xl:col-span-2" {
                    (render_eval_section(ctx))
                    (calibration_panel(&ctx.eval_jobs))
                }
                div class="space-y-6" {
                    (eval_jobs_panel(&ctx.eval_jobs))
                    (compare_panel(&ctx.eval_jobs, None))
                }
            }
        },
        breadcrumbs: vec![
            Breadcrumb::home(),
            Breadcrumb::new("Evaluation", Some("/eval")),
        ],
        callouts: vec![PageCallout::info(
            "Dataset store",
            "Dataset dropdown is backed by refractive_swan_eval::DatasetStore so UI + CLI stay in sync."
        )],
    })
    .into_string()
}

pub fn render_eval_jobs_fragment(jobs: &[EvalJobView]) -> String {
    eval_jobs_panel(jobs).into_string()
}

pub fn render_eval_calibration_fragment(jobs: &[EvalJobView]) -> String {
    calibration_panel(jobs).into_string()
}

pub fn render_eval_compare_fragment(
    jobs: &[EvalJobView],
    selection: Option<[String; 2]>,
) -> String {
    compare_panel(jobs, selection).into_string()
}

pub(crate) fn render_no_match_explorer(results: Option<&MappingResultsView>) -> Markup {
    card(html! {
        (card_header("NoMatch Explorer", None))
        (card_body(html! {
            div class="grid gap-6 lg:grid-cols-2" {
                (render_no_match_table(results))
                (render_remediation_panel(results.map_or(false, |view| !view.no_matches.is_empty())))
            }
        }))
    })
}

fn render_no_match_table(results: Option<&MappingResultsView>) -> Markup {
    html! {
        @if let Some(view) = results {
            @if view.no_matches.is_empty() {
                p class="text-sm text-emerald-700 font-medium" { "✓ No unmapped codes found in this run." }
            } @else {
                (table_container(html! {
                    (table_header(&["ServiceRequest", "Code", "Reason"]))
                    tbody class="bg-white divide-y divide-gray-200" {
                        @for row in &view.no_matches {
                            (table_row(html! {
                                (table_cell(html! {
                                    p class="font-mono text-xs font-medium" { (&row.sr_id) }
                                    p class="text-xs text-gray-500" { (&row.system) }
                                }))
                                (table_cell(html! {
                                    p class="font-mono text-xs font-semibold" { (&row.code) }
                                    p class="text-xs text-gray-500" { (&row.display) }
                                }))
                                (table_cell(html! {
                                    span class="inline-flex rounded bg-rose-50 px-2 py-0.5 text-xs font-medium text-rose-800 border border-rose-100" {
                                        (row.reason.as_deref().unwrap_or("unknown"))
                                    }
                                }))
                            }))
                        }
                    }
                }))
            }
        } @else {
            div class="text-center py-12" {
                svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" {}
                }
                h3 class="mt-4 text-sm font-semibold text-gray-900" { "No Data Yet" }
                p class="mt-2 text-sm text-gray-500 max-w-sm mx-auto" { "Submit a FHIR Bundle to identify codes that couldn't be mapped to NCIt concepts." }
            }
        }
    }
}

fn render_remediation_panel(has_results: bool) -> Markup {
    html! {
        div class="rounded-md border border-dashed border-gray-200 bg-gray-50 p-4 space-y-3" {
            h4 class="text-xs font-semibold text-gray-700 uppercase tracking-wide" { "Remediation tips" }
            ul class="text-xs text-gray-600 space-y-2" {
                li {
                    span class="font-semibold text-navy-900" { "Inspect CLI explanations:" }
                    br;
                    code class="text-[11px] bg-white px-2 py-1 rounded border border-gray-200 block mt-1" {
                        "refractive_swan_cli map_codes --explain ./codes.ndjson"
                    }
                }
                li {
                    span class="font-semibold text-navy-900" { "Review docs:" }
                    br;
                    a href="/docs" class="text-sky-700 hover:underline" target="_blank" {
                        "Mapping eval quickstart"
                    }
                    span { " (bundle requirements & troubleshooting)" }
                }
                li {
                    span class="font-semibold text-navy-900" { "Gather context:" }
                    br;
                    span { "Leverage upload history and HTMX replay to iterate on fixes without re-uploading bundles." }
                }
            }
            @if !has_results {
                p class="text-[11px] text-gray-500" { "Hints stay available even when no results yet so new users can plan remediation workflows." }
            }
        }
    }
}

fn render_eval_section(ctx: &PageContext) -> Markup {
    html! {
        (card(html! {
            (card_header("Evaluation", Some(html! {
                span class="text-sm text-gray-500" { "Refractive Swan mapping eval datasets" }
            })))
            (card_body(html! {
                 form hx-post="/eval/run" hx-target="#eval-jobs-panel" hx-swap="outerHTML" class="flex flex-wrap gap-3 items-center text-sm" {
                    (label_text("Dataset"))
                    select id="dataset" name="dataset" class="rounded-md border-gray-300 px-3 py-1.5 text-sm" {
                        @for ds in &ctx.datasets {
                            @let manifest = &ds.manifest;
                            option
                                value=(manifest.name)
                                selected[(ctx.selected_eval_dataset == manifest.name)]
                                disabled[ds.disabled]
                            { (format!("{} ({} rows)", manifest.name, manifest.n_cases)) }
                        }
                    }
                    (label_text("Top K"))
                    input type="number" id="top_k" name="top_k" value="1" min="1" max="5" class="w-16 rounded-md border-gray-300 px-2 py-1 text-sm" {}
                    (primary_button(ButtonProps {
                        text: "Run eval",
                        type_: "submit",
                        ..ButtonProps::default()
                    }))
                }
                div class="mt-4" {
                    @if let Some(eval) = &ctx.eval {
                        (render_eval_summary(&eval.summary, &eval.dataset))
                    } @else {
                        p class="text-sm text-gray-500" { "No eval summary available yet." }
                    }
                }
            }))
        }))
    }
}

fn render_eval_summary(summary: &EvalSummary, dataset: &str) -> Markup {
    html! {
        div class="space-y-6" {
            div class="flex items-center justify-between" {
                (subsection_heading(&format!("Dataset: {}", dataset)))
                span class="text-sm text-gray-500" { (format!("Total cases: {}", summary.total_cases)) }
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Precision", &format!("{:.1}%", summary.precision * 100.0), None))
                (metric_card("Recall", &format!("{:.1}%", summary.recall * 100.0), None))
                (metric_card("Coverage", &format!("{:.1}%", summary.coverage * 100.0), None))
            }
            div class="grid gap-4 md:grid-cols-3" {
                (metric_card("Top1 accuracy", &format!("{:.1}%", summary.top1_accuracy * 100.0), None))
                (metric_card("Top3 accuracy", &format!("{:.1}%", summary.top3_accuracy * 100.0), None))
                (metric_card("AutoMapped precision", &format!("{:.1}%", summary.auto_mapped_precision * 100.0), None))
            }
            div class="bg-gray-50 rounded-md border border-gray-200 p-4" {
                h4 class="text-xs font-bold text-gray-500 uppercase tracking-wide mb-2" { "State counts" }
                ul class="text-sm text-gray-600 space-y-1 font-mono" {
                    @for (state, count) in &summary.state_counts {
                        li { (format!("{state}: {count}")) }
                    }
                }
            }
            div class="bg-gray-50 rounded-md border border-gray-200 p-4" {
                h4 class="text-xs font-bold text-gray-500 uppercase tracking-wide mb-2" { "Top NoMatch reasons" }
                ul class="text-sm text-gray-600 space-y-1 font-mono" {
                    @for (reason, count) in &summary.reason_counts {
                        li { (format!("{reason}: {count}")) }
                    }
                }
            }
        }
    }
}

fn render_dataset_tiers(ctx: &PageContext) -> Markup {
    const TIER_ORDER: [DatasetTier; 4] = [
        DatasetTier::Gold,
        DatasetTier::Silver,
        DatasetTier::Bronze,
        DatasetTier::Uncategorized,
    ];
    fn tier_index(tier: &DatasetTier) -> usize {
        match tier {
            DatasetTier::Gold => 0,
            DatasetTier::Silver => 1,
            DatasetTier::Bronze => 2,
            DatasetTier::Uncategorized => 3,
        }
    }
    let mut grouped: Vec<Vec<&refractive_swan_contracts::eval::DatasetManifest>> =
        vec![Vec::new(); TIER_ORDER.len()];
    for entry in &ctx.datasets {
        let manifest = &entry.manifest;
        let idx = tier_index(&manifest.tier);
        grouped[idx].push(manifest);
    }

    card(html! {
        (card_header("Dataset catalog", Some(html! {
            span class="text-xs text-gray-500" { "Tier metadata pulled from refractive_swan_eval manifests" }
        })))
        (card_body(html! {
            @if ctx.datasets.is_empty() {
                p class="text-sm text-gray-500 italic" { "Dataset store not configured. Set refractive_swan_eval env vars to load manifests." }
            } @else {
                div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4 text-sm" {
                    @for (bucket, manifests) in TIER_ORDER.iter().zip(grouped.into_iter()) {
                        (render_tier_column(bucket, &manifests))
                    }
                }
            }
        }))
    })
}

fn render_tier_column(
    tier: &DatasetTier,
    manifests: &[&refractive_swan_contracts::eval::DatasetManifest],
) -> Markup {
    let TierMeta {
        label,
        description,
        badge_class,
        badge_text,
    } = tier_meta(tier);
    html! {
        div class="rounded-md border border-gray-100 bg-gray-50 p-4 space-y-3" {
            div class="flex items-center justify-between" {
                h4 class="text-xs font-semibold text-gray-700 uppercase tracking-wide" { (label) }
                span class=(format!("text-[11px] font-semibold px-2 py-0.5 rounded-full border {}", badge_class)) {
                    (badge_text)
                }
            }
            p class="text-xs text-gray-500" { (description) }
            @if manifests.is_empty() {
                p class="text-xs text-gray-500 italic" { "No datasets assigned yet." }
            } @else {
                @for manifest in manifests {
                    div class="rounded bg-white border border-gray-200 p-3 space-y-1 shadow-sm" {
                        p class="font-semibold text-navy-900" { (&manifest.name) }
                        p class="text-xs text-gray-500" { (format!("Version {} \u{2022} {} cases", manifest.version, manifest.n_cases)) }
                        @if let Some(source) = &manifest.source {
                            p class="text-xs text-gray-500" { (format!("Source: {}", source)) }
                        }
                        @if let Some(license) = &manifest.license {
                            p class="text-xs text-gray-500" { (format!("License: {}", license)) }
                        }
                        @if let Some(notes) = &manifest.notes {
                            p class="text-xs text-gray-500" { (notes) }
                        }
                    }
                }
            }
        }
    }
}

struct TierMeta {
    label: &'static str,
    description: &'static str,
    badge_class: &'static str,
    badge_text: &'static str,
}

fn tier_meta(tier: &DatasetTier) -> TierMeta {
    match tier {
        DatasetTier::Gold => TierMeta {
            label: "Gold tier",
            description: "Fully curated reference datasets with operator review + compliance sign-off.",
            badge_class: "bg-amber-50 text-amber-800 border-amber-200",
            badge_text: "gold",
        },
        DatasetTier::Silver => TierMeta {
            label: "Silver tier",
            description: "High-signal mixes ready for demo + accuracy checks.",
            badge_class: "bg-slate-50 text-slate-700 border-slate-200",
            badge_text: "silver",
        },
        DatasetTier::Bronze => TierMeta {
            label: "Bronze tier",
            description: "Seed datasets used for smoke/regression runs.",
            badge_class: "bg-orange-50 text-orange-700 border-orange-200",
            badge_text: "bronze",
        },
        DatasetTier::Uncategorized => TierMeta {
            label: "Uncategorized",
            description: "New or ad-hoc payloads that still need metadata.",
            badge_class: "bg-gray-50 text-gray-600 border-gray-200",
            badge_text: "uncategorized",
        },
    }
}

fn eval_jobs_panel(jobs: &[EvalJobView]) -> Markup {
    html! {
        div id="eval-jobs-panel" hx-get="/eval/jobs" hx-trigger="load, every 7s" hx-target="#eval-jobs-panel" hx-swap="outerHTML" {
            (card(html! {
                (card_header("Eval job queue", Some(html! {
                    span class="text-xs text-gray-500" { "Jobs execute via backend /api/eval/run and cache previews here." }
                })))
                (card_body(html! {
                    @if jobs.is_empty() {
                        p class="text-sm text-gray-500 italic" { "No eval jobs queued yet. Use the form above to kick off a run." }
                    } @else {
                        div class="space-y-3" {
                            @for job in jobs {
                                (render_job_card(job))
                            }
                        }
                    }
                }))
            }))
            form id="compare-run-form"
                hx-post="/eval/compare"
                hx-target="#eval-compare-panel"
                hx-swap="outerHTML"
                class="mt-3 flex flex-wrap items-center justify-between gap-2 rounded-md border border-dashed border-gray-200 bg-gray-50 px-3 py-2 text-xs text-gray-600" {
                span { "Select any two completed runs below to diff precision/recall deltas." }
                button type="submit" class="inline-flex items-center rounded-md border border-navy-200 bg-white px-3 py-1 font-semibold text-navy-900 shadow-sm hover:bg-navy-50" {
                    "Compare runs"
                }
            }
        }
    }
}

fn render_job_card(job: &EvalJobView) -> Markup {
    let compare_ready = matches!(job.status, EvalJobStatusView::Completed) && job.summary.is_some();
    html! {
        div class="rounded border border-gray-200 bg-white p-3 shadow-sm space-y-2" {
            div class="flex items-center justify-between" {
                div {
                    p class="text-sm font-semibold text-navy-900" { (&job.dataset) }
                    p class="text-xs text-gray-500" { (format!("Submitted {}", job.submitted_at)) }
                }
                (job_status_badge(job))
            }
            p class="text-xs text-gray-500" { (format!("Top K: {}", job.top_k)) }
            @if let Some(summary) = &job.summary {
                (render_summary_preview(summary))
            } @else if let Some(error) = &job.error {
                p class="text-xs text-rose-600" { (format!("Error: {}", error)) }
            } @else {
                p class="text-xs text-gray-500" { "Awaiting summary..." }
            }
            div class="flex items-center justify-between border-t border-dashed border-gray-200 pt-2 text-xs text-gray-500" {
                label class="inline-flex items-center gap-2" {
                    input type="checkbox"
                        form="compare-run-form"
                        name="job_ids"
                        value=(&job.id)
                        disabled[!compare_ready]
                        class="rounded border-gray-300 text-navy-600 focus:ring-navy-500" {};
                    span { "Compare" }
                }
                span class="text-[11px]" {
                    @if compare_ready {
                        "Ready"
                    } @else {
                        "Pending completion"
                    }
                }
            }
        }
    }
}

fn job_status_badge(job: &EvalJobView) -> Markup {
    let label = match job.status {
        EvalJobStatusView::Queued => "queued",
        EvalJobStatusView::Running => "running",
        EvalJobStatusView::Completed => "completed",
        EvalJobStatusView::Failed => "failed",
    };
    status_badge(label)
}

fn render_summary_preview(summary: &EvalSummaryViewData) -> Markup {
    html! {
        div class="grid grid-cols-3 gap-2 text-center text-xs" {
            div class="rounded bg-emerald-50 px-2 py-1" {
                p class="font-semibold text-emerald-900" { (format!("{:.1}%", summary.precision * 100.0)) }
                p class="text-[10px] text-emerald-700" { "Precision" }
            }
            div class="rounded bg-sky-50 px-2 py-1" {
                p class="font-semibold text-sky-900" { (format!("{:.1}%", summary.recall * 100.0)) }
                p class="text-[10px] text-sky-700" { "Recall" }
            }
            div class="rounded bg-violet-50 px-2 py-1" {
                p class="font-semibold text-violet-900" { (format!("{:.1}%", summary.coverage * 100.0)) }
                p class="text-[10px] text-violet-700" { "Coverage" }
            }
        }
    }
}

fn calibration_panel(jobs: &[EvalJobView]) -> Markup {
    html! {
        div id="eval-calibration-panel" hx-get="/eval/calibration" hx-trigger="load, every 15s" hx-target="#eval-calibration-panel" hx-swap="outerHTML" {
            (card(html! {
                (card_header("Calibration & stratified metrics", Some(html! {
                    span class="text-xs text-gray-500" { "Latest completed eval run" }
                })))
                (card_body(html! {
                    @if let Some((job, summary)) = latest_completed_summary(jobs) {
                        div class="space-y-4" {
                            div class="flex items-center justify-between text-xs text-gray-500" {
                                span { (format!("Dataset: {}", job.dataset)) }
                                span { (job.submitted_at.clone()) }
                            }
                            div class="grid gap-4 md:grid-cols-2" {
                                (render_score_buckets(summary))
                                (render_system_metrics(summary))
                            }
                        }
                    } @else {
                        p class="text-sm text-gray-500 italic" { "No completed runs yet. Kick off an eval to view calibration buckets." }
                    }
                }))
            }))
        }
    }
}

fn latest_completed_summary(jobs: &[EvalJobView]) -> Option<(&EvalJobView, &EvalSummaryViewData)> {
    jobs.iter()
        .find(|job| matches!(job.status, EvalJobStatusView::Completed) && job.summary.is_some())
        .and_then(|job| job.summary.as_ref().map(|summary| (job, summary)))
}

fn render_score_buckets(summary: &EvalSummaryViewData) -> Markup {
    let buckets = summary.score_buckets.iter().take(5);
    html! {
        div class="rounded-md border border-gray-200 bg-gray-50 p-3 space-y-2" {
            h4 class="text-xs font-semibold text-gray-600 uppercase tracking-wide" { "Score buckets" }
            @if summary.score_buckets.is_empty() {
                p class="text-xs text-gray-500" { "Bucketed accuracy data not available." }
            } @else {
                @for bucket in buckets {
                    div class="space-y-1" {
                        div class="flex items-center justify-between text-[11px] text-gray-600" {
                            span { (&bucket.label) }
                            span { (format!("{:.1}% (n={})", bucket.accuracy * 100.0, bucket.total)) }
                        }
                        div class="h-2 w-full rounded-full bg-gray-200" {
                            div class="h-2 rounded-full bg-navy-500" style=(format!("width: {:.0}%;", bucket.accuracy * 100.0)) {}
                        }
                    }
                }
            }
        }
    }
}

fn render_system_metrics(summary: &EvalSummaryViewData) -> Markup {
    let systems = summary.per_system.iter().take(4);
    html! {
        div class="rounded-md border border-gray-200 bg-white p-3 space-y-2" {
            h4 class="text-xs font-semibold text-gray-600 uppercase tracking-wide" { "Per-system calibration" }
            @if summary.per_system.is_empty() {
                p class="text-xs text-gray-500" { "No stratified metrics emitted." }
            } @else {
                @for system in systems {
                    div class="space-y-1 border-b border-dashed border-gray-200 pb-2 last:border-0 last:pb-0" {
                        div class="flex items-center justify-between text-xs font-semibold text-navy-900" {
                            span { (&system.label) }
                            span class="text-gray-500 font-normal" { (format!("n={}", system.total_cases)) }
                        }
                        div class="grid grid-cols-3 gap-2 text-[11px] text-gray-600" {
                            span { (format!("P {:.1}%", system.precision * 100.0)) }
                            span { (format!("R {:.1}%", system.recall * 100.0)) }
                            span { (format!("F1 {:.1}%", system.f1 * 100.0)) }
                        }
                    }
                }
            }
        }
    }
}

fn compare_panel(jobs: &[EvalJobView], selection: Option<[String; 2]>) -> Markup {
    let (primary, secondary) = pick_compare_jobs(jobs, selection.as_ref());
    html! {
        div id="eval-compare-panel" {
            (card(html! {
                (card_header("Compare runs", Some(html! {
                    span class="text-xs text-gray-500" { "Diff precision/recall/coverage across runs" }
                })))
                (card_body(html! {
                    div class="flex items-center justify-between text-xs text-gray-500" {
                        span { "Completed runs populate automatically; use the compare form to override." }
                        button type="button"
                            hx-get="/eval/compare"
                            hx-target="#eval-compare-panel"
                            hx-swap="outerHTML"
                            class="inline-flex items-center rounded-md border border-gray-200 px-2 py-0.5 text-[11px] font-semibold text-navy-900 hover:bg-gray-50" {
                            "Reset selection"
                        }
                    }
                    @if let (Some(a), Some(b)) = (primary, secondary) {
                        (render_compare_table(a, b))
                    } @else {
                        p class="mt-3 text-sm text-gray-500 italic" { "Need at least two completed runs with summaries to compare." }
                    }
                }))
            }))
        }
    }
}

fn pick_compare_jobs<'a>(
    jobs: &'a [EvalJobView],
    selection: Option<&[String; 2]>,
) -> (Option<&'a EvalJobView>, Option<&'a EvalJobView>) {
    let completed: Vec<&EvalJobView> = jobs
        .iter()
        .filter(|job| matches!(job.status, EvalJobStatusView::Completed) && job.summary.is_some())
        .collect();
    if let Some(ids) = selection {
        let first = completed.iter().copied().find(|job| job.id == ids[0]);
        let second = completed.iter().copied().find(|job| job.id == ids[1]);
        if first.is_some() && second.is_some() {
            return (first, second);
        }
    }
    let mut iter = completed.into_iter();
    (iter.next(), iter.next())
}

fn render_compare_table(a: &EvalJobView, b: &EvalJobView) -> Markup {
    let metrics = [
        (
            "Precision",
            a.summary.as_ref().map(|s| s.precision),
            b.summary.as_ref().map(|s| s.precision),
        ),
        (
            "Recall",
            a.summary.as_ref().map(|s| s.recall),
            b.summary.as_ref().map(|s| s.recall),
        ),
        (
            "Coverage",
            a.summary.as_ref().map(|s| s.coverage),
            b.summary.as_ref().map(|s| s.coverage),
        ),
        (
            "F1",
            a.summary.as_ref().map(|s| s.f1),
            b.summary.as_ref().map(|s| s.f1),
        ),
        (
            "Top1 accuracy",
            a.summary.as_ref().map(|s| s.top1_accuracy),
            b.summary.as_ref().map(|s| s.top1_accuracy),
        ),
        (
            "Top3 accuracy",
            a.summary.as_ref().map(|s| s.top3_accuracy),
            b.summary.as_ref().map(|s| s.top3_accuracy),
        ),
    ];
    html! {
        div class="mt-4 space-y-2" {
            div class="grid grid-cols-4 text-[11px] font-semibold text-gray-500" {
                span {}
                span { (&a.dataset) }
                span { (&b.dataset) }
                span { "Δ (B-A)" }
            }
            @for (label, lhs, rhs) in metrics {
                (compare_row(label, lhs.unwrap_or(0.0), rhs.unwrap_or(0.0)))
            }
        }
    }
}

fn compare_row(label: &str, a: f32, b: f32) -> Markup {
    html! {
        div class="grid grid-cols-4 items-center text-sm border-t border-gray-100 py-1" {
            span class="text-xs font-medium text-gray-600" { (label) }
            span class="font-semibold text-navy-900" { (format!("{:.1}%", a * 100.0)) }
            span class="font-semibold text-navy-900" { (format!("{:.1}%", b * 100.0)) }
            span class=(if b >= a { "text-emerald-700 font-semibold text-xs" } else { "text-rose-700 font-semibold text-xs" }) {
                (format!("{:+.1} pp", (b - a) * 100.0))
            }
        }
    }
}
