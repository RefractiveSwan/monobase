use std::{env, path::PathBuf};

use dfps_configuration::load_env;
use dfps_eval::{FileDatasetStore, default_data_root};
use env_logger::Builder;
use log::LevelFilter;

use super::{CliError, CliResult};

pub fn init_cli_env() -> CliResult<()> {
    load_env("app.cli").map_err(|err| CliError::config(format!("dfps_cli env error: {err}")))?;
    Ok(())
}

pub fn init_logging(level: &str) -> CliResult<()> {
    let filter = level
        .parse::<LevelFilter>()
        .map_err(|_| CliError::config(format!("invalid log level '{level}'")))?;
    let mut builder = Builder::from_default_env();
    builder.filter_level(filter);
    builder
        .try_init()
        .map_err(|err| CliError::external(err.to_string()))
}

pub fn dataset_store_from_env() -> FileDatasetStore {
    let root = env::var("DFPS_EVAL_DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_data_root());
    FileDatasetStore::new(root)
}
