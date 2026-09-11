# 05 — Prove the width term is unreachable where it lives

**What to build:** `every_declared_change_is_the_first_difference_for_some_pair` asserts that no pair of Functions reports `Width` as its first difference. In `lang` that assertion is tautological: `DECLARED_CHANGES` holds no `Width` row, so `replacing` cannot answer `Width` whatever it is handed. The test re-states what `04`'s sibling test already checks, and its comment reasons about `reserved_for` and `Reserved::Row` — both of which live in `orcvs` and neither of which that test executes.

The comment is honest that the assertion is documentation rather than evidence. The problem is that the `assert!` does not read that way, and the spec's claim — that the width term cannot fire at all — is the one claim in the reachability table that nothing proves.

Prove it where it can be proved. `reserved_for` is in `orcvs/src/source/tick.rs`, and the claim has two halves worth asserting separately: that no Function in `Function::ALL` declares a Sequence answer, and that a computation built from any of them therefore reserves a Cell pair rather than a row. The first is a fact about the Function table and belongs in `lang`; the second needs a Grid and belongs in `orcvs`.

This is the test that should fail the day a Function declares a Sequence answer — which is what makes it worth writing. It is the notice to whoever adds that Function that they have made the width term live.

**Blocked by:** 02

**Status:** resolved

- [x] The `Width` branch of the `lang` test is either removed or reduced to what it can honestly assert, with the reachability claim no longer resting on it.
- [x] A test in `lang` asserts no Function declares a Sequence answer.
- [x] A test in `orcvs` asserts that a computation over any Function in the table reserves a Cell pair, so the width comparison cannot differ.
- [x] Both tests name ADR 0036 and say, in a comment, that their failure means the width term has become reachable rather than that something broke.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --package <crate> --all-targets --locked -- -D warnings`, and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `lang`, `orcvs` and `console`.

## What the mutation did

The claim both tests carry is that no Function declares a Sequence answer, so the
mutation is to declare one: `Clock`'s answer column changed from `Elementwise` to
`Sequence` in `define_functions!`.

`PROPTEST_CASES=32 cargo nextest run --package lang --locked --no-fail-fast` then failed
`no_function_declares_a_sequence_answer` — "[Clock] declare a Sequence answer, so a
reserved width can now differ" — alongside the existing
`every_function_declares_how_wide_an_answer_it_gives`, which names the row.
`PROPTEST_CASES=32 cargo nextest run --package orcvs --locked` failed
`a_computation_over_any_function_reserves_a_cell_pair` with "Clock replacing computation 0
would reserve more than a Cell pair", which is the width term becoming reachable stated in
the place the reservation lives. The row was restored and the working tree checked clean
before the change was committed.
