use dfps_compliance::ComplianceAction;
use dfps_compliance::Policy;
use dfps_core::mapping::{CodeElement, DimNCITConcept, MappingResult, MappingStrategy};
use dfps_core::staging::StgSrCodeExploded;
use dfps_terminology::{CodeKind, EnrichedCode, TerminologyClient, TerminologyResult};

use crate::config::MappingConfig;
use crate::data::load_umls_xrefs;
use crate::engine::MappingEngine;
use crate::rankers::{LexicalRanker, VectorRankerMock};
use crate::traits::CandidateRanker;
use crate::types::MappingSummary;

use super::helpers::{attach_license_metadata, build_result_with_score, dim_concepts};

pub fn map_staging_codes<I>(codes: I) -> (Vec<MappingResult>, Vec<DimNCITConcept>)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let (results, dims, _) = map_staging_codes_with_summary(codes);
    (results, dims)
}

pub fn map_staging_codes_with_summary<I>(
    codes: I,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    map_staging_codes_with_summary_with_config(codes, None, &MappingConfig::default())
}

pub fn map_staging_codes_with_summary_and_policy<I>(
    codes: I,
    policy: &Policy,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_summary_with_config(codes, None, &config)
}

pub fn map_staging_codes_with_summary_with_policy<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
    policy: &Policy,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let config = MappingConfig::with_policy(policy.clone());
    map_staging_codes_with_summary_with_config(codes, client, &config)
}

pub fn map_staging_codes_with_summary_with_client<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    map_staging_codes_with_summary_with_config(codes, client, &MappingConfig::default())
}

pub fn map_staging_codes_with_summary_with_config<I>(
    codes: I,
    client: Option<&dyn TerminologyClient>,
    config: &MappingConfig,
) -> (Vec<MappingResult>, Vec<DimNCITConcept>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
{
    let dim_concepts = dim_concepts();
    let xrefs = load_umls_xrefs();
    let engine = MappingEngine::new(LexicalRanker, VectorRankerMock, crate::engine::RuleReranker);
    let (results, summary) = map_with_engine(codes, &engine, &xrefs, client, config);
    (results, dim_concepts, summary)
}

pub fn map_with_engine<I, L, V>(
    codes: I,
    engine: &MappingEngine<L, V>,
    xrefs: &std::collections::HashMap<(String, String), crate::data::UmlsXref>,
    client: Option<&dyn TerminologyClient>,
    config: &MappingConfig,
) -> (Vec<MappingResult>, MappingSummary)
where
    I: IntoIterator<Item = StgSrCodeExploded>,
    L: CandidateRanker,
    V: CandidateRanker,
{
    let mut results = Vec::new();
    let mut summary = MappingSummary::default();

    for staging in codes {
        let enriched = EnrichedCode::from_staging(staging.clone());
        let code_kind = enriched.code_kind();
        let element = CodeElement::from(staging);
        let system_value = enriched.staging.system.clone().unwrap_or_default();
        let code_value = enriched.staging.code.clone().unwrap_or_default();
        let key = (system_value.clone(), code_value.clone());

        summary.record(code_kind, enriched.license_label());

        if let Some(tier) = enriched.license_tier
            && !config.policy.is_allowed(ComplianceAction::Map, tier)
        {
            let mut blocked = build_result_with_score(
                &element,
                None,
                None,
                0.0,
                MappingStrategy::Unmapped,
                Some("license_blocked".into()),
                config,
            );
            attach_license_metadata(&mut blocked, &enriched);
            results.push(blocked);
            continue;
        }

        let mut result = match code_kind {
            CodeKind::MissingSystemOrCode => build_result_with_score(
                &element,
                None,
                None,
                0.0,
                MappingStrategy::Unmapped,
                Some("missing_system_or_code".into()),
                config,
            ),
            CodeKind::UnknownSystem => {
                if matches!(enriched.license_label(), Some(label) if label == "forbidden") {
                    summary.record_external_miss();
                    build_result_with_score(
                        &element,
                        None,
                        None,
                        0.0,
                        MappingStrategy::Unmapped,
                        Some("license_forbidden".into()),
                        config,
                    )
                } else {
                    let base = build_result_with_score(
                        &element,
                        None,
                        None,
                        0.0,
                        MappingStrategy::Unmapped,
                        Some("unknown_code_system".into()),
                        config,
                    );
                    if let Some(client) = client {
                        match external_lookup(client, &system_value, &code_value) {
                            Ok(Some((cui, ncit_id))) => {
                                summary.record_external_success();
                                build_result_with_score(
                                    &element,
                                    cui,
                                    ncit_id,
                                    0.95,
                                    MappingStrategy::Rule,
                                    Some("external_terminology_lookup".into()),
                                    config,
                                )
                            }
                            Ok(None) => {
                                summary.record_external_miss();
                                base
                            }
                            Err(_) => {
                                summary.record_external_error();
                                base
                            }
                        }
                    } else {
                        base
                    }
                }
            }
            _ => {
                if let Some(xref) = xrefs.get(&key) {
                    build_result_with_score(
                        &element,
                        Some(xref.cui.clone()),
                        Some(xref.ncit_id.clone()),
                        0.99,
                        MappingStrategy::Rule,
                        Some("umls_direct_xref".into()),
                        config,
                    )
                } else {
                    engine.map_with_config(&element, config)
                }
            }
        };

        attach_license_metadata(&mut result, &enriched);
        results.push(result);
    }

    (results, summary)
}

fn external_lookup(
    client: &dyn TerminologyClient,
    system: &str,
    code: &str,
) -> TerminologyResult<Option<(Option<String>, Option<String>)>> {
    let mut cui = None;
    if let Some(record) = client.lookup_cui(system, code)? {
        cui = Some(record.cui);
    }

    let ncit_id = client
        .lookup_ncit(cui.as_deref().unwrap_or(code))?
        .map(|record| record.ncit_id);

    if cui.is_some() || ncit_id.is_some() {
        Ok(Some((cui, ncit_id)))
    } else {
        Ok(None)
    }
}
