# Read, Write and Copy are separate families

Status: accepted. Amends [ADR 0008](0008-preserve-symbolic-arithmetic-functions.md)'s `&` and `@` families, [ADR 0019](0019-map-orca-capabilities-onto-orcvs-language-families.md)'s rows for Jump, Track, Push, Read, Query, Write, Generator and Konkat, [ADR 0049](0049-a-position-is-two-numbers.md)'s placement of the absolute Address Functions, and the wording of [ADR 0067](0067-track-reads-a-portal-as-jump-does.md), which names Jump and spells Track `@t`. [ADR 0071](0071-a-bang-travels-as-its-glyph.md) amends the `**` value: a Write writes `**` over its destination, as Orca's `X` does, and activates the roots aligned with it.

**A Function's family prefix names what it does to Source.** `&` reads Source, `@` writes Source, and `=` copies Source: it reads one Cell pair and writes another, leaving the first as it was. The action is what orders a Function's Turn: a Read waits on the writers of the Cells it reads, a Write is waited on by their readers, and a Copy does both. A reader can therefore tell from the first glyph which ordering rule applies.

**Jump is renamed Copy.** "Jump" came from Orca's `J`, named under a 26-letter limit. The operation reads one side and writes the other without clearing what it read, so it is a copy. The four Jumps `&^ &v &< &>` become the Copy Functions `=^ =v =< =>`. Their behaviour is unchanged: each writes the pair on its arrow's side, reads the pair on the opposite side, and has static Portals ([ADR 0014](0014-spatial-functions-preserve-performative-behaviour.md)).

**An arrow names the Cell the Function acts on.** A Read reads the pair on that side. A Write writes it. A Copy writes it and reads the opposite side, as the Jumps already did. The same arrows mean the same directions in every family: `*^`, `^^`, `=^`, `&^` and `@^` all point north.

| Action | Relative, by direction | Absolute, at a Position | Indexed along a row |
| --- | --- | --- | --- |
| Read `&` | `&^ &v &< &> n` | `&$ column row` | Track `&t index count` |
| Write `@` | `@^ @v @< @> n value` | `@$ column row value` | Push `@t index count value` |
| Copy `=` | `=^ =v =< =>` | `=$ src-column src-row dst-column dst-row` | — |

**A directional Read `&^ &v &< &> n` reads the pair `n` Portals from its `n` operand in its arrow's direction.** `n` is a Number operand like any other: a literal, a nested Function or Cells a Portal writes, and an unwritten one makes the Read invalid ([ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)). The Read reads that pair exactly as a Copy reads its Input Portal, answers it through its Output Portal one row south and, nested, returns it to its parent.

**A directional Write `@^ @v @< @> n value` writes `value` to the pair `n` Portals from its `n` operand in its arrow's direction,** counted exactly as the Read with the same arrow counts.

**The absolute forms take `$` as their second glyph, in every family.** `&$ column row` reads the pair at that Position, `@$ column row value` writes `value` there, and `=$ src-column src-row dst-column dst-row` reads the first Position's pair and writes it to the second. A Position is ADR 0049's: two Numbers, column then row, each `00`–`FF`, counted in Cells from `00 00` at the top-left of the 256 by 256 Grid ([ADR 0054](0054-a-grid-is-always-256-by-256.md)). A pair that starts at column `FF` is cut short by the row edge and diagnoses, as a Copy's Input Portal outside the Grid does. In a spreadsheet `$` marks an absolute reference, as in `$A$1`, which a copied formula keeps fixed. That is the same relative-versus-absolute distinction on a grid of Cells. `$` already ends `!$`, the Application Command, and ADR 0008 allows an operation glyph to recur in another family. `=$` takes 10 Cells; the composition `@$ c r &$ c r` takes 12.

**Track moves from `@t` to `&t` and keeps its behaviour.** It reads, so it belongs to the Read family. `@t index count value` becomes Push, which writes `value` into one of `count` pairs on the row below it (see the amendment on Push's lane). Track's `index` counts pairs, as the directional distances count Portals. Track and Push have no absolute form.

**Directional distances count Portals from the `n` operand, with no special cases.** `00` is the `n` operand itself. Each step is one Portal: one pair, two Cells, east or west, or one row north or south, in the operand's columns. So `&>00C4` answers `00`, its own operand; `&>01C4` answers `C4`; and `&>02C4D4` answers `D4`. A Write counts exactly as the Read with the same arrow does, and a Copy still steps one Portal, as the Jumps do. The absolute `$` forms address ADR 0049 Positions, whose columns count Cells.

**A nested `n` is counted from the slot it occupies, with no special case.** In `&>.+0101C4`, `n` is `.+0101`: step `00` is the nested `.+`'s spelling, steps `01` and `02` are its operands, and step `03` is `C4`. This `n` answers `02`, so the Read answers the second `01`; `&>.+0102C4` answers `C4`. Steps that land inside a nested Function are visible in the Source and harmless. Track keeps its own rule and counts its list from the end of its last operand, nested Functions included, because `index` points into a list rather than measuring a distance.

These consequences are intended:

- Every `&x 00` answers its own operand.
- A south Read never crosses its own Output Portal: the column below the operand lies beside the Output Portal, which is below the Function's spelling.
- `&< 01` reads the Function's own spelling, so it writes a copy of that Function.
- `@x 00 value` writes onto its own `n` operand, so on the next Tick it reaches a different distance. The Write steps itself.

**Generator stays a composition:** a Read nested in a Write's `value` operand.

**Ordering is unchanged.** Every Read, Track included, has a dynamic Input Portal resolved at its Turn under ADR 0067's amendment to [ADR 0032](0032-schedule-tick-execution-by-dependency.md): once its operands settle, every writer of the pair it reads that has not taken its Turn goes first, and a writer that waits on it forms a cycle handled by [ADR 0065](0065-an-error-never-stops-the-performance.md) and [ADR 0068](0068-a-discovered-cycle-preserves-completed-turns.md). The directional Copies keep their static Portals. `=$` reads through a dynamic Input Portal ordered by the same rule. How a dynamic Output Portal is ordered, for every `@` form and the write of `=$`, is decided by ADR 0067's amendment on dynamic Output Portals. What a Write's `value` carries is decided by the amendment below.

## Sources written with the old spellings

**Sources written with the old spellings are read under the new grammar, and a stored autosave is discarded once.** A Source File states Cells and no version (ADR 0054), so nothing in it can tell an old `&v` from a new one, and rejecting or flagging old spellings would need a version marker ADR 0054 rejected. Some old spellings diagnose under the new grammar: a lone `&v` is now a Read with an unwritten operand. Others are silently reinterpreted: `&<07` becomes a Read seven pairs west of its operand, `@<0201` becomes a Write, and `@t0103C4D4E4` becomes a Push that writes `C4` into the second pair of the row below it. The console's stored autosave is therefore discarded once, with the report ADR 0054 gives a Source it cannot load, after the respellings that change a stored spelling's meaning have all landed. Orcvs is pre-release, and every stored Source is a developer's autosave. The shipped Function reference and examples are rewritten by each respelling as it lands.

## Considered options

- **Keep reads and writes in `@`.** A second glyph cannot carry both a direction and an action, so eight directional forms and two absolute forms would not fit. ADR 0008 also read `@<` and `@>` as data flow, into or out of the Expression, while `*`, the Self-Banging Functions and the Jumps use arrows as Grid directions. Keeping both meanings would put two arrow vocabularies in one language.
- **A new write prefix, with `&` left for the Jumps.** `&` would stay an "address" family of four operand-less Functions, and the prefix would no longer tell a reader whether a Function reads Source, writes it or both, which is what decides how its Turn is ordered.
- **Letter glyphs for the directional reads**, such as `&n &s &e &w`. Every other family spells direction with arrows, so a reader would learn two direction vocabularies, and `v` would be an arrow in one family and a letter in another.
- **Fold the Jumps into the Reads,** as a Read with `n` of zero. A Copy writes in its arrow's direction with static Portals; a Read writes one row south through a dynamic Input Portal. Folding them either makes `&v` write over the column it reads or removes the directional write that Jump chains compose through (ADR 0014), and either makes every Jump wait at its Turn or needs a special case in ordering.
- **`"` (ditto) as the Copy prefix.** `"` is the likely delimiter for the text or message value that UDP, OSC and the Application Command wait for (ADRs 0008 and 0019). Spending it on a family would make that value ambiguous or push it to a less familiar delimiter.
- **Counting directional distances from the anchor.** `00` would land on the Function's own spelling, and `&v 01` would read the Function's own answer.
- **Counting a nested `n`'s distance from the end of the nested Function,** as Track counts its list. The pair a distance names would then move whenever the nested Function's width changed, and a distance is a measure from the operand, not an index into what follows it.
- **Counting directional distances in Cells.** An odd distance straddles a pair and diagnoses.
- **Skipping the Output Portal**, so that `&v 00` reads two rows south while `&^ 00` reads one row north. It makes north and south asymmetric.
- **`#` as the absolute glyph.** It looks like a grid but says nothing about absoluteness.
- **`@` as the absolute glyph.** `&@` reads as read plus write.
- **Doubled glyphs such as `&&`, `@@` and `==`.** `==` reads as equality beside `.=`, and doubled glyphs already mean other things: `^^` is a Self-Banging Function, `||` a Comment and `**` a Bang.

## Consequences

No form reads or writes at a relative diagonal offset. The two-axis relative Read `@< column row` built for ticket 02 is withdrawn; its selection and ordering code is adapted into the directional Reads rather than discarded. A pattern a voice reads lies in line with it, or is reached by Position.

ADR 0069's phrase "a Function that copies Cells" still covers every Function that moves a pair it read, Reads and Track among them. The Copy family is the narrower set whose arrow names the pair it writes.

A Function Replacement between two Functions with different Portal rules is refused as a change to the Source write it declares, so replacing a Read with a Copy or a Write is refused.

Other ADRs that name Jump, such as 0014, 0031, 0032, 0043, 0061 and 0069, are not rewritten. Jump in them means the Copy Function.

## Glossary changes

`CONTEXT.md` states these from acceptance, because the glossary names the language the ADRs define rather than only what is implemented.

- **Jump Function** becomes **Copy Function**: `=^ =v =< =>`, and `=$` for the absolute form. _Avoid_: Jump, Jumper, Jymper.
- **Source Read Function** becomes **Read Function**: the directional Reads `&^ &v &< &> n` and the Absolute Read `&$ column row`. _Avoid_: Source Read, Read Operator, Query Operator, Konkat Operator.
- New **Write Function**: the directional Writes `@^ @v @< @> n value` and the Absolute Write `@$ column row value`. _Avoid_: Source Write, Generator Function.
- **Track Function** is spelled `&t index count`.
- New **Push Function**: `@t index count value`, writing into a lane on the row below it.
- **Position** notes that the absolute forms take a Position as two Number operands.
- **Input Portal**, **Output Portal**, **Return**, **Bang** and **Operand Literal** say Copy where they say Jump, and the Input Portal entry lists every Read and `=$` as dynamic.
- **Output Portal** names static and dynamic Output Portals once ticket 04 decides their ordering.

## Amendment, 2026-10-10: a Write carries its value

**Reads and Writes have no opinions: they pass a pair through with the least ceremony possible.** A Write's `value` operand, and Push's, is the one operand that takes no type. It carries the two-Cell encoding it holds, unchanged, to the Write's destination, and whatever later reads that pair decodes it by its own type, exactly as if the pair had been typed there or a Copy had put it there. A Read already answers whatever pair it reads, and a nested Read returns it as an encoding the receiver decodes ([ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md)), so a Read nested in `value` passes any pair through: a Note, a Number, a Function spelling, which Function Replacement then applies to at the destination, or `**`, which the Write writes as a Bang. An empty `value` makes the Write invalid ([ADR 0069](0069-an-unwritten-operand-is-invalid-input.md)).

A Function spelling typed into `value` is a nested Function, as it is in every operand, so `@$0000.+0101` writes `02`; writing the spelling `.+` itself takes a nested Read of it. A `**` typed into `value` is an encoding like any other, so the Write writes a Bang at its destination each Turn it takes: a Write is a remote trigger, as Orca's `X` writing `*` is. A `**` typed into a typed operand stays invalid syntax.

### Considered options

- **A Number `value`.** A Read nested in it could pass on only the pairs that spell hex Numbers, so Generator would refuse Notes, Function spellings and Bangs, and a Write could suppress a Function but never replace it.
- **A Note `value`.** It refuses Numbers above the Note spellings and every Function spelling, the same limit in another shape.
- **A `value` that decodes whatever arrives, trying Function, Number and Note in turn.** It gives the Write an opinion about the pair it carries and a decode order to keep in step with the Parser, when the destination's reader decodes the pair anyway.

## Amendment, 2026-10-10: Push writes into the lane below it

**Push `@t index count value` writes `value` into the pair `index % count` pairs east of its default Output Portal:** a lane of `count` pairs on the row below Push, starting directly under its anchor. Pair `00` is the Output Portal one row south that every Function has by default; `index` slides the write east by whole pairs. So `@t0103C4` writes `C4` into Cells 2–3 of the row below it. This is Orca's `P`, whose `val` is the operand east of it and whose output lane is the row below, starting under `P`. A pair past the row edge, or a lane below the Grid, rejects the write and keeps the answer for a parent's Return, as every Write's destination does; a `count` of `00` diagnoses at the Turn, as Track's does.

**The lane is ordinary Source.** Push claims only its operands, as Track does (ADR 0067). Orca locks a Push's lane, so that operators standing in it do not run; Orcvs does not, and a Function standing in the lane takes its Turn and can be written over like any other.

**A lane below the Grid diagnoses as a lane, pair `00` included.** On the Grid's last row every pair of the lane is outside the Grid, and Push diagnoses that its result falls outside the Grid whichever pair `index` selects, rather than reporting pair `00` as a default Output Portal below the Source.

### Considered options

- **The pairs east of Push's last operand, `value`, on its own row.** It mirrors where Track's list lies, but it is not Orca's `P`, and it needs a counting rule of its own where the south default already gives one.
- **The pairs Track at the same anchor would read, counted from `count`.** Push's own `value` would be pair `00` of the lane it writes, and the lane would move whenever `value` held a nested Function.
