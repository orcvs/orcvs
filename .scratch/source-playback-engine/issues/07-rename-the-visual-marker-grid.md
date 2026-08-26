# 07 — Rename the visual marker grid

**What to build:** Free the word "grid" so it names one concept. The console draws a visual marker every few Cells and highlights the block around the cursor; those settings and that predicate currently use "grid" for something unrelated to Cell addressing. Rename them to marker vocabulary before a Grid module exists, so the two never overlap.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] The marker spacing and selected-dot spacing settings are named for markers, not for the grid.
- [x] The predicate that decides whether a position falls in the cursor's marker block lives with its only caller, the console's terminator rendering, and is no longer a method on the addressing type.
- [x] Rendering is unchanged: markers, highlights and cursor blocks appear exactly where they did before.
- [x] The tests covering the marker block move with the predicate and still pass.

## Comments

**2026-08-26 — implemented (agent)**

The marker spacing and highlight dot spacing settings are renamed off "grid" (`marker_spacing`, `highlight_dot_spacing`, and their defaults), and the cursor-block predicate moved off `Coord` to a private `in_marker_block` beside its only caller, `App::terminator`. Its body is unchanged and its test moved with it, assertions intact. `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` now derive from `DEFAULT_MARKER_SPACING`; their values are unchanged. Rendering is unchanged.

No `grid` identifier remains under `console/src` or `lang/src`. Five prose mentions remain in doc comments and test names in the Source module, all describing Source contents rather than the visual markers — left for tickets 08–10.

Review found nothing in this change. It did surface three pre-existing defects in the committed Source work, recorded as issues 11, 12 and 13.

**2026-08-26 — one sentence above no longer holds (review)**

The implementation comment records that `DEFAULT_COL_COUNT`/`DEFAULT_ROW_COUNT` "now derive from `DEFAULT_MARKER_SPACING`; their values are unchanged". That was true when written. It is not now: the issue 10 branch moved both constants into `console/src/grid.rs` and wrote them as plain literal `16`s, so a Source's shape is no longer derived from a visual constant. Left as written rather than corrected — it is a record of what this ticket did, and undoing the derivation was someone else's change.

Nothing this ticket asked for is affected. The settings are still named for markers, the predicate still lives beside its only caller, its test still moved with it, and rendering is still unchanged — the values are the same 16s either way. The consequence of the decoupling is recorded on issues 14 and 17, which are the two tickets that have to reason about the marker spacing.
