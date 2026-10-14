# 01 — Use one borrowed projection for Expression scheduling

**What to build:** Literal-only edits continue to reuse their schedule, while
edits to Expression scheduling facts invalidate it exactly as before. Equality
and computation construction obtain those facts through one borrowed projection
owned by the schedule module. See the accompanying spec for the confirmed design.

**Blocked by:** None — can start immediately

**Status:** ready-for-agent

- [ ] Equality and computation construction consume the same projection of
  scheduled Expression eligibility, Span, leading Function, entry Cell ranges,
  parents and three-way parse classification.
- [ ] Literal values, Tokens and Parser diagnostic text remain outside that
  projection; Source bytes are not copied.
- [ ] The projection borrows existing Parser structure, adds no projection
  allocations or stored duplicate structure, and owns the interpretation of
  scheduling facts rather than relocating a separate comparison.
- [ ] Unchanged rows remain shared, only dirty rows are compared, and Portal
  resolution, dependency derivation and ordering remain lazy.
- [ ] Tick Plans, diagnostics, activation, Turn order and the existing schedule
  invalidation distinctions remain unchanged. Language Unit comparison and lock
  construction retain their current behaviour until ticket 02.
- [ ] Existing fresh-versus-reused comparisons, stale-schedule counterexamples
  and edit properties pass; focused behavioural regressions cover any identified
  gaps without reproducing the projection representation.
- [ ] The repository's required crate and dependant gates pass with
  PROPTEST_CASES=32; the complete diff is reviewed and completion evidence recorded.
