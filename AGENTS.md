<!-- BEGIN:framework-agent-rules -->

# Read this before writing code

<!--
This file's whole job is to catch the gap between an AI agent's training
data and the actual, currently-installed version of whatever you're
building on. Keep it short. Add a line the FIRST time an agent gets
something wrong because it assumed an older API/convention — not before,
on spec, for something that hasn't happened yet.

This is a polyglot project (Rust/WASM sim core + separate Vite/TS frontend
+ separate Cloudflare Workers backend) — one block per toolchain, since a
Rust/wasm gotcha and a Workers gotcha have nothing to do with each other.
-->

## Rust / wasm-bindgen (simulation core)

`sim/Cargo.toml` targets edition `2024`, `wasm-bindgen = "0.2.126"`,
`rustfft = "6"`. No `rust-toolchain.toml` — CI
(`dtolnay/rust-toolchain@stable`) and local builds float on whatever
`stable` currently is (this machine: rustc/cargo 1.97.1). `wasm-pack`
is likewise unpinned (CI's `jetli/wasm-pack-action@v0.4.0` installs
`latest`; this machine has 0.15.0). If a build breaks on a toolchain
bump, check `cargo doc --open` for the installed version's actual API
shape before assuming training-data behavior.

<!-- Add one short, concrete bullet per real breaking change you hit. -->

## Vite + TypeScript (frontend)

`web/package.json` pins `vite ^8.1.1`, `typescript ~6.0.2`,
`oxlint ^1.76.0`. Node 22 in CI (`actions/setup-node@v4`).

<!-- Add one short, concrete bullet per real breaking change you hit. -->

## Cloudflare Workers (backend)

<!-- Fill in the pinned wrangler/workers-types version once wrangler.toml
     exists. Check the Workers docs before assuming an API shape. -->

<!-- Add one short, concrete bullet per real breaking change you hit. -->

<!-- END:framework-agent-rules -->
