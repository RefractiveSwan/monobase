use axum::{
    Router,
    routing::{get, post},
};

use crate::server::ApiState;
use crate::server::handlers::eval;

/// Evaluation endpoints.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/eval/summary", get(eval::eval_summary))
        .route("/eval/datasets", get(eval::list_eval_datasets))
        .route("/eval/run", post(eval::run_eval))
        .route("/eval/latest", get(eval::latest_eval))
}
