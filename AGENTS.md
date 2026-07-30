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

<!-- Fill in the pinned Rust edition + wasm-bindgen/wasm-pack version once
     Cargo.toml exists. Check `cargo doc --open` before assuming an API
     shape from training data. -->

<!-- Add one short, concrete bullet per real breaking change you hit. -->

## Vite + TypeScript (frontend)

<!-- Fill in the pinned Vite/TS version once package.json exists. -->

<!-- Add one short, concrete bullet per real breaking change you hit. -->

## Cloudflare Workers (backend)

<!-- Fill in the pinned wrangler/workers-types version once wrangler.toml
     exists. Check the Workers docs before assuming an API shape. -->

<!-- Add one short, concrete bullet per real breaking change you hit. -->

<!-- END:framework-agent-rules -->
