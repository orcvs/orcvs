# 02 — Broadcast Atomic Functions over Sequences

**What to build:** Apply compatible stateless Atomic Functions pervasively across Sequence operands
using ADR 0007's scalar and equal-length rules.

**Blocked by:** 01 — Add the Sequence language value; orcvs-language-migration/05; orcvs-language-migration/07.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Atom/Atom evaluates once and returns the ordinary result.
- [ ] Atom/Sequence and Sequence/Atom repeat the scalar across every element.
- [ ] Equal-length Sequences pair element-wise in order.
- [ ] Unequal non-scalar lengths diagnose and return no partial Sequence.
- [ ] Per-element type or evaluation failure diagnoses the complete operation.
- [ ] Unary `.v` and `.^` conversions broadcast atom-wise, preserve order, and return no partial
      Sequence when any element fails conversion.
- [ ] Equality remains ADR 0011's whole-value predicate: it produces one scalar Bang only when
      every broadcast pair is equal, otherwise no value; it never creates absent Sequence elements.
- [ ] Increment and Interpolation are not accidentally broadcast.
