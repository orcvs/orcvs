# 10 — Derive Output Portal destinations and prove agreement with the scheduler

**What to build:** Derive, from a Source revision before any Tick, the Cells each root's Output Portal highlight covers under the rule `05` records. Prove the independent derivation agrees with the Tick scheduler's own reservations.

**Blocked by:** 05 — Decide how a Result is known before a Tick.

**Status:** resolved

**Tags:** release/v1

- [x] Destinations are derived from the Language Map, reading `lang::Function`'s public facts and the Expression's own nesting. It must not reach into tick planning's `Computation` nodes. The inputs are named explicitly: `output_portal()`, `answers_sequence()`, `widens_over_a_sequence_operand()`, `source_effect()` (source-writing Functions, including the Advance anchor), the locking behaviour that leaves a Function with no Cell write site, and a Jump's declared displacement.
- [x] The derivation covers exactly the eligibility and coverage cases `05` records. That includes incomplete Function candidates, nested Functions, terminal output and locking Functions, Jumps, source-writing Functions (the Advance anchor), the row-edge scalar, and Sequence-capable rows.
- [x] An agreement test compares the derived ranges with the scheduler's reservations for the same revision, across the reference Source and generated Sources. Where `05` deliberately excludes Cells the scheduler reserves, the test names each exclusion.
- [x] The Render Frame answers the destination fact. Adding it as an input to the paint decision is `06`'s work.

## Answer

`LanguageMap::output_portal_cells(&self) -> Vec<bool>` (`orcvs/src/source/language_map.rs`) answers, in the Grid's row-major order, whether each Cell lies in a root Function's Output Portal Reservation. It walks `self.expressions()`, and for each `ExpressionEntry::function_candidate()` — matching the scheduler's own eligibility rather than `root()` — resolves the Reservation from `lang::Function`'s declared facts:

- `Function::locks_root()` (Halt) and a `None` from `Function::output_portal()` (a Terminal Output Function, whose `output_portal()` is `None` because `performs_terminal_output()` holds, and a Source-writing Function, `None` because `source_effect()` holds) cover nothing.
- Otherwise `Portal::named` (the same `orcvs::source::portal` resolution the scheduler itself calls) resolves the declared `PortalCoords` against the Grid, refusing off-Grid the same way for every direction — south past the last row, a Jump's own direction past an edge.
- Sequence-capability is read from a new private helper, `sequence_capable`, that mirrors `tick.rs`'s `derive_reservations`/`reserved_for` one reverse pass over the Expression's own `ExpressionEntry::positioned()` entries (using each entry's `parent` index), never over tick planning's `Computation` nodes: a Function entry may answer a Sequence when `answers_sequence()` says so outright, or when `widens_over_a_sequence_operand()` and a direct operand entry is itself a Function entry that may. A scalar reservation is the Cell pair from the Portal (refused where the row edge leaves no room); a Sequence-capable one runs to the end of that row.

`RenderCell::output_portal(&self) -> bool` (`orcvs/src/render_frame.rs`) carries the fact per Cell, populated in `RenderFrame::derive` alongside the existing claim derivation. (**Amended by `12`, 2026-09-20**: the Reservation is now derived once as `LanguageMap::output_portal_reservations`; `output_portal_cells` flattens it for the agreement test below and is `#[cfg(test)]`, and `RenderFrame::derive` reads `SourceRevision::output_portal_highlight`, which fits a Sequence-capable root's highlight to its answer inside the same Reservation. The Reservation, the scheduler and this agreement test are unchanged.) This is the smallest API `06` needs to add the fact to the paint decision.

An agreement test (`orcvs/src/source/tick.rs`, `mod output_portal_agreement`) compares `output_portal_cells` against an oracle read straight from the scheduler's own `Lookup` and `computations` (the one place that may reach them) for: a scalar root, an incomplete root, a nested Function, a Sequence-capable root, a root widened by a nested Sequence operand, a Terminal Output Function, Halt, every Jump direction, a Jump and an ordinary scalar off the Grid or at the row edge, the bottom row, and the checked-in Function reference Source (`console/assets/function_reference.orcvs`, embedded via `include_str!` since `orcvs` cannot depend on `console`) — plus a `proptest` property over generated Sources. The Self-Banging Advance and Directional Bang Emit exclusions are named in their own isolated tests rather than folded into the general comparison: `05`'s "Overlap" rule means a Source-writing Function's write site can coincide with an unrelated root's genuine Reservation (the generated-Source property test found exactly this case, `.|>>` on a 2-column Grid, where `.|`'s own scalar Reservation lands on `>>`'s anchor), so the exclusion is asserted only where the Source is built to rule that out.

Focused unit tests for every row of `05`'s coverage table, plus the row-edge and off-Grid geometry, live in `orcvs/src/source/language_map.rs`'s `mod output_portal`.

## Comments

`derive_reservations` (`orcvs/src/source/tick.rs:548`) works on `&mut [Computation]`, a tick-planning structure, so it cannot be moved over as-is. The scheduler picks Expressions by `function_candidate()` (`tick.rs:919`), not by `root()`. So `RenderExpression::root()` alone would leave out incomplete candidates that the scheduler still reserves for.

**2026-09-19.** `05` resolved. The derivation lives on the Language Map and the Render Frame carries it. Coverage, from `05`'s table: every root Function that answers a value (incomplete operands included, per `function_candidate()`) covers its Reservation from its Output Portal — the Cell pair for a scalar-only answer, to the end of the row for a Sequence-capable one; a Jump at its declared direction. Not covered: nested Functions, Terminal Output Functions, Halt, Source-writing Functions (including an Advance's cleared anchor — a named exclusion in the agreement test), and a scalar at the row edge where the pair cannot fit. No `lang` change: read what `lang::Function` already declares. Status moves to ready-for-agent.
