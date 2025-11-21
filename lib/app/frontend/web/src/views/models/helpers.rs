use refractive_swan_contracts::pipeline::{StgServiceRequestFlat, StgSrCodeExploded};
use std::collections::{BTreeMap, HashMap};

use super::types::{CountStat, ServiceRequestSummary};
use crate::client::MapBundlesResponse;

pub(super) fn summarize_flats(flats: &[StgServiceRequestFlat]) -> ServiceRequestSummary {
    let mut statuses = BTreeMap::new();
    let mut intents = BTreeMap::new();
    for flat in flats {
        *statuses.entry(flat.status.clone()).or_insert(0) += 1;
        *intents.entry(flat.intent.clone()).or_insert(0) += 1;
    }
    ServiceRequestSummary {
        total: flats.len(),
        statuses: map_counts(statuses),
        intents: map_counts(intents),
    }
}

fn map_counts(source: BTreeMap<String, usize>) -> Vec<CountStat> {
    source
        .into_iter()
        .map(|(label, count)| CountStat { label, count })
        .collect()
}

pub(super) fn build_code_lookup(response: &MapBundlesResponse) -> HashMap<String, CodeInfo> {
    response
        .exploded_codes
        .iter()
        .map(|code| {
            (
                code_element_id(code),
                CodeInfo {
                    sr_id: code.sr_id.clone(),
                    system: code.system.clone(),
                    code: code.code.clone(),
                    display: code.display.clone(),
                },
            )
        })
        .collect()
}

#[derive(Debug, Clone)]
pub(super) struct CodeInfo {
    pub sr_id: String,
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

fn code_element_id(code: &StgSrCodeExploded) -> String {
    format!(
        "{}::{}::{}",
        code.sr_id,
        code.system.as_deref().unwrap_or("unknown-system"),
        code.code
            .as_deref()
            .or(code.display.as_deref())
            .unwrap_or("unknown-code")
    )
}

impl CodeInfo {
    pub fn components(self) -> (String, String, String, String) {
        let CodeInfo {
            sr_id,
            system,
            code,
            display,
        } = self;
        let system = system.unwrap_or_else(|| "unknown-system".to_string());
        let display_value = display
            .clone()
            .unwrap_or_else(|| "(no display provided)".to_string());
        let code = code
            .or(display)
            .unwrap_or_else(|| "unknown-code".to_string());
        (sr_id, system, code, display_value)
    }
}
