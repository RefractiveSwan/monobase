# refractive_swan eval fixtures (fake_data module)

This directory anchors all checked-in datasets and regression fixtures consumed by
`refractive_swan_eval::fake_data` loaders, `refractive_swan_eval::FileDatasetStore`, CLIs, and downstream tests.

```
lib/domain/meta/evaluation/data/
├─ eval/         # NDJSON evaluation corpora (see eval/README.md)
├─ meta/         # Shared configs like eval thresholds
└─ regression/   # Deterministic bundles + mapping cases referenced by tests
```

Override the lookup root (for local datasets or private corpora) by configuring your
app/test harness (e.g., `refractive_swan_FAKE_DATA_ROOT`) and passing the resulting path into
`refractive_swan_eval::fake_data::fixtures::Registry::new_with_root(...)` or
`refractive_swan_eval::FileDatasetStore::new(...)`. The domain crates no longer read environment
variables directly; config lives in the app/platform layer.

## Meta artifacts

- `meta/eval_thresholds.json` — canonical CI gating thresholds consumed by the
  `eval_mapping` CLI and GitHub workflow. Point `--thresholds` at this file (or
  configure `refractive_swan_eval::FileDatasetStore`) to ensure local runs match automation.
