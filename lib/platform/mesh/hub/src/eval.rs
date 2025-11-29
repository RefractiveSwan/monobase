use refractive_swan_contracts::{EvalRunResponse, FederatedEvalSummary, aggregate_eval_summaries};
use refractive_swan_mesh_dto::{MeshJobDescriptor, MeshJobType};

use crate::{HubError, JobQueue};

/// Dispatch eval jobs to all nodes and aggregate summaries.
pub async fn federated_eval(
    queue: &JobQueue,
    dataset: &str,
) -> Result<FederatedEvalSummary, HubError> {
    let job = MeshJobDescriptor {
        job_id: format!("hub-eval-{dataset}"),
        job_type: MeshJobType::EvalDataset,
        parameters: serde_json::json!({ "dataset": dataset }),
        governance_context: None,
    };
    let results = queue.dispatch(&job, None).await?;
    let mut per_node = Vec::new();
    for result in results {
        if let Some(output) = result.output {
            if let Ok(eval) = serde_json::from_value::<EvalRunResponse>(output) {
                // TODO: capture node_id once NodeJobResult carries it.
                per_node.push((eval.dataset.clone(), eval.summary));
            }
        }
    }
    Ok(aggregate_eval_summaries(per_node))
}
