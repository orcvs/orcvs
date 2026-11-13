# 02: A Raw Play with a blank note plays nothing

**What to build:** Raw Play `!>` declares `note: Optional<Note>`. A blank note sends no Play Command and raises no diagnostic, whether the Source was written blank or a Portal write emptied the slot. A half-typed or mistyped note is still diagnosed. See `../spec.md`.

**Blocked by:** 01

**Status:** resolved

- [x] lang has an `Optional<O>` operand for any `O: Operand`. Its Token is `O`'s, it binds absent to `None` and a present value through `O::bind` to `Some`, and its bound type is `Option<O::Bound>`.
- [x] Each operand type answers whether it admits a blank slot (`true` only for `Optional<O>`), and the Function table derives each slot's admission from its declared type, with no separate list.
- [x] The Play body receives `Option<Note>` and returns no Play Command for `None`.
- [x] Through `Source::execute`: a written-blank or Portal-emptied note on `!>` sends nothing, and neither the Tick Plan nor the Language Map reports a diagnostic for it.
- [x] Through `Source::execute`: a half-typed note (`C `) and a mistyped one (`ZZ`) on `!>` are diagnosed as today.
- [x] A blank note still lets other operands bind. A blank channel, or a channel out of range, is diagnosed on a blank step.
- [x] A `!>` with a blank note costs no other Expression its Turn and keeps its place in Tick order.
- [x] `evaluate` tests: two blank Cells bind to absent, a valid note binds to a note, and anything else is refused as `Note` refuses it.
- [x] Reading a `!>` with a blank note allocates nothing per Tick, under the existing allocation harness.
- [x] ADR 0108 records that the note is optional by type. It supersedes 0075, 0104 and 0107 and amends 0069 and 0073 by operand type, and their status lines point to it.
