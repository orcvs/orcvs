# 04 — Compose and order spatial writes

Status: resolved

Blocked by: 02 — Carry Parser-owned positions through Source execution.

Tags: release/v1

## What to build

Make real producers compose whole and partial writes during the same Tick. Derive dependencies from fixed destinations, settle every potential writer before the receiver, and preserve deterministic Cell-wise publication. This slice establishes ordering and admission with Number receivers; pending literal decoding is delivered separately in issue 05.

## Completion

- [x] Use actual producer computations and derived dependencies. The lower-producer chain changes .+0101 to .+0021 and answers Number 21 during the same Tick, without intermediate Source parsing.
- [x] Same-boundary competing writers compute 02 and 03. Position orders them as 02 then 03 at the same destination; the receiver answers 31. Assert the surviving operand characters and resulting Source as well as the answer.
- [x] Different-producer partial overlap: 05 at column 2 then 02 at column 3 leaves .+0021 and answer 21. Preserve uncovered Cells from the first write; both potential writers settle before consumption.
- [x] One producer computes once and retains configured emission order: two 05 writes at columns 2 then 3 yield .+0051 and answer 51.
- [x] Validate each complete destination before applying any Cell. Rejected writes diagnose and apply no fragment; admitted writes compose Cell-wise with later coverage winning. Add a bounded independent Cell-overlay property beyond the fixtures.
- [x] Split old competing-writer/cycle expectations: competing writes succeed, while cycles and self-dependency prevent all Tick execution effects, including independent writes and Play Commands.
- [x] Assert Tick Plan, resulting Source and next parse at the existing boundary; do not supply the schedule or assert storage layout. Preserve atomic publication, activation and terminal command ordering.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
