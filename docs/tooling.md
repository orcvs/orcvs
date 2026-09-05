# Build and verification tooling

`rust-toolchain.toml` is the source of truth for the Rust toolchain and compilation targets.
`mise.toml` pins the additional tooling and defines verification commands used both locally and in
CI, so the same checkout executes the same gates.

Verification has two trigger tiers:

- `mise run check_pull_request` runs the tooling contract, the contract's own test suite, the
  roadmap planner's test suite, the dependency audit, formatting, clippy with and without
  `persistence`, the native tests under both feature sets, and doctests. `mise run check_wasm`
  compiles every crate's test targets for `wasm32-unknown-unknown` and builds the application. Pull
  requests run the first on Linux and macOS and the second on the WASM job.
- `mise run check_merge` runs the browser regression suite, the rustdoc gates, and the persistence
  tier at proptest's full case count. CI distributes these gates across the existing Linux and WASM
  jobs after a push to `main` or a manual dispatch.

`mise run check` runs both tiers locally. A failure in the merge tier makes `main` red and must be
fixed before normal development continues. What the delayed tier holds is behaviour rather than
compilation: a browser regression can still be found after merge, but a browser test that no longer
compiles fails the pull request that wrote it. The same is true one feature over — the persistence
tests live in a test-only module behind a dev-dependency, so no library build can reach them, and
the pull-request tier reaches them by building all targets with the feature enabled.

`mise run test_persistence` still runs in the merge tier, and its overlap with the pull-request tier
is deliberate rather than an oversight: `check_pull_request` sets `PROPTEST_CASES` to 32, so the
merge tier is the only place the properties run at proptest's 256-case default. What is genuinely
merge-only is the browser run, the rustdoc gates, and that full-case run.

Dependency auditing runs in the pull-request tier through `mise run audit_deps`, which checks
advisories, licences, and sources and prints the feature-resolved dependency tree. Dependabot's
weekly grouped bumps are exactly the pull requests it exists for, so it gates them rather than
reporting on them once they are already on `main`. `mise run check_merge_native` runs `cargo deny`
again after a merge; the audit is cheap, and each tier reading correctly on its own is worth more
than removing the overlap.

Every commit that reaches `main` gets its own merge-tier run. The workflow's concurrency group is
keyed on the pull request number for a pull request and on the commit for anything else, so a later
merge cannot cancel an earlier commit's run, and `cancel-in-progress` is confined to pull requests.
The merge-tier steps are guarded on the event not being a pull request rather than on its being a
push, so a manual dispatch — the obvious way to re-verify a commit — runs the merge tier instead of
reporting green having run only the pull-request tier.

Two gates advise rather than block, and both are recorded here rather than assumed. The benchmark
comparison in `.github/workflows/bench.yml` fails on a threefold regression but is not a required
status context, and `main`'s branch protection does not enforce against administrators, so a direct
push skips every required context. Closing either needs repository administration rather than a
change to a checkout, so
`.scratch/verification-gaps/issues/09-require-the-benchmark-and-close-the-protection-bypasses.md`
records the intended settings and holds the decision.

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
- `node` runs `scripts/roadmap.ts` and its test suite through its own type stripping and test
  runner. The pull-request tier runs both, and it needs both: the suite drives `buildRoadmap` and
  `planRelease` over temporary fixtures and never reads `.scratch/`, so the throws that catch tracker
  drift — a dangling `Blocked by:`, a dependency cycle, an untagged release blocker — are reached
  only by running the planner against the real tree. `package.json` declares the floor the type
  stripping needs and `mise.toml` pins the version the gate runs.

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
contract script pins the path for each property — `lang/proptest-regressions/parser.txt`,
`lang/proptest-regressions/interpreter.txt`, and `orcvs/proptest-regressions/grid.txt` — by asking
`git check-ignore` rather than reading
`.gitignore`, which catches a broad glob or a nested ignore file as well as a literal rule. A
counterexample that CI can see and a developer cannot reproduce is worse than no property at all, so
the shrunk input travels with the repository and the next run replays it before generating anything
new.

The same `AGENTS.md` sentence defers fuzzing to "when exposure warrants it", so no fuzzing harness is
installed. That is a separate decision with its own cost, and it is not taken here.

Upgrade each version deliberately in its source-of-truth file, then run `mise run check`, the
affected platform or feature gates, and `mise run audit_deps`.
