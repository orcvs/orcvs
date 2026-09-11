# 04 — Close the gap above the table

**What to build:** `02` put a test around `DECLARED_CHANGES` that proves each named change is compared exactly once. The test iterates `ReplacementChange::ALL`, and `ALL` is hand-maintained — so a variant that is never added to it escapes the test that exists to catch exactly this.

The review proved it by mutation: adding a sixth variant to the enum, giving it its `Display` arm, and omitting it from `ALL` leaves **both** new tests green. The `Display` match forces an arm because it is exhaustive; nothing forces `ALL` membership. A sixth fact added to a replacement's refusal would therefore ship with no table row and no failing test, which is the defect class this whole effort exists to close, one level up from where it was closed.

`Function::ALL` does not have this problem: it is generated from the `define_functions!` rows, so it is complete by construction. The fix is to give `ReplacementChange` the same property — declare its variants once and generate both the enum and `ALL` from that declaration, so omitting a variant from `ALL` is not expressible rather than merely untested.

Do not solve this with a hand-written count assertion. `assert_eq!(ALL.len(), 5)` is a second hand-maintained fact beside the first, and the next person adds a variant and bumps the number.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] `ReplacementChange::ALL` is derived from the variant declaration rather than written out beside it.
- [ ] Adding a variant without adding it to `ALL` does not compile, or is not expressible.
- [ ] The mutation the review ran — add a variant, give it a `Display` arm, omit it from `ALL` — now fails a test or fails to build. Record which, and that it was tried.
- [ ] No change to what ships: the five facts and the four table rows are unchanged.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package <crate> --all-targets --locked -- -D warnings`, and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `lang` and `orcvs`, which depends on it.
