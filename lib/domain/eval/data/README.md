# DFPS eval fixtures (fake_data module)

This directory anchors all checked-in datasets and regression fixtures consumed by
`dfps_eval::fake_data` loaders, `dfps_eval::FileDatasetStore`, CLIs, and downstream tests.

```
lib/domain/evaluation/eval/data/
├─ eval/         # NDJSON evaluation corpora (see eval/README.md)
├─ meta/         # Shared configs like eval thresholds
└─ regression/   # Deterministic bundles + mapping cases referenced by tests
```

Override the lookup root (for local datasets or private corpora) by configuring your
app/test harness (e.g., `DFPS_FAKE_DATA_ROOT`) and passing the resulting path into
`dfps_eval::fake_data::fixtures::Registry::new_with_root(...)` or
`dfps_eval::FileDatasetStore::new(...)`. The domain crates no longer read environment
variables directly; config lives in the app/platform layer.

## Meta artifacts

- `meta/eval_thresholds.json` — canonical CI gating thresholds consumed by the
  `eval_mapping` CLI and GitHub workflow. Point `--thresholds` at this file (or
  configure `dfps_eval::FileDatasetStore`) to ensure local runs match automation.
