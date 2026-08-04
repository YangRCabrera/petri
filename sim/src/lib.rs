//! Simulation core for a [Lenia](https://chakazul.github.io/lenia.html)
//! continuous cellular automaton, compiled to WASM for the `web` frontend.
//!
//! - [`kernel`] builds the convolution kernel from a set of ring weights.
//! - [`growth`] maps a cell's convolved potential to a rate of change.
//! - [`grid`] handles toroidal (wrap-around) grid coordinates.
//! - [`universe`] owns the grid state and exposes the `Universe` type the
//!   frontend drives via `wasm-bindgen`.
//!
//! The kernel, growth, and update-rule math ([`kernel`], [`growth`],
//! [`universe::Universe::tick`]) implement the formulas from Chan,
//! B. W.-C. (2019). Lenia: Biology of Artificial Life. _Complex Systems_,
//! 28(3), 251–286. [arXiv:1812.05433](https://arxiv.org/abs/1812.05433)

mod fft_convolution;
mod grid;
mod growth;
mod kernel;
mod universe;

pub use universe::Universe;
