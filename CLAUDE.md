@AGENTS.md

# Project rules

<!--
This file is loaded into every agent session and treated as ground truth.
A stale section is worse than a missing one — it causes wrong assumptions
instead of an honest gap. Keep every claim here true as of today; the
doc-sync skill exists specifically to catch drift.

Fill in sections in the order they become TRUE, not the order they look
impressive. Most of these have nothing to say on day 1. That's fine —
leave the header and the HTML comment until there's real content.
-->

## Commands

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
| `npm run preview` | Preview the production build locally |

### Backend (Cloudflare Workers)

<!-- Not scaffolded yet. -->

## <Framework/toolchain> version notes

<!-- Only if AGENTS.md's gotchas for that toolchain grow past a couple of
     lines and deserve more room, or there's project-specific config
     layered on top of the toolchain's own breaking changes. -->

---

## Architecture

<!-- Write this once the shape has repeated enough to name, not on a
     fixed feature count and not before there's real code to name it
     from. For a typical CRUD app that's usually the 2nd feature
     vertical; for a project built as distinct sequential layers (e.g.
     simulation -> rendering -> editing -> sharing) it's whenever the
     second layer confirms the boundary the first one implied. Don't
     design this on paper before any code exists to name. -->

## Separation of concerns

<!-- One short paragraph per file/folder/toolchain whose responsibility
     is easy to blur with another's. Skip anything self-evident from its
     name. -->

## <Core data/error shape — e.g. a Result type>

<!-- Only formalize this once you've hit the same "expected failure"
     shape by hand 2-3 times. Premature abstraction here is worse than a
     little duplication for a while. -->

---

## Security requirements

<!-- Write this the moment more than one user's data can live in the same
     table/store. Nothing to enforce before that point — don't pad this
     section with hypotheticals. -->

## Frontend conventions

<!-- Same reasoning as Architecture — once a convention has actually
     repeated, not on a fixed feature count. -->

---

## Testing

<!-- What's actually tested, what's deliberately not, and why — not an
     aspirational list. Update the same day a new test layer is added
     (unit, e2e, a11y, etc.); this is a doc-sync trigger every time. -->

## Documentation

<!-- The comment contract: what gets a JSDoc/docstring/rustdoc, what
     doesn't, and the rule that comments explain WHY not WHAT. Cheap to
     write once, early — write it before habits drift in either
     direction. -->

---

## Not yet, but on the radar

<!--
Start this section on day 1, empty is fine. This is the single
highest-value section in the whole file: it's where a deliberate
scope cut gets RECORDED instead of silently forgotten, and it's the
first place a future session — or an interviewer — should look to see
you know what's missing on purpose versus by accident.

Format: one bullet per deferred thing, one clause on why it's deferred.
Delete a bullet the day it actually gets built — don't let it go stale.
-->
