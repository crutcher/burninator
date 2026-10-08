//! Cross-tests of `Tensor::sin` and `Tensor::cos` around the circle, and
//! around it again after shifts of whole turns.

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use bunsen::{
        audit::AuditProbe,
        burner::descriptors::TolerancePolicy,
        errors::{
            BunsenResult,
            WithOkOrPanic,
        },
    };
    use burn::tensor::{
        Device,
        Tensor,
        TensorData,
    };

    use crate::testing::{
        audit_targets_against_reference,
        reference_device,
        reports_options,
        target_devices,
    };

    /// Steps per turn: every multiple of π/12, which takes in 0, π/6, π/4,
    /// π/3 and π/2 in each quadrant.
    const STEPS: usize = 24;

    /// Whole turns added to the circle: none, then larger shifts both ways,
    /// where each backend's range reduction starts to show.
    const SHIFT_TURNS: [i32; 9] = [0, 1, -1, 10, -10, 100, -100, 1000, -1000];

    /// The circle shifted by `turns`: `τ * (turns + k / STEPS)`, each rounded
    /// once to `f32`.
    fn shifted_circle(turns: i32) -> Vec<f32> {
        (0..STEPS)
            .map(|k| (TAU * (turns as f64 + k as f64 / STEPS as f64)) as f32)
            .collect()
    }

    /// The tolerance for `sin` and `cos` of angles up to `max_abs` in size.
    ///
    /// The outputs are bounded by 1, so this is absolute. Range reduction loses
    /// about an `f32` ulp of the angle, so it widens with the angle.
    fn tolerance(max_abs: f64) -> TolerancePolicy {
        TolerancePolicy::Absolute {
            absolute: 1e-5 + 4.0 * f32::EPSILON as f64 * max_abs,
        }
    }

    /// For each shift: the angles, exactly, then their `sin` and `cos`.
    fn sin_cos_around_the_circle(
        probe: &mut AuditProbe<'_>,
        device: &Device,
    ) -> BunsenResult<()> {
        for turns in SHIFT_TURNS {
            let angles = shifted_circle(turns);
            let max_abs = angles.iter().fold(0.0f64, |m, &a| m.max(a.abs() as f64));
            let theta = Tensor::<1>::from_data(TensorData::new(angles, [STEPS]), device);

            let at = format!("θ = τ * ({turns} + k / {STEPS})");
            probe.assert_eq_as::<f32>(&at, &theta)?;
            probe.assert_approx_eq_as::<f32>(
                &format!("sin({at})"),
                &theta.clone().sin(),
                tolerance(max_abs),
            )?;
            probe.assert_approx_eq_as::<f32>(
                &format!("cos({at})"),
                &theta.cos(),
                tolerance(max_abs),
            )?;
        }
        Ok(())
    }

    #[test]
    fn test_sin_cos_around_the_circle() {
        audit_targets_against_reference(
            "tensor/float/trig/sin",
            &sin_cos_around_the_circle,
            &reports_options().ok_or_panic(),
            &reference_device(),
            &target_devices(),
        )
        .ok_or_panic();
    }
}
