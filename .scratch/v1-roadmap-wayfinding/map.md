# Find the First Release Candidate route

Tags: wayfinder:map

## Destination

An implementation-ready First Release Candidate roadmap: the shipped language inventory,
release membership, dependency graph, Definition of Done, and proof gate are mutually consistent,
evidence-backed, and ready for implementation sessions to execute.

## Notes

- This effort plans and may correct issue-tracker artifacts; it does not change production code.
- The release includes the accepted numeric, Tick, spatial, Read/Write/Copy, and MIDI slice;
  ADR 0063 retired the Sequence value. Messaging transports and cross-version backward
  compatibility are not release promises.
- `CONTEXT.md` and accepted ADRs constrain the route. A concrete contradiction becomes its own
  decision rather than an incidental documentation edit.
- An `Improvement` joins the release route only when correctness, safe implementation, or measured
  release evidence requires it.
- Criterion history and its merge gate are authoritative performance evidence. Do not infer an
  inlining change from the older audit's now-stale benchmark premise.
- Every grilling ticket uses the `grilling` and `domain-modeling` skills. Architecture decisions
  also use `codebase-design`.
- Primary evidence includes `docs/research/lang-artifact-audit-2026-09-01.md`, the current code and
  manifests, `CONTEXT.md`, accepted ADRs, and the existing roadmap tickets.
- Execution is permitted only for applying resolved decisions to planning artifacts after the route
  is fully decided.

## Decisions so far

- The FRC ships the complete accepted numeric, Sequence, Tick, spatial, and five-Function MIDI
  slices. Bangs may occur in Sequences with compatibility checked by consuming Functions. General
  Source addressing, UDP, OSC, Application Command encoding, compatibility, and unrelated
  Improvement work are explicit deferrals. ADR 0085 has since settled UDP and OSC, which send a
  Payload of Cells, and they stay deferred from v1. See
  [01 — Name the shipped language inventory](issues/01-name-the-shipped-language-inventory.md).
- Expression/runtime pairing is limited to successfully parsed evaluable entries; incomplete,
  invalid, comment, diagnostic, and future non-evaluable syntax remain explicit non-value records
  behind the Language Map. Self-Banging Functions `^^`, `vv`, `<<`, and `>>` are root-only Source
  Functions with no operand or Sequence behaviour (`activation-representation/01`, CONTEXT.md). See
  [02 — Decide the Expression, Language Unit, and runtime value invariant](issues/02-decide-the-expression-language-unit-runtime-invariant.md).
- FRC evidence is conjunctive and candidate-bound: exhaustive finite laws, 256-case structured
  properties, exact target/feature gates, product persistence, fake plus physical MIDI proof, four
  rendered captures, named-baseline benchmark review, inventory traceability, and an unwaived human
  GO/NO-GO record are all required. See
  [03 — Define the release evidence contract](issues/03-define-the-release-evidence-contract.md).
- Release membership contains every open implementation and proof prerequisite for the decided
  inventory, while four unrelated maintenance tickets remain Improvements. Existing tickets are
  corrected in place, the obsolete Expression Map plan is retired, and new Activation, persistence,
  physical-MIDI, and exact-candidate work fills real gaps. Implementation edges express only real
  prerequisites; final evidence ordering belongs to the gate. See
  [04 — Reconcile roadmap membership and dependencies](issues/04-reconcile-roadmap-membership-and-dependencies.md).
- `v1-release/01` is the true sink over every tagged release branch; one longest chain is displayed
  as critical while all parallel work remains mandatory. The inventory-backed DoD, conjunctive
  evidence bundle, and unwaived GO/NO-GO close it, and the roadmap retains a settled gate as release
  history. See [05 — Design the release proof gate](issues/05-design-the-release-proof-gate.md).
- The decided route is applied: 41 tagged release tickets form one enforced Gate closure, four
  maintenance tickets remain Improvements, new Activation/persistence/candidate/MIDI evidence work
  fills the gaps, and the inventory-backed DoD and final gate now match the graph. See
  [06 — Apply the decided release roadmap](issues/06-apply-the-decided-release-roadmap.md).
- Four review-flagged ticket statements are reconciled: `04` no longer makes `sequence-values/01`
  wait on a representation prototype; Directional Bang emission is two Cells outside the emitter's
  Span; `midi-output-family/01` is resolved; `native-midi/02` records the WASM feature matrix.
  See [07 — Correct the ticket statements flagged in review](issues/07-correct-ticket-statements-flagged-in-review.md).
- The inventory's Current state is refreshed against `98bd5d0d` (2026-10-01). Each member links
  the tests that prove it, and `01` lists the
  [members still short of the release](issues/01-name-the-shipped-language-inventory.md#members-still-short-of-the-release). See
  [08 — Refresh the inventory's Current state against the code](issues/08-refresh-the-inventorys-current-state.md).

## Out of scope

- Implementing production Rust, UI, persistence, MIDI, or WASM behavior.
- Designing concrete Source Read/Write addressing or an Application Command transport. ADR 0085
  settles UDP and OSC.
- Promising Source, persistence-format, or Rust-API compatibility across pre-release versions.
- Reopening accepted language semantics without a concrete contradiction.
- Pulling independent cleanup into the release solely because it is inexpensive.

## Comments

### Audit against origin/main cf01e3c — 2026-10-13

The Notes bullet on release scope was corrected for ADR 0063 and ADR 0070. Three Decisions entries
are history that later ADRs and tickets overtook. The first entry's Sequence slice, its Bangs in
Sequences and its "General Source addressing" deferral predate ADR 0063, which retired the Sequence
value, and ADR 0070, which shipped Read and Write Functions on main. The `07` entry's
"`native-midi/02` records the WASM feature matrix" records a feature that `midi-port-ownership/03`
and `05` later removed: `orcvs/Cargo.toml` has no `native-midi` feature and `midir` is a `console`
dependency (`console/Cargo.toml:130-131`). The "Designing concrete Source Read/Write addressing"
line under Out of scope bounded this effort, not the language. `01`'s inventory was refreshed on
the same date.

### Decided 2026-10-13 by Toby

The Functions ADR 0070 added (the Reads, Writes, Track, Push and the absolute Copy) join the
release contract; the Notes bullet and `01`'s inventory state it, and Source Read/Write addressing
is no longer a deferral.
