# 07 — Deliver one nested result through both paths

Status: resolved

Blocked by: 04 — Compose and order spatial writes.

Tags: release/v1

## What to build

Allow a nested Function to compute once, return its typed answer to its parent and project that answer through fixed Portals during the same execution. Both consumers wait for that computation, and each failure is handled at the correct boundary.

## Completion

- [x] Nested Multiply computes 0C once; its parent answers 0E and its spatial consumer answers 0D. The dependency schedule includes nested ownership and both delivery paths.
- [x] Extend the existing internal fixed-destination configuration for nested anchors and ordered emissions. No new Source operation, authored Portal syntax or production tracing API is introduced.
- [x] An inactive parent suppresses the child and every child Portal without evaluation-time diagnostics; syntax and configuration validation remain independently visible.
- [x] A child’s complete write crossing the Grid boundary applies neither Cell and diagnoses; its successful typed 0C remains available so the parent answers 0E.
- [x] A Portal configured on a Terminal Output Function diagnoses as invalid configuration. Reject only that Portal, creating no spatial edge or write, and retain ordinary activation and terminal effect behavior. Configuration validation must not activate an inactive root.
- [x] A computation still answers a value or a terminal effect. Portal projection does not turn the Evaluator result into both. Observe exactly-once execution using the existing test boundary if its two outputs alone cannot distinguish duplicate computation.
- [x] Check nested cycles and self-dependency prevent publication of all effects, including independent terminal work. Preserve typed Numeric Conversion and existing Sequence behavior.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

Existing nested evaluation supplies the starting point. This ticket and issue 06 can proceed independently after issue 04; issue 10 owns their joined parent-failure case.

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
