# 19 — Cache the Tick schedule across revisions that change no parse

**What to build:** A Tick re-plans its dependency order only when the parse it depends on changes. Today every Tick rebuilds the whole schedule — `tick::plan` → `schedule` → `computations` → `Lookup::new` → `order_turns`: the per-Expression lookup, the edge set and the topological order — although the schedule depends only on the Grid and the Language Map. The Language Map revision is not a usable key: every committed Tick mints a new `LanguageMapId` and `RevisionId`, even one that writes nothing, and `rebuild_rows` deliberately re-derives rows a Tick rewrote with identical bytes. A cache keyed on the Map revision would miss on every Tick.

**Blocked by:** 06 — Reuse unchanged Language Map rows across revisions.

**Status:** ready-for-agent

Related: 07 moves planning outside the write lock; the cache must be readable wherever planning runs.

- [ ] Choose and record a key covering every scheduling input from the Grid and Language Map. It survives both no-write commits and commits that rewrite identical bytes. Merely keeping the revision for no-write commits does not satisfy the latter case; sharing clean rows alone does not cover rewritten rows.
- [ ] Define which changes alter scheduling inputs, including Function identity, portal relationships, occupancy and syntax diagnostics. Do not equate unchanged Expression shape with unchanged scheduling inputs.
- [ ] A schedule is computed once per key and reused by every Tick planned against it.
- [ ] Tests change Function identity, portal relationships, occupancy and syntax between Ticks and compare cached planning with fresh planning, including ordering and scheduling diagnostics. Tests separately prove reuse after no-write and identical-byte-write commits.
- [ ] Tick ordering and scheduling diagnostics are unchanged across the existing Tick tests.
- [ ] A benchmark shows Tick planning cost falls for a run whose Ticks write Cells without changing any Expression — "steady state" means that.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The earlier premise, that a steady-state run does not change the Language Map, was false: every commit mints a new Map revision. The ticket now asks for a key that survives a commit and defines "steady state". The 06 blocker is now justified by that.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Closed the gap between the no-write cache-key proposal and the steady-state benchmark. Added explicit input coverage and cached-versus-fresh invalidation tests.

**2026-09-25 — note from 06 (epic PR 6).** A commit that writes no row now keeps the held Language Map and its `LanguageMapId`; it still mints a new `RevisionId`. A commit that writes any row, identical bytes included, still mints a new Map identity. The key this ticket chooses must still cover the identical-byte case on its own.
