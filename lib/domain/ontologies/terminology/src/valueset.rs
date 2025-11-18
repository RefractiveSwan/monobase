/// Metadata describing a ValueSet and its component systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueSetMeta {
    pub url: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub include_systems: &'static [&'static str],
}

impl ValueSetMeta {
    pub const fn new(
        url: &'static str,
        name: &'static str,
        description: &'static str,
        include_systems: &'static [&'static str],
    ) -> Self {
        Self {
            url,
            name,
            description,
            include_systems,
        }
    }
}

static VALUE_SETS: [ValueSetMeta; 2] = [
    ValueSetMeta::new(
        "http://terminology.dfps/ValueSet/pet-imaging-procedures",
        "DFPS PET Imaging Procedures",
        "Subset of CPT and SNOMED codes relevant to PET/CT workflows.",
        &["http://www.ama-assn.org/go/cpt", "http://snomed.info/sct"],
    ),
    ValueSetMeta::new(
        "http://terminology.dfps/ValueSet/imaging-ordering",
        "DFPS Imaging Ordering",
        "LOINC observations and NCIt OBO concepts used for ordering context.",
        &["http://loinc.org", "http://purl.obolibrary.org/obo/NCIT"],
    ),
];

pub fn list_value_sets() -> &'static [ValueSetMeta] {
    &VALUE_SETS
}

pub fn lookup_value_set(url: &str) -> Option<&'static ValueSetMeta> {
    VALUE_SETS
        .iter()
        .find(|vs| vs.url.eq_ignore_ascii_case(url))
}
