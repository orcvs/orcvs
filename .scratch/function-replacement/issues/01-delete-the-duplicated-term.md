# 01 — Delete the duplicated term

**What to build:** The replacement guard at `orcvs/src/source/tick/execution.rs:448-486` compares `replacement.source_effect() != target.source_effect()` twice, at `:463` and `:485`. The two expressions are byte-identical. Delete the second, and keep whichever of the two justification comments survives a merge of both.

The duplicate is inert: `a || b || b` is the predicate `a || b`, so no Tick has behaved differently while it stood and no diagnostic has been wrong. This is dead code arriving through a merge, not a defect. `git log -S` puts the two on the same day, `17c4078` and `f63f5b2`, the second resolving a conflict about the mover-collision rule.

The two comments argue for the same expression from different directions — one about Portal direction, telling `^^` from `>>`; the other about ADR 0004 write offsets and why the whole effect is compared rather than its fields. Both facts are true and both are worth keeping, so the surviving comment states both. If they turn out not to merge, that is a finding: it would mean the two terms were asking different questions and one of them is now missing. Say so in the ticket rather than picking one.

This lands before `02` so that the deepening's diff is not credited with removing a line it did not have to touch.

**Status:** ready-for-agent

- [ ] Exactly one `source_effect` comparison remains in the guard.
- [ ] The surviving comment carries both arguments, or the ticket records why they could not be merged.
- [ ] No test changes. The predicate is unchanged, so the suite is the evidence.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`, `cargo nextest run --package orcvs --locked`, and the same two for `console`, which depends on `orcvs`.
