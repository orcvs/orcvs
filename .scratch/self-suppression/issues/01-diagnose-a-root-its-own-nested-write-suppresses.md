# 01 — Diagnose a dynamic write that lands on its own Expression

Status: resolved
Blocked by: None (can start immediately)

**What to build:**

A dynamic writer can write over the Expression it belongs to. In `.<.|00.v~?000000=$35BA@<.x64FBE506`, placed in the middle row of a 34-column Grid, the root is a Minimum `.<` at column 0. The nested `@<` (Write West) answers 229 and writes `E5` over Cells 0–1, which are the root's own spelling. Its destination is known only at its Turn, so no Dependency orders anything against it. When the root's Turn arrives, ADR 0067 suppresses it and every unattempted Function beneath it. The Tick Plan holds the `E5` write and no diagnostic.

A static writer that does the same thing is already diagnosed. Its Reservation covers a Function whose subtree contains the writer, so `producer_edges` orders the writer after itself, and the Tick reports "same-Tick dependency cycle". The dynamic version differs only in when its destination becomes known. The owner decided (2026-10-12) that the two agree: a dynamic write that lands on a Function of its own Expression is diagnosed.

Rule: when a dynamic write's covered Cells contact a Function whose root is the writer's own root, and that Function's Turn is still to come, the write is applied and diagnosed at the writer. ADR 0067's feedback rule still decides what the write does, so the contacted Function and its unattempted subtree are suppressed as today; the diagnostic says why. The owner chose applying over refusing (2026-10-12), so the Source carries the write into the next Tick.

A dynamic write from any other Expression stays quiet feedback under ADR 0067, as `a_write_onto_a_function_still_to_come_suppresses_or_replaces_it` asserts.

The property `a_tick_over_every_nested_expression_the_parser_accepts_settles_its_root` (`orcvs/src/source/tick.rs`) already demands this: an intrinsically active root is interpreted or diagnosed. Today it fails on about 1 run in 30 at `PROPTEST_CASES=32`, and it removed PR #214 from the merge queue.

Defaults, for the implementer to keep unless the owner says otherwise:

- The diagnostic is reported at the writer's anchor, once per write, and says the dynamic output reached its own Expression.
- Only a Function in the writer's own Expression triggers the rule; owner equality decides that.
- ADR 0067's feedback rule gains the exception, through an amendment or a short new ADR.
- Start by giving Turn state an `activate` and a `suppress` verb. Execution writes the suppression loop twice today (`deliver` and `lock_portal`), and this rule belongs at the one place both would call. This is candidate 6 of the Tick architecture review.

## Acceptance criteria

- [x] A dynamic write landing on a Function of its own Expression whose Turn is still to come is applied, suppresses as ADR 0067 says, and produces one diagnostic at the writer.
- [x] The shrunk case above is pinned as a stated example beside the property. In that example the Tick Plan holds the `E5` write and the diagnostic, and the root is not interpreted.
- [x] A dynamic write from another Expression onto a Function still to come is still quiet feedback.
- [x] A static writer landing on its own Expression is still diagnosed as a same-Tick dependency cycle, unchanged.
- [x] The nested-expression property passes at the merge tier's case count.
- [x] The ADR text covers the exception.

## Comments

2026-10-12 — Re-scoped from "a root its own nested write suppresses" after tracing the shrunk case on `main` (d5d2ff4). The trace showed the writer is dynamic and lands on the root's own anchor, and that the static equivalent already diagnoses as a cycle. The owner confirmed the narrowed rule.

2026-10-12 — The owner chose "applied and diagnosed" over refusing the write.

2026-10-13 — Implemented. `deliver_output` diagnoses `dynamic output reached its own Expression` at a dynamic writer whose write suppresses a Function with the writer's own owner, after applying the write. Tests: `write::a_write_onto_its_own_expression_still_to_come_is_applied_and_diagnosed`, and the shrunk case pinned as `a_nested_write_over_its_own_root_leaves_the_root_diagnosed`; both fail without the check. ADR 0067 gains the paragraph. The `activate`/`suppress` verbs were not needed: the rule sits in the one suppression loop a write passes through, since a lock's suppression in `lock_portal` is not a write. The property passes eight runs at 256 cases, but also passed four runs without the fix, so the pinned example is the evidence. Other-Expression writes stay quiet (`a_write_onto_a_function_still_to_come_suppresses_or_replaces_it`), and static self-writes stay cycles (`live_cycles_stop_only_the_expressions_they_reach`).
