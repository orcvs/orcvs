# 02 — Carry Parser-owned positions through Source execution

Status: resolved

Blocked by: language-map/10 — Reconcile historical scheduler tickets.

Tags: release/v1

## What to build

Prefactor the existing Source-to-Tick path so operand positions and nested ownership come from the Parser. Preserve current observable behavior while making subsequent live updates possible without a reconstructed layout. This does not redo the shipped arity-based row partition or choose a mandatory storage representation.

## Completion

- [x] The Parser records original Function anchors, directly owned literal positions and nested connections, including invalid or missing inputs within the available row and Comment boundary.
- [x] Source/Tick execution consumes those positions end to end; independent reconstruction from token widths is removed from migrated execution callers. Any temporary compatibility adapter reads the new product rather than deriving positions again.
- [x] Existing fixed-Portal arithmetic, nested typed evaluation, Tick/anchor inputs, activation and resulting Source remain correct through the unchanged Source/Tick acceptance seam.
- [x] Long Expressions retain complete ownership, edits remain valid within the Grid, and no retired 32-record cap or three-policy claim experiment is reinstated.
- [x] A valid initial parse can be reused or rebuilt by affected row; no Source parsing is inserted between Function executions.
- [x] Keep each commit green by introducing the positioned form before moving callers. Do not require a CellRole array, fixed identifier widths, or a new public testing API.
- [x] Run the affected crate and dependant checks required by the repository; record migrated callers and any temporary adapter for the final contract cleanup.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
