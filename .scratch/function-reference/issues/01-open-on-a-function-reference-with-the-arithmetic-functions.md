# 01 — Open on a Function reference, starting with the Arithmetic Functions

**What to build:** The console opens on a checked-in reference Source instead of a blank Grid when no Source is stored, and a console command loads the same reference on demand. The first group is the Arithmetic Functions, laid out in the column shape every later group follows.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] With no stored Source, with persistence off, and when a stored Source is refused, the console opens on the reference Source rather than a blank Grid.
- [ ] A console command, reachable from the menu, replaces the current Source with the reference, including its Grid. It is an explicit action, not a single unmodified key. With persistence on, the reference is then saved like any other Source.
- [ ] The reference is one checked-in Source text, not built up in code, and its Grid dimensions come from it. Both dimensions are multiples of the Sector Seam spacing, so every column edge falls on a Sector Seam.
- [ ] Columns are 16 Cells wide (two Sectors). A group is headed by a `||` Comment row. Each example is its Expression row, its result row directly south, and one blank row.
- [ ] The Arithmetic group has one example each for `.+ .- .| .x ./ .% .< .> .=`. Equality shows both outcomes: a Bang when equal and nothing when not.
- [ ] Result rows are written as the value one Tick writes, for example `.+0101` over `02` and `.-1001` over `09`.
- [ ] A test ticks the reference once and asserts every result row reads exactly as written. This is the test later groups extend.
- [ ] The tests that pinned a blank default Source are updated to the reference. The window still fits the Grid.
- [ ] The column layout, including which columns later groups occupy, is recorded where the later tickets can follow it, so groups can be added without moving each other.

## Comments

Layout facts from research: Sector Seam spacing is fixed at 8 Cells; a stored Source always wins over the default at startup; playback starts stopped, so the reference shows as authored until played. Examples wider than one Sector (MIDI at 10 Cells, nested Sequence Functions, Sequence results up to 14 Cells) are why a column is two Sectors.
