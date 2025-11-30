use std::collections::HashMap;

use refractive_swan_contracts::AnalyticsSummaryResponse;
use refractive_swan_mesh_dto::{MeshJobDescriptor, MeshJobType};

use crate::{HubError, JobQueue, NodeJobResult};

/// Dispatch NCIt summary jobs to all nodes and aggregate rows.
pub async fn global_ncit_summary(queue: &JobQueue) -> Result<AnalyticsSummaryResponse, HubError> {
    let job = MeshJobDescriptor {
        job_id: "hub-ncit-summary".into(),
        job_type: MeshJobType::AnalyticsQuery,
        parameters: serde_json::json!({ "query_type": "ncit_summary" }),
        governance_context: None,
    };
    let results: Vec<NodeJobResult> = queue.dispatch(&job, None).await?;
    type SummaryBucket = (String, Option<String>, Option<String>, Option<String>);
    let mut buckets: HashMap<SummaryBucket, usize> = HashMap::new();

    for result in results {
        if let Some(output) = result.result.output
            && let Ok(summary) = serde_json::from_value::<AnalyticsSummaryResponse>(output)
        {
            for row in summary.rows {
                let key = (
                    row.ncit_id,
                    row.preferred_name,
                    row.mapping_state,
                    row.time_bucket,
                );
                *buckets.entry(key).or_default() += row.count;
            }
        }
    }

    let rows = buckets
        .into_iter()
        .map(
            |((ncit_id, preferred_name, mapping_state, time_bucket), count)| {
                refractive_swan_contracts::analytics::AnalyticsSummaryRow {
                    ncit_id,
                    preferred_name,
                    mapping_state,
                    time_bucket,
                    count,
                }
            },
        )
        .collect();

    Ok(AnalyticsSummaryResponse { rows })
}
