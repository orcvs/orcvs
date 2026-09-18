# 05 — Add the MIDI output Functions

**What to build:** The reference gains a MIDI group with an example of each Terminal Output Function, each with a Bang source so it sends when played.

**Blocked by:** 01 — Open on a Function reference, starting with the Arithmetic Functions.

**Status:** resolved

- [x] One example each for `!>`, `!~`, `!%`, `!c`, `!b`, with operands that name a plausible channel, note, velocity, controller or bend.
- [x] Each has a Bang source that fires at a rate slow enough to hear distinct events at the default tempo.
- [x] No result row: a Terminal Output Function writes nothing, and the example says so in its layout (no row reserved south).
- [x] A test proves ticking the group writes no Cell.

## Resolution

Added column band `k = 6` (columns `96..112`, header `|| MIDI` on row 6, examples starting row 7)
to `console/assets/function_reference.orcvs`, with one example each for the five Terminal Output
Functions the Function table actually holds: Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`,
Control Change `!c`, and Pitch Bend `!b`. Application Command `!$`, the family's sixth member named
in CONTEXT.md and the module doc's original plan, has no variant in `lang::Function` — ADR 0008 and
ADR 0019 defer it until Orcvs has a command value encoding — so it has nothing to write an example
of and ticket 06's completeness test has nothing to ask for there either.

Operands: `!>0064C4` (Raw Play, channel `00`, velocity `64`, note C4), `!~0064E408` (Timed Play,
channel `00`, velocity `64`, note E4, length `08` Ticks), `!%0164G404` (Monophonic Play, channel
`01`, velocity `64`, note G4, length `04` Ticks), `!c000764` (Control Change, channel `00`,
controller `07` — Volume — value `64`), `!b000050` (Pitch Bend, channel `00`, lsb `00`, msb `50` —
above centre). Each is a worked, plausible example rather than a domain-boundary probe; boundary
values already live in `lang`'s own operand tests.

Each example is gated by its own Euclidean, `~%0104` — the `X...` pattern
(`lang::functions::tick::euclidean_places_its_hits_where_the_adr_formula_does` proves this pattern
from the same formula), one Bang every four Ticks starting at Tick 0, roughly three seconds apart
at the default 20 BPM tempo (`orcvs::opts::Opts::new`'s `Bpm::new(20)`, 750 ms per Tick) — slow
enough to hear each event distinctly. Neither of ticket 04's two Bang sources fit: a Delay whose
cycle outlasts the tick budget (band 5's choice) Bangs once and never again, which does not "fire
at a rate"; an always-true Equality Bangs every Tick, and unlike a Directional Bang's emission (which
occupies a Cell and so diagnoses on every later re-attempt) a Terminal Output Function has no
destination Cell to occupy, so it would perform cleanly forever — just not as a distinct event.
Activation is vertical, the same alignment
`orcvs::source::tick::a_select_bang_activates_an_aligned_terminal_root` proves for a Select-forwarded
Bang: each Euclidean's Expression, its own Bang-display Cells one row south at the same column, and
the gated Terminal Output Function's root one row further south at that same column, with a blank
row after — four rows per example, since a Terminal Output Function's own row replaces a value
group's result row rather than sitting beside one.

`ticking_the_midi_group_writes_no_cell_and_emits_every_terminal_output_function` (added test-first,
seen failing first against the un-widened 96-column Grid and again against an overly strict "no
write anywhere in the band" check before the Euclidean sources' own legitimate Bang-display writes
were carved out) ticks the reference across two Euclidean cycles (Ticks 0 through 8) and asserts,
every Tick: no diagnostic; no `CellWrite` inside the MIDI band outside the five Bang-display Cells
(a Terminal Output Function never writes a Cell — CONTEXT.md's Terminal Output Function entry); and
no Cell in the MIDI band changes except those same five Bang-display Cells, which the test names
explicitly via `midi_bang_result_areas`. Across the whole run it also asserts every one of the five
expected `PlayCommand`s (`orcvs::source::TickPlan::play_commands`, the same field
`a_select_bang_activates_an_aligned_terminal_root` reads) was emitted on at least one Tick — proof
the group plays, not merely that it declines to write.
`playing_the_reference_repeatedly_changes_only_each_examples_own_area` (ticket 04's repeated-tick
containment test) now excludes those same five Bang-display Cells via the same
`midi_bang_result_areas` helper, the same way it already excludes the Tick group's own six dynamic
result Cells, since they legitimately change every few Ticks for a reason unrelated to either
ticket's own concern.

`console/src/console/kittest_tests.rs`'s `the_file_menu_loads_the_function_reference_on_demand`
pinned the reference's Grid at `(96, 32)`; updated to `(112, 32)` — the row count is unchanged (the
MIDI band's five four-row examples reach only row 26, still inside the height Arithmetic already
pins), but Timed Play's and Monophonic Play's ten-Cell Expressions, starting at column 96 like
every MIDI example, reach column 105, rounding the Grid's width up from 96 to 112 — exactly band
6's own upper bound.

## Correction

Timed Play's own example, `!~0064E408`, held a note for 8 Ticks while its `~%0104` Euclidean
source fires every 4 — so the note was asked to sound again 4 Ticks before the one before it had
finished, an internal inconsistency this ticket's own acceptance criteria did not catch because
nothing here checked one example's rate against another's. Fixed by shortening the length to `04`
Ticks (`!~0064E404`), matching the source's own period exactly, the same length Monophonic Play's
own example already used. `expected_midi_commands`'s `PlayCommand::Timed` expectation in
`console/src/function_reference.rs` was updated from `Length::from(0x08)` to `Length::from(0x04)`
to match.
