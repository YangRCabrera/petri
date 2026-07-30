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
- [Misc Patches](#misc-patches)

## Project scaffold

Set up the repo layout for the three toolchains ahead of any real
simulation logic: `sim/` (Rust lib crate) and `web/` (Vite + vanilla TS).
A Cloudflare Workers backend for sharing configs comes later, once
there's something worth sharing.

- `sim/`: `cargo new --lib sim`, then set `crate-type = ["cdylib",
  "rlib"]` in `[lib]` and added `wasm-bindgen` as a dependency. `rlib` is
  kept alongside `cdylib` so `cargo test` can still run natively — a
  `cdylib`-only crate can't be unit tested without a full wasm runtime.
  Confirmed it builds clean with `cargo build --target
  wasm32-unknown-unknown` (the `wasm32-unknown-unknown` target was
  already installed via rustup). `cargo new` didn't generate a
  `.gitignore` here since it detected the repo root's existing git repo,
  so added `sim/.gitignore` (`/target`) by hand.
- `web/`: scaffolded via `npm create vite@latest web -- --template
  vanilla-ts` — no framework, since this is fundamentally a canvas app
  with a parameter panel, not a component tree. Confirmed `tsc --noEmit`
  and `npm run build` both pass on the untouched template.
- Filled in README/CLAUDE.md's Commands tables and the `verify` skill
  with the real per-toolchain commands now that manifests exist. Left
  the backend rows as explicit "not scaffolded yet" rather than guessing
  at wrangler commands ahead of time.
- No linter or unit test framework installed for `web/` yet (the vanilla
  Vite template doesn't include either) — noted as a gap in `verify`
  rather than silently skipped.

## Workspace + WASM build pipeline

Wired `sim/` and `web/` together, following the same shape as an earlier
Conway's Game of Life prototype (workspace root, nodemon watching the
Rust source, lazy WASM loader exposing raw memory) rather than
re-deriving it from scratch.

- Converted the repo root into an npm workspace (`workspaces: ["web"]`)
  with orchestration scripts: `build:wasm` (`wasm-pack build sim
  --target web --out-dir ../web/src/wasm`), `watch:wasm` (nodemon,
  watches `sim/src` + `sim/Cargo.toml`, 300ms debounce), `dev`
  (build once, then run the watcher and `web`'s Vite dev server together
  via `concurrently`), and `build` (build once, then `web`'s production
  build). `concurrently` and `nodemon` added as root devDependencies.
  Crate stays named `sim`, not renamed to match the prototype's `core`.
- Added `web/src/wasm-loader.ts`: a lazily-initialized, promise-cached
  loader that returns both the generated bindings and the raw
  `WebAssembly.Memory` export. Exposing the memory directly (rather than
  serializing frame data across the JS/WASM boundary) is the whole point
  of the `mem::swap`-based rendering approach from the original spec —
  this loader is the file that makes that possible on the JS side.
  `web/src/wasm/` (the `wasm-pack` output) is gitignored, generated, and
  never hand-edited.
- `web/package-lock.json` was replaced by a single root-level
  `package-lock.json` now that `web` is a workspace member — npm
  hoists shared deps to the root `node_modules`.
- Verified end to end: `npm run build:wasm` actually produces
  `web/src/wasm/sim.{js,d.ts}` + `sim_bg.wasm`; `tsc -p web --noEmit`
  passes against the real generated output (not just against the
  hand-written loader in isolation); `npm run build` (root) succeeds;
  and a bounded smoke test of `npm run dev` confirmed nodemon does its
  initial `build:wasm` pass and `concurrently` brings up the Vite dev
  server alongside it, with clean shutdown on SIGTERM.

## Misc Patches

<!-- One line each, newest first: small, cosmetic, no-schema-change
     updates that don't warrant a section of their own. -->
