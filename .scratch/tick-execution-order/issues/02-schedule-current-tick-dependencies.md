# 02 — Schedule and execute current-Tick dependencies

Status: resolved
Tags: release/v1
Blocked by: 01

**What to build:** Replace the provisional row-major Tick traversal with the bounded dependency scheduler from [ADR 0032](../../../docs/adr/0032-schedule-tick-execution-by-dependency.md), behind the existing Source Tick interface.

- [x] Establish data and activation dependencies from parser-owned input positions and fixed output destinations before executing roots.
- [x] Order each root once; use row-major position only to break ties between independent ready roots. Keep nested evaluation inside the containing Expression.
- [x] Make current-Tick operand encodings visible before dependent evaluation, including supplied missing operands. Do not silently substitute old data when a supplier fails.
- [x] Wait for potential Bang producers to settle even when they produce no Bang. Empty results perform no write and retain ordinary data.
- [x] Activate only from current-Tick Function results. Discard valid prior `**` display without activation; manually entered `**` is a no-op. Never clear invalid operand spellings as display.
- [x] Demonstrate Note C4 plus Bang playing C4 during T with producers above and below MIDI, including Bang producer `(0,4)` through Portal `(0,3)` to MIDI `(0,2)`.
- [x] Verify no replay on T+1 after the producer stops, and that multiple fresh neighboring Bangs do not execute one root twice.
- [x] Diagnose competing writers, same-Tick cycles, partial operand projections, and unsupported writes to Function structure before committing the graph's effects. For this initial scope graph errors reject the Tick, as in the accepted prototype.
- [x] Retain whole-write admission, deterministic command order, atomic publication, and the Source/Playback separation. Keep ordinary result encoding and Sequence result admission helpers coherent; do not claim variable-width scheduling is implemented.
- [x] Remove obsolete Bang cleanup turns and repeated whole-Source reparsing from the execution path rather than layering scheduling over the fallback.

Use existing below-root routing in normal Source execution and the internal Portal destination mechanism to verify fixed upward routing. This ticket adds no invented Portal authoring syntax. Run the `orcvs` scoped gate and behavioral tests through Source/Tick execution.

## Comments

2026-09-06: The HTML prototype proves the bounded scheduling model; it is not production code. `orcvs/src/source/tick.rs` still contains the earlier row-major fallback. No production scheduler implementation has begun. This ticket owns the timing/lifetime replacement required by `language-map/06` and `spatial-tick-planning/02`.

2026-09-06: Resolved for ADR 0032's initial scope. Tick planning now fixes candidates and scalar destinations, validates graph conflicts, topologically orders data and activation edges, binds current operands, evaluates every root at most once, and publishes atomically. Fixed upward routing at `Y-1`, below-root routing, same-Tick Note plus Bang delivery, false-result absence, no replay, duplicate activations, competing writers, and cycles have regression coverage. Variable-width projections and authorable arbitrary Portals remain follow-up scope.

2026-09-08: ADR 0034 integration evidence is captured on local throwaway branch
`prototype/integrated-live-execution-adr34` at
`lang/prototypes/integrated-live-execution/integrated-live-execution.prototype.html`.
The bounded model and presentation handlers support 24 exercised cases with derived ordering,
pending spatial encodings, typed nesting, suppression, replacement, failure and next-Tick parse.
No contract contradiction was observed in those cases. Browser visual verification is incomplete
because browser security blocked opening the local artifact. See the adjacent README and HANDOFF
for evidence and scope. This does not reopen this resolved ADR 0032 ticket or change production:
its failed-supplier regression still implements the earlier policy. Continue through specification
and follow-up tickets after assessment. The HTML remains throwaway primary-source evidence.

2026-09-08: Resolved delivery history retained. [ADR 0034](../../../docs/adr/0034-execute-against-live-typed-expressions.md)
now revises the old binding and nested scheduling seam, partial/competing-write restrictions,
failed-spatial-supplier suppression, and original-anchor replacement limits. Initial partitioning,
Bang lifetime, activation gating, deterministic ordering, cycle atomicity and terminal behavior
remain in force. Production successor and regression mapping: [live typed execution](../../live-typed-execution/evidence.md).
