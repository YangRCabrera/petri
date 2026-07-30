---
name: verify
description: Verify a change to this app actually works — static checks, unit tests, then drive the affected flow at runtime. Use before committing any nontrivial change.
---

# Verifying a change

Work through the cheapest layer that can catch the problem first, but
don't stop before the layer that actually exercises the change.

<!-- Fill in real commands as soon as they exist. Keep this file in sync
     with the project's manifest(s) — same doc-sync trigger as README
     and CLAUDE.md's Commands sections. Three toolchains here: sim
     (cargo), web (vite), backend (wrangler) — keep their blocks
     separate rather than interleaving unrelated commands. -->

## 1. Static + unit (always)

### sim (cargo) — run from `sim/`

```
cargo fmt --check
cargo clippy
cargo build --target wasm32-unknown-unknown
cargo test
```

### web (vite) — run from `web/`

```
<lint command — no linter installed yet>
tsc --noEmit
<unit test command — no test framework installed yet>
```

### backend (wrangler)

<!-- Not scaffolded yet. -->

## 2. Runtime

```
web: npm run dev   # run from web/
<sim: no standalone runtime yet — wasm-pack / the actual JS<->WASM wiring
      doesn't exist until the frontend imports the crate>
<backend: wrangler dev — not scaffolded yet>
```

<!--
If there's an auth wall or any other boundary that blocks straightforward
verification, document the workaround here explicitly, in order of
preference — e.g. a dedicated test account, a way to bypass auth for a
service-layer-only check, a fallback of asking the user to click through
manually. This section is where the actual friction of THIS project's
setup gets written down, once you've hit it once.
-->

### Service/logic-layer verification (bypasses any UI/auth wall)

<!-- For changes whose interesting behavior lives below the UI: a
     scratch script pattern that exercises the function directly. -->

### Screenshot-based UI verification

<!-- For visual/layout changes: drive the flow headlessly, capture a
     screenshot, actually read it back — a screenshot never looked at
     isn't verification. -->

## 3. What "verified" means

State what you exercised and what you observed — not just "should work."
If a layer couldn't be reached (auth wall, missing test data), say so
explicitly rather than implying it was covered.
