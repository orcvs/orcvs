# 02 — Let movers enter Cells vacated by earlier Turns

**What to build:** West and north trains advance into Cells their leaders have vacated without
rejecting the Grid's Tick. Blocked movers retain their ordinary refusal and unrelated effects
still commit. Ordinary overwrites continue to interact with movers in actual Turn order.

**Blocked by:** None — can start immediately.

**Status:** claimed

**Parent:** [01 — Reconcile placement with value readiness](01-reconcile-placement-with-value-readiness.md).

The [placement semantics spec](../spec.md) is the contract. This slice establishes shared
placement admission and delivers it for Self-Banging Functions; emission scheduling follows in
issue 03. Parent issue 01 is a scope reference, not a prerequisite.

- [ ] Before changing execution, record the ADR distinguishing rule 3a, value inputs wait for
      potential writers, from rule 3b, placement observes occupancy at its Turn. Reconcile the
      relevant clauses of ADRs 0006, 0014, 0032, and 0034, preserving ADR 0036. State that emission
      scheduling is the remaining delivery slice rather than claiming the entire spec ships.
- [ ] Add failing regressions through `Source::execute` for adjacent west and north trains and
      an eastbound mover approaching a west train. Assert multiple committed Tick Grids,
      diagnostics, and an unrelated calculation or musical effect that previously lost its Tick.
- [ ] Derive successful-placement treatment from the existing Advance/Emit declaration. A
      Snapshot computation having attempted its Turn does not prohibit entering its now-empty
      Cells. Keep ordinary overwrite guards intact and avoid per-contact exceptions or new
      independently configurable admission flags.
- [ ] Preserve east/south blocked-follower behavior, odd and even gaps, static obstacles,
      horizontal self-overlap, alignment diagnostics, and Grid-edge refusals. Exercise trains
      reaching an edge on a later Tick and confirm that movement remains atomic.
- [ ] Verify that value inputs and Input Portal reads still wait for their potential suppliers,
      including placement into empty claimed operand Cells. Preserve collision Bang delivery,
      Halt, generated-code deferral, prior Bang cleanup, and genuine cycle rejection.
- [ ] Pin ordinary-overwrite/placement contests in both orders with initially empty destination
      Cells. An earlier nonempty overwrite blocks movement. A later ordinary overwrite can
      replace a mover's complete new Span without a late-write diagnostic; assert both the
      cleared origin and replacement content. The guard still protects any attempted Snapshot
      computation covered by an overwrite, including one whose attempted Turn failed.
- [ ] Add a mover-only property over well-formed arrangements and successive Ticks: movement
      and local refusal produce neither an executed-computation rejection nor a dependency
      cycle. Include adjacent and offset placements. Keep this property scoped to movers and
      empty Cells; the emitter extension belongs to issue 03.
- [ ] Keep each test at the existing production Source seam, using Tick Grids and Tick Plan
      effects rather than internal graph edges or test-only shipped parameters. Preserve the
      existing emission expectations until issue 03 intentionally changes their scheduling.
- [ ] Run scoped repository gates for changed crates and dependents with 32 local property
      cases. Record exact commands, results, and deferred CI coverage; review the complete diff.

## Comments

### 2026-10-02 — Approved slice, with ADR review coverage

The shared admission change precedes emission scheduling. The mixed overwrite cases are explicit
language consequences: occupancy admission does not grant protection from a later ordinary write.
No separate prefactor is required.

### 2026-10-03 — Implemented; verification on PR 195

[PR 195](https://github.com/orcvs/orcvs/pull/195) records ADR 0060, removes the stale
Snapshot-owner guard from successful placement, and adds nine behavior-level regressions,
including the mover property. Emission scheduling and its existing expectations remain for 03.

`PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(placement_)'`
against the pre-fix implementation selected 23 tests: 19 passed and four failed as expected
(west train, north train, convergence, and the generated mover property). The property shrank
to two offset northbound movers; its seed is retained. The first sandboxed attempt failed in
sccache, and the escalated retry completed after a long compilation.

`cargo fmt --all -- --check`, `node --test scripts/tests/roadmap.test.ts`,
`node scripts/roadmap.ts > /dev/null`, and `git diff --check` passed locally.
The user requested post-fix verification through CI instead of further local compilation.
The issue remains claimed until the PR checks verify the implementation.
