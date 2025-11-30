use crate::{NodeMetadata, NodePolicy, NodeStatus};
use refractive_swan_contracts::{
    MeshError, MeshErrorCode, MeshErrorKind, MeshJobDescriptor, MeshJobResult, MeshJobStatus,
};

#[derive(Clone, Debug, Default)]
pub struct HubPolicy {
    /// Nodes with these tags are allowed; if None, all tags are allowed.
    pub allowed_tags: Option<Vec<String>>,
    /// Compliance modes (case-insensitive) that are blocked.
    pub blocked_compliance_modes: Vec<String>,
}

/// Basic allow/deny check before dispatching to a node.
#[allow(clippy::result_large_err)]
pub fn allow_federated_job(
    job: &MeshJobDescriptor,
    node: &NodeMetadata,
    node_policy: &NodePolicy,
    hub_policy: &HubPolicy,
) -> Result<(), MeshJobResult> {
    if node.status == NodeStatus::Offline {
        return Err(policy_denied(job, format!("node_offline:{}", node.node_id)));
    }
    if let Some(allowed) = hub_policy.allowed_tags.as_ref() {
        let node_tags = &node.capabilities.tags;
        if !node_tags
            .iter()
            .any(|t| allowed.iter().any(|a| a.eq_ignore_ascii_case(t)))
        {
            return Err(policy_denied(job, "hub_denied:tag".into()));
        }
    }
    if hub_policy
        .blocked_compliance_modes
        .iter()
        .any(|mode| mode.eq_ignore_ascii_case(&node.capabilities.compliance_mode))
    {
        return Err(policy_denied(
            job,
            format!(
                "hub_denied:compliance_mode={}",
                node.capabilities.compliance_mode
            ),
        ));
    }
    if matches!(
        job.job_type,
        refractive_swan_contracts::MeshJobType::ExportJob
    ) && !node_policy.export_allowed
    {
        return Err(policy_denied(job, "hub_denied:export_not_allowed".into()));
    }
    Ok(())
}

fn policy_denied(job: &MeshJobDescriptor, reason: String) -> MeshJobResult {
    let code = policy_error_code(&reason);
    MeshJobResult {
        job_id: job.job_id.clone(),
        status: MeshJobStatus::Denied,
        metrics: None,
        output: None,
        error: Some(MeshError {
            kind: MeshErrorKind::PolicyDenied,
            code,
            message: reason,
            context: None,
        }),
    }
}

fn policy_error_code(reason: &str) -> MeshErrorCode {
    if reason.contains("dp_budget_exceeded") {
        MeshErrorCode::new("policy_denied:dp_budget_exceeded")
    } else if reason.contains("export_not_allowed") {
        MeshErrorCode::new("policy_denied:export_blocked_for_tier_licensed")
    } else {
        MeshErrorCode::new("policy_denied:hub")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refractive_swan_contracts::MeshJobType;
    use url::Url;

    fn node_meta(compliance_mode: &str, tags: Vec<&str>) -> NodeMetadata {
        let caps = crate::NodeCapabilities {
            node_id: crate::MeshNodeId("node-test".into()),
            vector_backend: "mock".into(),
            warehouse_backend: "sqlite".into(),
            compliance_mode: compliance_mode.into(),
            max_dataset_size: 1,
            tags: tags.into_iter().map(|t| t.to_string()).collect(),
        };
        NodeMetadata {
            node_id: caps.node_id.clone(),
            url: Url::parse("http://localhost:8080/").unwrap(),
            capabilities: caps,
            last_seen_ms: None,
            status: NodeStatus::Online,
        }
    }

    #[test]
    fn denies_blocked_compliance_mode() {
        let job = MeshJobDescriptor {
            job_id: "job-1".into(),
            job_type: MeshJobType::AnalyticsQuery,
            parameters: serde_json::json!({"query_type":"ncit_summary"}),
            governance_context: None,
        };
        let node = node_meta("restricted", vec!["prod"]);
        let node_policy = NodePolicy::default();
        let hub_policy = HubPolicy {
            blocked_compliance_modes: vec!["restricted".into()],
            ..HubPolicy::default()
        };
        let result = allow_federated_job(&job, &node, &node_policy, &hub_policy);
        assert!(result.is_err());
        let denied = result.err().unwrap();
        assert_eq!(denied.status, MeshJobStatus::Denied);
        assert!(
            denied
                .error
                .as_ref()
                .map(|e| e.code.as_str() == "policy_denied:hub")
                .unwrap_or(false)
        );
    }

    #[test]
    fn export_denial_maps_to_license_code() {
        let job = MeshJobDescriptor {
            job_id: "job-2".into(),
            job_type: MeshJobType::ExportJob,
            parameters: serde_json::Value::default(),
            governance_context: None,
        };
        let node = node_meta("internal", vec!["prod"]);
        let node_policy = NodePolicy {
            export_allowed: false,
        };
        let hub_policy = HubPolicy::default();
        let result = allow_federated_job(&job, &node, &node_policy, &hub_policy);
        assert!(result.is_err());
        let denied = result.err().unwrap();
        let code = denied.error.as_ref().unwrap().code.as_str();
        assert_eq!(code, "policy_denied:export_blocked_for_tier_licensed");
    }
}
