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

A third tier measures rather than checks. `mise run bench` runs the `lang` criterion benchmarks and
prints them in the bencher output format. `.github/workflows/bench.yml` runs the same command in two
jobs and fails either when a benchmark is more than three times slower than the previous stored
result. The publishing job runs after a push to `main` and appends the result to the series on the
`gh-pages` branch. The pull-request job compares against that series and stores nothing. Permissions
are declared per job, so only the publishing job can write repository contents. Both triggers are
filtered to the paths that can move a measurement, so a change that cannot touch `lang` performance
runs no benchmark. `mise run check` does not run either.

This benchmark gate is the one exception to the equivalence above. The measurement is reproducible
from a checkout; the comparison is not, because it lives in the action rather than in `mise.toml`.
`.scratch/benchmarks/spec.md` records what the gate can and cannot detect.

- `criterion` measures the `lang` parse and interpret paths; `benchmark-action/github-action-benchmark`
  stores and compares the results.
- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.

Upgrade each version deliberately in its source-of-truth file, then run `mise run check`, the
affected platform or feature gates, and `mise run audit_deps`.
