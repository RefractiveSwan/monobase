use dfps_eval::{self, EvalCase, EvalSummary};

use crate::map_staging_codes;

#[deprecated(note = "Use dfps_eval::run_eval_with_mapper and pass map_staging_codes instead")]
pub fn run_eval(cases: &[EvalCase]) -> EvalSummary {
    dfps_eval::run_eval_with_mapper(cases, |rows| map_staging_codes(rows).0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(deprecated)]
    #[test]
    fn mapping_pipeline_still_maps_eval_cases() {
        let cases = vec![
            EvalCase {
                system: "http://www.ama-assn.org/go/cpt".into(),
                code: "78815".into(),
                display: "PET with concurrently acquired CT for tumor imaging".into(),
                expected_ncit_id: "NCIT:C19951".into(),
            },
            EvalCase {
                system: "http://loinc.org".into(),
                code: "24606-6".into(),
                display: "FDG uptake PET".into(),
                expected_ncit_id: "NCIT:C17747".into(),
            },
        ];

        let summary = run_eval(&cases);
        assert_eq!(summary.total_cases, 2);
        assert_eq!(summary.correct, 2);
    }
}
