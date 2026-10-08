# 02 — Resolve an Input Portal's position in lang

**What to build:** At its Turn, a Function's Input Portal resolves to its offset from the anchor through one operation that `lang` owns, given the operand values the Turn resolved and the columns its operands occupy. A static Input Portal resolves to the offset it declares; a dynamic one resolves to the pair its operands select, `index % count` pairs east of its last operand. Execution reads the resolved offset without knowing which Function declared it or how a selected pair becomes columns. A performer sees no change: Track reads, waits, relays and diagnoses exactly as before.

**Blocked by:** 01 — Name Input Portals by when their position is known.

**Status:** resolved

## Acceptance criteria

- [x] `lang` exposes one operation that resolves an Input Portal declaration to its offset from the Function's anchor, taking the operand values and the columns the operands occupy east of the anchor, nested operands included. The selection rule, the pair width and the conversion of a selected pair to columns live behind it.
- [x] Track's pair selection is no longer part of `lang`'s public interface. Execution calls no Track-specific function and names no Function when resolving an Input Portal.
- [x] The Turn's view of its Input Portal is a resolved offset or none; it no longer distinguishes static from dynamic after resolution, and the repeated branch on the declaration's kind within execution is gone. Scheduling still reads the declaration before the Tick to order a static Input Portal's writers.
- [x] A `count` of `00` still diagnoses at Track's Turn as a wrap by zero, and operands outside their domain diagnose as they would at evaluation.
- [x] An offset that cannot be represented is diagnosed rather than panicking on the Tick path. A pair past the row edge, or one straddling it, still diagnoses as a Jump's Input Portal outside the Grid does.
- [x] The columns the operands occupy are still supplied by `orcvs`, which owns how nested and suppressed operands lie in the Grid; a suppressed nested count still occupies the two Cells at its anchor, and a Track that retries after waiting resolves its Input Portal again from current state.
- [x] Existing Track Source tests — reading `D4` from `@t0103C4D4E4`, nested and suppressed operands, zero count, empty pairs, Bang relay, Function Replacement, row-edge cases, ordering against writers of the selected pair and not unselected ones, discovered cycles — pass unchanged. Add focused `lang` coverage for resolving each declaration kind, including the unrepresentable-offset diagnosis.
- [x] Review the complete diff, including the changed public `lang` interface for human API review, and run the repository's required verification for `lang` and its dependents, with the local proptest case count set to 32. Leave the broader checks designated for CI there.

## Scope and constraints

This moves where an existing rule lives; it does not change ADR 0067's semantics, ADR 0032's ordering or ADR 0065's cycle handling. This ticket ends at the resolved offset: reading the Cells at that offset from working Source, and classifying them as a Language Unit, belong to the working Source placement owner (`.scratch/working-source-placement/issues/01`), and the columns the operands occupy belong to live operand resolution. The Interpreter's own operand validation in Track's evaluation stays: the public Interpreter validates independently of the Turn. Eagerly scheduling literal Track selections remains outside scope; ADR 0067 requires benchmark evidence before that optimisation.

## Comments

**2026-10-08 — implemented in `59e71cbd`, after 01 and `working-source-placement/01`.** `InputPortal::resolve(operands, operand_columns)` answers the offset; `track_pair` is crate-private. The Turn holds `Option<PortalCoords>` and checks unresolved writers for any resolved Portal: the schedule orders a static Input Portal's writers and Function Replacement refuses a changed Input Portal, so only a dynamic one finds writers, as before. An offset past `i16` diagnoses as partial or invalid input. Human API review of the `lang` surface is still open on PR #205.
