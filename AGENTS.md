# Repository contract

## Sources of truth

- Toolchain and targets: `rust-toolchain.toml`.
- Workspace, edition, Rust version, features, dependencies, and lints: root and crate `Cargo.toml` files.
- Architecture and vocabulary: `CONTEXT.md` and `docs/adr/`.
- CI is authoritative when local and CI behaviour differ.

This is an internal application. Track current stable Rust deliberately; nightly is optional and
non-blocking while known nightly regressions exist. Neither workspace crate is publishable.

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
- Add dependencies, features, or build-time execution only with a recorded rationale.
- Support claims about performance with a reproducible benchmark or profile.
- Exercise explicit feature combinations; do not assume every feature composes.

## Verification

Run the scoped gate for the affected crate:

```sh
cargo fmt --all -- --check
cargo check --package <crate> --all-targets --locked
cargo clippy --package <crate> --all-targets --locked -- -D warnings
cargo nextest run --package <crate> --locked
cargo test --package <crate> --doc --locked
```

Run `mise run check` before completing repository-wide or high-risk work. Also run the applicable
risk gate:

- persistence: `mise run test_persistence`
- WASM or platform code: `mise run check_wasm`
- dependency, feature, lockfile, build script, or proc macro: `mise run audit_deps`
- unsafe, FFI, layout, raw pointer, or atomic changes: Miri and focused tests where supported
- public API: doctests, examples, rustdoc warnings, and human API review
- concurrency: cancellation, shutdown, ownership, backpressure, ordering, and race-sensitive tests
- parser/protocol boundary: boundary or property tests; fuzz when exposure warrants it

## Completion evidence

Review the complete diff. Report:

```text
Changed: ...
Tests added or updated: ...
Commands run: <exact command> — passed/failed
Not run: <required check> — <reason>
Risks: public API / unsafe / dependencies / features / performance
```

## Agent conventions

### Issue tracker

Issues are tracked as local Markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the five default canonical label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Domain documentation uses a single-context layout. See `docs/agents/domain.md`.
