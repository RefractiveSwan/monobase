//! Semantic mapping concepts/results derived from staging data.
//!
//! This layer translates staging codes into NCIt/UMLS concepts and keeps the
//! canonical mapping result types for downstream crates.
//!
//! See:
//! - docs/system-design/ncit/architecture/system-architecture.md
//! - docs/system-design/ncit/models/class-model.md
//! - docs/system-design/ncit/behavior/sequence-servicerequest.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04.

pub mod mapping;
