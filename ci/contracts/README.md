Mesh contract schemas
=====================

This directory stores JSON schemas generated from the canonical mesh contracts in `lib/domain/meta/contracts`.

Generate/refresh:

```bash
cd code
cargo run -p refractive_swan_contracts --bin contracts-schema
```

Outputs include:
- `mesh_job_descriptor.schema.json`
- `mesh_job_result.schema.json`
- `mapping_health_check_report.schema.json`
- `export_job_summary.schema.json`
- `node_introspection_view.schema.json`
- plus supporting analytics/eval/load schemas.

Consumers (API/web clients/tests) should use these schemas for validation and typed DTO updates.
