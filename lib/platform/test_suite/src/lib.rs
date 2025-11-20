//! Shared test utilities for the DFPS workspace.
//!
//! This crate exposes fixtures, assertions, and regression helpers that other
//! crates (or workspace integration tests) can pull in without duplicating code.

pub mod assertions;
mod datamart;
mod env;
pub mod fixtures;
pub mod regression;

pub use assertions::*;
pub use datamart::TempSqliteWarehouse;
pub use env::{ScopedEnvVar, ensure_eval_data_root, init_environment, scoped_env_var};
pub use fixtures::*;
pub use regression::*;

pub fn ping() -> &'static str {
    init_environment().expect("dfps_test_suite env");
    "test-suite-ready"
}
