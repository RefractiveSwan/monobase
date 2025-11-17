use std::collections::HashMap;

use thiserror::Error;
#[cfg(feature = "http-client")]
use {
    reqwest::Url, reqwest::blocking::Client as HttpClient,
    reqwest::blocking::ClientBuilder as HttpClientBuilder,
};

/// Configures how terminology lookups should behave when HTTP clients are present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyClientConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: Option<u64>,
    pub mode: TerminologyMode,
}

impl TerminologyClientConfig {
    /// Build a config from environment variables.
    ///
    /// - `DFPS_TERMINOLOGY_BASE_URL`
    /// - `DFPS_TERMINOLOGY_API_KEY`
    /// - `DFPS_TERMINOLOGY_TIMEOUT_SECS`
    /// - `DFPS_TERMINOLOGY_MODE` (`mock_only` | `http_fallback` | `http_only`)
    pub fn from_env() -> Self {
        let base_url = std::env::var("DFPS_TERMINOLOGY_BASE_URL").ok();
        let api_key = std::env::var("DFPS_TERMINOLOGY_API_KEY").ok();
        let timeout_secs = std::env::var("DFPS_TERMINOLOGY_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok());
        let mode = std::env::var("DFPS_TERMINOLOGY_MODE")
            .ok()
            .and_then(|raw| TerminologyMode::from_env_value(&raw))
            .unwrap_or(TerminologyMode::MockOnly);
        Self {
            base_url,
            api_key,
            timeout_secs,
            mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminologyMode {
    MockOnly,
    HttpFallback,
    HttpOnly,
}

impl TerminologyMode {
    fn from_env_value(raw: &str) -> Option<Self> {
        match raw {
            "mock_only" => Some(Self::MockOnly),
            "http_fallback" => Some(Self::HttpFallback),
            "http_only" => Some(Self::HttpOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuiRecord {
    pub cui: String,
    pub preferred_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NcitRecord {
    pub ncit_id: String,
    pub preferred_name: String,
    pub synonyms: Vec<String>,
}

/// Errors that can occur during terminology lookups.
#[derive(Debug, Error)]
pub enum TerminologyClientError {
    #[error("terminology lookup forbidden")]
    Forbidden,
    #[error("terminology lookup timed out")]
    Timeout,
    #[error("terminology service unavailable")]
    Unavailable,
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("network error: {0}")]
    Network(String),
}

pub type TerminologyResult<T> = Result<T, TerminologyClientError>;

/// Abstraction for UMLS/NCIt external services.
pub trait TerminologyClient: Send + Sync {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>>;

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>>;

    fn search_by_text(&self, _text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "http-client")]
#[derive(Clone)]
pub struct HttpTerminologyClient {
    base_url: Url,
    api_key: Option<String>,
    client: HttpClient,
}

#[cfg(feature = "http-client")]
impl HttpTerminologyClient {
    pub fn from_config(cfg: &TerminologyClientConfig) -> Option<Self> {
        let base = cfg.base_url.as_ref()?;
        let url = Url::parse(base).ok()?;
        let mut builder = HttpClientBuilder::new();
        if let Some(secs) = cfg.timeout_secs {
            builder = builder.timeout(std::time::Duration::from_secs(secs));
        }
        let client = builder.build().ok()?;
        Some(Self {
            base_url: url,
            api_key: cfg.api_key.clone(),
            client,
        })
    }

    fn request_json<T: for<'de> serde::Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> TerminologyResult<Option<T>> {
        let mut url = self
            .base_url
            .join(endpoint)
            .map_err(|err| TerminologyClientError::InvalidResponse(err.to_string()))?;
        {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in params {
                pairs.append_pair(k, v);
            }
        }
        let mut req = self.client.get(url);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req
            .send()
            .map_err(|err| TerminologyClientError::Network(err.to_string()))?;
        if resp.status().is_success() {
            let parsed: T = resp
                .json()
                .map_err(|err| TerminologyClientError::InvalidResponse(err.to_string()))?;
            Ok(Some(parsed))
        } else if resp.status().as_u16() == 404 {
            Ok(None)
        } else if resp.status().as_u16() == 403 {
            Err(TerminologyClientError::Forbidden)
        } else if resp.status().as_u16() == 429 {
            Err(TerminologyClientError::Unavailable)
        } else {
            Err(TerminologyClientError::InvalidResponse(format!(
                "status {}",
                resp.status()
            )))
        }
    }
}

#[cfg(feature = "http-client")]
impl TerminologyClient for HttpTerminologyClient {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>> {
        self.request_json("/lookup_cui", &[("system", system), ("code", code)])
    }

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
        self.request_json("/lookup_ncit", &[("id", cui_or_code)])
    }

    fn search_by_text(&self, text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        Ok(self
            .request_json::<Vec<NcitRecord>>("/search", &[("q", text)])?
            .unwrap_or_default())
    }
}

/// Simple in-memory client for tests and offline use.
#[derive(Debug, Clone, Default)]
pub struct MockTerminologyClient {
    cui_map: HashMap<(String, String), CuiRecord>,
    ncit_map: HashMap<String, NcitRecord>,
    text_index: HashMap<String, Vec<NcitRecord>>,
}

impl MockTerminologyClient {
    pub fn with_cui(
        mut self,
        system: impl Into<String>,
        code: impl Into<String>,
        record: CuiRecord,
    ) -> Self {
        self.cui_map.insert((system.into(), code.into()), record);
        self
    }

    pub fn with_ncit(mut self, key: impl Into<String>, record: NcitRecord) -> Self {
        self.text_index
            .entry(record.preferred_name.to_lowercase())
            .or_default()
            .push(record.clone());
        self.ncit_map.insert(key.into(), record);
        self
    }

    pub fn with_synonym(mut self, synonym: impl Into<String>, record: NcitRecord) -> Self {
        self.text_index
            .entry(synonym.into().to_lowercase())
            .or_default()
            .push(record);
        self
    }
}

impl TerminologyClient for MockTerminologyClient {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>> {
        Ok(self
            .cui_map
            .get(&(system.to_string(), code.to_string()))
            .cloned())
    }

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
        Ok(self.ncit_map.get(cui_or_code).cloned())
    }

    fn search_by_text(&self, text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        Ok(self
            .text_index
            .get(&text.to_lowercase())
            .cloned()
            .unwrap_or_default())
    }
}

/// Composite client that prefers local/mock data and falls back to a remote client when present.
pub struct CompositeTerminologyClient {
    mock: Option<MockTerminologyClient>,
    remote: Option<Box<dyn TerminologyClient>>,
}

impl CompositeTerminologyClient {
    pub fn new(
        mock: Option<MockTerminologyClient>,
        remote: Option<Box<dyn TerminologyClient>>,
    ) -> Self {
        Self { mock, remote }
    }
}

impl TerminologyClient for CompositeTerminologyClient {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>> {
        if let Some(mock) = &self.mock {
            if let Some(hit) = mock.lookup_cui(system, code)? {
                return Ok(Some(hit));
            }
        }
        if let Some(remote) = &self.remote {
            return remote.lookup_cui(system, code);
        }
        Ok(None)
    }

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
        if let Some(mock) = &self.mock {
            if let Some(hit) = mock.lookup_ncit(cui_or_code)? {
                return Ok(Some(hit));
            }
        }
        if let Some(remote) = &self.remote {
            return remote.lookup_ncit(cui_or_code);
        }
        Ok(None)
    }

    fn search_by_text(&self, text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        if let Some(mock) = &self.mock {
            let results = mock.search_by_text(text)?;
            if !results.is_empty() {
                return Ok(results);
            }
        }
        if let Some(remote) = &self.remote {
            return remote.search_by_text(text);
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_client_returns_hits() {
        let mock = MockTerminologyClient::default()
            .with_cui(
                "http://loinc.org",
                "24606-6",
                CuiRecord {
                    cui: "C0001".into(),
                    preferred_name: "FDG uptake".into(),
                },
            )
            .with_ncit(
                "C1234",
                NcitRecord {
                    ncit_id: "C1234".into(),
                    preferred_name: "FDG Uptake".into(),
                    synonyms: vec!["FDG".into()],
                },
            );
        let cui = mock.lookup_cui("http://loinc.org", "24606-6").unwrap();
        assert_eq!(cui.unwrap().cui, "C0001");
        let ncit = mock.lookup_ncit("C1234").unwrap();
        assert_eq!(ncit.unwrap().preferred_name, "FDG Uptake");
    }

    #[test]
    fn composite_prefers_mock_then_remote() {
        struct Remote;
        impl TerminologyClient for Remote {
            fn lookup_cui(
                &self,
                _system: &str,
                _code: &str,
            ) -> TerminologyResult<Option<CuiRecord>> {
                Ok(Some(CuiRecord {
                    cui: "REMOTE".into(),
                    preferred_name: "Remote record".into(),
                }))
            }

            fn lookup_ncit(&self, _cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
                Ok(Some(NcitRecord {
                    ncit_id: "REMOTE".into(),
                    preferred_name: "Remote NCIt".into(),
                    synonyms: vec![],
                }))
            }
        }

        let mock = MockTerminologyClient::default().with_cui(
            "http://loinc.org",
            "24606-6",
            CuiRecord {
                cui: "MOCK".into(),
                preferred_name: "Mock record".into(),
            },
        );
        let composite = CompositeTerminologyClient::new(Some(mock), Some(Box::new(Remote)));
        let cui = composite.lookup_cui("http://loinc.org", "24606-6").unwrap();
        assert_eq!(cui.unwrap().cui, "MOCK");
        let ncit = composite.lookup_ncit("C9999").unwrap();
        assert_eq!(ncit.unwrap().ncit_id, "REMOTE");
    }
}
