# 03 — Delete the answer-substitution fork, and give the scheduler one predicate in both builds

**What to build:** Delete the `cfg(test)` answer-substitution seam from the Tick scheduler and
executor. The reservation predicate today has three disjuncts under test and two in production, and
the computation record carries a field in one build and not the other, so the tests that exercise
scheduling exercise a module the release never contains. After this ticket the predicate has one
shape, the record has one shape, and no shipped module compiles differently under test.

Amend ADR 0034 in the same change. Its deferral of a Source operation producing Function values
stands; the sentence stating that the execution design can be tested with supplied typed Function
values is struck, because that sentence is what the fork was built on and it arrived in the same
commit as the fork.

**Blocked by:** 01 — Re-site the injected-value tests; 02 — Re-site the Sequence reservation tests.

**Status:** resolved

- [x] The answer-substitution map, the per-computation Sequence flag, and both arms of the
      predicate helper that reads it are gone.
- [x] The reservation predicate reads the Function table and the settled reservations of a
      computation's children, and nothing else, in every build.
- [x] The executor no longer substitutes a result after interpretation.
- [x] No `cfg(test)` or `cfg(not(test))` attribute remains on any field, function, or parameter of
      the Tick scheduler or executor that changes what the shipped module computes. Test modules
      themselves are unaffected.
- [x] ADR 0034 records the amendment and the reason, and no ADR sentence licenses a test-only
      capability in shipped code.

## Comments

Split out of the architecture review of the Sequence branch, 2026-09-10. The destination half of
the same struct is `04` through `08`; this ticket leaves it standing.

2026-09-10: Resolved by `57f705c`. A deletion rather than a migration, because `01` and `02` had
already removed every reader. 76 lines out, 17 in. The reservation predicate is
`answers_sequence() || widened` in every build, and the executor delivers what the Interpreter
answered.

With the substitution gone the `Execution` had no reader for its `Configuration`, so the field went,
and with it the parameter that fed it and `execute`'s own `Configuration` parameter. `execute` now
takes no `Configuration` at all. `Configuration` itself stays for `destinations`, which is `04`
through `08`.

One `cfg(test)` statement remains inside a shipped function — the record of what the Interpreter
was handed. It changes nothing a Tick computes, so it is not this fork; `10` owns it.
