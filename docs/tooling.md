# Build and verification tooling

`mise.toml` is the single source of truth for tool versions and verification commands used both
locally and in CI. Versions are pinned so the same checkout executes the same gates.

- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.

Upgrade these pins deliberately in `mise.toml`, then run `mise run check`, the affected platform or
feature gates, and `mise run audit_deps`.
