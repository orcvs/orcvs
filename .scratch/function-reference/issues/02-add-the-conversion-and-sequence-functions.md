# 02 — Add the Conversion and Sequence Functions

**What to build:** The reference gains a Conversion group and a Sequence group, each Function with a worked example and its written result.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** resolved

- [x] Conversion: `.^` (Number to Note) and `.v` (Note to Number).
- [x] Sequence: `:-`, `:#`, `:<`, `:&`, `:?`, `:=`. A Sequence operand is supplied by a nested Function, for example `:<:-0104` over `04030201`.
- [x] A Sequence result that spans more than one Sector stays inside its column.
- [x] One example shows a Pervasive Arithmetic Function applied elementwise to a Sequence.
- [x] The one-Tick test covers both groups.

## Comments

Mid-implementation, running the console from this worktree panicked at `function_reference.rs`'s
"must be a rectangle" assertion: the checked-in `.orcvs` asset had its trailing whitespace
stripped (a tool in the edit path strips it on save), turning the padded rectangle ragged. The
loader was too fragile to demand a shape a plain checkout could not guarantee it would keep. Fixed
as part of this ticket — see Resolution.

## Resolution

Built the Conversion and Sequence groups in `console/assets/function_reference.orcvs`, extending
the Grid from 16 columns to 48 (three 16-Cell column bands) while keeping the row count at 32.
Every example was verified by actually ticking the reference (`cargo test -p console
function_reference`), not computed by hand:

- Conversion (column `16..32`, header row 1, examples from row 2): `.^3C` → `C4` (Number to Note),
  `.vC4` → `3C` (Note to Number).
- Sequence (column `32..48`, header row 2, examples from row 3): `:-0104` → `01020304` (Number
  Range), `:#C4D4` → `C4c4D4` (Note Range), `:<:-0104` → `04030201` (Reverse over a nested
  Function operand, the ticket's own example), `:&01:-0203` → `010203` (Concatenate, an Atom
  promoted alongside a nested Sequence), `:?00:-0103` → `01` (Select), `:=01.+0102:-0103` →
  `010303` (Replace, over two nested Functions — exactly 16 Cells, the column's full width), and
  `.+10:-0103` → `111213` (`.+`, a Pervasive Arithmetic Function, broadcast elementwise over a
  Sequence — the required example, placed in the Sequence group per the ticket).

**Header row conflict, resolved as a staircase.** A Comment claims the rest of its *Grid* row, not
just its own 16 Cells, so two group headers cannot share a row. The group in column band `k`
(0-based: Arithmetic `k = 0`, Conversion `k = 1`, Sequence `k = 2`, Tick `k = 3`, the movement
groups `k = 4..5`, MIDI `k = 6`) now puts its header on row `k` and starts its examples on row
`k + 1`. Arithmetic keeps `k = 0` and did not move. Recorded in `console/src/function_reference.rs`'s
module doc, replacing ticket 01's open question, so tickets 03–05 follow the same rule.

**Loader hardened against trailing-whitespace stripping.** `console/src/function_reference.rs`
no longer asserts the checked-in text is a padded rectangle already a multiple of the Sector Seam
spacing. `source_from_reference_text` (the parsing logic factored out of `function_reference`)
instead derives the Grid's width from the widest line and its height from the line count, each
rounded up to the Sector Seam spacing, and pads short lines and rows past the last line with empty
Cells — ordinary unset Cells, since `Source::get` already reads those back as empty. The checked-in
`.orcvs` asset now stores each line with trailing whitespace removed and stops at the last line
with content (no trailing blank rows), so it is stable under whatever strips trailing whitespace on
save. A new test, `a_ragged_text_with_stripped_trailing_whitespace_loads_at_rounded_up_dimensions`,
proves a ragged, never-checked-in text loads at the rounded-up dimensions; the existing
`the_reference_grid_dimensions_are_multiples_of_the_sector_seam_spacing` test still holds against
the real asset.

`console/src/console/kittest_tests.rs`'s `the_file_menu_loads_the_function_reference_on_demand`
pinned the reference's Grid at `(16, 32)`; updated to `(48, 32)`. `DEFAULT_VIEW_SIZE` is unrelated
to the reference's own dimensions (the console fits and letterboxes any Grid), so no window size
needed to change.
