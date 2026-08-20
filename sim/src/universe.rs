//! The simulation grid itself: state storage, the per-tick update loop, and
//! the WASM-facing API the frontend drives it through.

use wasm_bindgen::prelude::*;

use crate::fft_convolution::FftConvolver;
use crate::growth::*;
use crate::rle;

/// Packed RGBA pixel. `#[repr(C)]` pins the field layout so `Universe::get_ptr`
/// can be read directly out of WASM linear memory as a byte buffer (e.g. into
/// a canvas `ImageData`) without any conversion on the JS side.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rgba(u8, u8, u8, u8);

/// Which growth mapping [`Universe::apply_growth`] uses each tick — the
/// three variants Lenia's `gn` selects between (see [`crate::growth`]).
/// Numeric values match Chan's own `gn` convention (`growth_func[gn - 1]`
/// in both `LeniaF.py` and `LeniaND.py`) so a value read straight off an
/// imported pattern's `gn` field selects the right variant.
#[repr(C)]
#[wasm_bindgen]
pub enum GrowthFunction {
    Polynomial = 1,
    Gaussian = 2,
    Step = 3,
}

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
    growth_function: GrowthFunction,
}

#[wasm_bindgen]
impl Universe {
    /// Builds a `width` x `height` toroidal grid, all cells dead, with a
    /// kernel of radius `kernel_radius` shaped by `ring_weights` (see
    /// [`crate::kernel::generate_kernel_matrix`]), and the growth mapping
    /// parameters used each [`Self::tick`].
    #[allow(clippy::too_many_arguments)] // wasm-bindgen constructors can only take flat primitive args, not a params struct
    pub fn new(
        width: usize,
        height: usize,
        kernel_radius: usize,
        ring_weights: &[f32],
        growth_target: f32,
        growth_width: f32,
        time_step: f32,
        growth_function: GrowthFunction,
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
            growth_function,
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

    /// Sets which growth mapping [`Self::tick`] evaluates the potential
    /// through, applied starting next [`Self::tick`].
    pub fn set_growth_function(&mut self, growth_function: GrowthFunction) {
        self.growth_function = growth_function;
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

    /// Clears the grid and places a rectangular pattern of continuous cell
    /// values (row-major, `pattern_width * pattern_height` long), centered
    /// on the grid and wrapping at the edges like any other placement on
    /// this toroidal grid.
    pub fn load_pattern(&mut self, pattern_width: usize, pattern_height: usize, cells: &[f32]) {
        self.place_pattern(pattern_width, pattern_height, cells);
    }

    /// Decodes an RLE-encoded pattern (Golly-style run-length encoding,
    /// extended with multi-character codes for continuous cell values — see
    /// [`crate::rle`]) and places it the same way [`Self::load_pattern`]
    /// does. Errors if `rle` decodes to no cells (empty input, or nothing
    /// recognizable as pattern data).
    pub fn load_rle(&mut self, rle: &str) -> Result<(), JsValue> {
        self.try_load_rle(rle).map_err(JsValue::from_str)
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
    /// Clears the grid and places a rectangular pattern of continuous cell
    /// values (row-major, `pattern_width * pattern_height` long), centered
    /// on the grid and wrapping at the edges like any other placement on
    /// this toroidal grid. Shared by [`Self::load_pattern`] and
    /// [`Self::load_rle`].
    fn place_pattern(&mut self, pattern_width: usize, pattern_height: usize, cells: &[f32]) {
        self.cell_states.fill(0.0);

        let offset_x = (self.width / 2).saturating_sub(pattern_width / 2);
        let offset_y = (self.height / 2).saturating_sub(pattern_height / 2);

        for y in 0..pattern_height {
            for x in 0..pattern_width {
                let grid_x = (offset_x + x) % self.width;
                let grid_y = (offset_y + y) % self.height;
                let index = grid_y * self.width + grid_x;
                self.cell_states[index] = cells[y * pattern_width + x];
            }
        }

        self.update_colors();
    }

    /// Does the actual decode-validate-place work behind [`Self::load_rle`],
    /// kept free of `JsValue` (unusable outside a `wasm-bindgen` host, e.g.
    /// in native `cargo test`) so it stays directly testable.
    fn try_load_rle(&mut self, rle: &str) -> Result<(), &'static str> {
        let (pattern_width, pattern_height, cells) = rle::decode(rle);

        if pattern_width == 0 || pattern_height == 0 {
            return Err("Pattern is empty or could not be parsed.");
        }

        self.place_pattern(pattern_width, pattern_height, &cells);
        Ok(())
    }

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

            let growth_rate = match self.growth_function {
                GrowthFunction::Gaussian => {
                    compute_gaussian_growth_rate(potential, growth_target, growth_width)
                }
                GrowthFunction::Polynomial => {
                    compute_polynomial_growth_rate(potential, growth_target, growth_width)
                }
                GrowthFunction::Step => {
                    compute_step_growth_rate(potential, growth_target, growth_width)
                }
            };

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
        Universe::new(
            width,
            height,
            0,
            &[1.0],
            0.15,
            0.015,
            0.1,
            GrowthFunction::Gaussian,
        )
    }

    #[test]
    fn new_seeds_colors_for_a_dead_grid() {
        let universe = Universe::new(2, 2, 1, &[1.0], 0.15, 0.015, 0.1, GrowthFunction::Gaussian);

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
    fn load_pattern_centers_and_clears_existing_state() {
        let mut universe = make_universe(6, 6);
        universe.cell_states[0] = 1.0; // pre-existing state that should be cleared

        universe.load_pattern(2, 2, &[0.5, 0.6, 0.7, 0.8]);

        assert_eq!(universe.cell_states[2 * 6 + 2], 0.5);
        assert_eq!(universe.cell_states[2 * 6 + 3], 0.6);
        assert_eq!(universe.cell_states[3 * 6 + 2], 0.7);
        assert_eq!(universe.cell_states[3 * 6 + 3], 0.8);
        assert_eq!(universe.cell_states[0], 0.0);
    }

    #[test]
    fn load_rle_decodes_centers_and_clears_existing_state() {
        let mut universe = make_universe(6, 6);
        universe.cell_states[0] = 1.0; // pre-existing state that should be cleared

        // 2x2: row0 "AB" (states 1,2 -> 1/255, 2/255), row1 "o." (1, then dead)
        universe.try_load_rle("AB$o!").unwrap();

        assert_eq!(universe.cell_states[2 * 6 + 2], 1.0 / 255.0);
        assert_eq!(universe.cell_states[2 * 6 + 3], 2.0 / 255.0);
        assert_eq!(universe.cell_states[3 * 6 + 2], 1.0);
        assert_eq!(universe.cell_states[3 * 6 + 3], 0.0);
        assert_eq!(universe.cell_states[0], 0.0);
    }

    #[test]
    fn load_rle_of_unparseable_input_errors_instead_of_clearing_the_grid() {
        let mut universe = make_universe(4, 4);
        universe.cell_states[0] = 1.0;

        assert!(universe.try_load_rle("").is_err());
        assert_eq!(universe.cell_states[0], 1.0);
    }

    #[test]
    fn tick_keeps_state_and_colors_within_valid_ranges() {
        let mut universe =
            Universe::new(4, 4, 1, &[1.0], 0.15, 0.015, 0.1, GrowthFunction::Gaussian);
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
