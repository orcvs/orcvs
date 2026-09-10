# 02 — Re-site the Sequence reservation tests, and delete the two that assert a fake declaration

**What to build:** The tests that state a Sequence answer through the shipped scheduler's
`cfg(test)` field construct their reservation directly instead. What they prove is unchanged: a
Sequence result reaches its destination Cells, a result that leaves its row or its Grid writes no
Cell of it, two overlapping Sequence results resolve Cell by Cell, an empty Sequence plans no write
and no diagnostic, a reservation the write stopped short of suppresses nothing, a reservation
covering its own producer orders nothing against it, and a Sequence writing over its own producer
rejects the Tick.

Two tests in the set are deleted rather than moved: the one asserting that a Sequence-answering
computation reserves through the end of its row, and the one asserting that a pervasive parent
widens over a Sequence-answering child. Both assert that a stated answer derives a reservation
width, which is the fake declaration itself — there is nothing true in them to preserve while no
Function declares a Sequence answer. `sequence-values/05` reintroduces both against the Range rows
when those exist.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Each retained test assembles the reservation it exercises rather than stating an answer
      through the production planning entry point.
- [ ] The two derivation tests are deleted, and `sequence-values/05` carries an acceptance line
      requiring both behaviours to be proven against a declared Range row.
- [ ] No test in the crate reads or writes the answer-substitution map after this ticket.
- [ ] The crate builds and its tests pass in both the default and test configurations.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The two deleted tests are
the ones the review counted as wins for building Range first; they are coverage of the seam, not of
the scheduler.
