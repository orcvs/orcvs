# 12 — An operand type whose blank means a fixed value

**What to build:** A Function can take an operand that a musician may leave blank, and blank means a
stated value.

**Blocked by:** 01.

**Status:** resolved

- [x] An operand type gives blank a fixed value, beside ADR 0108's `Optional<O>`.
- [x] A new ADR records it, grounded in ADR 0108's deferral of a default for an empty operand.
- [x] The glossary describes the operand type.
- [x] Tests cover a blank operand reading the fixed value and a written operand reading what is written.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
