# 03 — Add Directional Bang movement

**What to build:** Implement `*^`, `*v`, `*<`, and `*>` plus Source-resident `^^`, `vv`, `<<`, and
`>>` Activation Characters using the representation selected by the focused prototype while
preserving the decided observable behavior.

**Blocked by:** 02 — Add Source Bang activation and expiry; activation-representation/01 — Prototype
the Activation representation.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] An active Directional Bang Function emits its matching Activation Character adjacent to itself.
- [ ] Activation Characters move one Cell per Tick while preserving their two-Cell encoding.
- [ ] Movement tests only newly entered Cells, not overlap with the current Footprint.
- [ ] Successful overlapping movement clears old Cells before writing new Cells.
- [ ] Blocked or out-of-Grid movement replaces the current Footprint with Bang.
- [ ] The selected representation does not accidentally grant Activation Characters ordinary
      Expression operand, Sequence, or automatically scheduled Function behavior.
- [ ] Complete root contact can activate; partial Language Unit contact diagnoses and activates nothing.
- [ ] Tick-by-Tick Source Grid tests cover all four directions and row edges.
