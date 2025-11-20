use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{MappingSourceVersion, MappingState, MappingStrategy, MappingThresholds};

/// Candidate concept returned by a ranker/mapper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingCandidate {
    pub target_system: String,
    pub target_code: String,
    pub cui: Option<String>,
    pub score: f32,
}

/// Overall mapping output, including NCIt concept selection + provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MappingResult {
    pub code_element_id: String,
    pub cui: Option<String>,
    pub ncit_id: Option<String>,
    pub score: f32,
    pub strategy: MappingStrategy,
    pub state: MappingState,
    pub thresholds: MappingThresholds,
    pub source_version: MappingSourceVersion,
    pub reason: Option<String>,
    pub license_tier: Option<String>,
    pub source_kind: Option<String>,
}

impl MappingResult {
    /// Builder for auto-mapped results with standardized reason/state handling.
    #[allow(clippy::too_many_arguments)]
    pub fn auto_mapped(
        code_element_id: impl Into<String>,
        ncit_id: impl Into<String>,
        score: f32,
        thresholds: MappingThresholds,
        source_version: MappingSourceVersion,
        strategy: MappingStrategy,
        reason: Option<String>,
        license_tier: Option<String>,
        source_kind: Option<String>,
        cui: Option<String>,
    ) -> Self {
        Self::with_state(
            code_element_id,
            Some(ncit_id.into()),
            score,
            thresholds,
            source_version,
            strategy,
            MappingState::AutoMapped,
            reason,
            license_tier,
            source_kind,
            cui,
        )
    }

    /// Builder for results that still need review even when a candidate exists.
    #[allow(clippy::too_many_arguments)]
    pub fn needs_review(
        code_element_id: impl Into<String>,
        ncit_id: impl Into<String>,
        score: f32,
        thresholds: MappingThresholds,
        source_version: MappingSourceVersion,
        strategy: MappingStrategy,
        reason: Option<String>,
        license_tier: Option<String>,
        source_kind: Option<String>,
        cui: Option<String>,
    ) -> Self {
        Self::with_state(
            code_element_id,
            Some(ncit_id.into()),
            score,
            thresholds,
            source_version,
            strategy,
            MappingState::NeedsReview,
            reason,
            license_tier,
            source_kind,
            cui,
        )
    }

    /// Builder for unmapped results that still carry thresholds and provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn no_match(
        code_element_id: impl Into<String>,
        score: f32,
        thresholds: MappingThresholds,
        source_version: MappingSourceVersion,
        strategy: MappingStrategy,
        reason: Option<String>,
        license_tier: Option<String>,
        source_kind: Option<String>,
    ) -> Self {
        Self::with_state(
            code_element_id,
            None,
            score,
            thresholds,
            source_version,
            strategy,
            MappingState::NoMatch,
            reason,
            license_tier,
            source_kind,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn with_state(
        code_element_id: impl Into<String>,
        ncit_id: Option<String>,
        score: f32,
        thresholds: MappingThresholds,
        source_version: MappingSourceVersion,
        strategy: MappingStrategy,
        state: MappingState,
        reason: Option<String>,
        license_tier: Option<String>,
        source_kind: Option<String>,
        cui: Option<String>,
    ) -> Self {
        let normalized_reason = match reason {
            Some(reason) if !reason.trim().is_empty() => Some(reason),
            _ => Some(Self::default_reason(state)),
        };

        Self {
            code_element_id: code_element_id.into(),
            cui,
            ncit_id,
            score,
            strategy,
            state,
            thresholds,
            source_version,
            reason: normalized_reason,
            license_tier,
            source_kind,
        }
    }

    fn default_reason(state: MappingState) -> String {
        match state {
            MappingState::AutoMapped => "auto_mapped",
            MappingState::NeedsReview => "needs_review",
            MappingState::NoMatch => "no_match",
        }
        .to_string()
    }
}
