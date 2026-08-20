//! The Lenia interaction kernel: the radially symmetric weight function
//! convolved over each cell's neighborhood to produce its "potential" each
//! tick (see [`crate::universe::Universe::tick`]).
//!
//! Kernel core/shell formulas per Chan (2019) — see the crate-level docs in
//! [`crate`] for the full citation. All four `kn` variants share the same
//! contract: a function of a single normalized radius in `[0, 1)`, shaped so
//! a convolution against it (via [`kernel_shell`]/[`generate_kernel_matrix`])
//! stays radially symmetric and (for the two smooth variants) vanishes at
//! both ends of the ring.

/// Lenia's `kn = 1` kernel core: the polynomial "quad4" bump `(4r(1-r))^4`
/// — compact support like [`compute_step_kernel_core`], but smooth (and its
/// derivative smooth) rather than a hard cutoff.
pub(crate) fn compute_polynomial_kernel_core(radius: f32) -> f32 {
    if radius <= 0.0 || radius >= 1.0 {
        0.0
    } else {
        (4.0 * radius * (1.0 - radius)).powi(4)
    }
}

/// Lenia's `kn = 2` kernel core: the exponential/Gaussian bump
/// `exp(4 - 1/(r(1-r)))`, the standard shape shared across the Lenia family
/// — chosen so it (and its derivative) vanish smoothly at both ends of the
/// ring. This is the shape this crate hard-coded before `kn` became
/// selectable.
pub(crate) fn compute_exponential_kernel_core(radius: f32) -> f32 {
    if radius <= 0.0 || radius >= 1.0 {
        0.0
    } else {
        f32::exp(4.0 - 1.0 / (radius * (1.0 - radius)))
    }
}

/// Lenia's `kn = 3` kernel core: a hard step, `1.0` anywhere within
/// `[0.25, 0.75]` and `0.0` everywhere else — no smooth falloff at all.
pub(crate) fn compute_step_kernel_core(radius: f32) -> f32 {
    const Q: f32 = 0.25;
    if (Q..=1.0 - Q).contains(&radius) {
        1.0
    } else {
        0.0
    }
}

/// Lenia's `kn = 4` kernel core: the "staircase" (life-like) shape — the
/// same step as [`compute_step_kernel_core`], except `radius < 0.25` holds
/// at `0.5` instead of dropping straight to `0.0`, giving a discontinuous
/// half-value plateau near the ring's center.
pub(crate) fn compute_staircase_kernel_core(radius: f32) -> f32 {
    const Q: f32 = 0.25;
    if (Q..=1.0 - Q).contains(&radius) {
        1.0
    } else if radius < Q {
        0.5
    } else {
        0.0
    }
}

/// Combines the shell `ring_weights` (relative weight of each concentric
/// ring, outermost last) with `kernel_core` to get the kernel's value at
/// `radius`.
fn kernel_shell(radius: f32, ring_weights: &[f32], kernel_core: fn(f32) -> f32) -> f32 {
    if !(0f32..1f32).contains(&radius) || ring_weights.is_empty() {
        return 0f32;
    }

    let ring_count = ring_weights.len() as f32;
    let scaled_radius = radius * ring_count;

    let ring_index = (scaled_radius.floor() as usize).min(ring_weights.len() - 1);

    let radius_within_ring = scaled_radius % 1f32;

    let ring_weight = ring_weights[ring_index];
    let core_value = kernel_core(radius_within_ring);

    ring_weight * core_value
}

/// Rasterizes the continuous kernel into a square weight matrix of side
/// `2 * kernel_radius + 1`, normalized to sum to 1 so a convolution against
/// it stays in the same range as the cell states being sampled.
///
/// Returns the flattened matrix in row-major order alongside its side
/// length.
pub(crate) fn generate_kernel_matrix(
    kernel_radius: usize,
    ring_weights: &[f32],
    kernel_core: fn(f32) -> f32,
) -> (Vec<f32>, usize) {
    let grid_size = 2 * kernel_radius + 1;
    let mut kernel = Vec::with_capacity(grid_size * grid_size);

    let kernel_radius = kernel_radius as f32;

    for y in 0..grid_size {
        for x in 0..grid_size {
            let delta_x = (x as f32) - kernel_radius;
            let delta_y = (y as f32) - kernel_radius;

            let pixel_distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            let normalized_radius = pixel_distance / kernel_radius;

            let weight = kernel_shell(normalized_radius, ring_weights, kernel_core);
            kernel.push(weight);
        }
    }

    let total_weight: f32 = kernel.iter().sum();
    if total_weight > 0.0 {
        for weight in kernel.iter_mut() {
            *weight /= total_weight;
        }
    }

    (kernel, grid_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn exponential_kernel_core_is_zero_at_and_beyond_its_bounds() {
        assert_eq!(compute_exponential_kernel_core(0.0), 0.0);
        assert_eq!(compute_exponential_kernel_core(1.0), 0.0);
        assert_eq!(compute_exponential_kernel_core(-0.1), 0.0);
        assert_eq!(compute_exponential_kernel_core(1.1), 0.0);
    }

    #[test]
    fn exponential_kernel_core_peaks_at_the_midpoint() {
        // exp(4 - 1/(0.5*0.5)) = exp(4 - 4) = exp(0) = 1
        assert!(approx_eq(compute_exponential_kernel_core(0.5), 1.0));
        assert!(compute_exponential_kernel_core(0.5) > compute_exponential_kernel_core(0.3));
        assert!(compute_exponential_kernel_core(0.5) > compute_exponential_kernel_core(0.7));
    }

    #[test]
    fn polynomial_kernel_core_is_zero_at_and_beyond_its_bounds() {
        assert_eq!(compute_polynomial_kernel_core(0.0), 0.0);
        assert_eq!(compute_polynomial_kernel_core(1.0), 0.0);
        assert_eq!(compute_polynomial_kernel_core(-0.1), 0.0);
        assert_eq!(compute_polynomial_kernel_core(1.1), 0.0);
    }

    #[test]
    fn polynomial_kernel_core_peaks_at_the_midpoint() {
        // (4*0.5*(1-0.5))^4 = 1^4 = 1
        assert!(approx_eq(compute_polynomial_kernel_core(0.5), 1.0));
        assert!(compute_polynomial_kernel_core(0.5) > compute_polynomial_kernel_core(0.3));
        assert!(compute_polynomial_kernel_core(0.5) > compute_polynomial_kernel_core(0.7));
    }

    #[test]
    fn polynomial_kernel_core_is_symmetric_around_the_midpoint() {
        assert!(approx_eq(
            compute_polynomial_kernel_core(0.3),
            compute_polynomial_kernel_core(0.7)
        ));
    }

    #[test]
    fn step_kernel_core_is_flat_inside_the_step_and_zero_outside_it() {
        assert_eq!(compute_step_kernel_core(0.0), 0.0);
        assert_eq!(compute_step_kernel_core(0.2), 0.0);
        assert_eq!(compute_step_kernel_core(0.25), 1.0);
        assert_eq!(compute_step_kernel_core(0.5), 1.0);
        assert_eq!(compute_step_kernel_core(0.75), 1.0);
        assert_eq!(compute_step_kernel_core(0.8), 0.0);
    }

    #[test]
    fn step_kernel_core_is_symmetric_around_the_midpoint() {
        assert_eq!(compute_step_kernel_core(0.3), compute_step_kernel_core(0.7));
    }

    #[test]
    fn staircase_kernel_core_holds_a_half_plateau_below_the_step() {
        // Unlike every other core shape, this one is non-zero at the origin.
        assert_eq!(compute_staircase_kernel_core(0.0), 0.5);
        assert_eq!(compute_staircase_kernel_core(0.2), 0.5);
        assert_eq!(compute_staircase_kernel_core(0.25), 1.0);
        assert_eq!(compute_staircase_kernel_core(0.5), 1.0);
        assert_eq!(compute_staircase_kernel_core(0.75), 1.0);
        assert_eq!(compute_staircase_kernel_core(0.8), 0.0);
    }

    #[test]
    fn kernel_shell_is_zero_outside_bounds_or_without_rings() {
        assert_eq!(
            kernel_shell(-0.1, &[1.0], compute_exponential_kernel_core),
            0.0
        );
        assert_eq!(
            kernel_shell(1.0, &[1.0], compute_exponential_kernel_core),
            0.0
        );
        assert_eq!(kernel_shell(0.5, &[], compute_exponential_kernel_core), 0.0);
    }

    #[test]
    fn kernel_shell_picks_the_matching_ring_weight() {
        // Two rings [0.5, 1.0]; radius 0.25 falls in ring 0, radius 0.75 in
        // ring 1, and both land on the ring-local midpoint where the
        // exponential core peaks at 1.0.
        let ring_weights = [0.5, 1.0];

        assert!(approx_eq(
            kernel_shell(0.25, &ring_weights, compute_exponential_kernel_core),
            0.5
        ));
        assert!(approx_eq(
            kernel_shell(0.75, &ring_weights, compute_exponential_kernel_core),
            1.0
        ));
    }

    #[test]
    fn kernel_shell_dispatches_to_the_given_kernel_core() {
        // Same ring-local midpoint as the exponential-core test above, but
        // the step core is flat at 1.0 across the whole [0.25, 0.75] band
        // rather than peaking only at the midpoint — picking a radius near
        // (but not at) a ring's edge distinguishes the two shapes.
        let ring_weights = [1.0];

        assert!(approx_eq(
            kernel_shell(0.3, &ring_weights, compute_step_kernel_core),
            1.0
        ));
        assert!(
            kernel_shell(0.3, &ring_weights, compute_exponential_kernel_core)
                < kernel_shell(0.3, &ring_weights, compute_step_kernel_core)
        );
    }

    #[test]
    fn generate_kernel_matrix_has_the_expected_shape() {
        let (kernel, side) = generate_kernel_matrix(3, &[1.0], compute_exponential_kernel_core);

        assert_eq!(side, 7);
        assert_eq!(kernel.len(), 49);
    }

    #[test]
    fn generate_kernel_matrix_normalizes_to_sum_one() {
        let (kernel, _) = generate_kernel_matrix(5, &[1.0, 0.5], compute_exponential_kernel_core);

        let total: f32 = kernel.iter().sum();
        assert!(approx_eq(total, 1.0));
    }

    #[test]
    fn generate_kernel_matrix_is_symmetric_across_both_axes() {
        let (kernel, side) =
            generate_kernel_matrix(4, &[1.0, 0.7, 0.3], compute_exponential_kernel_core);

        for y in 0..side {
            for x in 0..side {
                let mirrored_x = kernel[y * side + (side - 1 - x)];
                let mirrored_y = kernel[(side - 1 - y) * side + x];
                assert!(approx_eq(kernel[y * side + x], mirrored_x));
                assert!(approx_eq(kernel[y * side + x], mirrored_y));
            }
        }
    }

    #[test]
    fn generate_kernel_matrix_changes_shape_with_a_different_kernel_core() {
        let (exponential, _) = generate_kernel_matrix(4, &[1.0], compute_exponential_kernel_core);
        let (polynomial, _) = generate_kernel_matrix(4, &[1.0], compute_polynomial_kernel_core);

        assert_ne!(exponential, polynomial);
    }
}
