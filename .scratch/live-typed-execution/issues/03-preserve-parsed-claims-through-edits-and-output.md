# 03 — Preserve parsed claims through edits and output

Status: resolved

Blocked by: 02 — Carry Parser-owned positions through Source execution.

Tags: release/v1

## What to build

Make the console, diagnostics and Tick planner agree about arity-based Source claims. Outputs beside an Expression remain safe without the obsolete join guard, while an output reaching an inactive half-typed root cannot silently become activation. Derive presentation from the Parser product rather than a second spelling interpreter.

## Completion

- [x] Remove the second semantic spelling classifier and derive rendering/claim lookup from the positioned Parser product. Pin and preserve the current visible Glyph behavior of plausible data outside an Expression; do not promote it into an executable operand or assume the old exception fulfills the rendering promise.
- [x] Remove the adjacency/join guard. Replace each affected rejection regression with the safe result for the same Source, including output beside standalone data and output beside a Function. Overlap behavior remains governed by the implemented execution scope, not adjacency.
- [x] Check the exact Sources .+01 02, .+0102.+0304, .+0102Z, ***, and .=0101 !>007FC4, distinguishing inside-claim spaces, between-Expression spaces, recovery and separate roots.
- [x] A half-typed !>00 claims its arity within the available row. A Bang reaching a claimed typed operand does not activate a root or disappear without a diagnostic because the terminal root is inactive.
- [x] Preserve row/Comment confinement, complete long-Expression ownership, row-local rebuild equivalence and successful rewriting of cleared Bang display.
- [x] Demonstrate the outcomes through Source edits, Tick results and rendered classification; initial syntax diagnostics remain distinct from evaluation diagnostics.
- [x] Reconcile the transferred claim/rendering obligations with the final behavior and run the applicable Source/rendering and affected-crate checks.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
