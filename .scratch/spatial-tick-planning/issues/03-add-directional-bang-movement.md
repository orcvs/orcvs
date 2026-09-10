# 03 — Add Directional Bang movement

**What to build:** Implement `*^`, `*v`, `*<`, and `*>` plus the root-only Self-Banging Functions
`^^`, `vv`, `<<`, and `>>` using the representation selected by the focused prototype.

**Blocked by:** 02 — Add Source Bang activation and expiry; evaluation-machine/05 — Widen the Function kind to value or effect.

**Status:** ready-for-agent

**Tags:** release/v1

**Sources of truth:** `CONTEXT.md` defines Self-Banging Function and Portal; ADR 0006 defines
intrinsic Bang activation; ADRs 0004 and 0009 define validated Portal effect bundles; ADR 0020
defines producer and emission order.

- [ ] An active Directional Bang Function emits its matching Self-Banging Function into the two Cells
      immediately outside its own two-Cell Span in the selected direction.
- [ ] A Self-Banging Function in the Source Snapshot receives intrinsic Bang activation at its turn;
      the activation event does not write `**`.
- [ ] Each Self-Banging Function moves one Cell per Tick while preserving its two-Cell spelling.
- [ ] Movement tests only newly entered Cells, not overlap with the current Span.
- [ ] Successful movement preflights one complete Portal bundle, then writes spaces over the old
      Span before writing the Function spelling at the shifted destination.
- [ ] Stated destinations reach `computations` without a Terminal Output Function acquiring one:
      ADR 0009's refusal is raised in shipped code and covered by a test of its own, not by the
      `#[cfg(test)]` `carry` helper this work retires.
- [ ] Blocked or out-of-Grid movement replaces the current Span with Bang.
- [ ] Self-Banging Functions remain root-only Source effects, not operands, runtime values, or
      Sequence members.
- [ ] The Directional Bang Functions answer an effect rather than a value, so ADR 0025's single
      construction point refuses them by their declared kind, per ADR 0029.
- [ ] A generated Self-Banging Function first receives a turn from the next Source Snapshot.
- [ ] Complete root contact can activate; partial Language Unit contact diagnoses and activates nothing.
- [ ] Tick-by-Tick Source Grid tests cover all four directions and row edges.

## Comments

2026-09-10: `test-only-seams/08` deleted the scheduler `Configuration`, and with it the only shipped
raiser of ADR 0009's "a Terminal Output Function cannot have a Portal". The refusal now lives solely
in the `#[cfg(test)]` `carry` helper, whose doc says it goes away with the tests that needed it —
which is this ticket's work. So the change that first lets real input name a destination for a
Terminal Output Function is the same change that removes the last check for it, which is why the
acceptance line above is stated here rather than left to be noticed.

What holds the rule in the meantime is the ordering of `computations`' `if` chain: the
`performs_terminal_output()` arm comes first and hands back `vec![]`, so a Terminal Output Function
never reaches the arm that resolves a destination. A stated-destination arm placed before that gate
would admit the pairing silently. The gate itself is covered by one incidental test —
`a_late_spatial_write_rejects_the_tick_even_when_the_earlier_turn_failed` is the only test that
fails when its condition is replaced with `false`.
