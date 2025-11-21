use super::Registry;
use dfps_core::fhir;
use serde_json::from_reader;
use std::error::Error;

pub fn load(registry: &Registry, name: &str) -> Result<fhir::Bundle, Box<dyn Error>> {
    let file = registry.open_bundle(name)?;
    Ok(from_reader(file)?)
}
