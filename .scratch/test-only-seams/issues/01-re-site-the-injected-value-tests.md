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

**Status:** resolved

- [x] Each of these tests assembles the schedule it exercises rather than configuring the
      production planning entry point.
- [x] No test in this set reads or writes the answer-substitution map.
- [x] Every behaviour the set asserted before it moved is still asserted after, with the same
      expected outcomes.
- [x] The crate builds and its tests pass in both the default and test configurations; the fork
      still compiles because `02` has not landed yet.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The seam these tests use
was introduced in the same commit as ADR 0034 and the sentence in that ADR which licenses it, so
the ADR is provenance rather than authorisation. See `03` for the amendment.

2026-09-10: Resolved by `2f6ac3c`. The five tests state their answer through a test module one
call below the planning entry point, so the Source, the schedule and every other computation's
Turn stay production. Two behaviour-neutral extractions made that possible: `Execution::new` took
the construction and the Bang-clearing pre-pass out of `execute`, and `opens_turn` took the Turn
prologue out of `take_turn`, so a stated answer faces the same suppression, nesting, syntax and
arity refusals the Turn it replaces faces. `Source::commit_tick` widened to the Source module so
the tests commit a plan the one way a Tick commits one.

One deviation from the third acceptance line, taken deliberately. `live_replacement_checks_retained
_arity_and_never_runs_new_anchors` counted one Interpreter call and now counts none: the stated
producer no longer interprets an answer that was discarded anyway. The behaviour the assertion
exists for is still asserted, and more strictly, with the snapshot proving the replacement was
delivered and the covered anchor suppressed.

Two guard tests were added with the seam, covering a stated answer that faces a refusal and one
stated at a Cell no computation anchors.
