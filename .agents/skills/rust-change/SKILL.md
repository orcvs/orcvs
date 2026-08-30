---
name: rust-change
description: Implement, fix, or refactor Rust in this repository. Use for Rust source, manifests, features, tests, examples, or build scripts. Combine with rust-unsafe or rust-dependency-change when their risk triggers apply; use rust-review instead for independent review without edits.
---

# Rust change

1. Read `AGENTS.md`, the affected manifests, and analogous implementation and tests.
2. Classify public API, unsafe, concurrency, feature, dependency, platform, and performance risk.
3. Reproduce defects with a failing behaviour-level test when practical.
4. Make the smallest coherent change using local abstractions.
5. Run the affected crate's scoped gate from `AGENTS.md` and every applicable risk gate. For repository-wide or high-risk work, run `mise run check` before completion. Exercise every applicable native and WASM target and each declared feature combination; report any combination not run.
6. Review the complete diff and report exact evidence, omitted checks, and remaining risks.
