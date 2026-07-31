//! The Lenia interaction kernel: the radially symmetric weight function
//! convolved over each cell's neighborhood to produce its "potential" each
//! tick (see [`crate::universe::Universe::tick`]).
//!
//! Kernel core/shell formulas per Chan (2019) — see the crate-level docs in
//! [`crate`] for the full citation.

/// Lenia's smooth bell-shaped kernel core `K_c`, evaluated at a single
/// normalized radius.
///
/// `radius` must be in `[0, 1)`; the closed form `exp(4 - 1/(r(1-r)))` is
/// the standard bump shared across the Lenia family, chosen so it (and its
/// derivative) vanish smoothly at both ends of the ring.
fn kernel_core(radius: f32) -> f32 {
    if radius <= 0.0 || radius >= 1.0 {
        0.0
    } else {
        f32::exp(4.0 - 1.0 / (radius * (1.0 - radius)))
    }
}

/// Combines the shell `ring_weights` (relative weight of each concentric
/// ring, outermost last) with [`kernel_core`] to get the kernel's value at
/// `radius`.
fn kernel_shell(radius: f32, ring_weights: &[f32]) -> f32 {
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

            let weight = kernel_shell(normalized_radius, ring_weights);
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
    fn kernel_core_is_zero_at_and_beyond_its_bounds() {
        assert_eq!(kernel_core(0.0), 0.0);
        assert_eq!(kernel_core(1.0), 0.0);
        assert_eq!(kernel_core(-0.1), 0.0);
        assert_eq!(kernel_core(1.1), 0.0);
    }

    #[test]
    fn kernel_core_peaks_at_the_midpoint() {
        // exp(4 - 1/(0.5*0.5)) = exp(4 - 4) = exp(0) = 1
        assert!(approx_eq(kernel_core(0.5), 1.0));
        assert!(kernel_core(0.5) > kernel_core(0.3));
        assert!(kernel_core(0.5) > kernel_core(0.7));
    }

    #[test]
    fn kernel_shell_is_zero_outside_bounds_or_without_rings() {
        assert_eq!(kernel_shell(-0.1, &[1.0]), 0.0);
        assert_eq!(kernel_shell(1.0, &[1.0]), 0.0);
        assert_eq!(kernel_shell(0.5, &[]), 0.0);
    }

    #[test]
    fn kernel_shell_picks_the_matching_ring_weight() {
        // Two rings [0.5, 1.0]; radius 0.25 falls in ring 0, radius 0.75 in
        // ring 1, and both land on the ring-local midpoint where kernel_core
        // peaks at 1.0.
        let ring_weights = [0.5, 1.0];

        assert!(approx_eq(kernel_shell(0.25, &ring_weights), 0.5));
        assert!(approx_eq(kernel_shell(0.75, &ring_weights), 1.0));
    }

    #[test]
    fn generate_kernel_matrix_has_the_expected_shape() {
        let (kernel, side) = generate_kernel_matrix(3, &[1.0]);

        assert_eq!(side, 7);
        assert_eq!(kernel.len(), 49);
    }

    #[test]
    fn generate_kernel_matrix_normalizes_to_sum_one() {
        let (kernel, _) = generate_kernel_matrix(5, &[1.0, 0.5]);

        let total: f32 = kernel.iter().sum();
        assert!(approx_eq(total, 1.0));
    }

    #[test]
    fn generate_kernel_matrix_is_symmetric_across_both_axes() {
        let (kernel, side) = generate_kernel_matrix(4, &[1.0, 0.7, 0.3]);

        for y in 0..side {
            for x in 0..side {
                let mirrored_x = kernel[y * side + (side - 1 - x)];
                let mirrored_y = kernel[(side - 1 - y) * side + x];
                assert!(approx_eq(kernel[y * side + x], mirrored_x));
                assert!(approx_eq(kernel[y * side + x], mirrored_y));
            }
        }
    }
}
