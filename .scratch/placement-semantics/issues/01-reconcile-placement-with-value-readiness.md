# 01 — Reconcile placement with value readiness

**What to build:** Implement the [placement semantics spec](../spec.md): value inputs wait for
their potential writers; an Advance or Emit tests occupancy at its own Turn. Remove the stale
ownership rejection and placement-only ordering exceptions through that declared distinction,
while preserving genuine data, activation, and overwrite dependencies.

**Blocked by:** None.

**Status:** resolved

## Acceptance

- [x] Record an ADR before changing execution, and reconcile ADRs 0006, 0014, 0032, and 0034 with
      the separate value-readiness and placement-occupancy rules. Preserve ADR 0036's role.
      *(ADR 0060:5-8.)*
- [x] Add regressions through `Source::execute` for the selected contract, following the spec's
      Testing Decisions. Demonstrate the west/north train defect before implementing the fix.
      *(`orcvs/src/source/tick.rs:1030,1041`; red run in 02's 2026-10-03 comment.)*
- [x] Derive scheduler and executor placement behavior from the existing operation declaration;
      ordinary placement into vacated Cells is admitted without treating it as a late overwrite.
      *(`deliver_source_effect`, `orcvs/src/source/tick/execution.rs:759`, carries no
      Snapshot-owner guard; `orders_contacts` in `producer_edges`,
      `orcvs/src/source/tick/dependencies.rs:239`.)*
- [x] Preserve value-input and Input Portal readiness, collision Bang activation, Halt, ordinary
      overwrite protection, generated-code deferral, Bang lifetime, and atomic Tick publication.
      *(02's criteria; `orcvs/src/source/tick.rs:1074-1143`.)*
- [x] Replace the vacating-mover regression's workaround expectation with successful emission
      into the Cells vacated before its Turn; retain occupied-destination refusal coverage.
      *(`orcvs/src/source/tick.rs:1353,1370`.)*
- [x] Verify all specified placement cases, unrelated-effect preservation, and the constrained
      no-rejection property. Keep genuine cycle and late-overwrite regressions passing.
      *(`orcvs/src/source/tick.rs:1191,1210,1424`.)*
- [x] Run the repository's scoped gates and record exact commands, results, and deferred checks.
      Link the delivery evidence here and resolve this issue only when the full spec is met.
      *(02's 2026-10-03 "Commands run"; 03's 2026-10-11 comment.)*

## Comments

### 2026-10-02 — Specified from the language-design discussion

The decisive clarification is that visibility does not imply readiness: value inputs wait for
their suppliers, while placements see working Source at their Turn. The spec adopts that rule
within Orcvs's dependency schedule; Orca parity and simultaneous movement are not requirements.

### 2026-10-13 — Audit against origin/main cf01e3c

Resolved. Delivered by [02](02-let-movers-enter-vacated-cells.md) (PR #195, `825f41e`) and
[03](03-apply-turn-local-occupancy-to-emissions.md) (PR #213, `68fcc18`); their comments carry the
evidence each box above cites. ADR 0060:3 records both slices as delivered. Issue 04 is an
independent investigation and does not hold this open. ADRs 0065, 0068 and 0072 later amended the
cycle and activation clauses named above ("atomic Tick publication", "collision Bang activation").
