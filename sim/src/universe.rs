//! The simulation grid itself: state storage, the per-tick update loop, and
//! the WASM-facing API the frontend drives it through.

use wasm_bindgen::prelude::*;

use crate::fft_convolution::FftConvolver;
use crate::growth::compute_growth_rate;

/// Packed RGBA pixel. `#[repr(C)]` pins the field layout so `Universe::get_ptr`
/// can be read directly out of WASM linear memory as a byte buffer (e.g. into
/// a canvas `ImageData`) without any conversion on the JS side.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rgba(u8, u8, u8, u8);

/// A Lenia grid: cell states plus the precomputed kernel convolved over them
/// each tick.
///
/// `cell_states` and `buffer_cell_states` are swapped rather than cleared
/// each generation: [`Universe::compute_potential_grid`] reads the former and
/// writes potentials into the latter, [`Universe::apply_growth`] then
/// overwrites the latter in place with the next generation, and
/// [`Universe::swap_buffer`] makes it current. This avoids re-allocating a
/// grid-sized buffer every tick.
#[wasm_bindgen]
pub struct Universe {
    cell_states: Vec<f32>,
    buffer_cell_states: Vec<f32>,
    colors: Vec<Rgba>,
    width: usize,
    height: usize,
    fft_convolver: FftConvolver,
    kernel_radius: usize,
    ring_weights: Vec<f32>,
    growth_target: f32,
    growth_width: f32,
    time_step: f32,
}

#[wasm_bindgen]
impl Universe {
    /// Builds a `width` x `height` toroidal grid, all cells dead, with a
    /// kernel of radius `kernel_radius` shaped by `ring_weights` (see
    /// [`crate::kernel::generate_kernel_matrix`]), and the growth mapping
    /// parameters used each [`Self::tick`].
    pub fn new(
        width: usize,
        height: usize,
        kernel_radius: usize,
        ring_weights: &[f32],
        growth_target: f32,
        growth_width: f32,
        time_step: f32,
    ) -> Universe {
        let cell_states = vec![0.0; width * height];
        let buffer_cell_states = vec![0.0; width * height];
        let colors = vec![Rgba(0u8, 0u8, 0u8, 0u8); width * height];

        let fft_convolver = FftConvolver::new(width, height, kernel_radius, ring_weights);

        let mut universe = Universe {
            cell_states,
            buffer_cell_states,
            colors,
            width,
            height,
            fft_convolver,
            kernel_radius,
            ring_weights: ring_weights.to_vec(),
            growth_target,
            growth_width,
            time_step,
        };
        universe.update_colors();
        universe
    }

    /// Sets the growth mapping's target potential (peak of the growth
    /// curve), applied starting next [`Self::tick`].
    pub fn set_growth_target(&mut self, growth_target: f32) {
        self.growth_target = growth_target;
    }

    /// Sets the growth mapping's width (spread of the growth curve around
    /// its target), applied starting next [`Self::tick`].
    pub fn set_growth_width(&mut self, growth_width: f32) {
        self.growth_width = growth_width;
    }

    /// Sets the per-tick integration step, applied starting next
    /// [`Self::tick`].
    pub fn set_time_step(&mut self, time_step: f32) {
        self.time_step = time_step;
    }

    /// Sets the kernel's radius (in cells) and regenerates it from the
    /// current ring weights, applied starting next [`Self::tick`].
    pub fn set_kernel_radius(&mut self, kernel_radius: usize) {
        self.kernel_radius = kernel_radius;
        self.fft_convolver
            .set_kernel(self.kernel_radius, &self.ring_weights);
    }

    /// Sets the kernel's ring weights and regenerates it at the current
    /// radius, applied starting next [`Self::tick`].
    pub fn set_ring_weights(&mut self, ring_weights: &[f32]) {
        self.ring_weights = ring_weights.to_vec();
        self.fft_convolver
            .set_kernel(self.kernel_radius, &self.ring_weights);
    }

    /// Plops a comet-shaped blob: an offset, elongated core with a short
    /// nose and a long tail trailing behind it, facing `angle_radians`
    /// (0 = +x, increasing counter-clockwise).
    pub fn add_comet_blob(&mut self, radius: usize, angle_radians: f32) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let r_float = radius as f32;

        let dir_x = angle_radians.cos();
        let dir_y = angle_radians.sin();

        // Peak sits ahead of center, toward the nose.
        let peak_x = center_x + dir_x * r_float * 0.3;
        let peak_y = center_y + dir_y * r_float * 0.3;

        let nose_length = r_float * 0.8;
        let tail_length = r_float * 1.6;
        let lateral_radius = r_float * 0.9;

        for y in 0..self.height {
            for x in 0..self.width {
                let dx = (x as f32) - peak_x;
                let dy = (y as f32) - peak_y;

                let forward = dx * dir_x + dy * dir_y;
                let lateral = -dx * dir_y + dy * dir_x;

                let forward_extent = if forward >= 0.0 {
                    nose_length
                } else {
                    tail_length
                };

                let normalized_forward = forward / forward_extent;
                let normalized_lateral = lateral / lateral_radius;
                let normalized_dist_sq = normalized_forward * normalized_forward
                    + normalized_lateral * normalized_lateral;

                if normalized_dist_sq <= 1.0 {
                    let height = f32::exp(-4.0 * normalized_dist_sq);
                    let index = y * self.width + x;
                    self.cell_states[index] = height.clamp(0.0, 1.0);
                }
            }
        }

        self.update_colors();
    }

    /// Advances the simulation by one generation: convolve the kernel over
    /// every cell to get its potential, map that through the growth
    /// function, then swap the resulting generation into place.
    pub fn tick(&mut self) {
        self.compute_potential_grid();

        self.apply_growth(self.growth_target, self.growth_width, self.time_step);

        self.swap_buffer();

        self.update_colors();
    }

    /// Raw pointer to the Rgba color buffer, for the JS side to read
    /// directly out of WASM linear memory instead of paying to serialize it
    /// across the boundary every frame.
    pub fn get_ptr(&self) -> *const Rgba {
        self.colors.as_ptr()
    }
}

impl Universe {
    fn swap_buffer(&mut self) {
        std::mem::swap(&mut self.cell_states, &mut self.buffer_cell_states);
    }

    /// Maps continuous float states [0.0, 1.0] to visual RGBA pixels.
    fn update_colors(&mut self) {
        for (i, &state) in self.cell_states.iter().enumerate() {
            let red = (state * state * 255.0) as u8;
            let green = (state * 220.0) as u8;
            let blue = ((state.sqrt()) * 255.0) as u8;
            let alpha = 255;

            self.colors[i] = Rgba(red, green, blue, alpha);
        }
    }

    /// Convolves the kernel over every cell, wrapping at the grid edges
    /// (toroidal boundary), and stores each cell's resulting potential into
    /// `buffer_cell_states`. See [`FftConvolver`] for the actual math.
    fn compute_potential_grid(&mut self) {
        self.fft_convolver
            .convolve(&self.cell_states, &mut self.buffer_cell_states);
    }

    /// Maps each cell's potential (currently sitting in
    /// `buffer_cell_states` from [`Self::compute_potential_grid`]) through
    /// the growth function and integrates it into the next state, clamped
    /// to the valid `[0, 1]` cell-state range.
    fn apply_growth(&mut self, growth_target: f32, growth_width: f32, time_step: f32) {
        let total_cells = self.width * self.height;

        for i in 0..total_cells {
            let old_state = self.cell_states[i];
            let potential = self.buffer_cell_states[i];

            let growth_rate = compute_growth_rate(potential, growth_target, growth_width);

            let new_state = old_state + time_step * growth_rate;

            self.buffer_cell_states[i] = new_state.clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a `Universe` for tests that don't care about the convolution
    /// kernel's shape (swap/color/growth tests) — see `fft_convolution`'s
    /// own test module for kernel/convolution correctness coverage.
    fn make_universe(width: usize, height: usize) -> Universe {
        Universe::new(width, height, 0, &[1.0], 0.15, 0.015, 0.1)
    }

    #[test]
    fn new_seeds_colors_for_a_dead_grid() {
        let universe = Universe::new(2, 2, 1, &[1.0], 0.15, 0.015, 0.1);

        for color in &universe.colors {
            assert_eq!((color.0, color.1, color.2, color.3), (0, 0, 0, 255));
        }
    }

    #[test]
    fn swap_buffer_swaps_cell_and_buffer_states() {
        let mut universe = make_universe(2, 1);
        universe.cell_states = vec![1.0, 2.0];
        universe.buffer_cell_states = vec![3.0, 4.0];

        universe.swap_buffer();

        assert_eq!(universe.cell_states, vec![3.0, 4.0]);
        assert_eq!(universe.buffer_cell_states, vec![1.0, 2.0]);
    }

    #[test]
    fn update_colors_maps_state_to_expected_channels() {
        let mut universe = make_universe(2, 1);
        universe.cell_states = vec![0.0, 1.0];

        universe.update_colors();

        assert_eq!(
            (
                universe.colors[0].0,
                universe.colors[0].1,
                universe.colors[0].2,
                universe.colors[0].3
            ),
            (0, 0, 0, 255)
        );
        assert_eq!(
            (
                universe.colors[1].0,
                universe.colors[1].1,
                universe.colors[1].2,
                universe.colors[1].3
            ),
            (255, 220, 255, 255)
        );
    }

    #[test]
    fn apply_growth_clamps_result_to_valid_range() {
        let mut universe = make_universe(1, 1);
        universe.cell_states = vec![0.99];
        universe.buffer_cell_states = vec![0.15]; // potential sitting exactly at the growth target

        universe.apply_growth(0.15, 0.015, 1.0);

        assert_eq!(universe.buffer_cell_states[0], 1.0);
    }

    #[test]
    fn tick_keeps_state_and_colors_within_valid_ranges() {
        let mut universe = Universe::new(4, 4, 1, &[1.0], 0.15, 0.015, 0.1);
        universe.cell_states[5] = 0.8;

        universe.tick();

        assert_eq!(universe.cell_states.len(), 16);
        assert!(
            universe
                .cell_states
                .iter()
                .all(|&s| (0.0..=1.0).contains(&s))
        );
        assert!(universe.colors.iter().all(|color| color.3 == 255));
    }
}
