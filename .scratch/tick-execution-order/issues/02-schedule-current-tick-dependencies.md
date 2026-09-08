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
