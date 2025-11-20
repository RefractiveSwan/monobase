//! dfps_cli orchestrates all CLI surfaces (map_bundles/map_codes/eval/validate/etc.).
//! Layered as an application-edge adapter: arg parsing + NDJSON IO live here while
//! domain logic stays in `dfps_pipeline`, `dfps_mapping`, `dfps_ingestion`.
//! See:
//! - docs/system-design/base/directory-architecture.md#1-app--application-surfaces
//! - docs/kanban/feature/mvp/040-infra-and-docs/024-codebase-refactor.md#refr-15--cli-surfaces--orchestration-dfps_cli

pub mod cli_core;
