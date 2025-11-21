use refractive_swan_configuration::load_env;
use refractive_swan_eval::{FileDatasetStore, config::EvalDatasetConfig};
use env_logger::Builder;
use log::{LevelFilter, warn};

use super::{CliError, CliResult};

pub fn init_cli_env() -> CliResult<()> {
    load_env("app.cli").map_err(|err| CliError::config(format!("refractive_swan_cli env error: {err}")))?;
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
    EvalDatasetConfig::from_env()
        .map(|cfg| cfg.dataset_store())
        .unwrap_or_else(|err| {
            warn!("refractive_swan_cli dataset config error ({err}); using default fixtures");
            FileDatasetStore::default()
        })
}
