# Terminology APIs Quickstart

Use this runbook to enable external terminology lookups (UMLS/NCIt) for DFPS.

## Configure environment
Set the terminology env vars (see `data/environment/.env.domain.terminology.dev.example`):
```
export DFPS_TERMINOLOGY_BASE_URL=http://localhost:8081
export DFPS_TERMINOLOGY_API_KEY=replace-me
export DFPS_TERMINOLOGY_TIMEOUT_SECS=5
export DFPS_TERMINOLOGY_MODE=http_fallback   # mock_only | http_fallback | http_only
```

## Enable the client
- Build with the `http-client` feature in `dfps_terminology` (default enabled in workspace builds).
- Construct an `HttpTerminologyClient` from `TerminologyClientConfig::from_env()` and wrap it in `CompositeTerminologyClient` alongside `MockTerminologyClient` for safe fallbacks.

## Usage in mapping
- `dfps_mapping::map_staging_codes_with_summary_with_client(codes, Some(&client))` — enables external lookups for unknown systems.
- `MappingSummary` now includes `extern_lookup_success/miss/error` counters; surface these in logs/CLI as needed.

## Compliance modes (LIC-020)

- Set `DFPS_COMPLIANCE_MODE` to `internal` (default), `partner`, or `open_source`; optional overrides via `DFPS_COMPLIANCE_POLICY_PATH` (JSON/YAML).
- CLIs:
  - `map_codes --fail-on-license-block` exits non-zero if any mapping is blocked; stderr prints `license_blocked` counts and mode.
  - `map_bundles --fail-on-license-block` does the same for Bundle ingestion.
- Export/warehouse:
  - Datamart loader and API persistence call `dfps_compliance::assert_export_allowed(...)` before writing facts; adjust the mode or policy if licensed tiers should be permitted.
- Env template: `data/environment/.env.platform.compliance.dev.example` documents the compliance settings.

## Local mock server (for tests/demos)
- Provide endpoints:
  - `GET /lookup_cui?system=<url>&code=<code>` -> `{ "cui": "...", "preferred_name": "..." }`
  - `GET /lookup_ncit?id=<cui_or_code>` -> `{ "ncit_id": "...", "preferred_name": "...", "synonyms": [] }`
  - `GET /search?q=<text>`
- Pair with `DFPS_TERMINOLOGY_MODE=http_fallback` to keep deterministic mock behavior if the HTTP server is down.
