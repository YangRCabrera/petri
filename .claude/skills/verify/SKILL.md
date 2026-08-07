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
npm run lint        # oxlint
tsc --noEmit
<unit test command — no test framework installed yet>
```

### backend (wrangler)

<!-- Not scaffolded yet. -->

## 2. Runtime

This is an npm workspace (root `package.json`, `web` is the only member
so far). `npm run dev` from the repo root builds the WASM bindings once,
then runs the wasm watcher (nodemon, rebuilds on any `sim/src` or
`sim/Cargo.toml` change) and `web`'s Vite dev server together via
`concurrently`. Prefer this over running `web`'s `npm run dev` alone
whenever the change touches `sim/` — otherwise stale bindings get served.

```
npm run dev        # run from repo root — sim watcher + web dev server together
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

`chromium-cli` isn't available in this environment — fall back to a
scratch Playwright script. Chromium itself is already cached locally
(`~/AppData/Local/ms-playwright`), so only the `playwright` npm package
needs installing, and only into the scratchpad — don't add it to
`web`'s own `package.json` for a one-off check:

```bash
cd <scratchpad>
npm init -y && npm install playwright@1.62.1 --no-save
```

Start the dev server from the repo root (`npm run dev`, see above) and
wait for it to actually serve before driving it:

```bash
timeout 30 bash -c 'until curl -sf http://localhost:5173 >/dev/null; do sleep 1; done'
```

```js
import { chromium } from 'playwright';

const browser = await chromium.launch();
for (const viewport of [
  { width: 1280, height: 800 }, // desktop
  { width: 375, height: 667 }, // mobile
  { width: 320, height: 480 }, // smallest realistic phone
]) {
  const page = await browser.newPage({ viewport });
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text());
  });

  await page.goto('http://localhost:5173');
  await page.waitForSelector('#sim-canvas');
  await page.waitForTimeout(500); // let the WASM sim/render settle

  await page.screenshot({ path: `<scratchpad>/${viewport.width}.png` });
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
  );
  console.log(viewport, 'errors:', errors, 'horizontalOverflow:', overflow);
  await page.close();
}
await browser.close();
```

Then actually **Read** each screenshot back — that step catches what
computed-style/rect checks miss. E.g. Chromium hides a closed
`<details>`'s content through an internal `content-visibility` region
that isn't reachable by author CSS: `getComputedStyle` reported the
content as `display: flex` / `content-visibility: visible` and
`getBoundingClientRect` returned a real, in-viewport size, yet nothing
was actually painted — only the screenshot showed the panel was blank.

Kill the dev server after: `lsof -ti:5173 -sTCP:LISTEN | xargs -r kill`.

## 3. What "verified" means

State what you exercised and what you observed — not just "should work."
If a layer couldn't be reached (auth wall, missing test data), say so
explicitly rather than implying it was covered.
