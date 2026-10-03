# 01 — Reconcile placement with value readiness

**What to build:** Implement the [placement semantics spec](../spec.md): value inputs wait for
their potential writers; an Advance or Emit tests occupancy at its own Turn. Remove the stale
ownership rejection and placement-only ordering exceptions through that declared distinction,
while preserving genuine data, activation, and overwrite dependencies.

**Blocked by:** None.

**Status:** ready-for-agent

## Acceptance

- [ ] Record an ADR before changing execution, and reconcile ADRs 0006, 0014, 0032, and 0034 with
      the separate value-readiness and placement-occupancy rules. Preserve ADR 0036's role.
- [ ] Add regressions through `Source::execute` for the selected contract, following the spec's
      Testing Decisions. Demonstrate the west/north train defect before implementing the fix.
- [ ] Derive scheduler and executor placement behavior from the existing operation declaration;
      ordinary placement into vacated Cells is admitted without treating it as a late overwrite.
- [ ] Preserve value-input and Input Portal readiness, collision Bang activation, Halt, ordinary
      overwrite protection, generated-code deferral, Bang lifetime, and atomic Tick publication.
- [ ] Replace the vacating-mover regression's workaround expectation with successful emission
      into the Cells vacated before its Turn; retain occupied-destination refusal coverage.
- [ ] Verify all specified placement cases, unrelated-effect preservation, and the constrained
      no-rejection property. Keep genuine cycle and late-overwrite regressions passing.
- [ ] Run the repository's scoped gates and record exact commands, results, and deferred checks.
      Link the delivery evidence here and resolve this issue only when the full spec is met.

## Comments

### 2026-10-02 — Specified from the language-design discussion

The decisive clarification is that visibility does not imply readiness: value inputs wait for
their suppliers, while placements see working Source at their Turn. The spec adopts that rule
within Orcvs's dependency schedule; Orca parity and simultaneous movement are not requirements.
