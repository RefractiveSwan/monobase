use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub mod bundles;
pub mod eval;
pub mod mapping;

/// Resolve the root directory for fixture files (relative to the crate).
fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn open_under(root: &Path, subdir: &str, file: &str) -> std::io::Result<File> {
    let path = root.join(subdir).join(file);
    File::open(path)
}

/// Registry provides typed access to eval + regression fixtures.
#[derive(Debug, Clone)]
pub struct Registry {
    root: PathBuf,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            root: fixtures_root(),
        }
    }
}

impl Registry {
    pub fn new_with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn open_regression_file(&self, name: &str) -> std::io::Result<File> {
        open_under(&self.root, "regression", &format!("{name}.json"))
    }

    pub fn open_eval(&self, name: &str) -> std::io::Result<File> {
        open_under(&self.root, "eval", &format!("{name}.ndjson"))
    }

    pub fn open_bundle(&self, name: &str) -> std::io::Result<File> {
        self.open_regression_file(name)
    }

    pub fn open_mapping(&self, name: &str) -> std::io::Result<File> {
        self.open_regression_file(name)
    }

    pub fn open_regression(&self, name: &str) -> std::io::Result<File> {
        self.open_regression_file(name)
    }
}

/// Canonical evaluation dataset record structure.
#[derive(Debug, Clone, Deserialize)]
pub struct EvalRow {
    pub system: String,
    pub code: String,
    pub display: String,
    pub expected_ncit_id: String,
}

/// Stream NDJSON records from disk without loading everything in memory.
pub fn stream_ndjson<T>(file: File) -> impl Iterator<Item = serde_json::Result<T>>
where
    T: for<'de> Deserialize<'de>,
{
    BufReader::new(file).lines().map(|line| {
        let line = line.map_err(serde_json::Error::io)?;
        serde_json::from_str(&line)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvalCase;

    #[test]
    fn eval_fixtures_match_evalcase_schema() {
        let registry = Registry::default();
        for dataset in [
            "bronze_pet_ct_small",
            "bronze_pet_ct_unknowns",
            "silver_pet_ct_small",
            "gold_pet_ct_small",
        ] {
            let iter = super::eval::iter(&registry, dataset)
                .unwrap_or_else(|err| panic!("open {dataset}: {err}"));
            let mut count = 0usize;
            for row in iter {
                let row = row.unwrap_or_else(|err| panic!("parse {dataset}: {err}"));
                let case = EvalCase {
                    system: row.system.clone(),
                    code: row.code.clone(),
                    display: row.display.clone(),
                    expected_ncit_id: row.expected_ncit_id.clone(),
                };
                assert!(
                    !case.system.is_empty() && !case.code.is_empty(),
                    "{dataset} rows must include system/code"
                );
                assert!(
                    case.expected_ncit_id.starts_with("NCIT:"),
                    "{dataset} expected_ncit_id should look like NCIT:Cxxxx"
                );
                count += 1;
            }
            assert!(count > 0, "{dataset} should contain rows");
        }
    }
}
