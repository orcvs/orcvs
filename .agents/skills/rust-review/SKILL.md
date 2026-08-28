---
name: rust-review
description: Review Rust changes in this repository without editing. Use for diffs, branches, pull requests, or work-in-progress review; apply unsafe, dependency, concurrency, feature, platform, and public-API risk checks when present.
---

1. Read `AGENTS.md`, the review specification, affected manifests, and the complete diff from its fixed base.
2. Trace changed behaviour through callers, invariants, feature combinations, platform branches, and tests.
3. Apply every relevant risk gate from `AGENTS.md`; inspect unsafe invariants and dependency or lockfile changes directly.
4. Run focused read-only verification when it can confirm or reject a suspected finding.
5. Report only actionable findings, ordered by severity, with file and line evidence.
6. State residual risks and checks not run. If no findings remain, say so explicitly.
