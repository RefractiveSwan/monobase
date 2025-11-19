use super::mock::MockTerminologyClient;
use super::types::{CuiRecord, NcitRecord, TerminologyClient, TerminologyResult};

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
        if let Some(mock) = &self.mock
            && let Some(hit) = mock.lookup_cui(system, code)?
        {
            return Ok(Some(hit));
        }
        if let Some(remote) = &self.remote {
            return remote.lookup_cui(system, code);
        }
        Ok(None)
    }

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
        if let Some(mock) = &self.mock
            && let Some(hit) = mock.lookup_ncit(cui_or_code)?
        {
            return Ok(Some(hit));
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
    use super::super::mock::MockTerminologyClient;
    use super::super::types::{CuiRecord, NcitRecord, TerminologyClient, TerminologyClientError};
    use super::*;

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

    #[test]
    fn propagates_mock_errors() {
        let mock = MockTerminologyClient::default().with_cui_error(
            "http://loinc.org",
            "123",
            TerminologyClientError::Unavailable,
        );
        let composite = CompositeTerminologyClient::new(Some(mock), None);
        let err = composite.lookup_cui("http://loinc.org", "123").unwrap_err();
        assert!(matches!(err, TerminologyClientError::Unavailable));
    }
}
