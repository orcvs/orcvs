# 06 — Settle failed suppliers without erasing inputs

Status: resolved

Blocked by: 04 — Compose and order spatial writes.

Tags: release/v1

## What to build

Let spatial consumers proceed after a failed producer using the operand characters that remain, while nested consumers still require a returned typed value. Make failure diagnostics and publication reflect the delivery path rather than suppressing every dependent Function.

## Completion

- [x] A failed spatial supplier emits a diagnostic and no write; a consumer with surviving original operand characters executes using them.
- [x] An earlier successful writer supplies 05, a later supplier at that destination fails, and a receiver with other operand 01 answers 06. The failed supplier erases neither original Cells nor successful earlier writes.
- [x] A failed nested child supplies no typed result and its parent fails; it does not decode old child Source as fallback.
- [x] Keep successful absence distinct from evaluation failure and preserve current no-write behavior for absence.
- [x] Replace the old failed-data-supplier and suppressed-consumer expectations explicitly with the new regressions, retaining unrelated diagnostics and activation coverage.
- [x] Ordinary evaluation failure preserves successful independent effects; cycles still reject the whole Tick. Assert the resulting Source and relevant diagnostics, not only an empty command list.
- [x] Run the applicable affected-crate and Source/Tick tests; record the deliberate ADR 0034 migration in the delivery evidence.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

This ticket and issue 07 can proceed independently after issue 04. Their parent-failure-after-child-write interaction is accepted in issue 10.

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
