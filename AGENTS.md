# Repository contract

## Sources of truth

- Toolchain and targets: `rust-toolchain.toml`.
- Workspace, edition, Rust version, features, dependencies, and lints: root and crate `Cargo.toml` files.
- Architecture and vocabulary: `CONTEXT.md` and `docs/adr/`.
- CI is authoritative when local and CI behaviour differ.

This is an internal application. Track current stable Rust deliberately; nightly is optional and
non-blocking while known nightly regressions exist. Neither workspace crate is publishable.
The Orcvs language is pre-release and has no public compatibility contract; classify vocabulary
and semantics changes as active language design, not public-API breakage.

## Before editing

- Read this file, the affected manifests, and analogous code and tests.
- Preserve the native and WASM targets and each declared feature combination.
- Classify public-API, unsafe, concurrency, dependency, feature, and performance risk.
- Use the focused skill whose routing description matches the work.

## Rust policy

- Follow neighbouring abstractions and make the smallest coherent change.
- Prefer safe Rust. Keep each unsafe scope minimal and state its safety invariants.
- Propagate or handle fallible results intentionally; use assertions for proven invariants.
- Keep lint suppressions narrow and explain why the lint is inapplicable.
- Keep test-only inputs out of shipped code: no shipped function takes a parameter, or reaches a
  branch, that only a test populates. When a test needs an input production cannot construct, it
  builds that input itself — below the shipped entry point, in a test-only item beside the shipped
  one, never through a seam cut into shipped code.
- Add dependencies, features, or build-time execution only with a recorded rationale.
- Support claims about performance with a reproducible benchmark or profile.
- Exercise explicit feature combinations; do not assume every feature composes.

## Verification

Local gates are scoped to what the change can reach. CI is the authority across its two tiers:
the pull-request tier runs both feature sets on Linux and compiles the WASM target. The merge
queue repeats those checks and adds macOS, the headless browser suite, the rustdoc gates, and
proptest at full case count. The benchmarks run on pull requests and pushes to `main` with path
filters. `docs/tooling.md` records what each covers. Re-deriving any of it locally spends minutes
and gigabytes on an answer CI gives anyway.

Run this for every change, on the crate you edited and the crates that depend on it — `lang`
means `lang` and `orcvs`, `orcvs` means `orcvs` and `shell`:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
cargo nextest run --package <crate> --locked
```

`cargo check` is deliberately absent: clippy performs the same compilation and adds the lints,
which is why `.scratch/ci-tiers/issues/02-remove-the-redundant-check-pass.md` removed it from
`mise.toml`. Asking for both here would have restored the pass that issue deleted.

Once, before opening a pull request, so the crate-scoped runs above are not repeated per crate:

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
```

Then run only the gates whose inputs the change actually touched:

- `mise.toml`, `.github/workflows/`, or `scripts/`: `bash scripts/check-tooling-contract.sh`,
  `actionlint`, and `zizmor --offline .github/workflows`
- `scripts/roadmap.ts` or `.scratch/`: `node --test scripts/tests/roadmap.test.ts` and
  `node scripts/roadmap.ts > /dev/null`
- persistence: `cargo nextest run --workspace --tests --no-default-features --locked`.
  `shell` enables `persistence` by default, so the ordinary run already covers the feature-on
  path and `--features persistence` adds nothing; the arm worth running per change is the one
  that proves the feature still compiles out. `mise run test_persistence` is the merge tier's
  whole pass and belongs to CI.
- WASM or platform code: `mise run check_wasm`
- dependency, feature, lockfile, build script, or proc macro: `mise run audit_deps`
- unsafe, FFI, layout, raw pointer, or atomic changes: the clippy gate, which denies
  `unsafe_op_in_unsafe_fn` and `undocumented_unsafe_blocks` across the workspace, and focused
  tests on every affected target and platform. Miri is the tool this gate would prefer and it
  ships on nightly only, so it is a path you take on purpose rather than a check a change owes:
  `mise run miri` interprets the Source model tests locally, and the `Miri` workflow runs the same
  task on manual dispatch. Neither is a required status context, and no tier calls the task. Reach
  for it when the byte write in `Source::set_source`, or the Grid indexing that mints the index it
  takes, changes — it installs a nightly toolchain and interprets at roughly a hundred times native
  cost, which is why it is not on the list above.
- public API: doctests, examples, rustdoc warnings, and human API review
- concurrency: cancellation, shutdown, ownership, backpressure, ordering, and race-sensitive tests
- parser/protocol boundary: boundary or property tests; fuzz when exposure warrants it
- performance: a benchmark for any path whose cost the change claims to move. Adding the
  benchmark is local work; running the comparison is not — see below.

### Deferred to CI

Do not run these to pass a gate. Run one only to answer a specific question, and say that is why.

- `mise run check`, `mise run check_merge`, and `mise run test_wasm` — every feature and target
  at once, plus the headless browser suite.
- `mise run bench` — the measurement is reproducible from a checkout but the comparison lives in
  the action, which `.scratch/benchmarks/spec.md` states outright, so a local run produces a
  number that decides nothing.
- proptest's 256-case default — `check_pull_request` runs 32 cases and the merge tier runs the
  rest. Export `PROPTEST_CASES=32` locally to face what a pull request faces.

## Completion evidence

Review the complete diff. Report:

```text
Changed: ...
Tests added or updated: ...
Commands run: <exact command> — passed/failed
Not run: <required check> — <reason>
Risks: public API / unsafe / dependencies / features / performance
```

"Deferred to CI" is a complete reason for anything the section above defers. Name it on the
`Not run` line rather than leaving it off the report.

## Agent conventions

### Remotes

`origin` (`orcvs/orcvs`) is the primary remote. Push branches and open pull requests there, not
against the `fork` remote.

### Worktrees

Every worktree builds into its own `target/`. A shared target directory lets one worktree's
artefacts satisfy another's freshness check, which makes a local gate report on source the
checkout does not contain. See `docs/tooling.md`.

### Issue tracker

Issues are tracked as local Markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the five default canonical label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Domain documentation uses a single-context layout. See `docs/agents/domain.md`.

### Syntax prototypes

Syntax and evaluation prototypes use Tick-by-Tick Orcvs Source Grids with diagnostic context. See `docs/agents/syntax-prototypes.md`.

### Skills

The repository's skills live in `.agents/skills/` and are exposed to Claude Code through tracked
symlinks in `.claude/skills/`. See `docs/agents/skills.md`.
