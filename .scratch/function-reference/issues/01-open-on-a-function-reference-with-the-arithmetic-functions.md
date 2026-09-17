# 01 — Open on a Function reference, starting with the Arithmetic Functions

**What to build:** The console opens on a checked-in reference Source instead of a blank Grid when no Source is stored, and a console command loads the same reference on demand. The first group is the Arithmetic Functions, laid out in the column shape every later group follows.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [ ] ~~With no stored Source, with persistence off, and when a stored Source is refused, the console opens on the reference Source rather than a blank Grid.~~ Withdrawn on rebase onto `source-view`: a console that restores no Source opens the blank 64 by 40 default Grid (`source-view/03`, ADR 0045), and the reference is reached from the File menu only.
- [x] A console command, reachable from the menu, replaces the current Source with the reference, including its Grid. It is an explicit action, not a single unmodified key. With persistence on, the reference is then saved like any other Source.
- [x] The reference is one checked-in Source text, not built up in code, and its Grid dimensions come from it. Both dimensions are multiples of the Sector Seam spacing, so every column edge falls on a Sector Seam.
- [x] Columns are 16 Cells wide (two Sectors). A group is headed by a `||` Comment row. Each example is its Expression row, its result row directly south, and one blank row.
- [x] The Arithmetic group has one example each for `.+ .- .| .x ./ .% .< .> .=`. Equality shows both outcomes: a Bang when equal and nothing when not.
- [x] Result rows are written as the value one Tick writes, for example `.+0101` over `02` and `.-1001` over `09`.
- [x] A test ticks the reference once and asserts every result row reads exactly as written. This is the test later groups extend.
- [ ] ~~The tests that pinned a blank default Source are updated to the reference. The window still fits the Grid.~~ Withdrawn with the default above; ADR 0045 retired the fit, so the window is sized to the default Grid, not the reference.
- [x] The column layout, including which columns later groups occupy, is recorded where the later tickets can follow it, so groups can be added without moving each other.

## Comments

Layout facts from research: Sector Seam spacing is fixed at 8 Cells; a stored Source always wins over the default at startup; playback starts stopped, so the reference shows as authored until played. Examples wider than one Sector (MIDI at 10 Cells, nested Sequence Functions, Sequence results up to 14 Cells) are why a column is two Sectors.

## Resolution

Built `console/assets/function_reference.orcvs` (16 columns x 32 rows, both multiples of
8) as the one checked-in Source text, loaded by `console/src/function_reference.rs`'s
`function_reference()`. Grid dimensions are derived from the text itself (longest line,
line count), not stated separately in code. `starting_source` was briefly pointed at it
and restored to the blank default Grid on rebase onto `source-view` (see the withdrawn
criteria above). `Console`'s File menu gained "Load Function reference" (`Console::load_function_reference`), which
replaces the whole running Orcvs — Source and Grid — and re-derives MIDI device selection
over the new handle; saving afterwards persists it like any other Source.

The chosen worked examples (verified by actually ticking, not computed by hand):
`.+0102`→`03`, `.-0503`→`02`, `.|0307`→`04`, `.x0304`→`0C`, `./0902`→`04`, `.%0902`→`01`,
`.<0305`→`03`, `.>0305`→`05`, `.=0505`→`**` (equal), `.=0506`→ nothing written (not equal).

Column layout for later tickets (see the module doc on `console/src/function_reference.rs`
for the same table, kept beside the reference text):

| Columns   | Group                                                                           |
|-----------|----------------------------------------------------------------------------------|
| `0..16`   | Arithmetic (this ticket): `.+ .- .| .x ./ .% .< .> .=`                          |
| `16..32`  | Numeric Conversion: `.v .^`                                                     |
| `32..48`  | Sequence: `:- :# :< :& :? :=` (results up to 14 Cells)                          |
| `48..64`  | Tick: `~. ~* ~% ~+ ~> ~?`                                                       |
| `64..96`  | Jumps, Directional Bangs, Self-Banging, Halt (two columns — these move and need room to move without leaving their area) |
| `96..112` | MIDI: `!> !~ !% !c !b !$` (examples ~10 Cells wide)                             |

Open question left for whichever ticket adds the second column: each group's header
Comment claims the rest of its *Grid* row (the whole row, not just its own 16 Cells), so
two groups cannot both head at row 0 once they share a Grid. Not resolved here because
only one group exists so far.
