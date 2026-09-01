# 02 — Broadcast Atomic Functions over Sequences

**What to build:** Apply compatible stateless Atomic Functions pervasively across Sequence operands
using ADR 0007's scalar and equal-length rules.

**Blocked by:** 01 — Add the Sequence language value;
`.scratch/orcvs-language-migration/issues/07-complete-the-numeric-function-family.md`.

**Status:** ready-for-agent

- [ ] Atom/Atom evaluates once and returns the ordinary result.
- [ ] Atom/Sequence and Sequence/Atom repeat the scalar across every element.
- [ ] Equal-length Sequences pair element-wise in order.
- [ ] Unequal non-scalar lengths diagnose and return no partial Sequence.
- [ ] Per-element type or evaluation failure diagnoses the complete operation.
- [ ] Unary `.v` and `.^` conversions broadcast atom-wise, preserve order, and return no partial
      Sequence when any element fails conversion.
- [ ] Equality can produce empty element results without inventing an Atom.
- [ ] Increment and Interpolation are not accidentally broadcast.
