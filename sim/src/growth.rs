//! The growth mapping: turns a cell's convolved potential into a rate of
//! change for that cell.
//!
//! Growth function formula per Chan (2019) — see the crate-level docs in
//! [`crate`] for the full citation.

/// Lenia's growth mapping `G`: a Gaussian bump centered on `growth_target`
/// with width `growth_width`, rescaled from `[0, 1]` to `[-1, 1]` so
/// potential near the target grows a cell and potential far from it decays
/// it.
pub(crate) fn compute_growth_rate(potential: f32, growth_target: f32, growth_width: f32) -> f32 {
    let difference = potential - growth_target;
    let numerator = -(difference * difference);
    let denominator = 2.0 * (growth_width * growth_width);

    2.0 * f32::exp(numerator / denominator) - 1.0
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
        assert!(approx_eq(compute_growth_rate(0.15, 0.15, 0.015), 1.0));
    }

    #[test]
    fn approaches_negative_one_far_from_the_target() {
        let rate = compute_growth_rate(0.0, 0.15, 0.015);
        assert!(rate < -0.999);
    }

    #[test]
    fn is_symmetric_around_the_target() {
        let above = compute_growth_rate(0.20, 0.15, 0.015);
        let below = compute_growth_rate(0.10, 0.15, 0.015);
        assert!(approx_eq(above, below));
    }
}
