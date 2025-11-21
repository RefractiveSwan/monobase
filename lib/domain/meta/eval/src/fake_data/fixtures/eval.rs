use super::{EvalRow, Registry, stream_ndjson};
use serde_json::Result as JsonResult;
use std::io;

pub fn iter(
    registry: &Registry,
    name: &str,
) -> io::Result<impl Iterator<Item = JsonResult<EvalRow>>> {
    let file = registry.open_eval(name)?;
    Ok(stream_ndjson::<EvalRow>(file))
}
