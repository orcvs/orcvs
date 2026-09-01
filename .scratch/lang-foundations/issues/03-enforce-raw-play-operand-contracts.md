# 03 — Enforce Raw Play operand contracts

**What to build:** Make Raw Play accept only its documented channel Number, velocity Number, and
Note operands, including when nested evaluation or a direct evaluator call bypasses ordinary Source
parsing.

**Blocked by:** 02 — Make Function definitions compiler-checked; property-testing/07 — Make Number and Note Source encodings canonical.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Raw Play consumes a two-Cell channel Number, a velocity Number, and a Note in that order.
- [ ] Channels outside `00`–`0F` and velocities outside `00`–`7F` diagnose and emit no command.
- [ ] A Note in either Number slot and a Number or Char in the Note slot diagnose and emit no
      command.
- [ ] Direct and nested evaluation enforce the same operand contract as Source parsing.
- [ ] Broad Atom-to-byte coercion and dead conversion helpers are removed when no remaining caller
      has their loose semantics.
- [ ] Existing valid Raw Play commands retain their exact MIDI bytes and ordering.
