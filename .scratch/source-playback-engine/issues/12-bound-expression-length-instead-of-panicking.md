# 12 — Bound Expression length instead of panicking

**What to build:** Give an over-long Expression a diagnosis rather than a panic. Parsing has a fixed atom capacity, and a contiguous run of Cells that exceeds it aborts mid-edit — a crash reachable by ordinary typing.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] An Expression that exceeds the parser's atom capacity is diagnosed and leaves the Source unchanged; no edit path panics.
- [x] The edit error type covers this rejection, and the guarantee that a rejected edit never mutates the Source holds on this path too.
- [x] A Tick over an over-long Expression suppresses only that Expression and neither panics nor poisons the lock guarding the Source.
- [x] Tests reach the capacity from the Source interface by editing, not by constructing atoms directly.

## Notes

Found by review of the Source edit work, not yet triaged by a human.

Reproduction: 32 consecutive `+`, or 64 consecutive `id`, in contiguous Cells. Expressions currently span row boundaries (issue 02), and the default Source is 256 contiguous Cells, so the run is easy to reach.

Two things make this worse than it was. Editing now runs on the caller's thread, so the panic unwinds through the render loop. And when it happens during a Tick it poisons the lock, after which every subsequent read on the render path panics too.

## Answer

Parser accumulation is now fallible and reports a bounded-capacity syntax error instead of panicking. Source preflights the prospective row Expression before mutation and returns `SourceError::ExpressionTooLong` with its Cell range and capacity. If an over-long Expression enters through a non-edit path, rebuilding records its diagnostic and omits its parsed Atoms, so a Tick suppresses it safely. Regressions type to the limit through Source and SourceCommander, verify the rejected revision is unchanged, and prove later reads, edits, and Tick execution still work without a poisoned lock.
