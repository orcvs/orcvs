# Repository contract

## Sources of truth

- Toolchain and targets: `rust-toolchain.toml`.
- Workspace, edition, Rust version, features, dependencies, and lints: root and crate `Cargo.toml` files.
- Vocabulary: `CONTEXT.md`. Decisions: `docs/adr/`, which stay binding and append-only.
- Current required behaviour: `docs/requirements/`, a snapshot restating the accepted ADRs. Before
  changing behaviour, read the owning requirement, in the order `docs/requirements/README.md`
  states.
- CI is authoritative when local and CI behaviour differ.

This is an internal application. Track current stable Rust deliberately; nightly is optional and
non-blocking while known nightly regressions exist. Neither workspace crate is publishable.
The Orcvs language is pre-release and has no public compatibility contract; classify vocabulary
and semantics changes as active language design, not public-API breakage.

## Before editing

- Read the affected manifests and analogous code and tests.
- Preserve the native and WASM targets and each declared feature combination.
- Classify public-API, unsafe, concurrency, dependency, feature, and performance risk.
- Use the focused skill whose routing description matches the work.

## Rust policy

- Follow neighbouring abstractions and make the smallest coherent change.
- Prefer safe Rust. Keep each unsafe scope minimal and state its safety invariants.
- Propagate or handle fallible results intentionally; use assertions for proven invariants.
- Keep lint suppressions narrow and explain why the lint is inapplicable.
- Comments state what the code does, why, and the invariant it keeps; lineage goes in the commit
  message. When a comment holds history, a citation, `SAFETY:` or a doctest: `docs/agents/comments.md`.
- Keep test-only inputs out of shipped code: no shipped function takes a parameter, or reaches a
  branch, that only a test populates. When a test needs an input production cannot construct, it
  builds that input itself — below the shipped entry point, in a test-only item beside the shipped
  one, never through a seam cut into shipped code.
- Add dependencies, features, or build-time execution only with a recorded rationale.
- Support claims about performance with a reproducible benchmark or profile.
- Exercise explicit feature combinations; do not assume every feature composes.

## Verification

Local gates cover what the change can reach; CI runs every feature set and target, and
`docs/tooling.md` records which tier runs what.

Run this for every change, on the crate you edited and the crates that depend on it — `lang`
means `lang` and `orcvs`, `orcvs` means `orcvs` and `console`:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
cargo nextest run --package <crate> --locked
```

Clippy is the compile pass; run it in place of `cargo check`.

Once, before opening a pull request, so the crate-scoped runs above are not repeated per crate:

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
```

Then run only the gates whose inputs the change actually touched:

- `mise.toml`, `.github/workflows/`, or `scripts/`: `mise run check_docs`, `actionlint`, and
  `zizmor --offline .github/workflows`
- `docs/adr/`, `docs/requirements/`, `.scratch/`, or renaming or deleting a file, function or
  test that a requirement's Evidence line names: `mise run check_docs`
- persistence: `cargo nextest run --workspace --tests --no-default-features --locked`.
  `console` enables `persistence` by default, so the ordinary run already covers the feature-on
  path and `--features persistence` adds nothing; the arm worth running per change is the one
  that proves the feature still compiles out. `mise run test_persistence` is the merge tier's
  whole pass and belongs to CI.
- WASM or platform code: `mise run check_wasm`
- dependency, feature, lockfile, build script, or proc macro: `mise run audit_deps`
- `console`'s `inspection` feature, or the `eframe` feature set it selects: `mise run check_inspection`.
  The feature is off by default, so every other gate compiles the console without it.
- `console`'s `release-capture` feature, `console::kittest_tests::capture`, or a console test helper
  the capture calls: `mise run check_release_capture`. The merge tier runs it; no pull-request gate
  compiles the feature.
- egui or eframe presentation, UI tests, or inspection: the `egui` skill
- `site/`, or what the site renders (`docs/first-result.md`, `docs/functions.md`): `mise run site_build`
  then `mise run site_test`, and the `website` skill. The site builds around the console published
  from `main`; to show a `console/` change, run `mise run console_web_build` and build the site
  with `CONSOLE_WEB=../target/console-web`.
- unsafe, FFI, layout, raw pointer, or atomic changes: the clippy gate, which denies
  `unsafe_op_in_unsafe_fn` and `undocumented_unsafe_blocks` across the workspace, and focused
  tests on every affected target and platform. When to add the optional Miri run: the
  `rust-unsafe` skill.

Each risk the change carries owes these tests, and is done when its bound holds:

- public API: a doctest or example on every changed public item, a clean
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, and the changed API named
  for human review on the `Risks` line.
- concurrency: a test for each of cancellation, shutdown, ownership, backpressure and ordering
  that the change touches, plus a race-sensitive test for any new shared state.
- parser/protocol boundary: boundary or property tests over every input form the change accepts;
  where that input is untrusted, state on the `Risks` line whether it warrants fuzzing.
- performance: a benchmark for any path whose cost the change claims to move. Adding the
  benchmark is local work; running the comparison is deferred to CI.

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

Domain documentation uses a single-context layout. See `docs/agents/domain.md`. Adding an ADR, or
changing one's Status or amendments: update the requirements in the same change, as
`docs/requirements/README.md` states under "Keeping the snapshot current".

### Syntax prototypes

Building or reviewing a syntax or evaluation prototype (`lang/*-prototype.html`):
`docs/agents/syntax-prototypes.md`.

### Skills

Adding or editing a skill: `docs/agents/skills.md`.
