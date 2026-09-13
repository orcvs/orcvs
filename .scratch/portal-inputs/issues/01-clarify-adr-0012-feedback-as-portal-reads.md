# 01 — Clarify ADR 0012 feedback as Portal reads

**What to build:** Revise ADR 0012 so Increment and Interpolation feedback is documented as a Portal
read from working Source at Turn — not as hidden cross-Tick state and not as a third
`TickInputs` channel alongside Tick. Align CONTEXT.md glossary wording if it still implies the
engine tracks "previous values."

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] ADR 0012 distinguishes Playback Tick inputs (Clock, Delay, Euclidean) from Portal feedback
      reads (Increment, Interpolation).
- [x] The revision states feedback is read through the ordinary result Portal in working Source,
      consistent with ADR 0004, ADR 0009, and ADR 0024.
- [x] CONTEXT.md glossary entries for Increment and Interpolation describe Portal input, not
      engine-tracked previous state, if the current wording implies otherwise.

## Comments

ADR 0012 now separates Playback Tick inputs from Portal feedback binding, cites ADRs 0004,
0009, and 0024, and states that the ordinary result Portal is read from working Source at Turn,
including earlier same-Tick writes. Increment and Interpolation glossary entries use the same
Portal-input wording. Arithmetic and scalar behavior remain the semantic baseline.
