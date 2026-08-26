# 03 — Commit atomic Cell results through Tick Plans

**What to build:** Interpret every Expression from one Tick snapshot and atomically commit the resulting Cell writes, producing a deterministic Tick Plan and consistent post-Tick Source revision even when some Expressions fail.

**Blocked by:** 02 — Keep Expressions horizontal and diagnosable.

**Resolution:** implemented

- [x] Every Tick evaluates all Expressions from the same pre-Tick Source snapshot.
- [x] Cell writes commit together only after interpretation completes; earlier writes cannot change another Expression's input within that Tick.
- [x] Multi-Cell values write their complete encoding horizontally, with no first-byte truncation.
- [x] A multi-Cell value that cannot fit entirely before the row edge is discarded and reported; an output below the bottom row is likewise discarded and reported.
- [x] Overlapping results are applied at Cell granularity in Source order, so later Expressions win only the Cells they overlap.
- [x] An Expression evaluation failure suppresses only that Expression's result, records its diagnostic, and does not block unrelated results.
- [x] The returned snapshot and change set describe the fully committed post-Tick Source revision.
- [x] A Tick Plan commits its Cell writes through a path that does not reparse per character, so no Expression is reparsed and no parse state is mutated part-way through a Tick.

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

**2026-08-26 — a Tick commits through the full public edit path (review)**

Found by code review of the branch that implemented issue 10; the behaviour is pre-existing and belongs here.

`Source::execute` (`console/src/source/source.rs`) writes each character of a result with `self.set(idx + i, &c.to_string())` at line 326 — the same public entry point a user edit takes. Every character therefore runs `unparse_around`, `set_source`, an `ExpressionMap` update, `reparse_span`, and a full-grid scan to build a `Change` that is then dropped. Parse state is mutated repeatedly *during* a Tick, and a two-Cell result reparses its neighbourhood twice.

The precomputation claim was checked rather than assumed: lines 256-264 build `results` with a `.collect()` over `self.parsed` before the write loop at line 266 opens, so every Expression is interpreted from the pre-Tick snapshot and no committed Cell can feed an interpretation in the same Tick. That is what makes the current code correct — not the commit path itself.

So this is not a wrong-output defect today; it is cost and fragility. The cost is O(characters x reparse) per Tick. The fragility is that atomicity rests entirely on the precomputation: a later change that made any result depend on state read during the write loop would break it silently, with no test failing, because nothing in the commit path enforces that a Tick reads only the snapshot.

The Tick Plan is the place to fix it — a plan is already the complete set of writes, so committing it needs a path that applies Cells and reparses once for the whole Tick, not the per-character public `set`.

**2026-08-26 — implemented (agent)**

`Source::execute` now returns a `TickResult`: its deterministic `TickPlan`, the fully committed Source snapshot, and the Cells whose content or glyph classification changed. Interpretation builds the entire plan from the pre-Tick parsed state. Cell writes are resolved by target index in Source order, so a later Expression replaces only overlapping Cells, then every final write is applied directly before derived Expression state is rebuilt once. The plan includes its ordered Play Command list as required by the domain model; it remains empty until issue 04 adds Play Function interpretation.

Evaluation failures and out-of-Source results are range-addressed diagnostics in the plan. A below-bottom result is covered through the public Source seam together with its empty change set; the row-edge guard remains defensive because, after issue 02 confined Expressions to rows, no current complete Expression can start late enough for its at-most-two-Cell result to cross an edge. The same current width and spacing constraints make overlapping results unreachable, while the plan's per-Cell ordered insertion defines the required outcome when the language can produce one.
