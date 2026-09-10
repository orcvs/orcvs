# 01 — Re-site the injected-value tests below the Tick plan

**What to build:** The tests that state a typed Function or Atom answer in place of the one a
computation would compute stop reaching a `cfg(test)` field inside the shipped scheduler, and
construct their input directly instead. What they prove is unchanged: Function replacement at an
original anchor retains its inputs and connections, a replacement that changes activation
requirements or output kind is refused, a replacement's signature reinterprets retained literals,
and a scalar projection that is not a Cell pair is rejected at a row edge.

Nothing is deleted from the shipped module by this ticket. It removes users, so the fork can be
deleted without argument in `03`.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Each of these tests assembles the schedule it exercises rather than configuring the
      production planning entry point.
- [ ] No test in this set reads or writes the answer-substitution map.
- [ ] Every behaviour the set asserted before it moved is still asserted after, with the same
      expected outcomes.
- [ ] The crate builds and its tests pass in both the default and test configurations; the fork
      still compiles because `02` has not landed yet.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The seam these tests use
was introduced in the same commit as ADR 0034 and the sentence in that ADR which licenses it, so
the ADR is provenance rather than authorisation. See `03` for the amendment.
