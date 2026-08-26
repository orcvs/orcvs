# 01 — Make Source edits synchronous and consistent

**What to build:** Make every user edit a synchronous Source transition so the returned snapshot, change set, Cells, glyph classifications, Expression state, and diagnostics all describe the same revision. Live Editing must remain responsive without exposing asynchronous ordering or partial derived state.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] After an accepted edit returns, every observable part of the Source describes the new revision with no stale read window.
- [x] A Cell accepts exactly one single-byte ASCII character; invalid content and out-of-range positions return precise errors without mutation.
- [x] Editing and deleting through the console still move the cursor and update the rendered Cell and glyph classification immediately.
- [x] Source snapshots and change sets expose consistent observable outcomes while Expression ranges, parsed atoms, and caches remain internal.
- [x] Tests exercise editing through the Source interface rather than relying on async sleeps, locks, or internal cache inspection.
- [x] The workspace test run no longer fails on stale Source examples.

## Comments

**2026-08-26 — implemented (agent)**

`Source::set`/`unset` are now synchronous, fallible transitions: validation (bounds, exactly one printable single-byte ASCII character) happens before any mutation, and an accepted edit returns a `Change` — the set of Cells whose content or glyph classification differs from the previous revision. `Source::snapshot()` (also on `SourceCommander`) reads the whole grid at one revision. The async `Command::Set`/`Unset` channel is gone; the channel now carries only playback `Tick`. Console `write`/`delete` call the synchronous path and move the cursor only on an accepted edit.

Two latent bugs surfaced and fixed along the way: joining/splitting Expressions left stale parsed Atoms (a deleted Expression could still execute on the next Tick) — edits now invalidate and reparse every Expression intersecting the affected span; and an Expression completed near the grid end panicked painting operand-slot glyph hints past the last Cell — hints now truncate. `ExpressionMap` is no longer exported from the crate.

Review notes deferred to later tickets: Expressions still wrap across rows (issue 02); `execute` still clamps out-of-grid results onto the last Cell, truncates multi-Cell values to their first character, and commits sequentially rather than atomically from one snapshot (issue 03, behavior pre-existing); the Tick channel/task still lives in the source module and a Tick holds the write lock while edits wait (issues 03/05). Vocabulary gaps for /domain-modeling: "revision" is used in doc comments but not in CONTEXT.md; the Cell definition says "single-byte ASCII" while `set` accepts only the printable range (0x20–0x7E).
