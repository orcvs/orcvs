# 13 — Discard Tick results that overwrite their own Expression

**What to build:** Stop a Tick from destroying the Source it just read. An Expression's result is written one row below its start; when the Expression is wide enough to wrap a row, that destination can land inside the Expression's own Cells, silently overwriting Cells the user typed.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [ ] A result whose destination Cells overlap its own Expression is discarded and reported, never committed.
- [ ] The discard is observable as a diagnostic, not only as a log line.
- [ ] Tests cover an Expression wide enough for its result destination to fall within its own extent.

## Notes

Found by review of the Tick commit work, not yet triaged by a human.

Reproduction on a 10-column, 3-row Source: type `++0102id01A2` into the first twelve Cells, forming one Expression that wraps into the second row. The first Tick writes the result over the last two Cells, destroying the user-typed operands that are part of that same Expression's source. Stable across further Ticks, so it reads as data loss rather than as a divergence.

Issue 02 confines Expressions to a row, which removes the reproduction above. This ticket is the guard that holds regardless of extent, and is worth keeping after 02 lands.

## Answer

Resolved without a code change because Issue 02 made the reported state unreachable. Every Expression is confined to one row, while its result begins at the same column in the row below and must fit entirely within that row. Its result range therefore cannot overlap its own Expression range. The former reproduction is now two Expressions; preserving the existing rule that a later result may overwrite a different Expression is intentional Tick behavior.
