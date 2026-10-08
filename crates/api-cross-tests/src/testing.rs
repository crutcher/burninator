//! Shared cross-test setup: where reference recordings live, which backends
//! are checked against them, and the harness that runs one against the other.

use std::path::PathBuf;

use bunsen::{
    audit::{
        AuditBody,
        AuditProbe,
        AuditStreamVerifier,
        BaselineOutcome,
        audit_baseline,
        reports::{
            BaselineMode,
            ReportsOptions,
            backend_label,
            cargo_target_dir,
        },
    },
    errors::{
        BunsenResult,
        Multiple,
        ResultContext,
    },
};
use burn::tensor::Device;

/// The environment variable read for the [`BaselineMode`] of
/// [`reports_options`]: `auto` (unset), `record` or `verify`.
pub const REPORTS_MODE_VAR: &str = "API_CROSS_TESTS_REPORTS_MODE";

/// This crate's reports: `{target}/api-cross-tests-reports`, mode from
/// [`REPORTS_MODE_VAR`].
pub fn reports_options() -> BunsenResult<ReportsOptions> {
    let root = cargo_target_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("api-cross-tests-reports");
    Ok(ReportsOptions::new(root).with_mode(BaselineMode::from_env(REPORTS_MODE_VAR)?))
}

/// The device whose run is the reference recording: `flex`, the CPU backend.
pub fn reference_device() -> Device {
    Device::flex()
}

/// The devices checked against the reference recording: `flex` itself, then
/// one per backend feature enabled on this crate.
pub fn target_devices() -> Vec<Device> {
    #[allow(unused_mut)]
    let mut devices = vec![Device::flex()];
    #[cfg(feature = "cpu")]
    devices.push(Device::cpu());
    #[cfg(feature = "cuda")]
    devices.push(Device::cuda(0));
    #[cfg(feature = "rocm")]
    devices.push(Device::rocm(0));
    #[cfg(feature = "metal")]
    devices.push(Device::metal(burn::tensor::DeviceKind::DefaultDevice));
    #[cfg(feature = "vulkan")]
    devices.push(Device::vulkan(burn::tensor::DeviceKind::DefaultDevice));
    #[cfg(feature = "wgpu")]
    devices.push(Device::wgpu(burn::tensor::DeviceKind::DefaultDevice));
    devices
}

/// Runs `body` on `reference` against its stored baseline `name`, then
/// verifies each of `targets` against that one recording.
///
/// The reference run records the baseline, or verifies against it, as
/// `options.mode()` says. Every target is run, so the error names all that
/// diverge.
///
/// # Returns
/// The path of the reference recording.
///
/// # Errors
/// As [`audit_baseline`] for the reference run; a [`Multiple`] naming each
/// target that does not verify.
pub fn audit_targets_against_reference(
    name: &str,
    body: &impl AuditBody,
    options: &ReportsOptions,
    reference: &Device,
    targets: &[Device],
) -> BunsenResult<PathBuf> {
    let (BaselineOutcome::Recorded(path) | BaselineOutcome::Verified(path)) =
        audit_baseline(options, reference, name, body)
            .with_context(|| format!("reference {}", backend_label(reference)))?;

    println!("Cross-Test:: {name}");

    let expected = AuditStreamVerifier::load(&path)?;

    let failures: Vec<_> = targets
        .iter()
        .filter_map(|target| {
            println!("- {}: {:?}", backend_label(target), target);

            let mut verifier = expected.clone();
            body.run(&mut AuditProbe::new(vec![&mut verifier]), target)
                .and_then(|()| verifier.finish())
                .err()
                .map(|err| (backend_label(target), err))
        })
        .collect();
    if failures.is_empty() {
        return Ok(path);
    }

    Err(Multiple::new(
        format!(
            "{} of {} targets diverge from reference {}",
            failures.len(),
            targets.len(),
            path.display()
        ),
        failures,
    )
    .into())
}
