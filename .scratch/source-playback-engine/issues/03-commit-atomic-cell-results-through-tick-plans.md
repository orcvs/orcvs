# 03 — Commit atomic Cell results through Tick Plans

**What to build:** Interpret every Expression from one Tick snapshot and atomically commit the resulting Cell writes, producing a deterministic Tick Plan and consistent post-Tick Source revision even when some Expressions fail.

**Blocked by:** 02 — Keep Expressions horizontal and diagnosable.

**Status:** ready-for-agent

- [x] Every Tick evaluates all Expressions from the same pre-Tick Source snapshot.
- [ ] Cell writes commit together only after interpretation completes; earlier writes cannot change another Expression's input within that Tick.
- [x] Multi-Cell values write their complete encoding horizontally, with no first-byte truncation.
- [ ] A multi-Cell value that cannot fit entirely before the row edge is discarded and reported; an output below the bottom row is likewise discarded and reported.
- [ ] Overlapping results are applied at Cell granularity in Source order, so later Expressions win only the Cells they overlap.
- [ ] An Expression evaluation failure suppresses only that Expression's result, records its diagnostic, and does not block unrelated results.
- [ ] The returned snapshot and change set describe the fully committed post-Tick Source revision.

## Comments

**2026-08-26 — partially satisfied ahead of schedule while fixing review findings (agent)**

A code review of branch `make-source-edits-synchronous-and-consistent` found four correctness bugs in `Source::execute` that fall inside this ticket's scope. They were fixed in place rather than deferred, because two of them wrote visibly wrong values into the Source. The Tick Plan type was **not** built and `Command::Tick`/`SourceCommander` were not restructured — the work stayed inside the existing `execute()` shape.

Per acceptance criterion:

1. **Same pre-Tick snapshot — satisfied.** Verified rather than assumed: every result is `collect()`ed from `parsed` before the first write. Pinned by `test_every_expression_evaluates_from_the_same_pre_tick_snapshot`.
2. **Writes commit together — half.** "Earlier writes cannot change another Expression's input" holds and is tested. "Commit together" is still structurally open: writes are applied one `set` at a time. No observer can see an intermediate state today (`&mut self`, and `SourceCommander` holds the write lock across the whole call), so it is atomic in effect but there is no single commit step.
3. **Multi-Cell, no first-byte truncation — satisfied.** `execute` writes the result's complete encoding one Cell per character. Previously `++0102` wrote `0` instead of `03`; with hexadecimal rendering `Number(10)` would have written `0` instead of `0A`.
4. **Row-edge and below-bottom discards, reported — half.** Both discards are implemented and tested; the previous code clamped every bottom-row result onto the last Cell, overwriting whatever the user had there. "Reported" is a `debug!` line only — no diagnostic reaches a caller.
5. **Overlapping results at Cell granularity in Source order — holds by construction, unverified.** Results apply in ascending start order, one Cell at a time. Currently unreachable: Expression starts in a row are at least two Cells apart and every encoding is at most two Cells wide, so two results cannot yet overlap. No test — coverage is not claimed.
6. **Failure suppresses only its own result — half.** Suppression and non-blocking hold and are tested. Errors were previously swallowed with no logging at all; they now `warn!`. "Records its diagnostic" remains open.
7. **Post-Tick snapshot and change set — open.** `execute()` still returns `()`.

Also fixed here, adjacent to this ticket: committed results used to feed themselves, cascading one row further down the grid on every Tick forever. A Tick now commits a result only for an Expression that contains a Function, and never commits `Atom::Empty` (the absence of a result). Both guards are needed — `Atom::Empty` alone does not stop an incomplete Function such as a bare `id` from writing `_`.

**Residual for this ticket, needs Cell provenance** (distinguishing a Cell written by `execute` from user Source — deliberately not built):

- Deleting a source Expression leaves its last committed result on the grid permanently.
- A result that becomes narrower leaves a stale trailing Cell: `++0102` commits `03`, then replacing it with `id7` leaves `73`.

