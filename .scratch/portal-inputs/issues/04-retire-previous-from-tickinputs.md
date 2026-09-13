# 04 — Retire Previous from TickInputs

**What to build:** Remove the `Previous` channel from `TickInputs` and delete the special-case
Portal read path that existed only to feed it. `TickInputs` carries only non-Source interpretation
inputs (Tick; Anchor when Random lands). All Portal content reaches Functions through Portal input
binding.

**Blocked by:** 03 — Bind Portal inputs at Turn for Increment and Interpolation.

**Status:** resolved

- [x] `TickInputs` no longer carries a Previous field; nothing in the Evaluator reads Portal
      content except the Portal input binding seam.
- [x] The `Previous` enum and one-off Portal read helpers used only for that channel are removed.
- [x] Turn execution supplies Portal inputs exclusively through the binding introduced in ticket
      03.
- [ ] Workspace tests and gates pass with no remaining references to the retired path.

## Comments

Removed the Previous enum, TickInputs field/accessor, previous_number, and portal_previous.
TickInputs now contains only Tick and Anchor; all constructor callers and doctest examples
were migrated. The parent tick-functions/03 issue remains untouched.

Verification:
- The validation-order test failed against the original behavior (red), then passed.
- `cargo nextest run --package lang --locked -E 'test(functions::tick)'` passed: 30 tests.
- `cargo nextest run --package orcvs --locked -E 'test(increment) | test(interpolation) | test(a_previous_that_is_not_a_number)'` passed: 6 tests.
- `cargo clippy --package lang --package orcvs --all-targets --locked -- -D warnings` passed.
- `cargo nextest run --package lang --package orcvs --locked -E 'test(functions::tick) | test(feedback_reads_a_portal_write)'` passed: 31 tests.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` passed before the subsequent workspace test compilation was stopped.
- `node --test scripts/tests/roadmap.test.ts` passed: 10 tests.
- `node scripts/roadmap.ts > /dev/null` passed.
- `git diff --check` passed.

Final workspace tests (`PROPTEST_CASES=32 cargo nextest run --workspace --locked`) were stopped
during compilation at the user's request. Workspace doctests, rustdoc, full feature/platform
verification, full-case proptest, and benchmark comparisons are deferred to CI. No dependencies,
features, unsafe code, or concurrency changes were introduced; this is internal binding API and
active language design work. No performance improvement has been measured locally.
