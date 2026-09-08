# 08 — Replace computations with spatial values

Status: resolved

Blocked by: 05 — Decode pending spatial inputs at consumption; 07 — Deliver one nested result through both paths.

Tags: release/v1

## What to build

Let a computed value replace a nested or top-level Function before its turn. Suppress the replaced computation and its descendants, use the replacement as the parent input, and preserve every untouched character for the next Tick.

## Completion

- [x] An actual producer writes 05 over Multiply in .+02.x0304. The parent answers 07 and Source becomes .+02050304; Multiply does not execute or emit.
- [x] Include a grandchild with its own Portal. Replacement gates the computation and descendants before any can run, and no detached child emits or supplies a typed result.
- [x] Top-level value replacement suppresses the root and its nested Portals. Source 0502.x0304 preserves the former child spelling, which becomes a new candidate only at the next parse.
- [x] Spatial replacement is interpreted through the receiving literal signature, not the producer’s type. Retain other parent input positions and connections.
- [x] Introduce original-anchor structural write admission and descendant ordering gates without granting a turn to newly anchored code. An output reaching an already-executed computation is an ordering defect, never a reason for re-execution.
- [x] The next parse of .+02050304 claims .+0205 and diagnoses leftover 03 and 04. Deeper replacement can expose a former descendant as a next-Tick root; do not force the parsed forest to equal the live tree.
- [x] Full-write validation, cycles and inactive ownership remain correct when structural writes are involved. Combined failed-producer behavior is verified in issue 10 after the failure and replacement branches join.
- [x] Verify actual Source, admitted effects, suppression and next interpretation at Source/Tick; add no Function-producing operation.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

Pending decoding (05) and nested Portal ownership (07) are prerequisites. Failed-supplier integration joins in issue 10.

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
