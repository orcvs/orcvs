# 03 — Settle the read footprint and dependency contract

Status: needs-triage
Blocked by: 01, 02

## Problem

Positions are settled, but a computed read address, count or step may not be
known when the Tick schedule is built. Adding a late Source lookup would miss
current-Tick writers or read them in Grid order.

## Work

Specify how the chosen read contributes dependencies before execution. Evaluate
a declared bounded region with dynamic selection inside it before considering
general computed geometry. If all possible read positions are reserved, state
the resulting conservative edges and possible cycles. If geometry is restricted,
give unsupported input a precise diagnostic. Any relaxation of ADRs 0032/0034/0036
must be explicit; do not silently read the previous Snapshot as a fallback.

## Acceptance

- [ ] Address, index, count and stride suppliers have a defined evaluation order.
- [ ] Reading a Note written earlier in dependency order observes that Tick's
      surviving encoding, including partial and competing writes.
- [ ] Writers above and below the reader produce equivalent musical results.
- [ ] Bounds and span arithmetic are widened before validation; a two-Cell step
      starting at column `FF`, a region crossing a row, and zero extent are covered.
- [ ] A read overlapping its output, a read/write cycle, an out-of-Grid region,
      and changes to the extent during Playback have defined outcomes.
- [ ] Failed and absent writers follow the surviving-Source rules; typed nested
      failure remains distinct from surviving spatial characters.
- [ ] Publication stays atomic and roots execute at most once per Tick. There
      is no hidden cross-Tick cache of notes or an unbounded rescheduling loop.

Primary seams: `orcvs/src/source/tick.rs`, `source/portal.rs`,
`source/tick/execution.rs`, `lang/src/tick.rs`. Choose the bounded contract
before changing these mechanisms.
