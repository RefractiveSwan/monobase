use refractive_swan_core::mapping::{CodeElement, MappingCandidate, MappingResult};

pub trait Mapper {
    fn map(&self, code: &CodeElement) -> MappingResult;
}

pub trait CandidateRanker {
    fn rank(&self, code: &CodeElement) -> Vec<MappingCandidate>;
}
