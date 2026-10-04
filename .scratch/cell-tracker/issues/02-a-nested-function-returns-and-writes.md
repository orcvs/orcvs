# 02 — A nested Function returns its encoding and still writes south

Status: ready-for-agent
Blocked by: 01

**What to build:**

Implement ADR 0061 so nesting uses the same receiving-operand interpretation
as spatial delivery and nested feedback keeps advancing through Source.

## Acceptance criteria

- [ ] Nested arithmetic writes each result under its own anchor and supplies the parent with the same encoding.
- [ ] Identical Note and Number characters delivered by Return or Portal decode identically at the receiving operand, including invalid encodings and diagnostics. Check `.v.vC4` and `.^.^3C` diagnose from receiver-context decoding, while `.+.^3C01` reads returned `C4` as Number `C4`; conversions do not pass through the child type.
- [ ] Replace the pass-through assertions in `nested_conversions_compose_and_stay_idempotent` (`lang/src/functions/numeric_conversion.rs`, the `.^` of `.^3C` and `.v` of `.vC4` cases) with the diagnostics ADR 0061 now requires, keep the composing `.v.^3C` and `.^.vC4` cases; the conversion bodies no longer need an arm for an already-converted value.
- [ ] Nested Increment and Interpolation retain their state through their own Output Portals across multiple Ticks.
- [ ] A nested Bang writes its display and activates the aligned roots specified by ADR 0006.
- [ ] The Parser and Language Map refuse a nested effect Function from Source alone, without waiting for a Tick.
- [ ] Reservations include nested Output Portals and order their consumers through the existing Source Tick path.
- [ ] A refused spatial destination preserves a valid Return; parent failure does not undo a successful child write.
- [ ] Update existing nested-projection and replacement regressions to assert the new south writes through observable Source/Tick behavior.
- [ ] The domain glossary, Function declarations and Function reference agree with the implemented behavior. Preserve both ADR 0060's placement-admission amendment and this nested-Return amendment on ADR 0034 when integrating the mover branch; update ADR 0060's typed-nested-input wording to follow ADR 0061.
