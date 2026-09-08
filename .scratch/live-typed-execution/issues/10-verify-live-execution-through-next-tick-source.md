# 10 — Verify live execution through next-Tick Source

Status: resolved

Blocked by: 03 — Preserve parsed claims through edits and output; 06 — Settle failed suppliers without erasing inputs; 09 — Replace Functions with retained inputs.

Tags: release/v1

## What to build

Close the complete bounded live-execution contract through the production Source/Tick seam. Demonstrate that the migrated Parser, rendering, scheduling and evaluation work together, remove unused compatibility paths, and record the release evidence without shipping the throwaway prototype.

## Completion

- [x] All 26 specification acceptance cases execute through production paths. Record a case-to-implementing-slice and regression mapping, including same-boundary 02 then 03 yielding 31, different-producer partial overlap yielding 21, and Terminal Portal misconfiguration. Retain the exact transferred claim/rendering cases and their diagnostic context.
- [x] Integrate failed suppliers (06) with dual delivery (07): a Divide parent fails after Multiply has returned and written 0C; that child write survives and its other consumer answers 0D. This joined case belongs here so neither independent implementation ticket waits on the other. Also verify failed producers preserve surviving inputs when structural writes are involved.
- [x] No second spelling interpreter, independent layout reconstruction or obsolete adjacency guard remains. Remove any compatibility adapter introduced during the prefactor only after its final caller has migrated.
- [x] Add or retain meaningful bounded properties for Cell-wise composition, complete-write rejection, deterministic fixed-configuration Ticks and incremental/full next-parse equivalence; the expected model does not use the scheduler under test.
- [x] No Source parsing occurs between Function executions. Typed intermediate state remains Tick-local, nested answers alone do not rewrite Source, and the next parse may expose leftover literals or new roots.
- [x] Preserve Bang production, activation and no replay, terminal effect ordering, ordinary Sequence evaluation, long-Expression ownership, native/WASM support and declared feature combinations.
- [x] The tracker test and roadmap generation gates pass with every open release-tagged slice inside the release dependency closure. Report remaining genuine blockers; do not close or modify the parent issue from this ticket.
- [x] Keep prototype assets on the throwaway branch. Record the implementation verdict and any remaining excluded structural cases so release assessment can distinguish delivered behavior from deferred language questions.
- [x] After the production acceptance cases and applicable verification pass, change ADR 0034 from proposed to accepted using the repository prose Status convention. Record the delivery evidence and resolve any obsolete provisional wording against the confirmed contract; do not mark acceptance merely because the prototype was approved.
- [x] Update ADR 0024 to identify the spelling-only Map separation revised by ADR 0034, and ADR 0031 to identify the working-character binding seam revised by ADR 0034. Update ADR 0032's status/supersession text and affected clauses to identify the partial-write, competing-writer and original-anchor literal-signature revisions. Link each targeted revision to ADR 0034; preserve unrelated decisions and existing supersession history, including ADR 0033's initial partition and ADR 0028's value-or-effect distinction.
- [x] Reconcile runtime documentation with those targeted ADR revisions and retain resolved delivery history. Make no performance claim without reproducible benchmark evidence.

## Verification

- [x] Run `cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `cargo nextest run --package <crate> --locked` for each affected crate and its dependants. Use `PROPTEST_CASES=32` locally.
- [x] Run only additional gates triggered by the changed inputs under the repository contract. Tracker edits require `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.
- [x] Record exact commands and results, remaining risks, and checks deferred to CI. Run the once-before-PR workspace and doctest checks when opening a PR; defer the whole merge/browser suite and benchmark comparison to CI.

## Context

This implements part of the [delivery issue](01-execute-against-live-typed-expressions.md) under the canonical [specification](../spec.md).

This joins the rendering, failed-supplier and replacement branches. Issue 09 includes dual delivery and pending decoding transitively. The delivery issue 01 remains blocked until this ticket resolves.


Acceptance ownership (issue 10 verifies the integrated set):

| Specification case | Implementing issue |
| --- | --- |
| Cross-boundary chain | [04](04-compose-and-order-spatial-writes.md) |
| Competing writers | [04](04-compose-and-order-spatial-writes.md) |
| Different producers partially overlap | [04](04-compose-and-order-spatial-writes.md) |
| Pending Note repaired | [05](05-decode-pending-spatial-inputs-at-consumption.md) |
| Pending Note invalid | [05](05-decode-pending-spatial-inputs-at-consumption.md) |
| EA as Number | [05](05-decode-pending-spatial-inputs-at-consumption.md) |
| Whole spatial Note | [05](05-decode-pending-spatial-inputs-at-consumption.md) |
| Typed nested Note | [05](05-decode-pending-spatial-inputs-at-consumption.md) |
| Dual delivery | [07](07-deliver-one-nested-result-through-both-paths.md) |
| Terminal Portal configuration | [07](07-deliver-one-nested-result-through-both-paths.md) |
| Rejected child write | [07](07-deliver-one-nested-result-through-both-paths.md) |
| Parent failure | [10](10-verify-live-execution-through-next-tick-source.md) |
| Failed spatial supplier | [06](06-settle-failed-suppliers-without-erasing-inputs.md) |
| Failed nested supplier | [06](06-settle-failed-suppliers-without-erasing-inputs.md) |
| Inactive parent | [07](07-deliver-one-nested-result-through-both-paths.md) |
| Deep nested value replacement | [08](08-replace-computations-with-spatial-values.md) |
| Exact nested replacement | [08](08-replace-computations-with-spatial-values.md) |
| Top-level value replacement | [08](08-replace-computations-with-spatial-values.md) |
| Function replacement | [09](09-replace-functions-with-retained-inputs.md) |
| Replacement keeps nesting | [09](09-replace-functions-with-retained-inputs.md) |
| Replacement signature | [09](09-replace-functions-with-retained-inputs.md) |
| Replacement arity | [09](09-replace-functions-with-retained-inputs.md) |
| Misaligned replacement | [09](09-replace-functions-with-retained-inputs.md) |
| Emission order | [04](04-compose-and-order-spatial-writes.md) |
| Cycle with independent work | [04](04-compose-and-order-spatial-writes.md) |
| Self-dependency | [04](04-compose-and-order-spatial-writes.md) |

## Comments

2026-09-08: Revised decomposition after review; split write composition from pending decoding and retain explicit integration ownership.

2026-09-08: Production slice completed and verified. See [delivery evidence](../evidence.md) for acceptance mapping, exact commands, migrated policy regressions and deferred CI checks.
