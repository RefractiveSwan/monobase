use serde::Deserialize;

#[derive(Deserialize)]
pub struct EvalQuery {
    pub dataset: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

#[derive(Deserialize)]
pub struct EvalRunRequest {
    pub dataset: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    1
}
