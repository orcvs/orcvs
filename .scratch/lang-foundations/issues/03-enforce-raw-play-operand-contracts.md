# 03 — Enforce Raw Play operand contracts

**What to build:** Make Raw Play accept only its documented channel Number, velocity Number, and
Note operands, including when nested evaluation or a direct evaluator call bypasses ordinary Source
parsing.

**Blocked by:** 02 — Make Function definitions compiler-checked; property-testing/07 — Make Number and Note Source encodings canonical.

**Status:** resolved

**Tags:** release/v1

- [x] Raw Play consumes a two-Cell channel Number, a velocity Number, and a Note in that order.
- [x] Channels outside `00`–`0F` and velocities outside `00`–`7F` diagnose and emit no command.
- [x] A Note in either Number slot and a Number or Char in the Note slot diagnose and emit no
      command.
- [x] Direct and nested evaluation enforce the same operand contract as Source parsing.
- [x] Broad Atom-to-byte coercion and dead conversion helpers are removed when no remaining caller
      has their loose semantics.
- [x] Existing valid Raw Play commands retain their exact MIDI bytes and ordering.

## Answer

Raw Play now rejects channels above `0F` at evaluation time, alongside its existing velocity and
typed operand checks. Behaviour-level coverage exercises Source, nested evaluation, and direct
evaluator calls. The unused permissive stack conversion from any Atom to String was removed.
