use refractive_swan_configuration::{string_var, u64_var};
use refractive_swan_contracts::{MeshJobDescriptor, MeshJobType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClass {
    MappingJob,
    AnalyticsJob,
    EvalJob,
    ExportJob,
    NodeIntrospection,
}

#[derive(Debug, Clone)]
pub struct QueryDescriptor {
    pub class: QueryClass,
    pub expected_cardinality: Option<u64>,
    pub requester: Option<String>,
    #[allow(dead_code)]
    pub time_range: Option<TimeRange>,
}

impl QueryDescriptor {
    pub fn from_job(job: &MeshJobDescriptor) -> Self {
        let class = match job.job_type {
            MeshJobType::AnalyticsQuery => QueryClass::AnalyticsJob,
            MeshJobType::EvalDataset => QueryClass::EvalJob,
            MeshJobType::ExportJob => QueryClass::ExportJob,
            MeshJobType::MappingHealthCheck => QueryClass::MappingJob,
            MeshJobType::NodeIntrospection => QueryClass::NodeIntrospection,
        };
        let expected_cardinality = job
            .parameters
            .get("expected_cardinality")
            .and_then(|v| v.as_u64());
        let time_range = TimeRange::from_params(&job.parameters);
        let requester = job
            .governance_context
            .as_ref()
            .and_then(|v| v.get("requester"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Self {
            class,
            expected_cardinality,
            requester,
            time_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeRange {
    pub start: Option<String>,
    pub end: Option<String>,
}

impl TimeRange {
    pub fn from_params(params: &serde_json::Value) -> Option<Self> {
        let start = params
            .get("start_date")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let end = params
            .get("end_date")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if start.is_some() || end.is_some() {
            Some(TimeRange { start, end })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePolicy {
    pub dp_budget_daily: f64,
    pub dp_budget_consumed: f64,
    pub max_cardinality_no_dp: u64,
    pub export_allowed: bool,
    pub hub_registration_allowed: bool,
}

impl Default for NodePolicy {
    fn default() -> Self {
        Self {
            dp_budget_daily: 1.0,
            dp_budget_consumed: 0.0,
            max_cardinality_no_dp: 100,
            export_allowed: true,
            hub_registration_allowed: true,
        }
    }
}

impl NodePolicy {
    pub fn from_env() -> Self {
        let mut policy = Self::default();
        policy.dp_budget_daily = string_var("refractive_swan_DP_BUDGET_DAILY")
            .ok()
            .and_then(|v| v.and_then(|s| s.parse::<f64>().ok()))
            .unwrap_or(policy.dp_budget_daily);
        policy.dp_budget_consumed = 0.0;
        policy.max_cardinality_no_dp = u64_var("refractive_swan_MAX_CARDINALITY_NO_DP")
            .ok()
            .and_then(|v| v)
            .unwrap_or(policy.max_cardinality_no_dp);
        policy.export_allowed = string_var("refractive_swan_EXPORT_ALLOWED")
            .ok()
            .and_then(|v| v.map(|s| !s.eq_ignore_ascii_case("false")))
            .unwrap_or(policy.export_allowed);
        policy.hub_registration_allowed = string_var("refractive_swan_HUB_REGISTRATION_ALLOWED")
            .ok()
            .and_then(|v| v.map(|s| !s.eq_ignore_ascii_case("false")))
            .unwrap_or(policy.hub_registration_allowed);
        policy
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GovernanceDecision {
    Allow,
    AllowWithNoise { epsilon: f64 },
    Deny(String),
}

pub struct GovernanceEngine;

impl GovernanceEngine {
    pub fn evaluate(descriptor: &QueryDescriptor, policy: &NodePolicy) -> GovernanceDecision {
        let requester_note = descriptor
            .requester
            .as_deref()
            .map(|r| format!(" (requester={r})"))
            .unwrap_or_default();
        match descriptor.class {
            QueryClass::MappingJob | QueryClass::NodeIntrospection => GovernanceDecision::Allow,
            QueryClass::AnalyticsJob => {
                if let Some(card) = descriptor.expected_cardinality
                    && card > policy.max_cardinality_no_dp
                {
                    if policy.dp_budget_consumed + 0.1 > policy.dp_budget_daily {
                        return GovernanceDecision::Deny(format!(
                            "dp_budget_exceeded{requester_note}"
                        ));
                    }
                    return GovernanceDecision::AllowWithNoise { epsilon: 0.1 };
                }
                GovernanceDecision::Allow
            }
            QueryClass::EvalJob => GovernanceDecision::Allow,
            QueryClass::ExportJob => {
                if !policy.export_allowed {
                    return GovernanceDecision::Deny(format!("export_not_allowed{requester_note}"));
                }
                if policy.dp_budget_consumed + 0.5 > policy.dp_budget_daily {
                    return GovernanceDecision::Deny(format!("dp_budget_exceeded{requester_note}"));
                }
                GovernanceDecision::AllowWithNoise { epsilon: 0.5 }
            }
        }
    }

    pub fn consume_budget(policy: &mut NodePolicy, epsilon: f64) {
        policy.dp_budget_consumed += epsilon;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_contracts::MeshJobType;

    fn job(job_type: MeshJobType, expected_cardinality: Option<u64>) -> MeshJobDescriptor {
        MeshJobDescriptor {
            job_id: "job-1".into(),
            job_type,
            parameters: match expected_cardinality {
                Some(card) => serde_json::json!({ "expected_cardinality": card }),
                None => serde_json::json!({ "start_date": "2025-01-01", "end_date": "2025-01-31" }),
            },
            governance_context: None,
        }
    }

    #[test]
    fn analytics_requires_dp_when_cardinality_high() {
        let policy = NodePolicy {
            dp_budget_daily: 1.0,
            dp_budget_consumed: 0.0,
            max_cardinality_no_dp: 10,
            export_allowed: true,
            hub_registration_allowed: true,
        };
        let descriptor = QueryDescriptor::from_job(&job(MeshJobType::AnalyticsQuery, Some(100)));
        let decision = GovernanceEngine::evaluate(&descriptor, &policy);
        match decision {
            GovernanceDecision::AllowWithNoise { epsilon } => {
                assert!(epsilon > 0.0);
            }
            other => panic!("expected AllowWithNoise, got {:?}", other),
        }
    }

    #[test]
    fn descriptor_extracts_time_range() {
        let descriptor = QueryDescriptor::from_job(&job(MeshJobType::AnalyticsQuery, None));
        assert!(descriptor.time_range.is_some());
        let range = descriptor.time_range.unwrap();
        assert_eq!(range.start.as_deref(), Some("2025-01-01"));
        assert_eq!(range.end.as_deref(), Some("2025-01-31"));
    }

    #[test]
    fn export_denies_when_not_allowed() {
        let policy = NodePolicy {
            export_allowed: false,
            ..NodePolicy::default()
        };
        let descriptor = QueryDescriptor::from_job(&job(MeshJobType::ExportJob, None));
        let decision = GovernanceEngine::evaluate(&descriptor, &policy);
        assert!(matches!(decision, GovernanceDecision::Deny(_)));
    }

    #[test]
    fn export_denies_on_budget_exceeded() {
        let policy = NodePolicy {
            dp_budget_daily: 0.1,
            dp_budget_consumed: 0.1,
            export_allowed: true,
            ..NodePolicy::default()
        };
        let descriptor = QueryDescriptor::from_job(&job(MeshJobType::ExportJob, None));
        let decision = GovernanceEngine::evaluate(&descriptor, &policy);
        match decision {
            GovernanceDecision::Deny(reason) => {
                assert!(reason.contains("dp_budget_exceeded"));
            }
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn budget_consumption_accumulates() {
        let mut policy = NodePolicy::default();
        GovernanceEngine::consume_budget(&mut policy, 0.2);
        assert_eq!(policy.dp_budget_consumed, 0.2);
        GovernanceEngine::consume_budget(&mut policy, 0.3);
        assert!((policy.dp_budget_consumed - 0.5).abs() < f64::EPSILON);
    }
}
