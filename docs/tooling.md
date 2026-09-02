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

A third tier measures locally and checks only in CI. `mise run bench` runs the criterion benchmarks
in both `lang` and `orcvs` and prints them in the bencher output format. `lang` covers language
execution: parsing an Expression and interpreting it. `orcvs` covers a populated Source: reading an
unchanged revision, deriving its Render Frame, and applying an edit with the Language Map rebuild it
forces. Each is measured over several Source sizes, so whole-map work shows as growth across the
series rather than hiding inside one fixed size. `.github/workflows/bench.yml` runs the same command
in two jobs and fails either when a benchmark is more than three times slower than the previous
stored result. The publishing job runs after a push to `main` and appends the result to the series
on the `gh-pages` branch. The pull-request job compares against that series and stores nothing.
Permissions are declared per job, so only the publishing job can write repository contents. Both
triggers are filtered to the paths that can move a measurement, so a change that cannot touch `lang`
or `orcvs` performance runs no benchmark. `mise run check` does not run either.

This benchmark gate is the one exception to the equivalence above. The measurement is reproducible
from a checkout; the comparison is not, because it lives in the action rather than in `mise.toml`.
`.scratch/benchmarks/spec.md` records what the gate can and cannot detect.

- `criterion` measures both benchmarked paths — language execution in `lang`, and populated Source
  reading, rendering, and editing in `orcvs`; `benchmark-action/github-action-benchmark` stores and
  compares the results.
- `proptest` generates the property tests that encode the invariants `CONTEXT.md` and the ADRs
  already state.
- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.

`AGENTS.md` already obliges a change at the parser boundary to bring "boundary or property tests;
fuzz when exposure warrants it", and the parser is the widest input surface in the workspace because
every keystroke reaches it. That obligation is the rationale for the dependency. `proptest` is
declared once in `[workspace.dependencies]` and consumed by `lang` and `orcvs` under
`[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` only. Every invariant these
properties encode is platform-independent logic, so running them under `wasm-pack` would add a
dependency graph to maintain and no signal; the property modules carry the same `cfg`, so a WASM
build never sees the dependency at all. Default features are off and only `std` is enabled, because
`std` is what the counterexample files are written through and the `fork`, `timeout`, and `bit-set`
features buy nothing for properties over pure logic. `scripts/check-tooling-contract.sh` pins that
confinement, so a later change cannot quietly move `proptest` into a shipped `[dependencies]` table.

The case count follows the verification tier. `mise run check_pull_request` sets `PROPTEST_CASES` to
32, so a pull request trades coverage for latency; the merge tier leaves proptest's 256-case default
in place. The setting is task-level env rather than an inline assignment on a run line, because the
contract script pins the exact text of those lines.

Counterexample files are committed like source. proptest writes them to a `proptest-regressions`
directory beside each crate's `src`, one file per module, and no ignore rule excludes them. The
contract script pins the representative path for each crate — `lang/proptest-regressions/parser.txt`
and `orcvs/proptest-regressions/grid.txt` — by asking `git check-ignore` rather than reading
`.gitignore`, which catches a broad glob or a nested ignore file as well as a literal rule. A
counterexample that CI can see and a developer cannot reproduce is worse than no property at all, so
the shrunk input travels with the repository and the next run replays it before generating anything
new.

The same `AGENTS.md` sentence defers fuzzing to "when exposure warrants it", so no fuzzing harness is
installed. That is a separate decision with its own cost, and it is not taken here.

Upgrade each version deliberately in its source-of-truth file, then run `mise run check`, the
affected platform or feature gates, and `mise run audit_deps`.
