# Read, Write and Copy Source by direction and Position

Status: ready-for-agent

## Problem Statement

A performer coming from Orca cannot build a tracker with several voices in Orcvs. In Orca, each voice offsets a shared Clock, reads the Note at that row of its own column with `O` or `Q`, and triggers a MIDI note. Orcvs can only read Cells through a Jump's fixed Input Portal or through Track, which reads a pair east of its own operands on its own row. No Function reads a Cell its operands address, so a pattern laid out in columns cannot be played, and patterns cannot be stacked one voice per column.

ADR 0019 mapped Orca's Read, Query and Konkat onto a Source Read `@<`, Write and Generator onto a Source Write `@>`, and Push onto a replacement followed by a Write. ADR 0049 settled that a Position is two Numbers and left absolute Address Functions as follow-up, and ADR 0067 added the dynamic Input Portal. A first Read, `@< column row` at a relative offset on both axes, was built under ticket 02. Building it showed that one `@` family holding both reads and writes cannot spell directional and absolute forms of each, and that the Jumps sit in an "address" family named for neither action.

[ADR 0070](../../docs/adr/0070-read-write-and-copy-are-separate-families.md) (proposed) splits the Functions that touch Source into three families by action: Read `&`, Write `@` and Copy `=`. Jump is renamed Copy. Every family has directional forms whose arrow names the Cell the Function acts on, and an absolute form with `$` as its second glyph.

## Solution

| Action | Relative, by direction | Absolute, at a Position | Indexed along a row |
| --- | --- | --- | --- |
| Read `&` | `&^ &v &< &> n` | `&$ column row` | Track `&t index count` |
| Write `@` | `@^ @v @< @> n value` | `@$ column row value` | Push `@t index count value` |
| Copy `=` | `=^ =v =< =>` | `=$ src-column src-row dst-column dst-row` | — |

A directional Read reads the pair `n` Portals from its `n` operand in its arrow's direction through a dynamic Input Portal it reads exactly as a Copy does, and answers it through its Output Portal one row south or, nested, as its Return. A voice whose `&v` sits at the head of its column, with a Clock-driven `n` offset by `.+ 01`, steps down that column. A shipped example ports the Orca tracker patch, with several voices playing one shared pattern grid through Timed Play.

The four Jumps keep their behaviour as the Copy Functions `=^ =v =< =>`. Track moves to `&t`. The absolute Read `&$` reads at a Position. Writes, Push and the absolute Copy `=$` follow once the ordering of a dynamic Output Portal is decided. Generator is a Read nested in a Write's `value` operand, not a separate Function.

Nothing in the language changes until ADR 0070 is accepted (ticket 07), which also settles its remaining proposed default.

## User Stories

1. As a performer, I want to read a Note from the pair `n` rows below a voice, so that patterns can be laid out in columns as in Orca.
2. As a performer, I want reads to be relative to the Function, so that copying a voice elsewhere in the Source keeps it working.
3. As a performer, I want a Clock-driven `n`, so that a voice steps down its column in time.
4. As a performer, I want several voices to share one Clock with different offsets, so that I can stagger or phase them as the Orca patch does.
5. As a performer, I want each voice to drive its own Timed Play, so that voices sound independently on their own MIDI channels.
6. As a performer, I want an empty pair to leave a step silent, so that rests keep their place in a column.
7. As a performer, I want edits to a column to reach the next eligible Tick, so that Live Editing a pattern feels immediate.
8. As a composer, I want a same-Tick write to a Read's operands to affect which pair it reads that Tick, so that spatial composition is not a Tick late.
9. As a composer, I want a same-Tick write to the addressed pair to reach the Read that Tick, so that the sound reflects current Source.
10. As a composer, I want a writer of the addressed pair that waits on the Read to be a diagnosed cycle, so that ordering never silently guesses.
11. As a composer, I want a Read to answer `**` as a Bang and a Function spelling as that Function, so that it behaves exactly as Copy and Track do.
12. As a composer, I want a partial pair, a Comment, or Cells straddling two Language Units to diagnose, so that malformed data is located at the Read.
13. As a performer, I want an address past the Grid's edge to diagnose and write nothing, so that a misplaced voice is reported rather than wrapping.
14. As a composer, I want a nested Read to return its pair to its parent's slot, so that Reads compose like any value Function.
15. As a composer, I want a nested Read of empty Cells to leave its parent invalid, so that unwritten operands follow ADR 0069.
16. As a reader of Source, I want the first glyph of a Function to say whether it reads, writes or copies Source, so that I can tell how its Turn is ordered.
17. As a reader of Source, I want an arrow to point at the Cell the Function acts on, in every family, so that I learn one direction vocabulary.
18. As a performer migrating from Orca, I want Orca's `J` and `Y` to be the Copy Functions with their behaviour unchanged, so that spatial chains keep working.
19. As a composer, I want to read and write at a Position, so that a shared table can be reached from anywhere in the Grid.
20. As a composer, I want to write a value in a direction or at a Position, so that Orca's `X` has an Orcvs counterpart.
21. As a composer, I want a Read nested in a Write to copy one pair to another place, so that Orca's `G` is a composition rather than a Function.
22. As a composer, I want an absolute Copy, so that copying between two Positions takes one 10-Cell Function rather than a 12-Cell composition.
23. As a composer, I want to write a value into the pair an index selects within a row, so that Orca's `P` has the write counterpart of Track.
24. As a composer, I want the Turn that sees a dynamically addressed write to be stated, so that Read and Write agree on what a Tick shows.
25. As a performer, I want a working Source File and guide for the tracker with several voices, so that I can load it and see how Orca's patch maps onto Orcvs.
26. As a performer, I want Open, Save and reopen to preserve the tracker, so that the Source File reproduces it.
27. As a reader of the language docs, I want ADR 0019's table and the domain glossary to name every Read, Write and Copy, so that the Orca capability audit stays authoritative.

## Implementation Decisions

1. ADR 0070 is accepted, with its remaining proposed default confirmed or changed, before any spelling changes (ticket 07). On acceptance its amendments to ADRs 0008, 0019, 0049 and 0067 land as Status-line amendments.
2. Each Read, Write and Copy with operands is an ordinary Function in the Function table. Every operand is a Number: a literal, nested, or written by a Portal, and an unwritten one makes the Function invalid (ADR 0069).
3. Directional distances count Portals: one pair, two Cells, east or west, or one row north or south. Track's and Push's `index` count pairs. Absolute `$` columns are ADR 0049 Positions and count Cells.
4. Directional distances count from the `n` operand, with no special cases (ADR 0070). `00` is the operand itself, so every `&x 00` answers its own operand; `&>01C4` answers `C4`. A south Read reads the column below its operand, beside its Output Portal, and never crosses it. `&< 01` reads the Function's own spelling. A Write counts exactly as the Read with the same arrow does, so `@x 00 value` writes onto its own `n` operand and steps itself. A Copy steps one Portal, as the Jumps do.
5. A Position is ADR 0049's two Numbers, counted in Cells from `00 00`. A pair cut short by the row edge, or an address outside the Grid, diagnoses and writes nothing, as a Copy's Input Portal outside the Grid does.
6. Every Read, Track included, and the read of `=$`, has a dynamic Input Portal ordered by ADR 0032's rule completed at its Turn (ADR 0067): once its operands settle, every writer of the addressed pair that has not taken its Turn goes first. A writer that waits on the Read forms a cycle handled by ADRs 0065 and 0068. A Source holding one does not reuse the schedule built in advance, as a Source holding Track does not.
7. The directional Copies keep static Portals and are ordered before the Tick, as the Jumps are.
8. Reads read their pair with the Copy reading rules and have none of their own. They answer through the default Output Portal one row south and, nested, as their Return.
9. Every `@` form and the write of `=$` wait on a recorded decision on dynamic Output Portal ordering (ticket 04), written as an amendment to ADR 0067.
10. The two-axis relative Read `@< column row` is withdrawn. Its `PairSelection` rule and Turn-time ordering are adapted into the directional Reads (ticket 02), and `@<` becomes the west Write.
11. Query reads one Atom per Read; a multi-Atom Query is several Reads. Konkat's hidden variable lookup becomes a Read of visible Source.
12. A Source is read under the grammar current when it is opened; a Source File has no version to check (ADR 0054). Per ADR 0070's proposed default, the console's stored autosave is discarded once, with ADR 0054's report, after every respelling that changes a stored spelling's meaning has landed (ticket 12).
13. ADR 0070's acceptance updates ADR 0019's capability table and the domain glossary for every Function it names. Each ticket updates the Function reference and any shipped Source File for the Functions it ships or respells, keeping one authoritative language contract. Code names follow the glossary: Jump becomes Copy in identifiers, diagnostics and test modules.

## Testing Decisions

- The principal seam is the existing Source Tick interface: assert observable Source Cells, ordered Play Commands and diagnostics. Do not add test-only inputs to shipped code.
- The Function table's existing sweeps cover arity and operand binding for each new row.
- The Jump→Copy rename and Track's move are respellings: their existing tests pass with only spellings and names changed.
- Prior art: the Source Read tests in `orcvs/src/source/tick/read.rs`, Track's Turn-time ordering tests, Jump's reading-rule tests, the nested-Return and invalid-operand tests, and the reused-versus-fresh schedule comparison.
- For each Read, cover literal, nested and Portal-written operands; offsets at zero, at the row edge and past the Grid in each direction; writers of the addressed pair before and after the Read in Grid order; a writer that waits on the Read; same-Tick writes to its operands; and empty, `**`, Function-spelling, partial, Comment and straddling pairs.
- Load the exact shipped tracker Source File and verify two complete loops for every voice through Source/Tick and Playback, including empty steps and Note Off timing. Assert each Tick's diagnostics exactly.
- Use real console input and file workflows for the example through the console edit-and-file checks `cell-tracker/05` establishes, following the egui skill.
- Record an audible MIDI smoke test with device, channels and BPM. A silent automated run does not replace it.
- Run proptest at 32 local cases. Broader platform and performance gates stay with CI.

## Out of Scope

- Orca's Variable `V`, which ADR 0019 omits entirely.
- UDP, OSC and the Application Command, which wait on a text or message value.
- Reading or writing at a relative diagonal offset.
- Absolute forms of Track and Push.
- Multi-Atom reads or writes, and any variable-width result.
- A dedicated tracker editor or moving playhead.
- Migrating Sources written under the old spellings.
- Changes to MIDI encoding, Timed Play lifetimes or Playback scheduling.
- New dependencies, unsafe code, feature combinations or performance claims.

## Further Notes

This is active pre-release language design with no public compatibility contract. Once ADR 0070 is accepted, the rename, Track's move and the directional and absolute Reads can proceed without waiting on the Write decision. The tracker example builds on `cell-tracker`, whose console verification (`cell-tracker/05`) it reuses.
