# 09 — Replace Functions with retained inputs

Status: resolved

Blocked by: 08 — Replace computations with spatial values.

Tags: release/v1

## What to build

Extend structural admission to Function-valued replacement at an original anchor. Execute the replacement with the original parameters and connections, applying its literal signature without reshaping the Expression. Preserve misaligned Source for subsequent interpretation.

## Completion

- [x] A supplied typed Multiply at the original Addition anchor produces .x0204 and answer 08. Only the deferred Function-producing operation uses a supplied answer; other producers compute normally.
- [x] Replacing the outer Addition with Multiply keeps its nested child and Portal: the child computes 0C once and the replacement answers 18.
- [x] Replacing .v with .^ leaves .^C4: retained C4 decodes as Number C4, then fails the 00–7F domain. Retained typed nested results remain typed and follow the Function’s existing typed-input checks.
- [x] A unary replacement of binary Addition fails against two retained inputs during T. The next parse of .^0204 claims .^02 and diagnoses standalone 04. Do not recruit or discard inputs to fit a signature.
- [x] Function output .x at the + Cell leaves ..x101 and discards the original Addition. The new anchor gets no current-Tick turn; assert next-parse diagnostics against the real row width.
- [x] Preserve fixed Portal configuration, nested connections, complete-write admission and already-executed-target invariants established by prior slices.
- [x] Replacements changing activation requirements or output kind, changing destinations and same-Tick execution at new anchors remain explicitly outside the supported scope; do not silently authorize them.
- [x] Check success, arity/type/domain failure, rendered Source and next parse end to end; preserve unrelated activation and Sequence behavior.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
