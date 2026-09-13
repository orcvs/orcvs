# 09 — Build background runs in the derive pass

**What to build:** fold `Paint::background_runs` into `Paint::derive` so runs close as the walk
goes, and retire the half-open `Range` leak at the shape step if that follows.

Filed from ticket `05`'s cascade question. With `CellVisuals.background` already
`Option<Color32>`, the second pass over derived Cells is no longer forced by a filter that lives
one module away — the derive walk already knows each Cell's fill. Closing runs there would also let
`BackgroundRun` stop exposing a half-open `Range<usize>` whose consumer subtracts one under a
`debug_assert!` (`console/src/console.rs` around the background-run shape step).

**Do not treat this as free.** `source-paint` deliberately chose "derived, never stored" for runs so
the fold stays testable in isolation. A single-pass fold is a different derivation shape with its
own trade-offs; this ticket exists so that choice can be reopened with a named acceptance bar rather
than as a drive-by on `05`.

**Blocked by:** `05`

**Status:** needs-triage

- [ ] Decide whether runs stay derived-on-demand or are built during `Paint::derive`.
- [ ] If built during derive: runs are closed in the walk; `background_runs()` either returns the
      stored answer or goes; tests that exercise the fold in isolation are rewritten against the new
      seam.
- [ ] If the shape step can take inclusive endpoints (or a pair of Positions) instead of
      `columns.end - 1`, do that and delete the `debug_assert!` that guards the subtraction.
- [ ] Nothing about what is drawn changes.

## Comments

Filed by ticket `05`: the cascade holds as a possible follow-up once the background is decided at
source. Not done there on purpose.
