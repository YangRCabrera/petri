//! The simulation grid itself: state storage, the per-tick update loop, and
//! the WASM-facing API the frontend drives it through.

use wasm_bindgen::prelude::*;

use crate::grid::wrap_coordinate;
use crate::growth::compute_growth_rate;
use crate::kernel::generate_kernel_matrix;

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
    kernel: Vec<f32>,
    kernel_radius: usize,
}

#[wasm_bindgen]
impl Universe {
    /// Builds a `width` x `height` toroidal grid, all cells dead, with a
    /// kernel of radius `kernel_radius` shaped by `ring_weights` (see
    /// [`generate_kernel_matrix`]).
    pub fn new(
        width: usize,
        height: usize,
        kernel_radius: usize,
        ring_weights: &[f32],
    ) -> Universe {
        let cell_states = vec![0.0; width * height];
        let buffer_cell_states = vec![0.0; width * height];
        let colors = vec![Rgba(0u8, 0u8, 0u8, 0u8); width * height];

        let (kernel, _) = generate_kernel_matrix(kernel_radius, ring_weights);

        let mut universe = Universe {
            cell_states,
            buffer_cell_states,
            colors,
            width,
            height,
            kernel,
            kernel_radius,
        };
        universe.update_colors();
        universe
    }

    /// Advances the simulation by one generation: convolve the kernel over
    /// every cell to get its potential, map that through the growth
    /// function, then swap the resulting generation into place.
    pub fn tick(&mut self) {
        let growth_target = 0.15;
        let growth_width = 0.015;
        let time_step = 0.1;

        self.compute_potential_grid();

        self.apply_growth(growth_target, growth_width, time_step);

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
    /// `buffer_cell_states`.
    fn compute_potential_grid(&mut self) {
        let kernel_size = 2 * self.kernel_radius + 1;
        let radius_signed = self.kernel_radius as isize;

        for world_y in 0..self.height {
            for world_x in 0..self.width {
                let mut weighted_sum = 0f32;

                for delta_y in -radius_signed..=radius_signed {
                    for delta_x in -radius_signed..=radius_signed {
                        let target_x = wrap_coordinate((world_x as isize) - delta_x, self.width);
                        let target_y = wrap_coordinate((world_y as isize) - delta_y, self.height);

                        let kernel_x = (delta_x + radius_signed) as usize;
                        let kernel_y = (delta_y + radius_signed) as usize;

                        let cell_value = self.cell_states[target_y * self.width + target_x];
                        let kernel_weight = self.kernel[kernel_y * kernel_size + kernel_x];

                        weighted_sum += kernel_weight * cell_value;
                    }
                }

                self.buffer_cell_states[world_y * self.width + world_x] = weighted_sum;
            }
        }
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

    fn make_universe(
        width: usize,
        height: usize,
        kernel_radius: usize,
        kernel: Vec<f32>,
    ) -> Universe {
        let cell_count = width * height;
        Universe {
            cell_states: vec![0.0; cell_count],
            buffer_cell_states: vec![0.0; cell_count],
            colors: vec![Rgba(0, 0, 0, 0); cell_count],
            width,
            height,
            kernel,
            kernel_radius,
        }
    }

    #[test]
    fn new_seeds_colors_for_a_dead_grid() {
        let universe = Universe::new(2, 2, 1, &[1.0]);

        for color in &universe.colors {
            assert_eq!((color.0, color.1, color.2, color.3), (0, 0, 0, 255));
        }
    }

    #[test]
    fn swap_buffer_swaps_cell_and_buffer_states() {
        let mut universe = make_universe(2, 1, 0, vec![0.0]);
        universe.cell_states = vec![1.0, 2.0];
        universe.buffer_cell_states = vec![3.0, 4.0];

        universe.swap_buffer();

        assert_eq!(universe.cell_states, vec![3.0, 4.0]);
        assert_eq!(universe.buffer_cell_states, vec![1.0, 2.0]);
    }

    #[test]
    fn update_colors_maps_state_to_expected_channels() {
        let mut universe = make_universe(2, 1, 0, vec![0.0]);
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
        let mut universe = make_universe(1, 1, 0, vec![0.0]);
        universe.cell_states = vec![0.99];
        universe.buffer_cell_states = vec![0.15]; // potential sitting exactly at the growth target

        universe.apply_growth(0.15, 0.015, 1.0);

        assert_eq!(universe.buffer_cell_states[0], 1.0);
    }

    #[test]
    fn compute_potential_grid_identity_kernel_copies_state() {
        // 3x3 kernel with weight 1 only at its center: each cell's potential
        // should just be that cell's own current state.
        let mut kernel = vec![0.0; 9];
        kernel[4] = 1.0;
        let mut universe = make_universe(3, 3, 1, kernel);
        universe.cell_states = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        universe.compute_potential_grid();

        assert_eq!(universe.buffer_cell_states, universe.cell_states);
    }

    #[test]
    fn compute_potential_grid_wraps_around_edges() {
        // Kernel weighted only toward delta_x = -1 on its middle row, so
        // each output cell should pick up its left-wrapping neighbor's state.
        let kernel = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        let mut universe = make_universe(3, 1, 1, kernel);
        universe.cell_states = vec![10.0, 20.0, 30.0];

        universe.compute_potential_grid();

        assert_eq!(universe.buffer_cell_states, vec![30.0, 10.0, 20.0]);
    }

    #[test]
    fn tick_keeps_state_and_colors_within_valid_ranges() {
        let mut universe = Universe::new(4, 4, 1, &[1.0]);
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
