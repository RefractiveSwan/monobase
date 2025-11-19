use crate::{DatasetError, EvalCase};
use std::io::{BufRead, Lines};

/// Streaming NDJSON reader for `EvalCase` rows.
pub struct EvalCaseStream<R: BufRead> {
    lines: Lines<R>,
    line_number: usize,
}

impl<R: BufRead> EvalCaseStream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lines: reader.lines(),
            line_number: 0,
        }
    }

    pub fn next_case(&mut self) -> Result<Option<EvalCase>, DatasetError> {
        while let Some(line) = self.lines.next() {
            let line = line.map_err(|source| DatasetError::Io {
                source,
                path: std::path::PathBuf::from("<stream>"),
            })?;
            self.line_number += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let case = serde_json::from_str(trimmed).map_err(|source| DatasetError::Parse {
                line: self.line_number,
                source,
            })?;
            return Ok(Some(case));
        }
        Ok(None)
    }
}

impl<R: BufRead> Iterator for EvalCaseStream<R> {
    type Item = Result<EvalCase, DatasetError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_case() {
            Ok(Some(case)) => Some(Ok(case)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}
