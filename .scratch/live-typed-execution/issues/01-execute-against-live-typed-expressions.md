# 01 — Execute against live typed expressions

Status: resolved

Blocked by: language-map/10 — Reconcile historical scheduler tickets with the implemented partition and ADR 0034; 10 — Verify live execution through next-Tick Source.

Tags: release/v1

## What to build

Deliver the [live typed execution specification](../spec.md), synthesized from the confirmed
ADR 0034 contract and the integration prototype accepted by the user. The linked spec is the
canonical requirements document; this issue publishes it to the local tracker without duplicating it.

Use Parser-owned positioned expressions, derived execution order, pending spatial encodings and
typed nested results. Include nested suppression, original-anchor replacement, the revised failure
rules and correct next-Tick Source. Preserve fixed destinations and existing activation behavior.

## Completion

- [x] The specification's observable acceptance cases hold at the Source/Tick boundary.
- [x] Superseded partial-write, competing-writer and failed-supplier expectations are revised
      explicitly; unrelated partition and activation regressions remain protected.
- [x] Delivery preserves the completed partition work and covers the implementation obligations
      transferred from the audited, superseded cell-indexed-parse issues; it does not reimplement
      abandoned storage or candidate-policy requirements.
- [x] Applicable repository verification passes. Tracker changes pass both roadmap tests and
      generation; the release gate includes every open release-tagged issue in its dependency closure.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

Prototype capture: local branch `prototype/integrated-live-execution-adr34`, commit
`e0383397ba735e2677fbae8dea2d650d75be21f8`. It supports 24 exercised cases; it is not production
code or a general-scheduler proof. The reviewer reports a successful direct-file render at
1200×2600, including Grids, live state, all 24 original tabs and disabled controls. The adjacent
evidence documents distinguish that observation from subsequent prototype checks.

This is the specification's delivery issue. Issues 02–10 implement its acceptance contract;
issue 10 joins all implementation branches and records ADR acceptance. This issue cannot be
resolved while that final integration remains open. `language-map/10` also remains a direct
predecessor for historical scheduler reconciliation.

## Comments

2026-09-08: Predecessor audit completed before decomposition. `cell-indexed-parse/01` is resolved
against `593613c`; `02–07` are `wontfix` as superseded, with unfinished behavior transferred to the
spec. `64291cc` was planning documentation, not an array implementation. The delivery remains
blocked by the genuinely open historical-reconciliation task `language-map/10`.

The release gate now includes this issue and its predecessor. Both
`node --test scripts/tests/roadmap.test.ts` (10 tests) and
`node scripts/roadmap.ts > /dev/null` pass. The earlier missing-closure failure is repaired.

Review also added different-producer partial overlap and an explicit diagnostic for a Terminal
Output Function's invalid Portal configuration. The revised prototype has 26 model/presentation
walkthroughs; the attributed visual evidence covers the original 24-tab artifact.

2026-09-08: Review split the original issue 04 and renumbered later slices; final integration moved from 09 to 10. Added 10 as a delivery blocker, so completing historical reconciliation alone cannot make this issue actionable.

2026-09-08: Delivery resolved after historical reconciliation and implementation issues 02–10. All 26 acceptance cases are exercised through production Source/Tick; [delivery evidence](../evidence.md) records verification and bounded exclusions. This closure is the parent delivery assessment, separate from issue 10.
