use thiserror::Error;

#[derive(Debug, Error)]
pub enum OboError {
    #[error("unsupported graph id: {0}")]
    UnsupportedGraph(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("failed to load graph from {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
