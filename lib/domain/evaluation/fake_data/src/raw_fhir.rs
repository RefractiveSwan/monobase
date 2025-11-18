use crate::value::{
    fake_encounter_id_with_rng, fake_patient_id_with_rng, fake_service_request_id_with_rng,
    fake_service_request_intent_with_rng, fake_service_request_status_with_rng,
};
use dfps_core::{
    fhir,
    order::{ServiceRequestIntent, ServiceRequestStatus},
};
use rand::{Rng, SeedableRng, rng, rngs::StdRng, seq::IndexedRandom};
use serde_json::to_value;

#[derive(Debug, Clone)]
struct ProcedureCoding {
    system: &'static str,
    code: &'static str,
    display: &'static str,
}

const SNOMED_CODES: &[ProcedureCoding] = &[
    ProcedureCoding {
        system: "http://snomed.info/sct",
        code: "2460006",
        display: "Positron emission tomography of whole body",
    },
    ProcedureCoding {
        system: "http://snomed.info/sct",
        code: "441567006",
        display: "PET-CT for neoplasm staging",
    },
];

const CPT_CODES: &[ProcedureCoding] = &[
    ProcedureCoding {
        system: "http://www.ama-assn.org/go/cpt",
        code: "78815",
        display: "PET with concurrently acquired CT for tumor imaging",
    },
    ProcedureCoding {
        system: "http://www.ama-assn.org/go/cpt",
        code: "78816",
        display: "PET with concurrently acquired CT; whole body",
    },
];

const LOINC_CODES: &[ProcedureCoding] = &[
    ProcedureCoding {
        system: "http://loinc.org",
        code: "24606-6",
        display: "FDG uptake PET",
    },
    ProcedureCoding {
        system: "http://loinc.org",
        code: "88029-4",
        display: "PET whole body uptake",
    },
];

const CATEGORY_CODES: &[ProcedureCoding] = &[
    ProcedureCoding {
        system: "http://snomed.info/sct",
        code: "363679005",
        display: "Imaging",
    },
    ProcedureCoding {
        system: "http://snomed.info/sct",
        code: "103693007",
        display: "Diagnostic procedure",
    },
];

/// Synthetic bundle with Patient + Encounter + ServiceRequest resources.
pub struct FhirBundleScenario {
    pub bundle: fhir::Bundle,
    pub patient: fhir::Patient,
    pub encounter: fhir::Encounter,
    pub service_request: fhir::ServiceRequest,
}

pub fn fake_fhir_patient() -> fhir::Patient {
    let mut rng = rng();
    fake_fhir_patient_with_rng(&mut rng)
}

pub fn fake_fhir_patient_with_seed(seed: u64) -> fhir::Patient {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_fhir_patient_with_rng(&mut rng)
}

fn fake_fhir_patient_with_rng<R: Rng + ?Sized>(rng: &mut R) -> fhir::Patient {
    let id = fake_patient_id_with_rng(rng).0;
    fhir::Patient {
        resource_type: "Patient".into(),
        id: Some(id),
    }
}

pub fn fake_fhir_encounter_for(patient: &fhir::Patient) -> fhir::Encounter {
    let mut rng = rng();
    fake_fhir_encounter_for_with_rng(patient, &mut rng)
}

pub fn fake_fhir_encounter_for_with_seed(seed: u64, patient: &fhir::Patient) -> fhir::Encounter {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_fhir_encounter_for_with_rng(patient, &mut rng)
}

fn fake_fhir_encounter_for_with_rng<R: Rng + ?Sized>(
    _patient: &fhir::Patient,
    rng: &mut R,
) -> fhir::Encounter {
    let id = fake_encounter_id_with_rng(rng).0;
    fhir::Encounter {
        resource_type: "Encounter".into(),
        id: Some(id),
    }
}

pub fn fake_fhir_servicerequest(
    patient: &fhir::Patient,
    encounter: Option<&fhir::Encounter>,
) -> fhir::ServiceRequest {
    let mut rng = rng();
    fake_fhir_servicerequest_with_rng(patient, encounter, &mut rng)
}

pub fn fake_fhir_servicerequest_with_seed(
    seed: u64,
    patient: &fhir::Patient,
    encounter: Option<&fhir::Encounter>,
) -> fhir::ServiceRequest {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_fhir_servicerequest_with_rng(patient, encounter, &mut rng)
}

fn fake_fhir_servicerequest_with_rng<R: Rng + ?Sized>(
    patient: &fhir::Patient,
    encounter: Option<&fhir::Encounter>,
    rng: &mut R,
) -> fhir::ServiceRequest {
    let id = fake_service_request_id_with_rng(rng).0;
    let status = fake_service_request_status_with_rng(rng);
    let intent = normalize_intent(status, fake_service_request_intent_with_rng(rng));
    let coding = sample_procedure_codings(rng);
    let code_text = coding
        .first()
        .and_then(|coding| coding.display.clone())
        .unwrap_or_else(|| "PET/CT procedure".to_string());

    fhir::ServiceRequest {
        resource_type: "ServiceRequest".into(),
        id: Some(id),
        status: Some(status_to_fhir(status).into()),
        intent: Some(intent_to_fhir(intent).into()),
        subject: Some(fhir::Reference {
            reference: patient.id.as_ref().map(|id| format!("Patient/{id}")),
            display: None,
        }),
        encounter: encounter.and_then(|enc| {
            enc.id.as_ref().map(|id| fhir::Reference {
                reference: Some(format!("Encounter/{id}")),
                display: None,
            })
        }),
        requester: None,
        supporting_info: vec![],
        code: Some(fhir::CodeableConcept {
            coding,
            text: Some(code_text),
        }),
        category: vec![fhir::CodeableConcept {
            coding: vec![sample_category_coding(rng)],
            text: None,
        }],
        description: Some("PET/CT order from fake data".into()),
        authored_on: Some("2024-05-01T12:00:00Z".into()),
    }
}

pub fn fake_fhir_bundle_scenario() -> FhirBundleScenario {
    let mut rng = rng();
    fake_fhir_bundle_scenario_with_rng(&mut rng)
}

pub fn fake_fhir_bundle_scenario_with_seed(seed: u64) -> FhirBundleScenario {
    let mut rng = StdRng::seed_from_u64(seed);
    fake_fhir_bundle_scenario_with_rng(&mut rng)
}

fn fake_fhir_bundle_scenario_with_rng<R: Rng + ?Sized>(rng: &mut R) -> FhirBundleScenario {
    let patient = fake_fhir_patient_with_rng(rng);
    let encounter = fake_fhir_encounter_for_with_rng(&patient, rng);
    let service_request = fake_fhir_servicerequest_with_rng(&patient, Some(&encounter), rng);

    let entries = vec![
        to_entry(&patient),
        to_entry(&encounter),
        to_entry(&service_request),
    ];

    FhirBundleScenario {
        bundle: fhir::Bundle {
            resource_type: "Bundle".into(),
            bundle_type: Some("collection".into()),
            entry: entries,
        },
        patient,
        encounter,
        service_request,
    }
}

fn to_entry(resource: &impl serde::Serialize) -> fhir::BundleEntry {
    fhir::BundleEntry {
        full_url: None,
        resource: Some(
            to_value(resource).expect("fake FHIR resources should serialize to JSON value"),
        ),
    }
}

fn sample_procedure_codings<R: Rng + ?Sized>(rng: &mut R) -> Vec<fhir::Coding> {
    let mut entries: Vec<fhir::Coding> =
        vec![pick_code(SNOMED_CODES, rng), pick_code(CPT_CODES, rng)];

    if rng.random_range(0..=1) == 1 {
        entries.push(pick_code(LOINC_CODES, rng));
    }

    entries
}

fn sample_category_coding<R: Rng + ?Sized>(rng: &mut R) -> fhir::Coding {
    pick_code(CATEGORY_CODES, rng)
}

fn pick_code<R: Rng + ?Sized>(pool: &[ProcedureCoding], rng: &mut R) -> fhir::Coding {
    pool.choose(rng)
        .map(|coding| fhir::Coding {
            system: Some(coding.system.to_string()),
            code: Some(coding.code.to_string()),
            display: Some(coding.display.to_string()),
        })
        .unwrap()
}

fn status_to_fhir(status: ServiceRequestStatus) -> &'static str {
    match status {
        ServiceRequestStatus::Draft => "draft",
        ServiceRequestStatus::Active => "active",
        ServiceRequestStatus::OnHold => "on-hold",
        ServiceRequestStatus::Completed => "completed",
        ServiceRequestStatus::Cancelled => "cancelled",
        ServiceRequestStatus::Revoked => "revoked",
        ServiceRequestStatus::EnteredInError => "entered-in-error",
    }
}

fn intent_to_fhir(intent: ServiceRequestIntent) -> &'static str {
    match intent {
        ServiceRequestIntent::Proposal => "proposal",
        ServiceRequestIntent::Plan => "plan",
        ServiceRequestIntent::Order => "order",
        ServiceRequestIntent::OriginalOrder => "original-order",
        ServiceRequestIntent::ReflexOrder => "reflex-order",
        ServiceRequestIntent::FillerOrder => "filler-order",
    }
}

fn normalize_intent(
    status: ServiceRequestStatus,
    intent: ServiceRequestIntent,
) -> ServiceRequestIntent {
    match status {
        ServiceRequestStatus::Draft => match intent {
            ServiceRequestIntent::Plan | ServiceRequestIntent::Proposal => intent,
            _ => ServiceRequestIntent::Plan,
        },
        ServiceRequestStatus::Completed | ServiceRequestStatus::Cancelled => {
            ServiceRequestIntent::Order
        }
        _ => intent,
    }
}
