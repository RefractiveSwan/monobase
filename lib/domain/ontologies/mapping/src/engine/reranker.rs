use dfps_core::mapping::MappingCandidate;

#[derive(Debug, Default)]
pub struct RuleReranker;

impl RuleReranker {
    pub fn apply(&self, candidates: &mut [MappingCandidate]) {
        for candidate in candidates {
            if candidate.target_system == "NCIT" {
                candidate.score = (candidate.score + 0.05).min(1.0);
            } else if candidate.target_system.contains("SNOMED")
                || candidate.target_system.contains("CPT")
            {
                candidate.score = (candidate.score + 0.02).min(1.0);
            }
        }
    }
}
