# 20 — Make the Language Map's Sequence capability linear and declare the scalar width once

**What to build:** The Language Map's Sequence-capability derivation is linear in positioned entries and allocates nothing per Expression, and the scalar Output Portal width is declared once. Today `root_may_answer_a_sequence` collects a `Vec` per Expression, and `sequence_capable` scans every entry for children inside its loop, which is quadratic in the positioned entries of one Expression; a single nested Expression holds many entries. The width is declared three times: `SCALAR_WIDTH` (`source/tick.rs`), `OUTPUT_PORTAL_SCALAR_WIDTH` (`source/language_map.rs`) and `PAIR_WIDTH` (`source/portal.rs`). Sharing one derivation between the Language Map and Tick scheduling is syntax-highlighting/11; this ticket follows it and does the rest.

**Blocked by:** syntax-highlighting/11 — Share one Output Portal derivation with the scheduler.

**Status:** resolved

Related: 06 changes how rows are carried across revisions, but Sequence capability is computed on demand in `output_portal_reservations`, not stored in rows.

- [x] The derivation syntax-highlighting/11 leaves visits each positioned entry a bounded number of times, so its cost is linear in positioned entries, not in Expressions.
- [x] It performs no heap allocation per Expression or per entry; any scratch storage is allocated at most once per derivation call, or is inline, and the choice is recorded here.
- [x] Evidence: an allocation test pins the per-call allocation count, and a benchmark over a Source with deeply nested Expressions shows the derivation's cost growing linearly with entry count. The comparison runs in CI, not locally.
- [x] The scalar width is declared once.
- [x] Language Map and Tick tests, including property tests, pass unchanged. syntax-highlighting/11, on the same branch, removed its own agreement tests and their property test by its own criteria; no test this ticket touches changed.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The "one derivation" half duplicated syntax-highlighting/11 (ready-for-agent), which now owns it; the comment at `language_map.rs` on the width constants already points at it. Width declarations corrected from two to three. 06 weakened from a blocker to a relation.

**2026-09-25 — performance contract specified.** "Linear in the number of Expressions" measured the wrong quantity: the quadratic scan is over the entries inside one Expression. "Allocation-free" is now stated as a per-Expression and per-entry bound, and the ticket requires allocation and benchmark evidence.

**2026-09-25 — resolved (epic PR 7).** Delivered with syntax-highlighting/11 on one branch, as separate commits: 11 left one derivation, `SequenceCapability` in `orcvs/src/source/language_map.rs`, which both `LanguageMap::output_portal_reservations` and tick planning's `computations` read.

- **Linear.** `SequenceCapability::derive` is one reverse pass over the Expression's positioned entries, visiting each once. Preorder puts every operand after its owner, so a child that may answer a Sequence sets its parent's slot before the parent is reached; the slot holds "some operand may" until the parent's own answer overwrites it. `lang::Expression::positioned` now returns a `DoubleEndedIterator` (it already iterated a double-ended chain) so the pass walks the entries in reverse without collecting them.
- **Scratch storage.** One `Vec<bool>` owned by `SequenceCapability`, one flag per entry, created once per derivation call by `SequenceCapability::for_map`, which sizes it for the revision's widest Function candidate, so no Expression grows it. Chosen over inline storage because an Expression's entry count is bounded only loosely (missing operands at the row edge claim no Cells), and an inline-with-fallback container would add a dependency for no measured gain. `output_portal_reservations` also sizes its answer list to the candidate count before the walk.
- **Allocation evidence.** `output_portal_reservations_allocate_the_same_blocks_for_every_shape` (`language_map.rs`, `mod output_portal`) pins one call at the same block count, at most two (the answer list and the scratch buffer), across a shallow root, one 60-deep root, several mixed roots, and four 60-deep roots. Making `derive` allocate per Expression moves the count with the number of roots and fails it. The counter is `orcvs/src/lib.rs`'s `allocation` module: `tests/allocation.rs`'s counting allocator, installed in the unit-test binary because the derivation is crate-private and every public path to it (Render Frame, Tick) allocates per entry for other reasons.
- **Benchmark.** `source_render_frame_nested` (`orcvs/benches/source.rs`): 16 rows, each one Expression nested as deep as the row allows, at 32, 64, 128 and 256 columns. The derivation is reached through `Orcvs::render_frame`, the public path that runs it; the rest of a frame is linear in Cells, so the series grows linearly exactly when the derivation does. A local run only to check that the series discriminates (not a comparison; CI owns that): 10.9, 21.9, 41.6, 83.9 µs with this change; 12.1, 23.4, 47.7, 130.3 µs with the quadratic scan substituted back in.
- **Scalar width.** Declared once as `SCALAR_WIDTH` in `orcvs/src/source/portal.rs`; `tick.rs`'s `SCALAR_WIDTH`, `language_map.rs`'s `OUTPUT_PORTAL_SCALAR_WIDTH` and `portal.rs`'s `PAIR_WIDTH` are gone. The literal `2` for a Function's own two-Cell spelling in `Lookup::new` stays: it is the glyph width, a different fact.
- No Language Map or Tick test changed; `orcvs` 637 and `console` 494 tests pass with `PROPTEST_CASES=32`.
