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

## Misc Patches

<!-- One line each, newest first: small, cosmetic, no-schema-change
     updates that don't warrant a section of their own. -->
