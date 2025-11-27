use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct MapQuery {
    pub vector: Option<String>,
}
