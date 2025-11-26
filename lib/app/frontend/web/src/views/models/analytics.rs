use chrono::{Duration, NaiveDate};
use std::collections::{BTreeMap, HashMap};

use super::types::{
    AnalyticsConceptTile, AnalyticsSummaryFilter, AnalyticsSummaryView, AnalyticsTimeBucket,
    CohortRowView, CohortView, CountStat,
};
use crate::client::{AnalyticsSummaryResponse, CohortResponse};

impl AnalyticsSummaryView {
    pub fn from_response(resp: &AnalyticsSummaryResponse) -> Self {
        Self::from_response_with_filter(resp, &AnalyticsSummaryFilter::default())
    }

    pub fn from_response_with_filter(
        resp: &AnalyticsSummaryResponse,
        filter: &AnalyticsSummaryFilter,
    ) -> Self {
        let filtered_rows = filter_rows(&resp.rows, filter);
        let (state_tally, concept_tally, time_buckets) = aggregate_analytics_data(&filtered_rows);

        let top_concepts = build_top_concepts(concept_tally);
        let state_counts = build_state_counts(state_tally);
        let time_buckets = build_time_buckets(time_buckets);

        Self {
            top_concepts,
            state_counts,
            time_buckets,
        }
    }
}

fn filter_rows(
    rows: &[crate::client::AnalyticsSummaryRow],
    filter: &AnalyticsSummaryFilter,
) -> Vec<crate::client::AnalyticsSummaryRow> {
    let mut filtered = rows.to_vec();
    if let Some(state) = filter.state.as_ref().map(|value| value.to_lowercase()) {
        filtered.retain(|row| {
            row.mapping_state
                .as_ref()
                .map(|value| value.to_lowercase() == state)
                .unwrap_or(false)
        });
    }
    if let Some(days) = filter.range_days {
        if let Some(max_date) = filtered
            .iter()
            .filter_map(|row| parse_bucket(row.time_bucket.as_deref()))
            .max()
        {
            let threshold = max_date - Duration::days(days as i64 - 1);
            filtered.retain(|row| {
                parse_bucket(row.time_bucket.as_deref())
                    .map(|date| date >= threshold)
                    .unwrap_or(true)
            });
        }
    }
    filtered
}

fn parse_bucket(value: Option<&str>) -> Option<NaiveDate> {
    value.and_then(|bucket| NaiveDate::parse_from_str(bucket, "%Y-%m-%d").ok())
}

fn aggregate_analytics_data(
    rows: &[crate::client::AnalyticsSummaryRow],
) -> (
    HashMap<String, usize>,
    HashMap<(String, String), usize>,
    BTreeMap<String, HashMap<String, usize>>,
) {
    let mut state_tally: HashMap<String, usize> = HashMap::new();
    let mut concept_tally: HashMap<(String, String), usize> = HashMap::new();
    let mut buckets: BTreeMap<String, HashMap<String, usize>> = BTreeMap::new();

    for row in rows {
        let state = row
            .mapping_state
            .clone()
            .unwrap_or_else(|| "unknown".into());

        aggregate_state_count(&mut state_tally, &state, row.count);
        aggregate_concept_count(&mut concept_tally, row);
        aggregate_time_bucket(&mut buckets, row, &state);
    }

    (state_tally, concept_tally, buckets)
}

fn aggregate_state_count(state_tally: &mut HashMap<String, usize>, state: &str, count: usize) {
    *state_tally.entry(state.to_string()).or_default() += count;
}

fn aggregate_concept_count(
    concept_tally: &mut HashMap<(String, String), usize>,
    row: &crate::client::AnalyticsSummaryRow,
) {
    let preferred = row
        .preferred_name
        .clone()
        .unwrap_or_else(|| "Unknown concept".into());
    *concept_tally
        .entry((row.ncit_id.clone(), preferred))
        .or_default() += row.count;
}

fn aggregate_time_bucket(
    buckets: &mut BTreeMap<String, HashMap<String, usize>>,
    row: &crate::client::AnalyticsSummaryRow,
    state: &str,
) {
    if let Some(bucket) = &row.time_bucket {
        let bucket_tally = buckets.entry(bucket.clone()).or_default();
        *bucket_tally.entry(state.to_string()).or_default() += row.count;
    }
}

fn build_top_concepts(
    concept_tally: HashMap<(String, String), usize>,
) -> Vec<AnalyticsConceptTile> {
    let mut top_concepts = concept_tally
        .into_iter()
        .map(|((ncit_id, preferred_name), total)| AnalyticsConceptTile {
            ncit_id,
            preferred_name,
            total,
        })
        .collect::<Vec<_>>();
    top_concepts.sort_by(|a, b| b.total.cmp(&a.total));
    top_concepts.truncate(5);
    top_concepts
}

fn build_state_counts(state_tally: HashMap<String, usize>) -> Vec<CountStat> {
    let mut state_counts = state_tally
        .into_iter()
        .map(|(label, count)| CountStat { label, count })
        .collect::<Vec<_>>();
    state_counts.sort_by(|a, b| b.count.cmp(&a.count));
    state_counts
}

fn build_time_buckets(
    buckets: BTreeMap<String, HashMap<String, usize>>,
) -> Vec<AnalyticsTimeBucket> {
    buckets
        .into_iter()
        .map(|(bucket, tally)| AnalyticsTimeBucket {
            bucket,
            state_counts: tally
                .into_iter()
                .map(|(label, count)| CountStat { label, count })
                .collect(),
        })
        .collect()
}

impl CohortView {
    pub fn from_response(resp: &CohortResponse) -> Self {
        let rows = resp
            .rows
            .iter()
            .map(|row| CohortRowView {
                sr_id: row.sr_id.clone(),
                patient_id: row.patient_id.clone().unwrap_or_else(|| "unknown".into()),
                encounter_id: row.encounter_id.clone().unwrap_or_else(|| "unknown".into()),
                ncit_id: row.ncit_id.clone().unwrap_or_else(|| "NO_MATCH".into()),
                description: row.description.clone(),
                status: row.status.clone(),
                intent: row.intent.clone(),
                ordered_at: row.ordered_at.clone().unwrap_or_else(|| "unknown".into()),
                mapping_state: row
                    .mapping_state
                    .clone()
                    .unwrap_or_else(|| "unknown".into()),
            })
            .collect();
        Self {
            total: resp.total,
            rows,
        }
    }
}
