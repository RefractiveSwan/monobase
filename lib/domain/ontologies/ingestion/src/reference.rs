use dfps_core::fhir;

/// Extracts the ID component from a `"ResourceType/id"` reference string.
pub fn reference_id_from_str(reference: &str) -> Option<&str> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }

    trimmed
        .split('/')
        .filter(|segment| !segment.is_empty())
        .last()
}

/// Convenience helper to extract the ID from a FHIR `Reference`.
pub fn reference_id(reference: &fhir::Reference) -> Option<String> {
    reference
        .reference
        .as_deref()
        .and_then(reference_id_from_str)
        .map(|value| value.to_string())
}
