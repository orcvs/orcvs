# 03 — Add the Tick Functions

**What to build:** The reference gains a Tick group: Clock, Delay, Euclidean, Increment, Interpolation and Random, each with an example whose result row is what the first Tick writes, so playing it shows the value changing.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** resolved

- [x] One example each for `~.`, `~*`, `~%`, `~+`, `~>`, `~?`.
- [x] Each result row is written as the first Tick writes it.
- [x] Increment and Interpolation read their own result Cell; their examples start from a written value that makes the change visible.
- [x] The one-Tick test covers the group. Later Ticks are expected to differ and are not asserted.

## Resolution

Built the Tick group in `console/assets/function_reference.orcvs`, extending the Grid from 48 to
56 columns (Random's eight-Cell Expression is the widest row) and keeping the row count at 32 —
the group's header plus six examples of 3 rows each only reaches row 21, well inside the height
Arithmetic already pins. Following ticket 02's staircase rule, the Tick group is column band
`k = 3` (columns `48..64`): its `|| Tick` header sits on row 3 and its examples start on row 4.
Every result was verified by actually ticking the reference (`cargo test -p console
function_reference`), not computed by hand: an initial placeholder was asserted for each new
`(column, row, expected)` entry, the test was run to fail, and the checked-in text and the
assertion were then set to what the failure (or, for Random, the panic's `left:` value) showed.

- Clock `~.0204` → `00` (step 0 of a 2-Tick, 4-step cycle; Tick 0 is always step 0).
- Delay `~*0302` → `**` (a Delay Bangs at Tick 0 for any non-zero rate and modulus, since 0 is a
  multiple of every cycle).
- Euclidean `~%0308` → `**` (the tresillo `X..X..X.` Bangs its first step).
- Increment `~+0104` over a checked-in previous of `03` → `00` asserted (`(03 + 01) % 04`).
- Interpolation `~>0210` over a checked-in previous of `00` → `02` asserted (steps up by rate `02`
  toward target `10`, short of it).
- Random `~?010010` → `10` (seed `01`, minimum `00`, maximum `10`, at this Function's own Grid
  Position `(48, 19)` and Tick `0`; ADR 0013 seeds the stream from exactly those facts, so the
  value is reproducible but tied to where this example sits — moving it changes the draw).

**Increment and Interpolation: checked-in text versus asserted result.** Both read their own
result Cell as the previous value before writing a new one, so the checked-in reference — which
is what a fresh, unplayed console shows — necessarily differs from what Tick 0 writes there.
`03` (Increment) and `00` (Interpolation) are the pre-Tick previous values chosen to make the
first Tick's change visible; `00` and `02` are what `ticking_the_reference_once_writes_every_result_row_exactly_as_written`
asserts after that Tick runs. Every other Function in this group (and every earlier group) writes
a Cell that depends only on its operands, the Tick, and — for Random — its own Position, never on
a Cell it or anything else wrote earlier, so the checked-in text and the asserted result are the
same string for those.

`console/src/console/kittest_tests.rs`'s `the_file_menu_loads_the_function_reference_on_demand`
pinned the reference's Grid at `(48, 32)`; updated to `(56, 32)`. `console/src/function_reference.rs`'s
module doc gained the Tick row's height and width accounting, and its column-layout table now
marks Conversion and Sequence as ticket 02's rather than "this ticket"'s, since this ticket is 03.
