# 01: Partition a row by parse, not by whitespace

**What to build:** A row is divided into Expressions by the Parser rather than by its spaces. The
Parser is handed the row's remaining Cells together with the Cell index they start at, and reports
where the Expression it read ends; the walk resumes at the Cell after it. A space becomes an
ordinary empty Cell that no Language Unit covers, rather than a partition rule.

Two Language Units written flush against each other are two units. A run of standalone Atoms such
as `**^^` is as many Expressions as the Parser finds, so the assembly path that built one Expression
from the partition's units — and the check that those units tile the Span end to end — has nothing
left to do and is deleted with it. The reconstruction that restored a trailing-content verdict by
adding a Span's start back to a consumed length goes too, because there is no longer a fragment
whose offsets need re-basing.

The Parser needs exactly one fact from the Grid: how wide a row is, so that a two-Cell spelling
cannot be read across a row edge. The Comment rule survives unchanged — nothing after `##` on a row
is Source — and stays stated in one place.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The walk advances by what the Parser read, and spaces are no longer a partition rule.
- [ ] The Parser receives the row's Cells and the Cell index they start at, and no Source bytes are
      copied into a fresh string per Expression.
- [ ] Two Language Units written with no space between them are established as two units.
- [ ] `**^^` establishes the Expressions the Parser finds, not one assembled from units.
- [ ] The standalone-run assembly path, its tiling check, and the caller-asserted consumed width it
      used are deleted.
- [ ] The trailing-content reconstruction is deleted, and the verdict it restored is unchanged for
      every Source that had one.
- [ ] A two-Cell spelling is never read across a row edge.
- [ ] The rebuild-equivalence property still holds: a rebuilt Map equals the Map a full build would
      have made.
- [ ] The parse boundary rule is recorded in an ADR beside the Comment rule.

## Comments

This absorbs `language-map/08`, which describes the same change. That ticket should be closed as
absorbed rather than worked separately.
