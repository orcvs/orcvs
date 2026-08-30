---
name: rust-dependency-change
description: Change or review Rust dependencies, features, Cargo.lock, build scripts, proc macros, registries, git sources, or cargo-deny policy in this repository. Combine with rust-change for implementation and rust-review for independent review.
---

# Rust dependency change

1. State the dependency or feature need and why existing code or dependencies cannot meet it.
2. Inspect release notes, MSRV, maintenance, advisories, licenses, default features, build scripts, proc macros, and source registry.
3. Select the narrowest explicit feature set and preserve the supported native and WASM targets.
4. Review `cargo tree -e features`, the complete manifest diff, and every `Cargo.lock` change.
5. Run the scoped gate, `mise run check`, explicit feature combinations, WASM checks when affected, and `cargo deny check`.
6. Report the rationale, transitive impact, commands run, omitted checks, and remaining supply-chain risk.
