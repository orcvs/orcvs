# Find the First Release Candidate route

Tags: wayfinder:map

## Destination

An implementation-ready First Release Candidate roadmap: the shipped language inventory,
release membership, dependency graph, Definition of Done, and proof gate are mutually consistent,
evidence-backed, and ready for implementation sessions to execute.

## Notes

- This effort plans and may correct issue-tracker artifacts; it does not change production code.
- The release includes the accepted numeric, Sequence, Tick, spatial, and MIDI slice. Source
  Read/Write addressing, messaging transports, and cross-version backward compatibility are not
  release promises.
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
  Improvement work are explicit deferrals. See
  [01 — Name the shipped language inventory](issues/01-name-the-shipped-language-inventory.md).
- Expression/runtime pairing is limited to successfully parsed evaluable entries; incomplete,
  invalid, comment, diagnostic, and future non-evaluable syntax remain explicit non-value records
  behind the Language Map. Activation's representation awaits a focused prototype while its
  spatial behavior stays fixed. See
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

## Not yet specified

- The inventory may expose contradictions between accepted language semantics and feasible release
  slices. Each concrete contradiction graduates into a dedicated decision ticket.
- The invariant decision may expose additional parser-recovery or diagnostic-model decisions that
  cannot yet be stated without assuming the relationship between syntax and runtime values.
- The evidence contract may expose target-specific proof work whose exact question depends on what
  native, WASM, persistence, or MIDI capability is actually shipped.
- Roadmap reconciliation may expose stale or overlapping tickets beyond the conflicts already
  created by the numeric-conversion and benchmark work merged from `origin/main`.

## Out of scope

- Implementing production Rust, UI, persistence, MIDI, or WASM behavior.
- Designing concrete Source Read/Write addressing or UDP, OSC, and Application Command transports.
- Promising Source, persistence-format, or Rust-API compatibility across pre-release versions.
- Reopening accepted language semantics without a concrete contradiction.
- Pulling independent cleanup into the release solely because it is inexpensive.
