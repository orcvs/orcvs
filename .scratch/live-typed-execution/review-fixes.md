# Live typed execution review follow-up

The user supplied five review findings against the live typed execution implementation in
this commit and confirmed the Source execution test seam: committed Source text, Tick
diagnostics and Play commands, with the existing fixed-Portal and supplied typed-result
fixture for values current syntax cannot produce. No fixture supplies a schedule.
Behavioral fixes follow one failing regression at a time.

## Finding ledger

All findings originate in the user's pasted review. Line numbers below identify the
implementation as it stood before these fixes.

| ID | Severity (original category) | Location | Claim and failure scenario | Outcome |
| --- | --- | --- | --- | --- |
| R1 | Major (correctness) | `orcvs/src/source/tick.rs:133` | Unchanged malformed nested literals suppress only their own computation, so ancestors repeat missing-result diagnostics each Tick. | Confirmed; fixed |
| R2 | Major (correctness) | `orcvs/src/source/tick.rs:458` | Scheduling assumes two Cells and omits row-edge outputs, while execution accepts other encoding widths. | Confirmed at the typed-result boundary; fixed |
| R3 | Major (robustness) | `orcvs/src/source/tick.rs:265,273` | Release assertions can panic under the Source lock if a write reaches an executed computation. | Confirmed latent robustness issue; hardened |
| R4 | Minor (dead-code) | `lang/src/expression.rs:44` | Unused `Expression::add` invents contiguous positions and root ownership, inconsistent with parser-owned records. | Confirmed; removed |
| R5 | Minor (dead-code) | `orcvs/src/glyph.rs:83` | `Glyph::to_glyphs` has no caller after `expression_parts` removal. | Confirmed; removed |

R1–R3 share the Tick implementation and are handled together. R4–R5 form a separate removal
group. Tracked Rust caller searches confirm both removed helpers were unused. Their removal
has no behavior to regress; existing parser and rendering tests provide verification.

## Regression evidence

R1: `live_unchanged_nested_syntax_errors_do_not_repeat_as_tick_failures` in
`orcvs/src/source/tick.rs:755` first failed because `.+01.x02.+03??` emitted two ancestor
“supplied no typed result” diagnostics. Syntax-blocked propagation now retains the Source
diagnostic without producing repeated Tick failures. The regression also covers editing the
literal and repairing it spatially in the same Tick; both restore the independently calculated
ancestor answer `0F`. Existing `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers`
still verifies genuine evaluation failures diagnose their nested consumers.

R2: `live_non_pair_scalar_projection_is_rejected_at_the_row_edge` in
`orcvs/src/source/tick.rs:801` first failed because a supplied `Atom::Char('7')` wrote a trailing
`7` into the last Cell despite the scheduler dropping that destination's edges. Restoring the
explicit two-byte scalar projection guard rejects that write. Typed nested results are stored
before projection validation, and computations with no destinations receive no projection
diagnostic. Current authored Functions emit scalar pairs; the supplied Atom fixture covers the
unsupported width without adding an operation to the language.

R3: the release assertions are replaced by a preflight check that returns an ordering diagnostic
and rejects all Tick writes and Play commands, including previously accumulated effects. No
supported two-Cell reproduction was found: an admitted output has a corresponding overlap edge
to every affected computation and descendant, inactive owners cannot emit, and replacements
cannot change activation/output kind. This is defensive hardening of a latent invariant failure,
not a reproduced current-language panic. No test — no reachable behavior to regress at the
confirmed seam; no internal schedule fault fixture was introduced.

R4 and R5: no test — no behavior to regress for unused helper removal.

Commands run during the red/green cycles:

- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(live_unchanged_nested_syntax_errors)'` — failed as expected, two cascading ancestor diagnostics.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(live_unchanged_nested_syntax_errors) or test(live_failed_suppliers)'` — passed, 2 tests after the fix.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(live_non_pair_scalar)'` — failed as expected, trailing `7` committed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(live_non_pair_scalar) or test(live_failed_suppliers)'` — passed, 2 tests after the fix.
- `cargo fmt --all` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(source::tick::)' --status-level fail` — passed, 43 tests.

## Verification and risk

The complete implementation diff was reviewed against the parent revision, including callers
and the ordering and publication invariants. Final gate results:

- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --locked --status-level fail` — passed, 505 tests.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --tests --features persistence --locked --status-level fail` — passed, 509 tests.
- `RUSTC_WRAPPER= cargo test --workspace --doc --locked` — passed, 9 doctests.
- `RUSTC_WRAPPER= RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` — passed.
- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.

Not run: `mise run check`, `mise run check_merge`, `mise run test_wasm`, `mise run bench`,
the full 256-case proptest run and other operating-system/WASM/browser matrix — deferred to CI.
No platform, dependency or tooling implementation changed, so their conditional local gates
are not triggered.

The removed methods were public only within unpublished internal crates. No unsafe code,
dependency, feature, platform implementation or concurrency lifecycle changed. The defensive
preflight adds an overlap/descendant traversal to spatial delivery; no performance improvement
is claimed or measured. All five findings have an outcome; none needs a design decision.
