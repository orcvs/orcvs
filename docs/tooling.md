# Build and verification tooling

`rust-toolchain.toml` is the source of truth for the Rust toolchain and compilation targets.
`mise.toml` pins the additional tooling and defines verification commands used both locally and in
CI, so the same checkout executes the same gates.

Verification has two trigger tiers:

- `mise run check_pull_request` runs formatting, clippy, native tests, and doctests. Pull requests
  run this tier on Linux and macOS.
- `mise run check_merge` runs the persistence, WASM build, browser test, dependency audit, and
  rustdoc gates. CI distributes these gates across the existing Linux and WASM jobs after a push
  to `main`.

`mise run check` runs both tiers locally. A failure in the merge tier makes `main` red and must be
fixed before normal development continues. The delayed tier means a WASM regression can be found
after merge rather than before it.

- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.

Upgrade each version deliberately in its source-of-truth file, then run `mise run check`, the
affected platform or feature gates, and `mise run audit_deps`.
