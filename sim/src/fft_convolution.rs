//! FFT-based toroidal (circular) convolution of the Lenia kernel over the
//! grid — an `O(W·H·log(W·H))` replacement for the naive `O(W·H·K²)` spatial
//! convolution, independent of kernel size.
//!
//! Toroidal wrap-around is exactly circular convolution, which is what the
//! convolution theorem gives for free via FFT: no linear-convolution
//! zero-padding is needed, just placing the kernel in wrap-around form (see
//! [`FftConvolver::set_kernel_from_matrix`]).
//!
//! The 2D FFT itself is done via row/column decomposition: FFT every row,
//! transpose, FFT every row again (now operating on what were columns),
//! transpose back. Transposing between passes keeps both FFT passes
//! working on contiguous memory.

use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::grid::wrap_coordinate;
use crate::kernel::generate_kernel_matrix;

/// Runs `fft` over every `row_len`-sized chunk of `data` independently.
fn fft_rows(fft: &Arc<dyn Fft<f32>>, data: &mut [Complex<f32>], row_len: usize) {
    for row in data.chunks_mut(row_len) {
        fft.process(row);
    }
}

/// Transposes a `height`-rows-of-`width` matrix into a `width`-rows-of-`height`
/// matrix.
fn transpose(src: &[Complex<f32>], width: usize, height: usize, dst: &mut [Complex<f32>]) {
    for y in 0..height {
        for x in 0..width {
            dst[x * height + y] = src[y * width + x];
        }
    }
}

/// Convolves the Lenia kernel over a `width` x `height` toroidal grid via
/// FFT, caching both the FFT plans (fixed for the grid's lifetime) and the
/// kernel's own FFT (recomputed only when the kernel changes, not per tick).
pub(crate) struct FftConvolver {
    width: usize,
    height: usize,
    forward_row: Arc<dyn Fft<f32>>,
    inverse_row: Arc<dyn Fft<f32>>,
    forward_col: Arc<dyn Fft<f32>>,
    inverse_col: Arc<dyn Fft<f32>>,
    scratch_a: Vec<Complex<f32>>,
    scratch_b: Vec<Complex<f32>>,
    kernel_fft: Vec<Complex<f32>>,
}

impl FftConvolver {
    /// Builds a convolver for a `width` x `height` grid with a kernel of
    /// radius `kernel_radius` shaped by `ring_weights` (see
    /// [`generate_kernel_matrix`]).
    pub(crate) fn new(
        width: usize,
        height: usize,
        kernel_radius: usize,
        ring_weights: &[f32],
    ) -> Self {
        let mut planner = FftPlanner::new();
        let cell_count = width * height;

        let mut convolver = FftConvolver {
            width,
            height,
            forward_row: planner.plan_fft_forward(width),
            inverse_row: planner.plan_fft_inverse(width),
            forward_col: planner.plan_fft_forward(height),
            inverse_col: planner.plan_fft_inverse(height),
            scratch_a: vec![Complex::new(0.0, 0.0); cell_count],
            scratch_b: vec![Complex::new(0.0, 0.0); cell_count],
            kernel_fft: vec![Complex::new(0.0, 0.0); cell_count],
        };
        convolver.set_kernel(kernel_radius, ring_weights);
        convolver
    }

    /// Regenerates the kernel from `kernel_radius`/`ring_weights` and
    /// recomputes its cached FFT, applied starting next [`Self::convolve`].
    pub(crate) fn set_kernel(&mut self, kernel_radius: usize, ring_weights: &[f32]) {
        let (kernel, _) = generate_kernel_matrix(kernel_radius, ring_weights);
        self.set_kernel_from_matrix(&kernel, kernel_radius);
    }

    /// Places a raw `(2 * kernel_radius + 1)`-square kernel matrix into the
    /// grid in wrap-around form — offset `(dx, dy)` goes to grid position
    /// `(dx.rem_euclid(width), dy.rem_euclid(height))`, summed rather than
    /// overwritten so a kernel whose support exceeds the grid still wraps
    /// correctly — then transforms it and caches the result.
    fn set_kernel_from_matrix(&mut self, kernel: &[f32], kernel_radius: usize) {
        let kernel_size = 2 * kernel_radius + 1;
        let radius = kernel_radius as isize;

        self.scratch_a.fill(Complex::new(0.0, 0.0));

        for delta_y in -radius..=radius {
            for delta_x in -radius..=radius {
                let kernel_x = (delta_x + radius) as usize;
                let kernel_y = (delta_y + radius) as usize;
                let weight = kernel[kernel_y * kernel_size + kernel_x];

                let target_x = wrap_coordinate(delta_x, self.width);
                let target_y = wrap_coordinate(delta_y, self.height);

                self.scratch_a[target_y * self.width + target_x] += Complex::new(weight, 0.0);
            }
        }

        self.forward_2d();
        self.kernel_fft.copy_from_slice(&self.scratch_a);
    }

    /// Convolves `cell_states` with the cached kernel, writing each cell's
    /// resulting potential into `output`.
    pub(crate) fn convolve(&mut self, cell_states: &[f32], output: &mut [f32]) {
        for (c, &v) in self.scratch_a.iter_mut().zip(cell_states.iter()) {
            *c = Complex::new(v, 0.0);
        }

        self.forward_2d();

        for (c, &k) in self.scratch_a.iter_mut().zip(self.kernel_fft.iter()) {
            *c *= k;
        }

        self.inverse_2d();

        let normalization = 1.0 / (self.width * self.height) as f32;
        for (out, c) in output.iter_mut().zip(self.scratch_a.iter()) {
            *out = c.re * normalization;
        }
    }

    /// Forward 2D FFT of `scratch_a` in place (via `scratch_b`), leaving the
    /// result back in `scratch_a`.
    fn forward_2d(&mut self) {
        fft_rows(&self.forward_row, &mut self.scratch_a, self.width);
        transpose(
            &self.scratch_a,
            self.width,
            self.height,
            &mut self.scratch_b,
        );
        fft_rows(&self.forward_col, &mut self.scratch_b, self.height);
        transpose(
            &self.scratch_b,
            self.height,
            self.width,
            &mut self.scratch_a,
        );
    }

    /// Inverse 2D FFT of `scratch_a` in place (via `scratch_b`), leaving the
    /// unnormalized result back in `scratch_a`.
    fn inverse_2d(&mut self) {
        fft_rows(&self.inverse_row, &mut self.scratch_a, self.width);
        transpose(
            &self.scratch_a,
            self.width,
            self.height,
            &mut self.scratch_b,
        );
        fft_rows(&self.inverse_col, &mut self.scratch_b, self.height);
        transpose(
            &self.scratch_b,
            self.height,
            self.width,
            &mut self.scratch_a,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-4;

    fn approx_eq_slices(a: &[f32], b: &[f32]) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < EPSILON)
    }

    fn make_convolver(
        width: usize,
        height: usize,
        kernel_radius: usize,
        kernel: Vec<f32>,
    ) -> FftConvolver {
        let mut convolver = FftConvolver::new(width, height, 0, &[1.0]);
        convolver.set_kernel_from_matrix(&kernel, kernel_radius);
        convolver
    }

    /// Reference brute-force convolution — the same algorithm
    /// `Universe::compute_potential_grid` used before switching to FFT —
    /// kept here purely to cross-check `FftConvolver`'s output.
    #[allow(clippy::too_many_arguments)]
    fn naive_convolve(
        cell_states: &[f32],
        width: usize,
        height: usize,
        kernel: &[f32],
        kernel_size: usize,
        kernel_radius: usize,
    ) -> Vec<f32> {
        let radius_signed = kernel_radius as isize;
        let mut output = vec![0.0; width * height];

        for world_y in 0..height {
            for world_x in 0..width {
                let mut weighted_sum = 0f32;

                for delta_y in -radius_signed..=radius_signed {
                    for delta_x in -radius_signed..=radius_signed {
                        let target_x = wrap_coordinate((world_x as isize) - delta_x, width);
                        let target_y = wrap_coordinate((world_y as isize) - delta_y, height);

                        let kernel_x = (delta_x + radius_signed) as usize;
                        let kernel_y = (delta_y + radius_signed) as usize;

                        weighted_sum += kernel[kernel_y * kernel_size + kernel_x]
                            * cell_states[target_y * width + target_x];
                    }
                }

                output[world_y * width + world_x] = weighted_sum;
            }
        }

        output
    }

    #[test]
    fn identity_kernel_copies_state() {
        let mut kernel = vec![0.0; 9];
        kernel[4] = 1.0;
        let mut convolver = make_convolver(3, 3, 1, kernel);
        let cell_states = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        let mut output = vec![0.0; 9];

        convolver.convolve(&cell_states, &mut output);

        assert!(approx_eq_slices(&output, &cell_states));
    }

    #[test]
    fn wraps_around_edges() {
        // Kernel weighted only toward delta_x = -1 on its middle row, so
        // each output cell should pick up its left-wrapping neighbor's state.
        let kernel = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let mut convolver = make_convolver(3, 1, 1, kernel);
        let cell_states = vec![10.0, 20.0, 30.0];
        let mut output = vec![0.0; 3];

        convolver.convolve(&cell_states, &mut output);

        assert!(approx_eq_slices(&output, &[30.0, 10.0, 20.0]));
    }

    #[test]
    fn matches_naive_convolution_for_a_realistic_kernel() {
        let width = 8;
        let height = 8;
        let kernel_radius = 2;
        let (kernel, kernel_size) = generate_kernel_matrix(kernel_radius, &[1.0, 0.6, 0.3]);

        let cell_states: Vec<f32> = (0..width * height)
            .map(|i| ((i as f32) * 0.037).sin().abs())
            .collect();

        let mut convolver = make_convolver(width, height, kernel_radius, kernel.clone());
        let mut fft_output = vec![0.0; width * height];
        convolver.convolve(&cell_states, &mut fft_output);

        let naive_output = naive_convolve(
            &cell_states,
            width,
            height,
            &kernel,
            kernel_size,
            kernel_radius,
        );

        assert!(approx_eq_slices(&fft_output, &naive_output));
    }
}
