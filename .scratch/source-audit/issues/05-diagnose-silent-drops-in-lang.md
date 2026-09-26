# 05 — Diagnose silent drops in lang

**What to build:** Two `lang` paths that currently produce a plausible wrong answer instead report a diagnostic: (a) interpreting a list of Atoms that leaves values beyond the first answers the first and silently drops the rest (`execute_context` pops one value); (b) Jump reads an empty Portal, or an all-space Portal of the wrong width, as Empty. Also corrects the `Answer::Sequence` doc, which claims no Function declares it while Concatenate, NoteRange, NumberRange, Replace and Reverse do.

Neither path is reached by shipped code today: `orcvs` evaluates each Turn through `Interpreter::execute_function`, which checks arity and runs one Function Atom, and it always hands Jump exactly `SCALAR_WIDTH` Cells. This hardens the `lang` public contract (`Interpreter::execute`, `PortalSource::from_cells`) rather than fixing a visible defect. Narrowing `execute` to test and bench use (09) is an acceptable alternative to (a); `cfg(test)` alone does not suffice because `lang/benches/lang.rs` and `lang/tests/allocation.rs` call it.

The third drop this ticket once named — a Sequence or Atom-or-Sequence operand bound through the scalar path yields an empty Sequence — is the token/bind disagreement 25 makes unrepresentable, and moves there.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Interpreting Atoms that leave more than one value is a diagnosed error, with a test, or `execute` is no longer public shipped API.
- [x] Jump diagnoses a Portal whose width is not the two-Cell unit, including an empty one, with a test.
- [x] The `Answer::Sequence` documentation matches the Function table.
- [x] Parser and interpreter property tests still pass.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `lang/` is unchanged since the audit baseline, so the findings hold. Reframed as contract hardening after tracing the shipped path; criterion (c) moved to 25.

**2026-09-25 — resolved on `lang/narrow-api`.** (a) took the narrowing alternative further: `Interpreter::execute` and the Atom-list walk behind it are deleted, so the path that dropped values no longer exists. Tests, the benchmark and the allocation test go through `execute_function`, the call a Turn makes; nested evaluation is `orcvs`'s and tested there (shared with 09). (b) Jump diagnoses `JumpInput` for any Portal that is not exactly two Cells, including an empty one (`a_portal_that_is_not_one_two_cell_unit_diagnoses`). The `Answer::Sequence` doc names Concatenate, Note Range, Number Range, Replace and Reverse. Parser property tests pass at `PROPTEST_CASES=32`.

**2026-09-26 — property coverage, corrected.** The ticked property criterion holds for the Parser only. `lang`'s interpreter property module and its nested-chain examples tested `Interpreter::execute`'s Atom-list walk and were deleted with it: `execute_function` takes resolved operands and cannot nest, so no generated nested Expression reaches it. Deep chains through the shipped path are covered by `orcvs` examples (`source/tick.rs`, the 24-deep division chain; `source/model.rs`, the 33-deep addition chain) and the effect-Function sweep in `source/model.rs`; no generated nested-evaluation property replaces the deleted one.
