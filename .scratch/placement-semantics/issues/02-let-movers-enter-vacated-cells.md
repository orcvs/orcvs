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


### 2026-10-03 — CI verified; contact review follow-up

The user-requested CI verification passed on `ba72330b`:
[Rust full gate, WASM and aggregate CI](https://github.com/orcvs/orcvs/actions/runs/37106846523)
and [benchmarks](https://github.com/orcvs/orcvs/actions/runs/37106846513).
This replaces the pending-CI statement above; it does not claim a local post-fix run.
The contact review below closes the newly identified stale classification when a mover blocks
against another mover that entered vacated Cells earlier in the Tick. Final review CI is recorded
on PR 195; this ticket remains claimed until that verification completes.

### 2026-10-03 — Address-code-review evidence

Changed: collision contact follows intact moved/emitted Function and blocked Bang spans, while
Source-effect writes mask their former Snapshot ownership. Every later write invalidates any
overlapping placed span. This preserves the fixed parse and gives generated units no new Turn.
Ordinary generated value contents are not reparsed into new Language Units between Turns.

Tests added or updated: whole moved-mover contact, partial moved-mover contact, partial contact
with a blocked mover's Bang, and unrelated Addition assertions in north/converging train tests.
The original 8-by-3 case failed first with `^^ contacts part of a Language Unit`; the additional
Bang case caught an intermediate fix losing that diagnostic (`left: []`). Both now pass.

Review ledger (user-supplied findings; R1 major, remaining actionable findings minor):

| ID | Outcome | Evidence |
| --- | --- | --- |
| R1 stale contact diagnostic | Fixed | `a_mover_contacts_the_whole_mover_that_entered_vacated_cells`, `a_mover_still_diagnoses_partial_contact_with_a_moved_mover`, and `a_mover_diagnoses_partial_contact_with_a_blocked_movers_bang` in `orcvs/src/source/tick.rs`; red evidence above. |
| R2 verification evidence | Fixed | Prior CI links above replace pending evidence. Omitting broad local recompilation was explicitly requested by the user; focused red/green now ran locally. |
| R3 unrelated effects | Fixed | North and converging trains assert Addition output each Tick. Coverage enhancement; no behavioral defect to regress. |
| R4 pair exception scope | Refuted | The accepted delivery split explicitly leaves emission scheduling and exception removal to issue 03. |
| R5 shared guard removal scope | Refuted | This ticket explicitly owns shared Advance/Emit admission; issue 03 owns scheduling. |
| R6 stale resolution lines | Refuted | Issue 07's resolution already cites test names. Line references belong to dated historical audit comments. |
| R7 commit evidence | Fixed traceability | The corrective commit records why the shared guard was removed. CLAUDE requires completion evidence, not commit headings; existing pushed history is retained. No behavior to regress. |
| R8 Advance rationale | Fixed | Restored the reason its two Portals write different content. No behavior to regress. |
| R9 predicate duplication | Refuted | Two exact bundle predicates answer different scheduling questions; a general dispatch helper adds no shared behavior. |
| R10 emitter predicate name | Fixed | `emits_without_vacating` states the relevant property. No behavior to regress. |
| R11 test names | Fixed | Sentence-style names match neighbouring tests; historical executed commands remain exact. No behavior to regress. |
| R12 native property explanation | Fixed | Native-only proptest dependency is stated beside the cfg. No behavior to regress. |
| R13 issue 07 complete | Verified; no defect | ADR 0006 collision cases retain their corresponding named regressions. |
| R14 no hard standards violations | Verified; no defect | No shipped test-only seam, lint suppression, unsafe or dependency changes. |

Commands run:

- `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(mover) | test(emission) | test(self_banging) | test(directional_bang)'` — failed red as described above; final run passed all 31 tests.
- `PROPTEST_CASES=32 target/debug/deps/orcvs-7273e44efb8dddf8 source::tick::test::` — passed all 185 Tick tests against the freshly compiled test binary, including 32 property cases.
- `cargo fmt --all -- --check` — passed.
- `node --test scripts/tests/roadmap.test.ts` — passed all 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.

Not run locally: workspace Clippy, workspace nextest, doctests and WASM — delegated to PR CI at
the user's request. macOS, browser tests, full property count and benchmark comparisons —
deferred to CI under repository policy.

Risks: no public API, unsafe, dependency, feature or concurrency change. Contact metadata adds
Tick-local span storage and scans; no performance improvement is claimed. Emission scheduling
remains issue 03 work.
