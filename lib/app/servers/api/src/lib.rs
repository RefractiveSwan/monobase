//! HTTP API gateway for the refractive_swan pipeline.
//!
//! The crate exposes a `run` function so integration tests (and eventual
//! binaries) can spin up the server in-process without binding to a global
//! executable.

pub mod dto;
pub mod server;
pub mod types;
pub mod utils;

pub use server::{router_with_state, run};
pub use types::*;
pub use utils::{
    ApiConfig, ApiError, ApiServerConfig, ApiState, ErrorResponse, ServerError, init_logging,
};
