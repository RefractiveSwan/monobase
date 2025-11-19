mod loader;
pub mod paths;
pub mod values;

pub use loader::{EnvLoadError, EnvLoadOutcome, load_env};
pub use paths::{ConfigPaths, config_paths, workspace_root};
pub use values::{EnvValueError, bool_var, port_var, u32_var, u64_var};

#[cfg(test)]
mod tests;
