//! Cross-domain bridges owned by the mapping layer.
//! Place all `From`/`TryFrom` impls that consume staging rows here.
//!
//! See:
//! - docs/system-design/ncit/architecture/system-architecture.md
//! - docs/kanban/feature/mvp/040-infra-and-docs/022-codebase-refactor.md#refr-04--domain-core--staging
//!
//! Card: REFR-04.

pub mod from_staging;
