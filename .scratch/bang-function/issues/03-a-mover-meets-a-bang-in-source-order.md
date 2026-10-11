# 03 — A mover meets a Bang in Source order

Status: resolved
Blocked by: 01, placement-semantics/03

**What to build:**

Pin D13 from ticket 07. A mover or emission reaching a Bang Function's Cells has no Bang-specific edge, so Source order decides under ADR 0060's rule 3b, as for a mover train. This matches Orca's reading order.

## Acceptance criteria

- [x] `**<<` and a `^^` south of a `**`: the `**` fires first and the mover enters the vacated Cells (E6).
- [x] `>>**` and a `vv` north of a `**`: the mover is blocked and writes `**`; the typed `**` still fires its aligned roots that Tick; the mover's `**` fires on the next (E6). The odd-gap test (`tick.rs:1646`) is flipped.
- [x] A writer covering the `**` and a mover entering its Cells order as writer-before-covered dictates, with the outcome asserted.
- [x] A locked `**` blocks a mover in every direction.
- [x] CONTEXT.md's Self-Banging Function entry states that a mover meets a Bang Function in Source order.

## Comments

2026-10-11 — Pinned in `orcvs/src/source/tick/bang_function.rs`; no code change beyond ticket 01. The vertical criteria were corrected with D13: Source order is row-major, so a `vv` above a `**` precedes it and is blocked, and a `^^` below follows it and enters. E6 and the flipped odd-gap test cover the horizontal cases. A writer covering the `**` goes first, suppresses it, and blocks the mover entering its Cells with what it wrote. A locked `**` blocks movers from the east, the west and the south; its north side is the Halt's.
