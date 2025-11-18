use criterion::{Criterion, criterion_group, criterion_main};
use dfps_core::mapping::{MappingResult, MappingState, MappingStrategy, MappingThresholds};
use dfps_eval::{self, DEFAULT_CHUNK_SIZE};
use std::io::BufReader;

fn bench_dataset(c: &mut Criterion, dataset: &str) {
    let path = dfps_eval::dataset_path(dataset);
    let file = std::fs::File::open(&path).expect("open dataset");
    c.bench_function(&format!("eval_{dataset}"), |b| {
        b.iter(|| {
            dfps_eval::run_eval_streaming_with_mapper(
                BufReader::new(file.try_clone().expect("clone file")),
                |rows| map_stub(rows),
                DEFAULT_CHUNK_SIZE,
            )
            .expect("streaming eval");
        })
    });
}

fn map_stub(rows: Vec<dfps_core::staging::StgSrCodeExploded>) -> Vec<MappingResult> {
    rows.into_iter()
        .map(|row| MappingResult {
            code_element_id: row.sr_id.clone(),
            ncit_id: row.code.clone().map(|c| format!("NCIT:{c}")),
            cui: None,
            score: 0.9,
            strategy: MappingStrategy::Lexical,
            state: MappingState::AutoMapped,
            thresholds: MappingThresholds::default(),
            source_version: dfps_core::mapping::MappingSourceVersion::new("bench", "bench"),
            reason: None,
            license_tier: Some("bench".into()),
            source_kind: Some("bench".into()),
        })
        .collect()
}

fn benchmarks(c: &mut Criterion) {
    bench_dataset(c, "pet_ct_small");
    bench_dataset(c, "pet_ct_extended");
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
