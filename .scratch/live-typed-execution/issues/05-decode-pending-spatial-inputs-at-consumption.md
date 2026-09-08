# 05 — Decode pending spatial inputs at consumption

Status: resolved

Blocked by: 04 — Compose and order spatial writes.

Tags: release/v1

## What to build

Keep spatial literal encodings pending until the receiving Function executes, then decode with its current literal signature. Extend the write-composition path from issue 04 without treating transient characters as intermediate execution errors or spatial producer types as nested values.

## Completion

- [x] Whole and partial spatial output use the current receiving literal signature. A computed Note C5 written into Addition with other input 01 yields C6; a nested Note remains typed and is rejected by arithmetic.
- [x] The Note sequence E4 to EA to E5 produces no intermediate execution error; the consumer answers 4C. The receiver waits for all writers and decodes only the surviving encoding.
- [x] EA left in a Note operand fails at consumption, while EA in a Number operand succeeds. Cover invalid pending encodings without creating an intermediate Source parse or coercing typed nested results.
- [x] Include every Cell of the full writes, resulting Source and next-parse diagnostics for leftover characters. Whole-write rejection remains atomic under the admission contract from issue 04.
- [x] Assert these outcomes through production Source/Tick execution using actual producers and fixed destination configuration. Function replacement and its changed receiving signature are covered by issue 09.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

Ordering and Cell-wise admission are established by issue 04; this ticket owns deferred literal decoding.

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
