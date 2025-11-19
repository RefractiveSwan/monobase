mod composite;
mod config;
#[cfg(feature = "http-client")]
mod http;
mod mock;
mod types;

pub use composite::CompositeTerminologyClient;
pub use config::TerminologyClientConfig;
#[cfg(feature = "http-client")]
pub use http::HttpTerminologyClient;
#[cfg(all(feature = "http-client", feature = "ncit-http"))]
pub use http::NcitTerminologyClient;
#[cfg(all(feature = "http-client", feature = "umls-http"))]
pub use http::UmlsTerminologyClient;
pub use mock::MockTerminologyClient;
pub use types::{
    CuiRecord, NcitRecord, TerminologyClient, TerminologyClientError, TerminologyMode,
    TerminologyResult,
};
