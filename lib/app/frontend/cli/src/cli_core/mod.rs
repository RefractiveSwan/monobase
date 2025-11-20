pub mod compliance;
pub mod env;
pub mod error;
pub mod io;
pub mod vector;

pub use compliance::{enforce_license_blocks, enforce_metrics_gate, load_policy, tag_metrics};
pub use env::{dataset_store_from_env, init_cli_env, init_logging};
pub use error::{CliError, CliResult, ExitCode, run_bin};
pub use io::{
    JsonStream, input_reader, json_stream, parse_json_file, read_to_string, write_record,
};
pub use vector::{load_vector_config, mapping_vector_store, pipeline_vector_context_from_env};
