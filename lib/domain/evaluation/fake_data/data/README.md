# DFPS fake_data fixtures

This directory anchors all checked-in datasets and regression fixtures consumed by
`dfps_fake_data` loaders, CLIs, and downstream tests.

```
lib/domain/fake_data/data/
├─ eval/         # NDJSON evaluation corpora (see eval/README.md)
├─ meta/         # Shared configs like eval thresholds
└─ regression/   # Deterministic bundles + mapping cases referenced by tests
```

Override the lookup root (for local datasets or private corpora) via
`DFPS_FAKE_DATA_ROOT`. By default the registry resolves paths relative to this
directory when `dfps_fake_data::fixtures::Registry::default()` is used.

## Meta artifacts

- `meta/eval_thresholds.json` — canonical CI gating thresholds consumed by the
  `eval_mapping` CLI and GitHub workflow. Point `--thresholds` at this file (or
  override `DFPS_FAKE_DATA_ROOT`) to ensure local runs match automation.
