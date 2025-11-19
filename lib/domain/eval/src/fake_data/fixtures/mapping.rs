use super::Registry;
use serde::Deserialize;
use serde_json::from_reader;
use std::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct MappingCase {
    pub sr_id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

pub fn load(registry: &Registry, name: &str) -> Result<MappingCase, Box<dyn Error>> {
    let file = registry.open_mapping(name)?;
    Ok(from_reader(file)?)
}
