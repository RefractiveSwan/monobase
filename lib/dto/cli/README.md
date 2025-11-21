# CLI DTO Veneer (`refractive_swan_cli_dto`)

Re-exports the contract types consumed by the CLI binaries. Keeps
`lib/app/frontend/cli` decoupled from the full `refractive_swan_contracts` surface.

Currently exposes the evaluation + pipeline DTOs required by:
- `map_bundles` / `map_codes`
- `eval_mapping`
- `load_datamart`

Add new re-exports here whenever CLI needs additional payloads.
