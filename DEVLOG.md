# Dev Log

<!--
Organized by feature/theme, not by day. Each section is a feature-sized
chunk of work — open a new one when that chunk lands, not on a fixed
daily cadence and not per commit. Small cosmetic or copy-only patches
don't get their own section; log them as a single line under Misc
Patches instead.

Write a section the same session the work lands, from what's fresh, not
reconstructed from memory later. Don't self-edit for polish — a wrong
turn documented honestly (see: the E2E-in-CI entry from a past project)
is worth more than a version that always gets it right the first time.
Where relevant, include: what shipped, the decision and the reasoning
behind it — not just the outcome, "chose X over Y because Z" beats
"added X" — what broke, what got learned, and where/how AI tools were
or weren't involved.

Update the Contents list below whenever a section is added — that's
what keeps this navigable once it's more than a handful of entries.
-->

## Contents

- [Project scaffold](#project-scaffold)
- [Workspace + WASM build pipeline](#workspace--wasm-build-pipeline)
- [CI + Vercel deploy config](#ci--vercel-deploy-config)
- [Lenia simulation math](#lenia-simulation-math)
- [Render loop + blob seeding](#render-loop--blob-seeding)
- [Misc Patches](#misc-patches)

> **AI usage note:** every step across the three sections below happened
> in one sitting, executed by Claude Code (CC) on direct human
> instruction. The _shape_ of the work — the Rust→WASM→TS pipeline, the
> `sim` crate's `cdylib`/`rlib` split, the workspace + nodemon
> orchestration, and the lazy WASM loader exposing raw memory — follows
> an earlier Conway's Game of Life prototype built by hand, not
> something CC designed from scratch. README.md, CLAUDE.md, the skill
> docs, CHANGELOG.md, and this DEVLOG were likewise built on
> pre-existing personal templates, populated by CC. The one exception is
> CI + Vercel deploy config: no prior template existed for that half, so
> it's CC's own design, built from scratch this session.

## Project scaffold

Set up the repo layout for the three toolchains ahead of any real
simulation logic: `sim/` (Rust lib crate) and `web/` (Vite + vanilla TS).
A Cloudflare Workers backend for sharing configs comes later, once
there's something worth sharing.

- `sim/`: `cargo new --lib sim`, then set `crate-type = ["cdylib",
"rlib"]` in `[lib]` and added `wasm-bindgen`. `rlib` stays alongside
  `cdylib` so `cargo test` can still run natively — a `cdylib`-only
  crate can't be unit tested without a full wasm runtime. Builds clean
  with `cargo build --target wasm32-unknown-unknown`. `cargo new`
  skipped generating a `.gitignore` here (it saw the repo root's
  existing git repo), so added `sim/.gitignore` (`/target`) by hand.
- `web/`: scaffolded via `npm create vite@latest web -- --template
vanilla-ts` — no framework, since this is fundamentally a canvas app
  with a parameter panel, not a component tree. `tsc --noEmit` and
  `npm run build` both pass on the untouched template.
- Filled in README/CLAUDE.md's Commands tables and the `verify` skill
  with real per-toolchain commands now that manifests exist. Backend
  rows left as explicit "not scaffolded yet" rather than guessing at
  wrangler commands ahead of time.
- No linter or unit test framework installed for `web/` yet (the
  vanilla Vite template ships neither) — noted as a gap in `verify`
  rather than silently skipped.

## Workspace + WASM build pipeline

Wired `sim/` and `web/` together, following the same shape as an earlier
Conway's Game of Life prototype (workspace root, nodemon watching the
Rust source, lazy WASM loader exposing raw memory) rather than
re-deriving it from scratch.

- Converted the repo root into an npm workspace (`workspaces: ["web"]`)
  with orchestration scripts: `build:wasm` (`wasm-pack build sim
--target web --out-dir ../web/src/wasm`), `watch:wasm` (nodemon,
  watches `sim/src` + `sim/Cargo.toml`, 300ms debounce), `dev` (build
  once, then run the watcher and `web`'s Vite dev server together via
  `concurrently`), and `build` (build once, then `web`'s production
  build). `concurrently` and `nodemon` added as root devDependencies.
  Crate stays named `sim`, not renamed to match the prototype's `core`.
- Added `web/src/wasm-loader.ts`: a lazily-initialized, promise-cached
  loader that returns both the generated bindings and the raw
  `WebAssembly.Memory` export. Exposing memory directly (rather than
  serializing frame data across the JS/WASM boundary) is what makes the
  prototype's `mem::swap`-based rendering approach possible on the JS
  side. `web/src/wasm/` (the `wasm-pack` output) is gitignored,
  generated, and never hand-edited.
- `web/package-lock.json` was replaced by a single root-level
  `package-lock.json` now that `web` is a workspace member — npm hoists
  shared deps to the root `node_modules`.
- Verified end to end: `npm run build:wasm` produces
  `web/src/wasm/sim.{js,d.ts}` + `sim_bg.wasm`; `tsc -p web --noEmit`
  passes against the real generated output; `npm run build` (root)
  succeeds; and a bounded smoke test of `npm run dev` confirmed nodemon
  does its initial `build:wasm` pass and `concurrently` brings up the
  Vite dev server alongside it, with clean shutdown on SIGTERM.

## CI + Vercel deploy config

First CI setup for a Rust-in-CI project — no prior template to lean on
for that half, so it was built from scratch alongside the more familiar
Node-side CI.

- Added `oxlint` to `web` (a real `npm run lint` script) so CI has an
  actual lint step, not just typecheck — closes the gap noted in the
  project-scaffold entry. Same tool the prototype used.
- `.github/workflows/ci.yml`: two jobs. `sim` runs `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, a
  `wasm32-unknown-unknown` build, and `cargo test`
  (`dtolnay/rust-toolchain` + `Swatinem/rust-cache` for the toolchain
  and caching). `web` installs Rust + `wasm-pack`
  (`jetli/wasm-pack-action`) first, runs `npm run build:wasm`, then
  lints and typechecks/builds `web` — it has to build the wasm bindings
  before typecheck can even resolve `wasm-loader.ts`'s import, same
  dependency order as the local `npm run build` pipeline.
- Vercel: `vercel.json` (`buildCommand`, `outputDirectory: web/dist`,
  `installCommand`) plus `scripts/vercel-build.sh`, since Vercel's build
  image has no Rust toolchain by default — the script installs `rustup`
  (minimal profile) + `wasm-pack` if they're not already present, then
  calls the normal `npm run build`. Ran `vercel link --yes` (already
  logged in via CLI) to create and link the
  `yangthepersons-projects/petri` project — the CLI added `.vercel` and
  `.env*` to `.gitignore` on its own.
- Pushed the repo to GitHub, made it public, and ran a real `vercel
deploy` this session instead of leaving it for a separate deploy pass
  as originally planned. Both came back clean on the first try: CI
  green on the actual Actions run (not just the by-hand local checks
  the workflow was written against), and the Vercel build successfully
  bootstrapped `rustup` + `wasm-pack` from scratch on its Rust-less
  image. Live at https://petri-one.vercel.app/.

## Lenia simulation math

Filled in `sim` with the actual Lenia rules: a
convolution kernel, a growth mapping, toroidal grid coordinates, and the
`Universe` type tying them into a per-tick update loop.

Unlike the three sections above, the math here wasn't CC's to design. Every function is human-written, derived from Chan, B. W.-C. (2019). _Lenia: Biology of Artificial Life_, _Complex Systems_, 28(3), 251–286 ([arXiv:1812.05433](https://arxiv.org/abs/1812.05433)), now cited in `sim/src/lib.rs`'s crate doc comment and in README's References. CC's part was refining the file layout (the `kernel`/`growth`/`grid`/ `universe` split) and, more importantly, the commenting: the math itself isn't intuitive to me, so function/variable names (`kernel_core`, `growth_target`, `wrap_coordinate`, ...) and doc comments were pushed to be as explicit as possible, otherwise I genuinely can't hold the shape of what each piece is doing. Verbosity here is deliberate, not an AI-ism to trim later.

- `kernel.rs`: `kernel_core` is Lenia's bell-shaped bump
  `exp(4 - 1/(r(1-r)))`, zero outside `[0, 1)`. `kernel_shell` layers
  multiple rings on top of it by dividing `[0, 1)` into `ring_weights.len()`
  bands and re-running `kernel_core` within whichever band a given radius
  falls into. `generate_kernel_matrix` rasterizes that continuous
  function onto a `(2r+1) x (2r+1)` grid and normalizes it to sum to 1,
  so convolving it doesn't push cell states out of range.
- `growth.rs`: `compute_growth_rate` is the other half of the rule — a
  Gaussian centered on `growth_target`, rescaled from `[0, 1]` to
  `[-1, 1]` so a cell's potential lands it somewhere between full decay
  and full growth depending on distance from the target band.
- `grid.rs`: `wrap_coordinate` (`rem_euclid`) gives the grid toroidal
  (wrap-around) edges instead of hard boundaries, used on both axes
  during convolution.
- `universe.rs`: `Universe` owns `cell_states` and `buffer_cell_states`
  and reuses the `mem::swap` pattern from the WASM pipeline entry rather
  than reallocating a grid-sized buffer every tick —
  `compute_potential_grid` convolves the kernel and writes potentials
  into the buffer, `apply_growth` overwrites the buffer in place with
  the next generation (clamped to `[0, 1]`), and `swap_buffer` makes it
  current. A separate `colors: Vec<Rgba>` is recomputed from state each
  tick and exposed read-only via `get_ptr`, the same raw-memory-pointer
  approach `wasm-loader.ts` already expects.
- Added `#[cfg(test)]` coverage in all four modules — kernel core/shell
  shape and normalization, growth peak/symmetry/decay, coordinate
  wrapping, and the `Universe` loop (buffer swap, color mapping,
  wrap-around convolution, growth clamping). First real use of the
  `cargo test` CI step; before this it was only running the crate's
  default scaffold test.

## Render loop + blob seeding

Wired the WASM `Universe` up to an actual `<canvas>`: `main.ts` now
creates and seeds a `Universe` in place of the Vite starter template,
and a new `render.ts` owns everything that happens per frame.

- `render.ts`: `setupCanvas` sizes `#sim-canvas` to the grid;
  `setupUniverseLoop` ticks the `Universe` each frame and blits its
  color buffer straight out of WASM linear memory into `ImageData`,
  reusing the same avoid-serialization approach as `wasm-loader.ts`;
  `setupLifetime` wraps `requestAnimationFrame` with FPS throttling (a
  frame-interval accumulator, not a fixed `setTimeout`) plus
  pause/resume. `main.ts` wires the three together, seeds via
  `add_comet_blob`, and binds Enter to pause/resume. `#sim-canvas` gets
  `image-rendering: crisp-edges` so the low-res grid doesn't blur when
  scaled up.
- `setupLifetime`'s `stop()` and `main.ts`'s `keyup` listener are
  scaffolding, not finished features: `stop()` isn't called anywhere
  yet (nothing tears the loop down), and the Enter-to-pause binding is
  a placeholder for real playback controls, not the intended UI.
- `Universe::add_comet_blob` (an offset, asymmetric blob with a short
  nose and a long tail) is what's seeded now, but it wasn't the first
  thing tried. The first seed was radially symmetric and it collapsed
  in on itself into static, tiled geometric patterns covering the
  whole canvas instead of producing a moving glider. Worried
  initially that this meant the kernel/growth math was wrong,
  but re-reading section 3.6 of the Lenia paper clarified that's an
  expected failure mode for symmetric seeds: gliders like Orbium
  need an asymmetric seed to break the symmetry and establish a
  heading, which is exactly why `add_comet_blob` replaced it.
- Chrome's console is flagging `[Violation] 'requestAnimationFrame'
handler took <N>ms` — the per-tick convolution is already expensive
  enough to miss frame budget at 60fps. Known, deferred: not worth
  optimizing before the UI has real controls (start/stop, parameter
  inputs) to make the sim worth interacting with in the first place.

## Misc Patches

<!-- One line each, newest first: small, cosmetic, no-schema-change
     updates that don't warrant a section of their own. -->
