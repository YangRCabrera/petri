---
name: doc-sync
description: Check whether the change just made drifted README.md or CLAUDE.md out of sync with the codebase, and fix whichever sections drifted. Use at the end of any nontrivial change, alongside `verify`, before committing.
---

# Keeping README.md and CLAUDE.md in sync

Stale docs mislead the next session exactly as badly as they mislead a
human — CLAUDE.md is loaded into context every session and treated as
ground truth. Treat a drifted doc as a bug, not cleanup.

## The judgment call: does this change need a doc update?

Reuse whatever PATCH/MINOR/MAJOR-shaped distinction CHANGELOG.md uses for
this project — don't invent a second rulebook:

- **PATCH-shaped** (copy/UI tweak, bug fix, refactor, no schema change,
  no new capability) → almost never touches the docs.
- **MINOR-shaped** (new capability, new route/table/function, existing
  behavior widened without breaking callers) → almost always does.
- **MAJOR-shaped** (breaking change) → touches the docs and probably
  deserves more than a one-line edit.

**Usually no doc update needed:** a new instance of an already-documented
pattern (a new feature vertical shaped exactly like existing ones — the
docs describe the pattern, not every instance of it); bug fixes/refactors
that don't change a convention; new tests for existing logic; copy tweaks.

**Check the docs:**
- A new or renamed script.
- A new dependency worth naming, or a new env var.
- A new architectural pattern, or a deliberate *exception* to one already
  documented.
- A new shared helper/primitive that other code is now expected to reuse.
- A new test layer or tool.
- Anything that makes an existing sentence in either doc **factually
  wrong** (not just incomplete) — these take priority over omissions.
- A "Not yet, but on the radar" item in CLAUDE.md that just got built —
  remove or reword it; a stale one is worse than none.

## Where a given signal belongs

| Signal | Update |
|---|---|
| New/renamed script | README Commands **and** CLAUDE.md Commands |
| New env var | README's setup/env block |
| New dependency worth naming | README Tech stack |
| New architectural pattern or exception | CLAUDE.md — the section whose claim it changes |
| New shared UI/logic primitive | CLAUDE.md's relevant conventions section |
| New test tooling/layer | CLAUDE.md Testing **and** README Tech stack/Commands |
| User-visible new feature | README Features |
| A "Not yet" item got implemented | Remove/reword that CLAUDE.md bullet |

## Applying the fix

- Match the existing tone: terse, technical, comments explain *why* not
  *what*. Read the surrounding paragraph before adding to it.
- Edit the specific section; don't rewrite the file for one change.
- After editing, re-read the touched section for internal consistency —
  does a Commands table match the real scripts exactly? Does removing a
  "Not yet" bullet leave another section still describing it as future
  work? Self-contradiction from incremental edits is the most common
  failure mode here.

## Reporting

State plainly what changed and why, in one or two lines. If you checked
and found no drift, say so explicitly — silence is indistinguishable
from forgetting to check.
