# 03 — Add Directional Bang movement

**What to build:** Implement `*^`, `*v`, `*<`, and `*>` plus the root-only Self-Banging Functions
`^^`, `vv`, `<<`, and `>>` using the representation selected by the focused prototype.

**Blocked by:** 02 — Add Source Bang activation and expiry; activation-representation/01 — Prototype
the Activation representation.

**Status:** ready-for-agent

**Tags:** release/v1

**Sources of truth:** `CONTEXT.md` defines Self-Banging Function and Portal; ADR 0006 defines
intrinsic Bang activation; ADRs 0004 and 0009 define validated Portal effect bundles; ADR 0020
defines producer and emission order.

- [ ] An active Directional Bang Function emits its matching Self-Banging Function adjacent to itself.
- [ ] A Self-Banging Function in the Source Snapshot receives intrinsic Bang activation at its turn;
      the activation event does not write `**`.
- [ ] Each Self-Banging Function moves one Cell per Tick while preserving its two-Cell spelling.
- [ ] Movement tests only newly entered Cells, not overlap with the current Footprint.
- [ ] Successful movement preflights one complete Portal bundle, then writes spaces over the old
      Footprint before writing the Function spelling at the shifted destination.
- [ ] Blocked or out-of-Grid movement replaces the current Footprint with Bang.
- [ ] Self-Banging Functions remain root-only Source effects, not operands, runtime values, or
      Sequence members.
- [ ] A generated Self-Banging Function first receives a turn from the next Source Snapshot.
- [ ] Complete root contact can activate; partial Language Unit contact diagnoses and activates nothing.
- [ ] Tick-by-Tick Source Grid tests cover all four directions and row edges.
