# 03 — Correct the stale test and cover Bang emission

**What to build:** Two gaps the reachability sweep in the spec turned up, both of which need `02`'s named outcomes to be stated properly.

**The stale comment.** `a_replacement_that_changes_only_the_activation_source_is_refused` at `orcvs/src/source/tick.rs:2456` says RawPlay and `^^` "agree on every other column it reads — neither answers a value, neither can return Bang, and both reserve a Cell pair — and they differ in where their activation comes from". That was true when the test was written at `e5155c1`; the `source_effect` term arrived later, at `3a676bc`. `^^` declares a write and RawPlay does not, so the fixture differs on two facts, not one. The test's name survives — Activation is still the first difference, so it still reports what the name claims — but the comment asserts an agreement that does not hold. Correct it to say that the pair differs on Activation first and on Write as well, and that no pair differing on Activation alone exists.

**The untested term.** `can_emit_bang` is the one term that is independently reachable and has no assertion anywhere. `Equality` against `Add` differs on it and agrees on every other fact: both answer a value, both are intrinsically active, neither declares a write, both reserve a Cell pair. Add a test that a replacement changing only Bang emission is refused, and that the diagnostic names it. `Delay` and `Euclidean` against any bang=false Value row would serve equally.

Neither of these is a behaviour change. The first corrects a comment; the second covers a term that has always been refused and has never been asserted.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] The activation test's comment states what its fixture actually differs on, and that no Activation-only pair exists.
- [ ] A test covers a replacement differing only in Bang emission, asserting the refusal and the named fact.
- [ ] No production code changes in this ticket.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`, `cargo nextest run --package orcvs --locked`, and the same two for `console`.
