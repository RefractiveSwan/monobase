use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::eval::EvalSummary;
use refractive_swan_eval::aggregate_summaries;

/// Mapping summary slice (aggregated) suitable for hub-level reporting.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MappingSummarySlice {
    pub total: usize,
    pub by_code_kind: BTreeMap<String, usize>,
    pub by_license_tier: BTreeMap<String, usize>,
    pub extern_lookup_success: usize,
    pub extern_lookup_miss: usize,
    pub extern_lookup_error: usize,
}

impl MappingSummarySlice {
    pub fn merge(&mut self, other: &MappingSummarySlice) {
        self.total += other.total;
        for (key, value) in &other.by_code_kind {
            *self.by_code_kind.entry(key.clone()).or_default() += value;
        }
        for (key, value) in &other.by_license_tier {
            *self.by_license_tier.entry(key.clone()).or_default() += value;
        }
        self.extern_lookup_success += other.extern_lookup_success;
        self.extern_lookup_miss += other.extern_lookup_miss;
        self.extern_lookup_error += other.extern_lookup_error;
    }
}

impl From<refractive_swan_mapping::MappingSummary> for MappingSummarySlice {
    fn from(value: refractive_swan_mapping::MappingSummary) -> Self {
        MappingSummarySlice {
            total: value.total,
            by_code_kind: value.by_code_kind,
            by_license_tier: value.by_license_tier,
            extern_lookup_success: value.extern_lookup_success,
            extern_lookup_miss: value.extern_lookup_miss,
            extern_lookup_error: value.extern_lookup_error,
        }
    }
}

/// Federated mapping summary across nodes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FederatedMappingSummary {
    pub aggregated: MappingSummarySlice,
    pub by_node: BTreeMap<String, MappingSummarySlice>,
}

/// Federated eval summary across nodes.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FederatedEvalSummary {
    pub aggregated: EvalSummary,
    pub by_node: BTreeMap<String, EvalSummary>,
}

/// Aggregate mapping summaries from multiple nodes.
pub fn aggregate_mapping_summaries(
    summaries: Vec<(String, MappingSummarySlice)>,
) -> FederatedMappingSummary {
    let mut aggregated = MappingSummarySlice::default();
    let mut by_node = BTreeMap::new();
    for (node, summary) in summaries {
        aggregated.merge(&summary);
        by_node.insert(node, summary);
    }
    FederatedMappingSummary {
        aggregated,
        by_node,
    }
}

/// Aggregate eval summaries from multiple nodes (weighting via `aggregate_summaries`).
pub fn aggregate_eval_summaries(summaries: Vec<(String, EvalSummary)>) -> FederatedEvalSummary {
    let mut aggregated = EvalSummary::default();
    let mut by_node = BTreeMap::new();
    for (node, summary) in summaries {
        aggregate_summaries(&mut aggregated, summary.clone());
        by_node.insert(node, summary);
    }
    FederatedEvalSummary {
        aggregated,
        by_node,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_eval::EvalSummary;
    use refractive_swan_mapping::MappingSummary;

    #[test]
    fn aggregates_mapping_summaries() {
        let mut a = MappingSummary::default();
        a.record(
            refractive_swan_terminology::CodeKind::KnownLicensedSystem,
            Some("licensed"),
        );
        let mut b = MappingSummary::default();
        b.record(
            refractive_swan_terminology::CodeKind::KnownOpenSystem,
            Some("open"),
        );
        let report = aggregate_mapping_summaries(vec![
            ("node-a".into(), MappingSummarySlice::from(a)),
            ("node-b".into(), MappingSummarySlice::from(b)),
        ]);
        assert_eq!(report.aggregated.total, 2);
        assert_eq!(report.by_node.len(), 2);
        assert_eq!(
            report
                .aggregated
                .by_license_tier
                .get("licensed")
                .copied()
                .unwrap_or(0),
            1
        );
    }

    #[test]
    fn aggregates_eval_summaries() {
        let mut s1 = EvalSummary::default();
        s1.total_cases = 1;
        s1.correct = 1;
        let mut s2 = EvalSummary::default();
        s2.total_cases = 2;
        s2.correct = 1;
        let report = aggregate_eval_summaries(vec![
            ("node-a".into(), s1.clone()),
            ("node-b".into(), s2.clone()),
        ]);
        assert_eq!(report.by_node.len(), 2);
        assert_eq!(report.aggregated.total_cases, 3);
        assert!(report.aggregated.correct >= 2);
    }
}
