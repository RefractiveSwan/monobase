mod composite;
#[cfg(feature = "http-client")]
mod http;
mod mock;

pub use composite::CompositeTerminologyClient;
#[cfg(feature = "http-client")]
pub use http::HttpTerminologyClient;
#[cfg(all(feature = "http-client", feature = "ncit-http"))]
pub use http::NcitTerminologyClient;
#[cfg(all(feature = "http-client", feature = "umls-http"))]
pub use http::UmlsTerminologyClient;
pub use mock::MockTerminologyClient;
pub use refractive_swan_terminology_port::{
    CuiRecord, NcitRecord, TerminologyClient, TerminologyClientConfig, TerminologyClientError,
    TerminologyMode, TerminologyResult,
};
