# 07 — Check each Function's answer against its declared kind

**What to decide:** Nothing checks that `execute_function` routes each Function to a body of its declared kind. Choose how to close that gap.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

**Sources of truth:** ADR 0028's third paragraph says the compiler checks dispatch for completeness only and leaves kind agreement to tests; issue 04 records the decision not to derive dispatch.

- [ ] A Function whose declared kind is value cannot answer `Interpretation::Play`, and a Terminal Output Function cannot answer `Interpretation::Cell`, without a test failing or a compile error.
- [ ] A value or Terminal Output Function listed in the `unreachable!` arm of `execute_function` fails a test or does not compile.

## Comments

The match in `execute_function` (`lang/src/interpreter.rs:113-176`) is exhaustive, but each arm's
return type ties a body only to its own wrapping. `Function::RawPlay => math::add(&mut ctx)?`
compiles and answers `Interpretation::Cell` for a Terminal Output Function. A value Function moved
into the `Halt | Bang | ...` arm compiles and panics when evaluated.
`locking_and_source_writing_functions_are_interpreted_from_their_kind` visits only Functions that
already return before dispatch, and `only_a_function_that_declares_it_ever_answers_with_bang` visits
only value Functions without an Input Portal, so neither sees a mis-route.

Two shapes close it:

- A test over `Function::ALL` that evaluates each Function from operands built from its signature
  and compares the `Interpretation` variant with `answers_value()`, `performs_terminal_output()`,
  `locks_root()` and `source_effect()`. Functions that read an Input Portal need a supplied Portal
  input, which the Bang sweep currently skips.
- The generated per-Function evaluation trait issue 06 proposed, with the answer type associated by
  kind, so a mis-route is a type error. That trait derives dispatch, which issue 04 declined on cost.
  Take it only if the test sweep cannot be made to cover every Function.
