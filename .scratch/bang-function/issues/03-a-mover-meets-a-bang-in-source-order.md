# 03 — A mover meets a Bang in Source order

Status: ready-for-agent
Blocked by: 01, placement-semantics/03

**What to build:**

Pin D13 from ticket 07. A mover or emission reaching a Bang Function's Cells has no Bang-specific edge, so Source order decides under ADR 0060's rule 3b, as for a mover train. This matches Orca's reading order.

## Acceptance criteria

- [ ] `**<<` and a `vv` north of a `**`: the `**` fires first and the mover enters the vacated Cells (E6).
- [ ] `>>**` and a `^^` south of a `**`: the mover is blocked and writes `**`; the typed `**` still fires its aligned roots that Tick; the mover's `**` fires on the next (E6). The odd-gap test (`tick.rs:1646`) is flipped.
- [ ] A writer covering the `**` and a mover entering its Cells order as writer-before-covered dictates, with the outcome asserted.
- [ ] A locked `**` blocks a mover in every direction.
- [ ] CONTEXT.md's Self-Banging Function entry states that a mover meets a Bang Function in Source order.
