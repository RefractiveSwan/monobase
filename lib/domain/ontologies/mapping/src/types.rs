use std::collections::BTreeMap;

use dfps_terminology::CodeKind;

pub const DEFAULT_VECTOR_TOP_K: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FusionWeights {
    pub lexical: f32,
    pub vector: f32,
}

impl FusionWeights {
    pub fn new(lexical: f32, vector: f32) -> Self {
        let mut weights = Self { lexical, vector };
        weights.ensure_valid();
        weights
    }

    fn ensure_valid(&mut self) {
        if self.lexical <= 0.0 && self.vector <= 0.0 {
            self.lexical = 1.0;
            self.vector = 1.0;
        } else {
            if self.lexical <= 0.0 {
                self.lexical = 0.01;
            }
            if self.vector <= 0.0 {
                self.vector = 0.01;
            }
        }
    }
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            lexical: 1.0,
            vector: 1.0,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MappingSummary {
    pub total: usize,
    pub by_code_kind: BTreeMap<String, usize>,
    pub by_license_tier: BTreeMap<String, usize>,
    pub extern_lookup_success: usize,
    pub extern_lookup_miss: usize,
    pub extern_lookup_error: usize,
}

impl MappingSummary {
    pub fn record(&mut self, kind: CodeKind, license_label: Option<&str>) {
        self.total += 1;
        let kind_key = kind.as_str().to_string();
        *self.by_code_kind.entry(kind_key).or_default() += 1;

        let license_key = license_label.unwrap_or("unknown").to_string();
        *self.by_license_tier.entry(license_key).or_default() += 1;
    }

    pub fn record_external_success(&mut self) {
        self.extern_lookup_success += 1;
    }

    pub fn record_external_miss(&mut self) {
        self.extern_lookup_miss += 1;
    }

    pub fn record_external_error(&mut self) {
        self.extern_lookup_error += 1;
    }
}
