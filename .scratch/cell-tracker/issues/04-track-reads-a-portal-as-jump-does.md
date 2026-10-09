# 04 — Track reads a Portal as Jump does

Status: resolved
Blocked by: 02, 03

**What to build:**

Implement [ADR 0067](../../../docs/adr/0067-track-reads-a-portal-as-jump-does.md)'s Track:
`@t index count`, an ordinary value Function with an Input Portal `index % count`
pairs east of its last operand, read with Jump's rules. Track claims nothing
beyond its operands, and the Cells it reads are ordinary Source. Its read is
ordered by ADR 0032's existing rule, completed at Track's Turn.

## Acceptance criteria

- [x] `@t` is a value Function in the Function table with Number operands `index` and `count`. It parses, nests and paints as any other Function; the Parser has no List-specific rule.
- [x] At its Turn Track reads the pair `index % count` pairs east of its last operand, through the same Portal read Jump uses: `@t0103C4D4E4` writes `D4` south. This holds with a nested `index` or `count`, where the last operand ends further east.
- [x] Track answers what it reads exactly as a Jump does: empty Cells are copied, clearing the Output Portal and leaving a nested parent pending (ADR 0066); `**` relays a Bang that activates a root at the Output Portal; a Function spelling answers that Function, so Function Replacement applies; a Number or Note answers that Atom; a partial pair, a Comment or Cells straddling two Language Units diagnose as partial or invalid input.
- [x] `index` and `count` are ordinary operands: literal, nested or written by a Portal during the Tick, and an empty one leaves Track pending. A `count` of `00` diagnoses at the Turn. A selected pair past the row edge diagnoses.
- [x] A Clock-driven Track selects the expected pair at Tick zero, holds it for the Clock's rate, wraps at the count and continues beyond Tick 255.
- [x] Ordering: once `index` and `count` settle at Track's Turn, every writer of the selected pair that has not taken its Turn goes first, whether it stands before or after Track in Grid order. Cover a Clock writing the index, a writer of the selected pair east of and below Track, and a writer of an unselected pair, which is not ordered against Track.
- [x] A writer of the selected pair that waits on Track forms a same-Tick dependency cycle diagnosed under ADR 0065; the rest of the Tick still runs.
- [x] The schedule built before the Tick is unchanged for every other Function, and schedule reuse still gives the same result as a freshly built schedule for Sources holding Track.
- [x] A nested Track supplies the pair it read to its parent and also writes its own Output Portal; a Timed Play whose note a nested Track supplies plays that Note.
- [x] `CONTEXT.md` gains a Track entry with no List vocabulary, and the Function reference shows Track.

## Comments

Rewritten for ADR 0067, which supersedes ADR 0063's List clauses. The earlier
version of this ticket (List claim, literal count fixed when the Source is
parsed, untyped Items) was implemented on PR #201 and abandoned.

Resolved by PR #205 (`681623a5`). The criteria are covered by one or more tests each in `orcvs/src/source/tick/track.rs`, the selection and `00` count in `lang/src/functions/track.rs`, and schedule reuse in `orcvs/src/source/tick/schedule_reuse.rs`. Commit `9380786d` then named Track's Input Portal dynamic and Jump's, Increment's and Interpolation's static.

2026-10-09: ADR 0069 superseded the pending clauses of criteria 3 and 4 (a nested parent left pending by empty Cells, and an empty `index` or `count` leaving Track pending). Under ADR 0069 an unwritten operand is invalid input, so Track with an empty `index` or `count` is invalid and diagnosed, and a nested Track that copies empty Cells leaves its parent invalid. That was implemented by `.scratch/pending-presentation/issues/01-treat-an-unwritten-operand-as-invalid-input.md`. The criteria above record what this ticket delivered under ADR 0066.
