# Petri

A [Lenia](https://en.wikipedia.org/wiki/Lenia) cellular automata simulator — tune the simulation parameters live and share a configuration with someone else via URL or a small backend.

<!-- TODO once deployed: -->
Live at [<url>](). The day-by-day build process is written up in
[DEVLOG.md](./DEVLOG.md).

<!-- TODO once there's a screenshot worth showing — don't force one early: -->
<!-- ![short alt text](docs/screenshots/example.png) -->

## Features

<!--
Write each bullet the week the feature actually ships, in the app's own
voice (what a user can do), not the day of a v1.0 push reconstructing
three weeks of memory. This is the section most likely to atrophy if
deferred — treat a new feature as unfinished until this is updated,
same as you'd treat a missing test.
-->

## Tech stack

- **Simulation core** — Rust, compiled to WASM. Runs the Lenia step and writes
  raw RGBA output directly into a stable, pointer-exposed buffer (swapped via
  `mem::swap`, not serialized) that the frontend reads straight into a Canvas
  each frame.
- **Frontend** — plain Vite + TypeScript. No UI framework; this is
  fundamentally a canvas app plus a parameter panel.
- **Backend** — Cloudflare Workers. Bare-bones: just enough to store/retrieve
  a shared parameter set, since the data model is flat.
- **CI** — GitHub Actions (`.github/workflows/ci.yml`): fmt/clippy/build/test
  for `sim`, lint (`oxlint`)/typecheck/build for `web`.
- **Deployment** — Vercel, via `vercel.json` + `scripts/vercel-build.sh`
  (installs Rust/wasm-pack at build time, since Vercel's build image
  doesn't ship them).

## Getting started

Prerequisites: a Rust toolchain with the `wasm32-unknown-unknown` target
(`rustup target add wasm32-unknown-unknown`) and `wasm-pack` installed
globally.

1. Install dependencies (root — this is an npm workspace, `web` is the
   only member so far):
   ```
   npm install
   ```
2. Start everything (builds the WASM bindings once, then runs the
   wasm watcher and Vite dev server together):
   ```
   npm run dev
   ```

## Commands

<!-- Table: command | purpose. Keep byte-for-byte in sync with CLAUDE.md's
     Commands section — same underlying manifest(s), two audiences. -->

### Workspace (root)

| Command | Purpose |
| --- | --- |
| `npm install` | Install all workspace dependencies |
| `npm run build:wasm` | One-shot `wasm-pack build` of `sim/`, output into `web/src/wasm` |
| `npm run watch:wasm` | Rebuild the WASM bindings on any change under `sim/src` or `sim/Cargo.toml` |
| `npm run dev` | Build WASM once, then run the wasm watcher and `web`'s dev server together |
| `npm run build` | Build WASM once, then production-build `web` |

### Simulation core (Rust/WASM) — `sim/`

| Command | Purpose |
| --- | --- |
| `cargo build --target wasm32-unknown-unknown` | Build the crate for WASM |
| `cargo test` | Run unit tests (native target) |
| `cargo fmt --check` | Check formatting |
| `cargo clippy` | Lint |

### Frontend (Vite/TS) — `web/`

| Command | Purpose |
| --- | --- |
| `npm run dev` | Start the dev server |
| `npm run build` | Typecheck (`tsc`) + production build |
| `npm run lint` | Lint (`oxlint`) |
| `npm run preview` | Preview the production build locally |

### Backend (Cloudflare Workers)

<!-- Not scaffolded yet. -->
| Command | Purpose |
| --- | --- |
| | |

## Architecture

<!-- Short prose + a folder-tree code block, written once the structure
     has actually settled — same timing as CLAUDE.md's Architecture
     section (once the shape repeats enough to name, not on a fixed
     schedule). Don't write this from an initial plan; write it from
     what's actually on disk. -->

## Database

<!-- Only if applicable. Migration workflow, one line. -->
