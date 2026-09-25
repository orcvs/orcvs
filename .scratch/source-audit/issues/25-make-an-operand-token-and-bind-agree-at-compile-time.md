# 25 — Make an operand's token and bind agree at compile time

**What to build:** A decision, then its implementation: the Function table's generated code makes a mismatch between an operand's declared token and its bind a compile error. Today fourteen cooperating macros generate the table (`define_functions!` and thirteen operand helpers, fed by `function_kind!`); a token and bind can disagree and still compile, falling through to an `unreachable!` that only the `#[cfg(test)] declaration_agreement` test catches. The Sequence and Atom-or-Sequence arms of `operand_bind!` bind an empty Sequence instead of failing, which is the same disagreement. The Number/Note conversion Functions declare typed operands but bypass the typed bind, leaving dead generated operand structs under a blanket `allow(dead_code)`, and Portal geometry sits in hand-written `Function::output_portal` and `Function::input_portal` matches outside the table.

**Blocked by:** 05 — Diagnose silent drops in lang; 09 — Move lang test-only API behind cfg(test).

**Status:** ready-for-human

- [ ] The chosen approach (fewer macros, a type-level operand description, or another) and its reason are recorded in this ticket.
- [ ] A mismatched token and bind fails to compile, shown by a compile_fail doctest.
- [ ] No `unreachable!` remains in the generated bind arms, and no operand bind answers an empty Sequence in place of a failure.
- [ ] The conversion Functions bind through the table like every other Function, and no dead-code allowance remains for generated operand structs.
- [ ] Conversion semantics are preserved when the conversion Functions move into the generated bind. The table declares `.^` with a Number operand and `.v` with a Note operand, but both accept either numeric type today; binding mechanically from the declared type would refuse values ADR 0021's idempotence relies on. Tests show both Functions still accept Numbers and Notes, stay idempotent, broadcast elementwise over Sequences (assembling nothing when any member fails), and report the same diagnostics in the same precedence as today.
- [ ] Function-table tests and parser/interpreter property tests pass unchanged.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid. "About eleven macros" corrected to fourteen. Absorbs 05's former criterion (c), the empty-Sequence bind.

**2026-09-25 — conversion criterion added.** `ConvertToNote` and `ConvertToNumber` are declared `[value: Number]` and `[value: Note]` in `lang/src/atom.rs`, while `Stack::convert` accepts both numeric types. The declaration alone does not describe their accepted inputs, so the move into the table carries an explicit behavioural contract.
