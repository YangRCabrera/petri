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

## Getting started

<!--
Write this the same day `<install> && <run>` first works end to end for
a clean checkout, not from memory later. Re-verify it before any push
that touches setup (new env var, new required service, new script). For
a polyglot project, repeat the numbered-list shape once per toolchain
rather than merging unrelated install/run steps into one block.
-->

1. Install dependencies:
   ```
   <command>
   ```
2. Environment variables (`.env`):
   ```
   <VAR>=              # what it's for
   ```
3. <Any DB migration / seed step>
4. Start the dev server:
   ```
   <command>
   ```

## Commands

<!-- Table: command | purpose. Keep byte-for-byte in sync with CLAUDE.md's
     Commands section — same underlying manifest(s), two audiences. -->

### Simulation core (Rust/WASM)

| Command | Purpose |
| --- | --- |
| | |

### Frontend (Vite/TS)

| Command | Purpose |
| --- | --- |
| | |

### Backend (Cloudflare Workers)

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
