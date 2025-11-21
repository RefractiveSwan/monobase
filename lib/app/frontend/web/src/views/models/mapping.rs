use refractive_swan_contracts::pipeline::MappingState;
use std::collections::HashMap;

use super::helpers::{CodeInfo, build_code_lookup, summarize_flats};
use super::types::{MappingResultsView, MappingRowView, NoMatchRowView};
use crate::client::MapBundlesResponse;

impl MappingResultsView {
    pub fn from_response(response: &MapBundlesResponse) -> Self {
        let request_summary = summarize_flats(&response.flats);
        let code_lookup = build_code_lookup(response);
        let concept_lookup = build_concept_lookup(&response.dim_concepts);

        let (rows, no_matches) =
            build_mapping_rows(&response.mapping_results, &code_lookup, &concept_lookup);

        Self {
            request_summary,
            rows,
            no_matches,
        }
    }
}

fn build_concept_lookup(
    concepts: &[refractive_swan_contracts::pipeline::DimNCITConcept],
) -> HashMap<String, String> {
    concepts
        .iter()
        .map(|concept| (concept.ncit_id.clone(), concept.preferred_name.clone()))
        .collect()
}

fn build_mapping_rows(
    mapping_results: &[refractive_swan_contracts::pipeline::MappingResult],
    code_lookup: &HashMap<String, CodeInfo>,
    concept_lookup: &HashMap<String, String>,
) -> (Vec<MappingRowView>, Vec<NoMatchRowView>) {
    let mut rows = Vec::with_capacity(mapping_results.len());
    let mut no_matches = Vec::new();

    for result in mapping_results {
        let row = create_mapping_row_view(result, code_lookup, concept_lookup);

        if row.state == MappingState::NoMatch {
            no_matches.push(NoMatchRowView::from(&row));
        }

        rows.push(row);
    }

    (rows, no_matches)
}

fn create_mapping_row_view(
    result: &refractive_swan_contracts::pipeline::MappingResult,
    code_lookup: &HashMap<String, CodeInfo>,
    concept_lookup: &HashMap<String, String>,
) -> MappingRowView {
    let (sr_id, system, code, display) =
        extract_code_components(&result.code_element_id, code_lookup);
    let ncit_label = result
        .ncit_id
        .as_ref()
        .and_then(|id| concept_lookup.get(id).cloned());

    MappingRowView {
        sr_id,
        system,
        code,
        display,
        ncit_id: result.ncit_id.clone(),
        ncit_label,
        state: result.state,
        reason: result.reason.clone(),
    }
}

fn extract_code_components(
    code_element_id: &str,
    code_lookup: &HashMap<String, CodeInfo>,
) -> (String, String, String, String) {
    code_lookup
        .get(code_element_id)
        .cloned()
        .unwrap_or_else(|| CodeInfo {
            sr_id: code_element_id.to_string(),
            system: Some("unknown-system".to_string()),
            code: Some(code_element_id.to_string()),
            display: None,
        })
        .components()
}
