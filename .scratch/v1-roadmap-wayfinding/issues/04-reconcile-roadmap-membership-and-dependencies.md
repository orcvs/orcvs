# Reconcile roadmap membership and dependencies

Type: grilling

Blocked by: 01 — Name the shipped language inventory; 02 — Decide the Expression, Language Unit, and runtime value invariant.

Status: resolved

## Question

Given the decided release inventory, the decided syntax-to-runtime invariant, current
`origin/main`, and the audited findings, which existing tickets belong to `release/v1`, which are
independent `Improvement` work, and which must be rewritten, renumbered, merged, closed, or newly
created? Produce a dependency graph whose edges express real implementation prerequisites rather
than review order or historical accident.

## Answer

Release membership follows the decided inventory and evidence contract. `Blocked by` expresses
only a prerequisite without which the ticket cannot be implemented correctly; candidate evidence
collection and final review order belong to the release proof gate instead of being projected back
through the implementation graph.

### Open work in `release/v1`

- `lang-foundations`: 02, 03, 06, 07, 08.
- `language-map`: 01–03.
- `orcvs-language-migration`: 05 and 07.
- `sequence-values`: 01–04.
- `tick-functions`: 01–04.
- `spatial-tick-planning`: 01–05, with Activation movement blocked by the new prototype.
- `midi-output-family`: 01–04.
- `property-testing`: 01–05 and 07.
- `restyle-egui-console`: 01–03.
- New Activation representation prototype, product-persistence proof, physical MIDI smoke, and
  exact-candidate verification work.
- The final `v1-release` proof gate.

Resolved language migration, Playback Engine, benchmark, CI-tier, crate-boundary, inherited-defect,
and MIDI-selection tickets remain satisfied prerequisites or evidence history. They do not become
open release work merely to make them appear in the route.

### Independent Improvement work

- `lang-foundations/01`: retire the unused Portal placeholder.
- `lang-foundations/04`: simplify parser borrowing and Atom handoff.
- `lang-foundations/05`: keep tracing setup out of the shipped dependency graph.
- `property-testing/06`: record the agent/glossary authority rule.

These remain real maintenance work but are not required for correctness, safe implementation, or
the decided release evidence. In particular, `lang-foundations/07` must not depend on parser
borrowing cleanup.

### Required corrections in place

- Rewrite `lang-foundations/07` around successfully parsed evaluable entries. Invalid and incomplete
  analysis records do not receive paired placeholder values. Remove its dependency on issue 04.
- Keep `lang-foundations/08` after compiler-checked Function definitions and the corrected paired
  entry model; it owns strict parsing versus permissive Live Edit analysis.
- Rewrite `property-testing/03` around strict-parser and permissive-analysis totality and recovery.
  Rewrite issue 04 against the implemented Language Map. Rewrite issue 05 as exhaustive proof for
  the complete numeric family and conversions and remove its false proptest-harness dependency.
- Rewrite `sequence-values/01` to admit Bang Atoms while granting the directional activation
  spelling no operand or Sequence behavior before the prototype decides its representation. Clarify issue
  02 that Equality is ADR 0011's whole-value predicate: it returns one scalar Bang only when every
  broadcast pair is equal, otherwise no value; it never creates missing Sequence elements.
- Make Directional Bang movement wait for the Activation prototype and then rewrite its internal
  representation criteria to the selected model without changing fixed observable behavior.
- Rewrite `restyle-egui-console/03` to produce candidate-bound native/WASM × wide/tall captures with
  the decided metadata and checklist.
- Mark `collapse-expression-map/spec.md` superseded by the Language Map effort and invariant. Do not
  create a competing ticket set.
- Preserve existing ticket identities and numbering except where new files require new numbers.

### New work

- A focused Activation prototype compares a distinct spatial Language Unit with a self-reproducing
  Function/value implementation across recognition, scheduling, Source writes, collision, and
  accidental Expression/Sequence capability. It blocks Activation movement, not Expression parsing.
- A product-persistence ticket proves model authority/rebuild and actual native/WASM
  save–restart–reload paths.
- A physical MIDI evidence ticket records the candidate SHA, OS, device, procedure, observation,
  result, and reviewer. Fake-adapter proof remains in implementation tests.
- A dedicated release-candidate task/workflow runs the full native, persistence, WASM, and Firefox
  matrix for the nominated SHA and publishes authoritative results without moving the full cost to
  every ordinary pull request.
- Candidate visual evidence reuses the corrected `restyle-egui-console/03`.
- Inventory traceability, Criterion named-baseline/history review, known-defect review, accepted
  deferrals, and final evidence assembly stay inside the final proof gate rather than becoming
  link-copying tickets.

### Minimal implementation graph

- Canonical encodings (`property-testing/07`) precede typed Raw Play and lexical Language Map work.
- Compiler-checked Functions (`lang-foundations/02`) precede typed Raw Play, typed extraction, strict
  analysis separation, and addition of the remaining Function families.
- Typed Raw Play (`lang-foundations/03`) precedes centralized typed extraction
  (`lang-foundations/06`).
- Correct valid-entry pairing (`lang-foundations/07`) and compiler-checked Functions precede strict
  versus permissive analysis (`lang-foundations/08`).
- Canonical encodings and compiler-checked Functions precede `language-map/01`; valid-entry pairing,
  strict/permissive analysis, and lexical partitioning precede `language-map/02`; issue 02 precedes
  migration of consumers in issue 03.
- Numeric conversions and the complete numeric family precede compatible Sequence broadcasting;
  `sequence-values/01` precedes Sequence issues 02–04.
- `language-map/03` precedes Portal result writes, spatial effect ordering, and Tick/Position inputs.
- Spatial ordering precedes Bang activation/expiry and Jump chains. Bang activation precedes Halt
  and the observable Directional Bang path; the Activation prototype additionally precedes
  Activation movement.
- Tick/Position input precedes all Tick Functions and Timed Play scheduling. Sequence broadcasting
  precedes deterministic Random; Portal result writes precede visible feedback Functions.
- Central typed extraction precedes the remaining numeric, Tick, and MIDI Function implementations.
- MIDI command construction is independent of Bang activation. Tick planning alone owns whether a
  root evaluates; integration tests prove inactive terminal roots emit nothing and active roots add
  commands in Tick Plan order.
- MIDI command generalization precedes Timed Play and Control Change/Pitch Bend. Timed Play precedes
  Monophonic ownership.
- The property harness precedes only large structured properties, not finite exhaustive tests.
- Restyle issues remain 01 → 02 → corrected 03.

The final proof phase begins only after all tagged implementation and semantic-proof work is
complete. It then collects exact-candidate target results, four captures, persistence evidence,
physical MIDI evidence, benchmark comparison/history, traceability and defect review before the
human GO/NO-GO. Those are gate dependencies and checklist requirements, not artificial edges among
implementation tickets.
