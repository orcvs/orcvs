---
name: rust-review
description: Review Rust changes in this repository without editing. Use for diffs, branches, pull requests, or work-in-progress review; apply unsafe, dependency, concurrency, feature, platform, and public-API risk checks when present.
---

# Rust review

1. Read `AGENTS.md`, the ticket or pull-request description the change answers, the affected manifests, and the complete diff from its fixed base.
2. Trace changed behaviour through callers, invariants, feature combinations, platform branches, and tests.
3. For each loop or iterator chain the diff adds or changes, check where its conditionals sit. A side-effect-free condition whose value cannot change between iterations goes before the loop. A test of each element's shape (an `Option`, or the one enum variant the loop expects) is settled where the elements are produced when the producer already knows it, so the looping code receives narrowed elements (`&[T]`, `Vec<T>`, or `impl Iterator<Item = T>`) rather than testing an `Option<T>` per element; a condition on an element's value stays in the loop. Raise it when the fix is local to the diff, and frame it as structure, not speed, unless a benchmark backs a cost claim.
4. Apply every relevant risk gate from `AGENTS.md`; inspect unsafe invariants and dependency or lockfile changes directly. Combine with `rust-unsafe`, `rust-dependency-change` or `egui` when the diff touches their triggers.
5. Run focused verification, without editing tracked files, when it can confirm or reject a suspected finding: the scoped gates from `AGENTS.md` for the crates the diff reaches, and each risk gate whose inputs it touches.
6. Hold every comment in the diff to `docs/agents/comments.md`: flag lineage, and a citation that names no constraint the code cannot show.
7. Report only actionable findings, ordered by severity, with file and line evidence.
8. State residual risks, and list each check not run in the `Not run:` form from `AGENTS.md`, where "Deferred to CI" is a complete reason only for the checks `AGENTS.md` defers. If no findings remain, say so explicitly.
