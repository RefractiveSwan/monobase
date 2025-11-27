use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct CohortQuery {
    pub ncit_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}
