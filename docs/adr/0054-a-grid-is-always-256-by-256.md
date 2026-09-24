# A Grid is always 256 by 256

Status: accepted. Amends [ADR 0049](0049-a-position-is-two-numbers.md): its limit becomes the only shape. Amends [ADR 0047](0047-the-source-view-has-a-margin.md): the default Grid it sized is no longer a default.

**Every Grid is 256 columns by 256 rows.** ADR 0049 made 256 by 256 the most a Grid could be and left a smaller shape to each Source. There is now one shape. A Source no longer carries its dimensions, because there is nothing left for them to say, and every pair of Numbers `00 00`–`FF FF` names a Position.

**A Source File states Cells, never a shape.** A Source written out as plain text is its rows, one line per row, one character per Cell, a space for an empty Cell. Line *n* is row *n* and character *m* is column *m*. A short line, a missing line, or trailing whitespace an editor stripped all read back as empty Cells, so no edit to the file can change the Grid it opens on. Text past column 256 or row 256 is not a Source File.

[ADR 0043](0043-the-grid-is-bounded-by-definition.md) stands unchanged: there is still an outside, and a Jump or Self-Banging Function that reaches it behaves as that decision says. ADR 0049's rule that a pair of Numbers the Grid does not reach names nothing still holds; under this decision no such pair can be spelled.

## Rejected alternatives

**A Grid sized from its content.** Derive the dimensions from the widest line and the line count of a Source File, as the Function reference loader rounds them up today. The Grid would then depend on how the file was last edited: stripping trailing whitespace, or deleting the last row that held content, would shrink it, and moving content would move where the edge is.

**Each Source keeps its own shape.** The status quo. A stored Source is restored on the Grid it was stored with, so a console that has stored anything never sees a later default: `.scratch/file-new/` exists because a Source stored at 64 by 40 stayed at 64 by 40 after the default became 128 by 80. A shape per Source also forces every Source File to state its dimensions, which is the first thing a hand edit breaks.

**A header in the Source File.** State the dimensions in the first line and keep a shape per Source. It is a second syntax in a file that is otherwise the Source verbatim, and it answers a question this decision removes.

## Consequences

`CONTEXT.md` states the Grid as one fixed 256 by 256 shape and adds Source File.

A stored Source whose Grid is not 256 by 256 no longer loads. It is refused through the existing persistence path — set aside, noticed, and replaced by an empty Grid — rather than migrated. Orcvs is pre-release and every stored Source is a developer's autosave.

The Grid holds 65,536 Cells, 6.4 times the 128 by 80 default. Paint covers only the Positions the viewport covers ([ADR 0038](0038-the-console-owns-the-source-grid-transform.md)) and Tick planning is proportional to Expressions, but any path that walks every Cell — Language Map derivation, Source snapshots, the stored value — has to be measured at the new size rather than assumed cheap.

A fresh console opens on the top-left corner of the Grid. ADR 0047's framing of the default Grid as twice the default window no longer describes anything; its margin and Pan reach stand.

`File > New` no longer needs to reach a newer default Grid, which was most of `.scratch/file-new/spec.md`'s reason to exist. It still replaces the environment with an empty Source.
