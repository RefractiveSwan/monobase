use super::*;

#[test]
fn bundle_iterates_servicerequests() {
    let bundle = Bundle {
        resource_type: "Bundle".into(),
        bundle_type: Some("collection".into()),
        entry: vec![
            BundleEntry {
                full_url: None,
                resource: Some(serde_json::json!({
                    "resourceType": "ServiceRequest",
                    "id": "sr-1",
                    "status": "active",
                    "intent": "order",
                    "subject": { "reference": "Patient/p1" }
                })),
            },
            BundleEntry {
                full_url: None,
                resource: Some(serde_json::json!({
                    "resourceType": "Patient",
                    "id": "p1"
                })),
            },
        ],
    };

    let collected: Vec<_> = bundle
        .iter_servicerequests()
        .map(|sr| sr.unwrap().id.unwrap())
        .collect();
    assert_eq!(collected, vec!["sr-1"]);
}
