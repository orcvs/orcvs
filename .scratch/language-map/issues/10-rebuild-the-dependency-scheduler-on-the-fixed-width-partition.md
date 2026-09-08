# 10 — Reconcile historical scheduler tickets with the accepted live-execution contract

**What to build:** Record which settled scheduler requirements remain implemented and which
ADR 0034 supersedes, before decomposing live-typed-execution. This is tracker reconciliation,
not a scheduler rebuild or a demand to implement the abandoned Cell-indexed array first.

**Blocked by:** cell-indexed-parse/01 — Implemented initial row partition under ADR 0033.

**Status:** resolved

**Tags:** release/v1

- [x] Review `tick-execution-order/01`, `02` and `03` against ADR 0034; preserve their resolved
      delivery history and identify each targeted supersession in the successor specification.
- [x] Record that ADR 0033's initial partition shipped while the join guard, reconstructed
      layout binding, second spelling classifier and old supplier/graph refusal behavior remain.
- [x] Reconcile ADR 0031/0032 references with ADR 0034's targeted revisions; retain unrelated
      activation, ordering, cycle atomicity and terminal behavior. Do not claim those ADRs are
      wholly unchanged or wholly repealed.
- [x] Ensure the successor owns the retained obligations from `cell-indexed-parse/02`–`07`,
      including half-typed diagnostics and same-Source regressions, without restoring mandatory
      storage or rejected candidate policies.
- [x] Keep release membership and the dependency closure valid; both
      `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` pass.

## Comments

2026-09-08: The seven Cell-indexed Parse statuses have been audited against `593613c` and
current code. Only ticket 01 is resolved; 02–07 are superseded with explicit residual routing.
`language-map/09` is superseded by ADR 0033/0034 and is no longer a human decision blocker.
This task precedes `live-typed-execution/01`; the successor implements the changed behavior.

The original branch-rebuild plan had already been withdrawn: it proposed rebasing scheduling
onto the corrected partition and dropping `4d6c17c` and `8e7bdce`. Those compensations were
implemented and reviewed. Their eventual migration must preserve their regression evidence;
rewriting branch history is neither necessary nor part of this task.

2026-09-08: Historical reconciliation completed for live-typed-execution delivery. Tickets
`tick-execution-order/01–03` retain their resolved implementation history. ADR 0033's initial
partition shipped at `593613c`; the remaining reconstructed binding, second spelling classifier,
join guard and supplier/graph refusals were the migration work, not unimplemented partitioning.
The successor specification and issues 02–10 own all transferred claim/rendering obligations;
mandatory arrays, identifier widths and the rejected candidate-policy experiment remain abandoned.
ADR 0034 revises only the named binding, partial/competing-write, nested and replacement rules;
activation lifetime, cycle atomicity, terminal ordering and Source/Playback separation survive.
See [delivery evidence](../../live-typed-execution/evidence.md) for implementation and checks.
