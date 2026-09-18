# 04 — Add the Jump, Bang and movement Functions

**What to build:** The reference gains groups for the Jumps, the Directional Bang Functions, the Self-Banging Functions and Halt, each laid out so that playing it acts only inside its own example area and never overwrites another example.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** resolved

- [x] Jumps: `&^`, `&v`, `&<`, `&>`, each with a value at its input to copy.
- [x] Directional Bangs: `*^`, `*v`, `*<`, `*>`, each with a Bang source (for example an Equality that holds) so it emits when played.
- [x] Self-Banging: `^^`, `vv`, `<<`, `>>`, each with a path that ends inside its own example area.
- [x] Halt: `*!`, with a Bang source and a root directly south for it to lock.
- [x] A test ticks the reference repeatedly (enough Ticks for every mover to stop) and asserts no Cell outside each example's own area changes.
- [x] The one-Tick exact-result test excludes these example areas, since movement is their result.

## Resolution

Built two column bands in `console/assets/function_reference.orcvs`: band `k = 4` (columns
`64..80`, header `|| Jumps & Halt`) holds the four Jumps and Halt, and band `k = 5` (columns
`80..96`, header `|| Bang & Move`) holds the four Directional Bangs and the four Self-Banging
Functions. Both headers are short enough to stay inside their own 16 Cells, matching every
earlier group's header, and both land on their staircase row (4 and 5) rather than a stacked
second header — `console/src/function_reference.rs`'s module doc records the row-6 constraint a
stacked header would owe ticket 05's MIDI header.

Every Jump and Halt example is stable after Tick 0, the same as Arithmetic, so its area is just
the Cells its Expression and result occupy. A Directional Bang's emission and a Self-Banging
Function's own Span are not stable — ADR 0006 moves them one Cell per Tick until something stops
them — so each of those eight examples reserves a small rectangle: a two-Cell blocking Language
Unit (`00` throughout) placed exactly where the mover's next full Span would land, aligned so a
horizontal mover's one entered Cell is the near edge of that Unit and a vertical mover's whole
Span matches it. Both give a clean block with no diagnostic. Each Directional Bang's Bang source
is a Delay, `~*1001` (cycle 16), rather than an Equality: an Equality holding forever re-Bangs
every Tick, and once its emission's destination is no longer empty every later attempt is refused
and diagnoses forever, which a Delay whose cycle outlasts this module's own tick budget avoids by
Banging exactly once. None of this was worked out on paper — a throwaway `orcvs` example binary
(never committed) ticked candidate layouts and read back what moved, what blocked cleanly, and
what diagnosed, before any of it went into the checked-in text or the test.

A blocked mover's Span becomes `**` for exactly one Tick and then clears to blank the Tick after,
the same way a Delay's own result Cell clears on a Tick it does not Bang, so every mover in both
bands is written, blocked, and cleared by the Tick indexed 2, and nothing in either band's area
changes again after that.

`playing_the_reference_repeatedly_changes_only_each_examples_own_area` (added test-first, run
against the unchanged text first to see it fail on the too-small Grid) ticks the reference through
index 4 — two Ticks past settling — and asserts, at every Tick, that no Cell outside a named area
changes, excepting the Tick group's own six dynamic result Cells (ticket 03), which change every
Tick by that group's own design and are unrelated to this ticket's movers. It also asserts the
exact settled content of every one of the thirteen areas at Tick 2, and that nothing in a settled
area differs between Tick 2 and Tick 4.
`ticking_the_reference_once_writes_every_result_row_exactly_as_written` gained no new entries: all
thirteen areas are excluded from it, since movement is their result.

`console/src/console/kittest_tests.rs`'s `the_file_menu_loads_the_function_reference_on_demand`
pinned the reference's Grid at `(56, 32)`; updated to `(96, 32)` — the row count is unchanged
(band 5's content reaches row 31, still inside the height Arithmetic already pins), but band 5's
West Directional Bang example's Delay Expression reaches column 91, rounding the Grid's width up
from 56 to 96.

## Correction — walling the movers in with Halt instead of an ordinary blocker

Playing the reference destroyed its own examples: every blocked mover's Span turned to `**` for
one Tick and then cleared to blank forever, per ADR 0006, so `>> << ^^ vv` and the emissions of
`*^ *v *< *>` all vanished once played, and with persistence on that vanished state was what got
saved. The acceptance criterion above ("a path that ends inside its own example area") was met
literally — the path did end inside the area — but the user later judged the result unacceptable:
an example that only reads correctly before it is ever played is not documenting the language
being played.

The fix walls each mover in with the Halt Function (CONTEXT.md's Halt entry) in place of the
ordinary blocking `00` this ticket used: Halt locks its target's Turn *before* it runs, so a Cell
it protects is never overwritten and never turns to `**` — the mover's own glyph stands forever.
This is possible for five of the eight movers and not for the other three, proven by ticking
rather than assumed:

- **`^^` and the `^^` a Directional Bang North (`*^`) emits** wall in cleanly with real, multi-Tick
  travel: a Self-Banging Function's Span occupies exactly one row, so moving between rows is a
  discrete jump with no partial state, and an already-active Halt several rows north locks the
  mover the instant it arrives with no diagnostic at any Tick along the way.
- **`<<` and `>>`** are placed at rest under their own Halt from Tick 0, rather than given real
  travel: closing a two-Cell gap by one column a Tick, the Tick immediately before full alignment
  always leaves the mover overlapping only one of Halt's two target Cells, and Halt reports
  "target is not an Expression root" for that partial match instead of locking it silently. Placed
  at rest from the start, Halt is already active before the mover exists, so no such Tick is ever
  reached.
- **`vv`** is placed at rest for a stricter reason: reaching a Cell south of Halt by moving south
  means passing through Halt's own row, which collides with Halt outright rather than merely
  misaligning with it, in flight or at rest apart from the one Cell directly south of it.
- **The Directional Bang South, West, and East Functions (`*v`, `*<`, `*>`) cannot be walled at
  all**, in flight or at rest. Each needs its own one-shot Delay (a permanently-true Equality
  re-attempts the emission every Tick once the destination is no longer empty and diagnoses
  forever, the reason this ticket already gives for using a Delay) touching it to emit exactly
  once, and every placement of that Delay that avoids colliding with the emission or with Halt's
  own target instead reaches into the one or two Cells a second, permanently active Bang source
  would need in order to keep that mover's own Halt lit — proven by exhausting the placements a
  Delay (six Cells: a two-Cell anchor and a four-digit operand) and a second Bang source can take
  around a target only one or two Cells from the Directional Bang itself, not by inspection alone.
  These three still emit into an ordinary blocking Cell and clear the Tick after, exactly as every
  mover in this group did before this correction.

`console/assets/function_reference.orcvs`'s Directional Bang and Self-Banging Function band was
rebuilt against this design, and `console/src/function_reference.rs`'s
`playing_the_reference_repeatedly_changes_only_each_examples_own_area` gained a
`walled_resting_glyphs` check: after settling (still by the Tick indexed 2), the five walled
movers' own glyphs are asserted present at their resting Cells, not merely that their areas stopped
changing. The Grid also grew taller, from 32 to 40 rows, to give the walled movers' own Halt and
Equality room; ticket 01's own correction covers the window-size consequence of that.
