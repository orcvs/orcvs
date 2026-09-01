# 02 — Make Function definitions compiler-checked

**What to build:** Make each real Function one compiler-checked definition of its canonical
two-Cell spelling and fixed operand signature, so adding a Function cannot leave parsing,
rendering, signature lookup, enumeration, or dispatch silently incomplete.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Parsing and rendering use the same canonical spelling definition.
- [ ] Every real Function has exactly one fixed operand signature.
- [ ] The complete real-Function enumeration cannot silently omit a newly added Function.
- [ ] Signature lookup and evaluator dispatch are exhaustive and contain no wildcard fallback for a
      real Function.
- [ ] Every enumerated Function parses from its spelling and renders back to the same spelling.
- [ ] Parser benchmark commands and before/after results are recorded; measured forced-inlining
      choices are retained unless the measurements justify a targeted change.

## Comments

Parser recovery sentinels are removed separately after Expression entries become structural. They
must not be treated as real Functions or included in the canonical enumeration.
