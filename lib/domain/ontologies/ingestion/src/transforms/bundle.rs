use dfps_core::{fhir, staging::StgSrCodeExploded};

use crate::transforms::errors::IngestionError;
use crate::transforms::service_request::{sr_to_domain, sr_to_staging};
use crate::validation::types::{Validated, ValidationMode};
use crate::validation::{
    ExternalValidationContext, validate_bundle, validate_bundle_with_external,
};

/// Convert a bundle into staging row collections.
pub fn bundle_to_staging(
    bundle: &fhir::Bundle,
) -> Result<
    (
        Vec<dfps_core::staging::StgServiceRequestFlat>,
        Vec<StgSrCodeExploded>,
    ),
    IngestionError,
> {
    bundle_to_staging_with_validation(
        bundle,
        ValidationMode::default(),
        ExternalValidationContext::default(),
    )
    .map(|validated| validated.value)
}

/// Convert a bundle into staging rows, returning validation metadata.
pub fn bundle_to_staging_with_validation(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    external: ExternalValidationContext<'_>,
) -> Result<
    Validated<(
        Vec<dfps_core::staging::StgServiceRequestFlat>,
        Vec<StgSrCodeExploded>,
    )>,
    IngestionError,
> {
    let report = match mode {
        ValidationMode::Strict | ValidationMode::Lenient => validate_bundle(bundle),
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict => {
            validate_bundle_with_external(bundle, mode, external)
        }
    };
    if matches!(
        mode,
        ValidationMode::Strict | ValidationMode::ExternalStrict
    ) && report.has_errors()
    {
        return Err(IngestionError::ValidationFailed(report.issues.clone()));
    }
    let (flats, exploded) = bundle_to_staging_inner(bundle)?;
    Ok(Validated::new((flats, exploded), report))
}

fn bundle_to_staging_inner(
    bundle: &fhir::Bundle,
) -> Result<
    (
        Vec<dfps_core::staging::StgServiceRequestFlat>,
        Vec<StgSrCodeExploded>,
    ),
    IngestionError,
> {
    let mut flats = Vec::new();
    let mut exploded = Vec::new();

    for entry in bundle.iter_servicerequests() {
        let sr = entry?;
        let (flat, codes) = sr_to_staging(&sr)?;
        flats.push(flat);
        exploded.extend(codes);
    }

    Ok((flats, exploded))
}

/// Convert a bundle into domain ServiceRequest aggregates.
pub fn bundle_to_domain(
    bundle: &fhir::Bundle,
) -> Result<Vec<dfps_core::order::ServiceRequest>, IngestionError> {
    bundle_to_domain_with_validation(
        bundle,
        ValidationMode::default(),
        ExternalValidationContext::default(),
    )
    .map(|validated| validated.value)
}

/// Convert a bundle into domain ServiceRequest aggregates, returning validation metadata.
pub fn bundle_to_domain_with_validation(
    bundle: &fhir::Bundle,
    mode: ValidationMode,
    external: ExternalValidationContext<'_>,
) -> Result<Validated<Vec<dfps_core::order::ServiceRequest>>, IngestionError> {
    let report = match mode {
        ValidationMode::Strict | ValidationMode::Lenient => validate_bundle(bundle),
        ValidationMode::ExternalPreferred | ValidationMode::ExternalStrict => {
            validate_bundle_with_external(bundle, mode, external)
        }
    };
    if matches!(
        mode,
        ValidationMode::Strict | ValidationMode::ExternalStrict
    ) && report.has_errors()
    {
        return Err(IngestionError::ValidationFailed(report.issues.clone()));
    }
    let output = bundle_to_domain_inner(bundle)?;
    Ok(Validated::new(output, report))
}

fn bundle_to_domain_inner(
    bundle: &fhir::Bundle,
) -> Result<Vec<dfps_core::order::ServiceRequest>, IngestionError> {
    let mut output = Vec::new();
    for entry in bundle.iter_servicerequests() {
        let sr = entry?;
        output.push(sr_to_domain(&sr)?);
    }
    Ok(output)
}
