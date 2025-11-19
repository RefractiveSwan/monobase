use super::config::TerminologyClientConfig;
use super::types::{
    CuiRecord, NcitRecord, TerminologyClient, TerminologyClientError, TerminologyResult,
};
use reqwest::Url;
use reqwest::blocking::{Client as HttpClient, ClientBuilder as HttpClientBuilder};

pub struct HttpTerminologyClient {
    base_url: Url,
    api_key: Option<String>,
    client: HttpClient,
}

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

#[cfg(feature = "umls-http")]
pub type UmlsTerminologyClient = HttpTerminologyClient;
#[cfg(feature = "ncit-http")]
pub type NcitTerminologyClient = HttpTerminologyClient;
