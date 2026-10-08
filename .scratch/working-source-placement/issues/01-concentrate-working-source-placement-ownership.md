# 01 — Concentrate working Source placement ownership

**What to build:** One private Tick-local module owns working Cells, intact
generated placements and obscured Snapshot spans together. It keeps those facts
coherent through admitted writes and classifies contact against the current
working Source. Execution retains activation, suppression, interpretation and
Tick Effects. Preserve the Source and diagnostics a performer observes.

**Blocked by:** None — can start immediately.

**Status:** resolved

## Acceptance criteria

- [x] Working Cells, intact generated placement geometry and obscured Snapshot spans have one owner. Callers do not independently update those facts or reconstruct their combined contact rule.
- [x] Each admitted write updates Cells and placement facts atomically, including invalidating intact generated placements overlapped by an ordinary output write. A refused write changes none of those facts.
- [x] Execution applies an admitted write and records its Tick Effect through one private operation, keeping the observable write and working Source mutation together. The placement module does not take ownership of activation, suppression, interpretation or Tick Effects.
- [x] The module owns contact classification against intact generated placements and Snapshot units still visible in the current working Source. It retains enough placement provenance to distinguish complete and partial contact without reparsing the Source.
- [x] Preserve ADR 0060's distinction between Snapshot ownership of computations and current working vacancy. Generated Functions become eligible for a Turn on a later Tick; placing their Cells does not create a current-Tick computation.
- [x] Preserve ADR 0034's retained parsed structure and atomic Tick publication. State belongs to one Tick and its Source Snapshot, with no placement metadata persisted across Ticks.
- [x] Keep the Source Tick interface as the principal test surface. Preserve coverage for moved and blocked placement contact, partial overlap, newly moved Cells overwritten after a Turn, supplied operands and Input Portals, and generated Functions first executing on a later Tick.
- [x] Inspect existing Source-level coverage for writes that invalidate intact generated placement geometry. Add a focused regression only where an observable overlap or contact outcome is missing; verify resulting Source and diagnostics rather than internal bookkeeping.
- [x] Keep the module private. Introduce no public interface, adapter without actual variation, dependency, or shipped parameter or branch populated only by tests.
- [x] Review the complete diff and run repository-required verification for affected crates with PROPTEST_CASES=32. Preserve native, WASM and declared feature behavior; defer the broader checks designated for CI.

## Scope and constraints

The improvement concentrates the compound working Source invariant and contact
decision. Moving fields or forwarding calls while leaving coordinated updates
and classification spread across callers does not satisfy the ticket.

Existing placement semantics remain governed by ADRs 0034 and 0060. No new
domain term or language-design ADR is required. Dependency ordering and live
operand resolution remain separate responsibilities.

This is one cohesive refactor: splitting state ownership from contact
classification would leave the invariant divided. Make no performance
improvement claim without a reproducible benchmark or profile.

## Comments

**2026-10-08 — implemented in `8d201b37`.** `orcvs/src/source/tick/execution/working.rs` holds the private `WorkingSource`: working Cells, intact placements and obscured spans change only through `WorkingSource::apply(WriteKind, &SpanWrite)`, and `WorkingSource::contact` classifies contact. `Execution::write` applies a write and records its Tick Effect together. With output-write invalidation of placements disabled the existing suite still passed, so `a_mover_meets_the_overwrite_of_a_moved_mover_as_source_content` pins it. A Jump's Input Portal still classifies against the Snapshot Map alone, so a Jump reading a unit placed this Tick across two Snapshot units is diagnosed as invalid; unchanged here and a candidate to revisit.
