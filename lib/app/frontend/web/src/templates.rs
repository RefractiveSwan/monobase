pub struct BundleTemplateMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

const PET_CT_TEMPLATE: &str = include_str!(
    "../../../../domain/meta/evaluation/data/regression/fhir_bundle_uppercase_status.json"
);
const ONCOLOGY_TEMPLATE: &str = include_str!(
    "../../../../domain/meta/evaluation/data/regression/fhir_bundle_extra_codings.json"
);
const BLANK_TEMPLATE: &str = r#"{
  "resourceType": "Bundle",
  "type": "collection",
  "entry": [
    {
      "resource": {
        "resourceType": "ServiceRequest",
        "id": "SR-NEW",
        "status": "active",
        "intent": "order",
        "subject": { "reference": "Patient/PAT-NEW" },
        "code": {
          "coding": [
            {
              "system": "http://loinc.org",
              "code": "00000-0",
              "display": "Example ServiceRequest"
            }
          ]
        }
      }
    },
    { "resource": { "resourceType": "Patient", "id": "PAT-NEW" } }
  ]
}"#;

pub const TEMPLATES: &[BundleTemplateMeta] = &[
    BundleTemplateMeta {
        id: "pet_ct",
        label: "PET/CT sample",
        description: "Baseline PET/CT order with SNOMED coding",
    },
    BundleTemplateMeta {
        id: "oncology",
        label: "Oncology sample",
        description: "Multiple codings + oncology staging order",
    },
    BundleTemplateMeta {
        id: "blank",
        label: "Blank template",
        description: "Minimal bundle skeleton",
    },
];

pub fn load_template(id: &str) -> &'static str {
    match id {
        "pet_ct" => PET_CT_TEMPLATE,
        "oncology" => ONCOLOGY_TEMPLATE,
        "blank" => BLANK_TEMPLATE,
        _ => BLANK_TEMPLATE,
    }
}
