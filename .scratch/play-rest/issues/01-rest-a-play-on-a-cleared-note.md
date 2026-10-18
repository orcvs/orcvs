# 01: Rest a Play whose note operand a nested Function cleared

**What to build:** A Raw, Timed or Monophonic Play whose note operand a nested
Function cleared sends no Play Command and raises no diagnostic. See
`../spec.md` and ADR 0075.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] The Function table declares `rest: note` on `!>`, `!~` and `!%`, and
      `Function::rest_operand` answers its index.
- [x] The Source tick rests a Turn whose rest operand a nested Function
      cleared, once every other operand decodes.
- [x] Each Play with a nested Read of blank Cells in its note slot sends
      nothing and raises no diagnostic.
- [x] A cleared channel, velocity or length, a Clear in Note to Number's
      operand, and a half-typed Note read into the slot still diagnose.
- [x] The nested-Track test in `outputs.rs` expects a rest.
- [x] ADR 0075 is written, ADR 0073's Status names it, and `CONTEXT.md`'s
      Return, Clear, Terminal Output and Rest entries match.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: Built on its own branch, apart from the tracker examples PR, whose
resting voices this quiets.
