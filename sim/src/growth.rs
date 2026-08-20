//! The growth mapping: turns a cell's convolved potential into a rate of
//! change for that cell.
//!
//! All three variants share the same contract: centered on `growth_target`,
//! rescaled to `[-1, 1]` so potential near the target grows a cell and
//! potential far from it decays it, and `growth_width` controls how far that
//! positive-growth region extends. Growth function formulas per Chan (2019)
//! — see the crate-level docs in [`crate`] for the full citation.

/// Lenia's `gn = 2` growth mapping: a Gaussian bump, so growth falls off
/// smoothly (and never quite reaches -1) the farther potential strays from
/// `growth_target`.
pub(crate) fn compute_gaussian_growth_rate(
    potential: f32,
    growth_target: f32,
    growth_width: f32,
) -> f32 {
    let difference = potential - growth_target;

    let numerator = -(difference * difference);
    let denominator = 2.0 * (growth_width * growth_width);

    2.0 * f32::exp(numerator / denominator) - 1.0
}

/// Lenia's `gn = 1` growth mapping: the "quad4" polynomial bump
/// `(1 - x²)⁴` (`x` the potential's distance from `growth_target`, scaled by
/// `3 * growth_width`) — compact support, flattening to exactly -1 for any
/// potential a full `3 * growth_width` or more from the target, instead of
/// the Gaussian's smooth asymptote.
pub(crate) fn compute_polynomial_growth_rate(
    potential: f32,
    growth_target: f32,
    growth_width: f32,
) -> f32 {
    let normalized = (potential - growth_target) / (3.0 * growth_width);
    let base = (1.0 - normalized * normalized).max(0.0);

    2.0 * base.powi(4) - 1.0
}

/// Lenia's `gn = 3` growth mapping: a hard step, +1 growth anywhere within
/// `growth_width` of `growth_target` and -1 everywhere else — no smooth
/// falloff at all.
pub(crate) fn compute_step_growth_rate(
    potential: f32,
    growth_target: f32,
    growth_width: f32,
) -> f32 {
    if (potential - growth_target).abs() <= growth_width {
        1.0
    } else {
        -1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn peaks_at_the_growth_target() {
        assert!(approx_eq(
            compute_gaussian_growth_rate(0.15, 0.15, 0.015),
            1.0
        ));
    }

    #[test]
    fn approaches_negative_one_far_from_the_target() {
        let rate = compute_gaussian_growth_rate(0.0, 0.15, 0.015);
        assert!(rate < -0.999);
    }

    #[test]
    fn is_symmetric_around_the_target() {
        let above = compute_gaussian_growth_rate(0.20, 0.15, 0.015);
        let below = compute_gaussian_growth_rate(0.10, 0.15, 0.015);
        assert!(approx_eq(above, below));
    }

    #[test]
    fn polynomial_peaks_at_the_growth_target() {
        assert!(approx_eq(
            compute_polynomial_growth_rate(0.15, 0.15, 0.015),
            1.0
        ));
    }

    #[test]
    fn polynomial_flattens_to_negative_one_beyond_three_times_the_width() {
        assert_eq!(compute_polynomial_growth_rate(0.2, 0.15, 0.015), -1.0);
        assert_eq!(compute_polynomial_growth_rate(0.25, 0.15, 0.015), -1.0);
    }

    #[test]
    fn polynomial_still_grows_within_three_times_the_width() {
        // 0.17 is only ~1.33x growth_width from target, inside the
        // 3x-growth_width compact support, so it should NOT have flattened
        // to -1 yet.
        assert!(compute_polynomial_growth_rate(0.17, 0.15, 0.015) > -1.0);
    }

    #[test]
    fn polynomial_is_symmetric_around_the_target() {
        // distances within the width, so the curve's actual shape is
        // exercised rather than both sides trivially flattening to -1
        let above = compute_polynomial_growth_rate(0.158, 0.15, 0.015);
        let below = compute_polynomial_growth_rate(0.142, 0.15, 0.015);
        assert!(approx_eq(above, below));
        assert!(above > -1.0 && above < 1.0);
    }

    #[test]
    fn step_is_positive_within_width_and_negative_outside_it() {
        assert_eq!(compute_step_growth_rate(0.15, 0.15, 0.015), 1.0);
        assert_eq!(compute_step_growth_rate(0.164, 0.15, 0.015), 1.0);
        assert_eq!(compute_step_growth_rate(0.166, 0.15, 0.015), -1.0);
    }

    #[test]
    fn step_is_symmetric_around_the_target() {
        let above = compute_step_growth_rate(0.20, 0.15, 0.015);
        let below = compute_step_growth_rate(0.10, 0.15, 0.015);
        assert_eq!(above, below);
    }
}
