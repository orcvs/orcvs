# 01 — Use one borrowed projection for Expression scheduling

**What to build:** Literal-only edits continue to reuse their schedule, while
edits to Expression scheduling facts invalidate it exactly as before. Equality
and computation construction obtain those facts through one borrowed projection
owned by the schedule module. See the accompanying spec for the confirmed design.

**Blocked by:** None — can start immediately

**Status:** resolved

- [x] Equality and computation construction consume the same projection of
  scheduled Expression eligibility, Span, leading Function, entry Cell ranges,
  parents and three-way parse classification.
- [x] Literal values, Tokens and Parser diagnostic text remain outside that
  projection; Source bytes are not copied.
- [x] The projection borrows existing Parser structure, adds no projection
  allocations or stored duplicate structure, and owns the interpretation of
  scheduling facts rather than relocating a separate comparison.
- [x] Unchanged rows remain shared, only dirty rows are compared, and Portal
  resolution, dependency derivation and ordering remain lazy.
- [x] Tick Plans, diagnostics, activation, Turn order and the existing schedule
  invalidation distinctions remain unchanged. Language Unit comparison and lock
  construction retain their current behaviour until ticket 02.
- [x] Existing fresh-versus-reused comparisons, stale-schedule counterexamples
  and edit properties pass; focused behavioural regressions cover any identified
  gaps without reproducing the projection representation.
- [x] The repository's required crate and dependant gates pass with
  PROPTEST_CASES=32; the complete diff is reviewed and completion evidence recorded.

## Comments

2026-10-14 — Implemented. `orcvs/src/source/tick/inputs.rs` owns the projection: `ScheduledExpression` borrows a row's Span, leading Function and parsed Expression, and yields each positioned entry's Cells, parent and `Parsed` classification (unparsed, value, or a particular Function). `computations` iterates `ScheduleInputs::expressions` instead of the Language Map, and row equality compares the same projection through its `PartialEq`, so `DerivedExpression::schedules_as` and its `pairwise` helper are gone. Nothing is stored or allocated for the projection. No new test: the schedule-reuse examples and the shared-versus-fresh property cover every invalidation the projection keeps, and no gap appeared. Gates with PROPTEST_CASES=32, all passed: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo test --workspace --doc --locked`, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, `cargo test --workspace --tests --no-default-features --locked`. Not run: `cargo nextest run` — not installable in this environment; `cargo test` ran the same tests in one process per binary, so per-test process isolation is unverified locally and left to CI. WASM, browser, macOS and benchmark comparison are deferred to CI.
