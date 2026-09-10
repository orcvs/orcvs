# 02 — Re-site the Sequence reservation tests, and delete the two that assert a fake declaration

**What to build:** The tests that state a Sequence answer through the shipped scheduler's
`cfg(test)` field construct their reservation directly instead. What they prove is unchanged: a
Sequence result reaches its destination Cells, a result that leaves its row or its Grid writes no
Cell of it, two overlapping Sequence results resolve Cell by Cell, an empty Sequence plans no write
and no diagnostic, a reservation the write stopped short of suppresses nothing, a reservation
covering its own producer orders nothing against it, and a Sequence writing over its own producer
rejects the Tick.

One test in the set is deleted rather than moved: the one asserting that a pervasive parent widens
over a Sequence-answering child. It asserts that a stated answer derives a reservation width, which
is the fake declaration itself — there is nothing true in it to preserve while no Function declares
a Sequence answer. `sequence-values/05` reintroduces it against the Range rows when those exist.

The coverage this costs is the whole of ADR 0036's reservation derivation, not just the widening.
Once no test states a Sequence answer through the scheduler, `supplies_sequence()` is false in every
build, and no Function declares a Sequence answer, so the bottom-up pass that derives reservations
can only ever answer `Reserved::Pair`: every `Reserved::Row` in the crate is one a fixture states.
`sequence-values/05` owes the pass as a whole against a declared Range row.

This ticket was written expecting two deletions, the second being a test asserting that a
Sequence-answering computation reserves through the end of its row. No test asserted that: the
closest, `live_a_sequence_reservation_orders_every_computation_its_write_can_reach`, asserts the
ordering and suppression a row-wide reservation buys, which is a consequence of the reservation and
stays true when the fixture states the width instead of deriving it. It is retained. The width
itself is written down as an acceptance line on `sequence-values/05` alongside the widening, so both
are owed against a declared Range row even though only one had a test to delete.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Each retained test assembles the reservation it exercises rather than stating an answer
      through the production planning entry point.
- [ ] The derivation test is deleted, and `sequence-values/05` carries an acceptance line requiring
      ADR 0036's reservation derivation as a whole — the row-wide width, the widening, and the
      ordering each buys — to be proven against a declared Range row.
- [ ] No test in the crate reads or writes the answer-substitution map after this ticket.
- [ ] The crate builds and its tests pass in both the default and test configurations.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The two deleted tests are
the ones the review counted as wins for building Range first; they are coverage of the seam, not of
the scheduler.
