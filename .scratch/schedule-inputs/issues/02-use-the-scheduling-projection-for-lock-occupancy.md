# 02 — Use the scheduling projection for lock occupancy

**What to build:** Halt classifies its target and schedule reuse sees the same
Language Unit facts, so a lock rule cannot silently acquire an input omitted
from equality. All schedule construction consumes the shared borrowed projection.
See the accompanying spec for the confirmed design.

**Blocked by:** 01 — Use one borrowed projection for Expression scheduling

**Status:** resolved

- [x] The projection includes Language Unit anchor, Span and kind, including
  Comments and units outside scheduled Expressions.
- [x] Equality and lock construction consume these shared facts, preserving
  the current full Language Unit equality and every existing invalidation rule.
- [x] Schedule construction, including lock occupancy classification, has no
  unrestricted Language Map access; execution retains the live access it needs.
- [x] The remaining independently maintained scheduling comparison is removed;
  Expression and Language Unit facts have one interpretation at the schedule seam.
- [x] Borrowing introduces no stored duplicate Parser structure, projection
  allocation or projection memoization. Row sharing and lazy construction remain.
- [x] Halt behaviour, Tick Plans, diagnostics, activation and Turn order remain
  unchanged under ADRs 0032, 0034 and 0068.
- [x] Existing Halt, schedule-reuse and edit-property tests pass; focused
  regressions cover occupancy or invalidation gaps identified during the change.
- [x] Required crate and dependant gates pass with PROPTEST_CASES=32, and required
  workspace gates pass before an implementation PR. The complete diff is reviewed
  and completion evidence recorded.

## Comments

2026-10-14 — Implemented. `RowInputs` adds each row's Language Units, compared whole (anchor, Span and kind), and its equality replaces `DerivedRow::schedules_as` in `LanguageMap::rebuild`. `ScheduleCache::schedule`, `schedule`, `order_turns` and `Locks::new` take `ScheduleInputs` rather than the Language Map, and Halt's occupancy classification reads `ScheduleInputs::units`; `Portal::occupancy` and `occupancy_of` take the units they classify against. Execution keeps the live Map. The Halt, schedule-reuse and edit-property tests pass unchanged, including the Comment occupancy example. Gates with PROPTEST_CASES=32, all passed: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo test --workspace --doc --locked`, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, `cargo test --workspace --tests --no-default-features --locked`. Not run: `cargo nextest run` — not installable in this environment; `cargo test` ran the same tests in one process per binary, so per-test process isolation is unverified locally and left to CI. WASM, browser, macOS and benchmark comparison are deferred to CI.

2026-10-14 — Review follow-up. `changing_only_a_language_units_kind_orders_a_new_schedule` pins the kind distinction: a Write's value operand changing between an Operand Literal and a Bang keeps every Expression fact and changes only the unit's kind; the test fails when `RowInputs` equality compares anchor and Span alone. `shared_schedule` takes the cache and the inputs from one Map, so no caller can fill one revision's cache from another's inputs. Halt occupancy still scans every unit rather than the destination's row; that cost predates this effort and is left out of scope.
