use refractive_swan_terminology_port::{
    CuiRecord, NcitRecord, TerminologyClient, TerminologyClientError, TerminologyResult,
};
use std::collections::HashMap;

/// Simple in-memory client for tests and offline use.
#[derive(Debug, Clone, Default)]
pub struct MockTerminologyClient {
    cui_map: HashMap<(String, String), CuiRecord>,
    ncit_map: HashMap<String, NcitRecord>,
    text_index: HashMap<String, Vec<NcitRecord>>,
    delay_ms: Option<u64>,
    cui_errors: HashMap<(String, String), TerminologyClientError>,
    ncit_errors: HashMap<String, TerminologyClientError>,
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

    pub fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.delay_ms = Some(delay_ms);
        self
    }

    pub fn with_cui_error(
        mut self,
        system: impl Into<String>,
        code: impl Into<String>,
        error: TerminologyClientError,
    ) -> Self {
        self.cui_errors.insert((system.into(), code.into()), error);
        self
    }

    pub fn with_ncit_error(
        mut self,
        key: impl Into<String>,
        error: TerminologyClientError,
    ) -> Self {
        self.ncit_errors.insert(key.into(), error);
        self
    }

    fn maybe_sleep(&self) {
        if let Some(ms) = self.delay_ms {
            std::thread::sleep(std::time::Duration::from_millis(ms));
        }
    }
}

impl TerminologyClient for MockTerminologyClient {
    fn lookup_cui(&self, system: &str, code: &str) -> TerminologyResult<Option<CuiRecord>> {
        self.maybe_sleep();
        if let Some(err) = self.cui_errors.get(&(system.to_string(), code.to_string())) {
            return Err(err.clone());
        }
        Ok(self
            .cui_map
            .get(&(system.to_string(), code.to_string()))
            .cloned())
    }

    fn lookup_ncit(&self, cui_or_code: &str) -> TerminologyResult<Option<NcitRecord>> {
        self.maybe_sleep();
        if let Some(err) = self.ncit_errors.get(cui_or_code) {
            return Err(err.clone());
        }
        Ok(self.ncit_map.get(cui_or_code).cloned())
    }

    fn search_by_text(&self, text: &str) -> TerminologyResult<Vec<NcitRecord>> {
        self.maybe_sleep();
        Ok(self
            .text_index
            .get(&text.to_lowercase())
            .cloned()
            .unwrap_or_default())
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
    fn mock_client_can_error_and_delay() {
        let mock = MockTerminologyClient::default()
            .with_cui_error(
                "http://loinc.org",
                "24606-6",
                TerminologyClientError::Unavailable,
            )
            .with_delay_ms(1);
        let start = std::time::Instant::now();
        let err = mock.lookup_cui("http://loinc.org", "24606-6").unwrap_err();
        assert!(matches!(err, TerminologyClientError::Unavailable));
        assert!(start.elapsed() >= std::time::Duration::from_millis(1));
    }
}
