# Live typed execution delivery evidence

The subsequent [indexed Tick lookup repair](performance.md) records the benchmark
regression, restored lookup design and independent prototype/issue audit.
The later [Parser allocation repair](parser-performance.md) addresses the
remaining failing `parse_source` benchmark.

The production Source/Tick path implements the bounded ADR 0034 contract. The Parser records
actual Source ranges and direct ownership, including truncated and invalid inputs. Rendering
and scheduling consume that product; `unit_kind`, independent layout reconstruction and
`bind_source` are retired. The ordinary arity-based partition and incremental row rebuild remain.

The scheduler derives fixed spatial, nested-result, activation and descendant-suppression
constraints before execution. Only potentially active owners contribute Portal edges. Actual
activation still requires a current-Tick Bang. Encodings remain pending in Tick-local Source
characters; nested results retain their types. No Parser call occurs between computations.
The existing Evaluator accepts already resolved typed operands and keeps its value-or-effect
answer, Numeric Conversion identity, Sequence and domain-checking behavior.

Every acceptance fixture below uses actual Source, production planning and Source commit through
the internal fixed-Portal test configuration. Supplied answers cover Function values for the
deferred Function-producing operation and, in the [review regressions](review-fixes.md), scalar
values whose spatial encoding is outside the scheduler's two-Cell contract. No fixture supplies
the schedule. No prototype Parser, Evaluator, HTML or tracing API is shipped.

## Acceptance mapping

Tests are in `orcvs/src/source/tick.rs` unless qualified otherwise. Some tests exercise multiple
cases; their assertions distinguish execution diagnostics, Source characters and next parsing.

| Specification case | Slice | Regression |
| --- | --- | --- |
| Cross-boundary chain | 04 | `live_cross_boundary_chain_uses_lower_producers` |
| Competing writers: 02 then 03, answer 31 | 04 | `live_competing_writers_follow_position_and_emissions_follow_configuration` |
| Different producers partially overlap, answer 21 | 04 | `partial_writers_settle_before_consumption` |
| Pending Note repaired | 05 | `live_pending_note_decodes_only_after_all_writers_settle` |
| Pending Note invalid | 05 | `live_pending_note_decodes_only_after_all_writers_settle` |
| EA as Number | 05 | `live_pending_note_decodes_only_after_all_writers_settle` |
| Whole spatial Note | 05 | `live_spatial_note_is_an_encoding_and_nested_note_stays_typed` |
| Typed nested Note | 05 | `live_spatial_note_is_an_encoding_and_nested_note_stays_typed` |
| Dual delivery | 07 | `nested_computation_returns_and_projects_once` |
| Terminal Portal configuration | 07 | `live_inactive_ownership_and_terminal_portal_configuration_are_independent` |
| Rejected child write | 07 | `live_child_write_survives_parent_failure_and_rejected_portal_keeps_typed_answer` |
| Parent failure | 10 | `live_child_write_survives_parent_failure_and_rejected_portal_keeps_typed_answer` |
| Failed spatial supplier | 06 | `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers`; Source model `a_failed_spatial_supplier_preserves_original_operand_cells` |
| Failed nested supplier | 06 | `live_failed_suppliers_preserve_spatial_data_but_not_nested_answers` |
| Inactive parent | 07 | `live_inactive_ownership_and_terminal_portal_configuration_are_independent`; `live_inactive_nested_portal_cannot_create_a_cycle` |
| Deep nested value replacement | 08 | `live_deep_and_top_level_replacement_suppress_descendant_portals` |
| Exact nested replacement | 08 | `a_value_replaces_nested_computation_and_preserves_next_tick_source` |
| Top-level value replacement | 08 | `live_deep_and_top_level_replacement_suppress_descendant_portals` |
| Function replacement | 09 | `original_anchor_function_replacement_retains_inputs` |
| Replacement keeps nesting | 09 | `live_function_replacement_keeps_nesting_and_reinterprets_only_literals` |
| Replacement signature | 09 | `live_function_replacement_keeps_nesting_and_reinterprets_only_literals` |
| Replacement arity | 09 | `live_replacement_checks_retained_arity_and_never_runs_new_anchors` |
| Misaligned replacement | 09 | `live_replacement_checks_retained_arity_and_never_runs_new_anchors` |
| Emission order | 04 | `live_competing_writers_follow_position_and_emissions_follow_configuration` |
| Cycle with independent work | 04 | `live_cycles_reject_independent_effects_and_self_dependency` |
| Self-dependency | 04 | `live_cycles_reject_independent_effects_and_self_dependency` |

## Transferred and preserved coverage

`live_claims_and_glyphs_survive_source_edits_and_publication` pins the exact Sources `.+01 02`,
`.+0102.+0304`, `.+0102Z`, `***` and `.=0101 !>007FC4`. It also pins the current Function Glyph
for plausible standalone data: those characters remain rejected Function candidates, with no
Language Units or execution. This explicitly records existing visible behavior rather than
relying on the removed standalone-literal exception. Invalid typed operands remain claimed and
retain their typed Glyph; a spelling-only Bang unit is no longer invented inside them.

`live_bang_in_half_typed_terminal_claim_diagnoses_without_activation` verifies the actual Source
claim, resulting `!>00**  `, Number Glyph, Source diagnostic and absence of terminal effects.
`outputs_beside_and_over_standalone_source_are_admitted`,
`overlapping_outputs_beside_standalone_source_both_contribute_cells`, and Source model
`a_result_beside_a_root_preserves_the_root_and_activates_it_each_tick` replace the join rejections
using their original Source geometries. Cleared-Bang rewriting, no replay and existing activation,
terminal ordering, long expressions, row and Comment confinement remain covered by the existing suite.

The former `a_failed_data_supplier_does_not_expose_stale_operand_cells`,
`a_suppressed_consumer_names_the_supplier_that_did_not_settle`, and truncated-supplier regression
now assert surviving characters and successful receivers. Competing writers are tested separately
from cycles. A nested terminal failure reports the invalid child and the parent's missing typed
answer, with no terminal output. Unchanged malformed inputs retain initial Source diagnostics;
spatial repairs receive a reserved turn, and changed invalid encodings diagnose at consumption.

`live_writes_agree_with_an_independent_cell_overlay` compares publication with a plain Cell-overlay
and hexadecimal-arithmetic oracle, repeats the same fixed configuration to check determinism,
and compares the incremental next parse with a full derivation. `live_rejected_complete_write_changes_no_cell`
checks complete-write rejection across the Number domain. Existing partition and incremental/full
rebuild properties remain in place. Local proptest uses 32 cases.

The new Evaluator entry point has a doctest showing typed Sequence subtraction with distinct
operands; the existing interpreter, Sequence, MIDI and playback regressions remain unchanged
except for explicitly migrated Source policy expectations.

## Verification

- `cargo fmt --all -- --check` — passed.
- `RUSTC_WRAPPER= cargo clippy --package lang --package orcvs --package shell --all-targets --locked -- -D warnings` — passed; covers each edited crate and its dependants, including examples.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package lang --locked` — passed, 186 tests.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --locked --status-level fail` — passed, 503 tests.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --tests --features persistence --locked --status-level fail` — passed, 507 tests.
- `RUSTC_WRAPPER= cargo test --workspace --doc --locked` — passed, 8 doctests before adding the new Evaluator example.
- Focused Source/Tick runs passed, including all 26 acceptance cases and the new properties. Initial failing tests demonstrated missing parser positions, partial-write refusal, failed-supplier suppression, missing nested Portal delivery, value/Function replacement refusal, the independent Bang classifier, and inactive-Portal cycles before their respective fixes.

The configured `sccache` failed with `Operation not permitted`, including an escalated retry.
Disabling `RUSTC_WRAPPER` allowed normal compilation. Builds use this worktree's `target/`.

Final checks:

- `RUSTC_WRAPPER= cargo test --package lang --doc --locked` — passed, including the new Sequence example.
- `RUSTC_WRAPPER= RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` — passed.
- `node --test scripts/tests/roadmap.test.ts` — passed, 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed.

The final clippy and formatting checks were repeated after test-helper and documentation edits.
The independent reviews compared baseline `593613c30b4d61154381e5cba89831ffb4482afc`
with immutable review snapshot `30e31cc25aad78d4c78ca9771cd0c6e2d2715e87`.

## Standards

No actionable code findings. The review checked parser ownership, safe Rust, fallible write
admission, typed nested delivery, deterministic scheduling and atomic publication.

Documentation finding D1 was confirmed and fixed: the cell-indexed-parse spec and seven issues
now label their implementation descriptions as a pre-delivery audit at `593613c` and link to
this delivery evidence. Historical statuses and checklists are preserved. This documentation-only
clarification requires no runtime regression test; diff checks and tracker checks validate it.

## Spec

No actionable findings. The review checked issues 01–10 and ADR 0034, including partial overlaps,
consumption-time decoding, typed nested results, inactive Portals, replacement and suppression
ordering, failure isolation and next-tick parsing. It found no intermediate parsing or unintended
production authoring API.

Reviewer verification: `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --locked -E 'test(source::tick::test::live_)'`
— passed, 17 tests.

Review totals: Standards — 0 actionable code findings, 1 documentation clarification fixed;
Spec — 0 findings. No findings remain open.

## Limits and risk

This is active pre-release language design with internal Parser/Evaluator API changes. No unsafe
scope, dependency, lockfile, feature declaration, platform branch or concurrency lifecycle changed.
Persistence was exercised because Source derivation is shared with restored Source. Native/WASM
code paths and feature declarations are retained; this is not a cross-platform validation claim.

Not run: `mise run check`, `mise run check_merge`, `mise run test_wasm`, `mise run bench`, the
256-case proptest default, and CI's other operating-system/WASM matrix — deferred to CI under
AGENTS.md. No platform code changed, so `mise run check_wasm` is not a triggered local gate.
No dependency or tooling implementation changed, so audit-deps, actionlint, zizmor and the tooling
contract script are not triggered. No performance improvement is claimed; benchmark comparisons
remain in CI. The public-interface review is part of the required Standards/Spec review.

Newly anchored code, changed destinations, variable-width spatial Sequence projection and
replacement changing activation/output behavior remain excluded. The implementation explicitly
rejects Function replacement that changes terminal status or Bang-output capability. No authored
Function-producing operation or arbitrary-Portal syntax was added. Prototype captures remain on
the throwaway branch; pre-existing untracked prototype/research/walkthrough files are not staged.
