use criterion::{Criterion, criterion_group, criterion_main};
use dfps_core::mapping::{MappingResult, MappingState, MappingStrategy, MappingThresholds};
use dfps_eval::{self, DEFAULT_CHUNK_SIZE, FileDatasetStore};
use std::io::BufReader;

fn bench_dataset(c: &mut Criterion, store: &FileDatasetStore, dataset: &str) {
    let path = store.dataset_path(dataset);
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
    let store = FileDatasetStore::default();
    bench_dataset(c, &store, "pet_ct_small");
    bench_dataset(c, &store, "pet_ct_extended");
    bench_fingerprint(c, &store, "pet_ct_small");
}

fn bench_fingerprint(c: &mut Criterion, store: &FileDatasetStore, dataset: &str) {
    let cases = store.load_dataset(dataset).expect("load dataset");
    let summary = dfps_eval::run_eval_with_mapper(&cases, |rows| map_stub(rows));
    c.bench_function(&format!("fingerprint_{dataset}"), |b| {
        b.iter(|| dfps_eval::fingerprint_summary(&summary))
    });
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
