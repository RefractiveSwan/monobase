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
            .and_then(|v| v.map(|s| s.to_ascii_lowercase() != "false"))
            .unwrap_or(policy.export_allowed);
        policy.hub_registration_allowed = string_var("refractive_swan_HUB_REGISTRATION_ALLOWED")
            .ok()
            .and_then(|v| v.map(|s| s.to_ascii_lowercase() != "false"))
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
        match descriptor.class {
            QueryClass::MappingJob | QueryClass::NodeIntrospection => GovernanceDecision::Allow,
            QueryClass::AnalyticsJob => {
                if let Some(card) = descriptor.expected_cardinality {
                    if card > policy.max_cardinality_no_dp {
                        if policy.dp_budget_consumed + 0.1 > policy.dp_budget_daily {
                            return GovernanceDecision::Deny("dp_budget_exceeded".into());
                        }
                        return GovernanceDecision::AllowWithNoise { epsilon: 0.1 };
                    }
                }
                GovernanceDecision::Allow
            }
            QueryClass::EvalJob => GovernanceDecision::Allow,
            QueryClass::ExportJob => {
                if !policy.export_allowed {
                    return GovernanceDecision::Deny("export_not_allowed".into());
                }
                if policy.dp_budget_consumed + 0.5 > policy.dp_budget_daily {
                    return GovernanceDecision::Deny("dp_budget_exceeded".into());
                }
                GovernanceDecision::AllowWithNoise { epsilon: 0.5 }
            }
        }
    }

    pub fn consume_budget(policy: &mut NodePolicy, epsilon: f64) {
        policy.dp_budget_consumed += epsilon;
    }
}
