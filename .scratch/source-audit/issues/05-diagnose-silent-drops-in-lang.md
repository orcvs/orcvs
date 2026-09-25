# 05 — Diagnose silent drops in lang

**What to build:** Two `lang` paths that currently produce a plausible wrong answer instead report a diagnostic: (a) interpreting a list of Atoms that leaves values beyond the first answers the first and silently drops the rest (`execute_context` pops one value); (b) Jump reads an empty Portal, or an all-space Portal of the wrong width, as Empty. Also corrects the `Answer::Sequence` doc, which claims no Function declares it while Concatenate, NoteRange, NumberRange, Replace and Reverse do.

Neither path is reached by shipped code today: `orcvs` evaluates each Turn through `Interpreter::execute_function`, which checks arity and runs one Function Atom, and it always hands Jump exactly `SCALAR_WIDTH` Cells. This hardens the `lang` public contract (`Interpreter::execute`, `PortalSource::from_cells`) rather than fixing a visible defect. Narrowing `execute` to test and bench use (09) is an acceptable alternative to (a); `cfg(test)` alone does not suffice because `lang/benches/lang.rs` and `lang/tests/allocation.rs` call it.

The third drop this ticket once named — a Sequence or Atom-or-Sequence operand bound through the scalar path yields an empty Sequence — is the token/bind disagreement 25 makes unrepresentable, and moves there.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Interpreting Atoms that leave more than one value is a diagnosed error, with a test, or `execute` is no longer public shipped API.
- [ ] Jump diagnoses a Portal whose width is not the two-Cell unit, including an empty one, with a test.
- [ ] The `Answer::Sequence` documentation matches the Function table.
- [ ] Parser and interpreter property tests still pass.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `lang/` is unchanged since the audit baseline, so the findings hold. Reframed as contract hardening after tracing the shipped path; criterion (c) moved to 25.
