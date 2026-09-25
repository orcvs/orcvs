# 06 — Reuse unchanged Language Map rows across revisions

**What to build:** A Source revision reuses clean row derivations, so it does not deep-clone their Expressions. Today `LanguageMap::rebuild` calls `DerivedRow::for_revision` for every clean row, which deep-clones the row and its Expressions only to restamp the map identity. The shipped Grid is fixed at 256 rows (ADR 0054), and `Source::commit_tick` rebuilds on every Tick, including one that writes nothing, so every rebuild visits all 256 rows, derives written rows and deep-clones the clean ones.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Unchanged rows are shared between consecutive revisions rather than cloned.
- [x] Map identity is still checked wherever a row is resolved against its Map.
- [x] Record the complexity and allocation target: clean Expression payloads are not copied, and any remaining O(rows) table traversal or shared-pointer copying is stated explicitly. Benchmarks through `test-grid-shapes` / `Grid::with_shape` measure scaling; `source_whole_grid/edit_rebuild_valid` measures the shipped-shape improvement. `source_file/read/256x256` is checked for regression.
- [x] The allocation tests in `orcvs/tests/allocation.rs` that record the deep-clone finding reflect the new shape.
- [x] Whether a commit with no writes mints a new Map revision is decided and recorded here. 19's cache key must survive no-write and identical-byte commits whichever way this is decided, so the decision does not supply that key.
- [x] Language Map property tests still pass.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid and more urgent: the fixed 256-row Grid makes every rebuild clone 256 rows. Benchmarks named; the no-write-commit decision added for 19.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Sharing row payloads does not by itself remove height-dependent table work. The target now distinguishes deep-clone removal from the remaining traversal cost.

**2026-09-25 — implementation (epic PR 6).** Implemented on branch `perf/share-language-map-rows`.

- *Sharing.* `LanguageMap` holds `Vec<Arc<DerivedRow>>`. `LanguageMap::rebuild` re-derives the dirty rows and `Arc::clone`s every other row from the previous revision; `DerivedRow::for_revision` and its deep clone are gone. Rows that derive nothing share one empty derivation per build or rebuild, so `LanguageMap::build` pays one allocation for all blank rows rather than one each.
- *Map identity.* A shared row cannot carry the identity of every revision that holds it, so the identity moved from the stored Expression to the view: `ExpressionEntry<'a>` is now a `Copy` handle of the Map's `LanguageMapId` and a reference to the private, shared `DerivedExpression`, minted by `LanguageMap::expressions`. `expression_units` still asserts the handle's identity against the Map's, so an Expression handed out by an earlier revision is refused even from a row the two revisions share (`an_edit_refuses_old_expression_entries_even_from_an_unchanged_row` passes unchanged). The accessors take `self` and return `'a` borrows; call sites are otherwise unchanged.
- *Complexity and allocation target.* A rebuild of `d` written rows over an `R`-row Grid parses `d` rows and builds one row table of `R` pointers: one allocation of `R × 8` bytes (2 KiB on the shipped 256-row Grid), `R − d` atomic reference-count increments, and on drop of the superseded Map `R` decrements. No carried Expression, Language Unit or Diagnostic payload is copied or allocated. That O(R) table traversal and pointer copying is the remaining height-dependent cost and is deliberate: a Map is a whole-Grid value. A commit that writes no row is O(1) and allocates nothing (see the decision below).
- *Allocation evidence* (`ORCVS_MEMORY_SERIES=1 cargo test -p orcvs --test allocation -- --nocapture`, deterministic counts, not timings). One valid Cell edit on the populated fixtures, before → after:

  ```
  16x16 (43 Expressions carried)   111 blocks,  46,003 bytes →  15 blocks,  4,994 bytes
  32x32 (160 Expressions carried)  326 blocks, 169,417 bytes →  29 blocks, 13,471 bytes
  64x64 (621 Expressions carried) 1110 blocks, 647,744 bytes →  53 blocks, 29,724 bytes
  empty Grid, any size              10 blocks                →  11 blocks (the re-derived row's `Arc`); bytes fall
  no-write commit, 32x32 populated 310 blocks, 161,309 bytes →   0 blocks, 0 bytes
  ```

  The populated series now grows with the edited row's width, not with the Expressions carried. New tests in `orcvs/tests/allocation.rs`: `a_rebuild_costs_the_same_however_many_expressions_the_rows_it_carries_hold` (same edit in a 32x32 Grid with 2 vs 32 written rows allocates identically) and `a_commit_that_writes_no_cell_allocates_nothing`. The deep-clone FINDING comments in that file were rewritten to the new shape; the published series names are unchanged.
- *Benchmarks.* `source_edit_rebuild_valid` / `source_edit_rebuild_invalid` (16–64 square through `Grid::with_shape`, `test-grid-shapes`) measure scaling; `source_whole_grid/edit_rebuild_valid` measures the shipped shape; `source_file/read/256x256` is the regression check for the extra per-non-empty-row `Arc` in `build`. Added `source_whole_grid/commit_no_write` for the no-write commit. The comparison is CI's bench workflow; no local run and no speedup is claimed here.
- *Decision — a no-write commit does not mint a new Map revision.* `Source::rebuild_rows` keeps the held `Arc<LanguageMap>`, identity included, when no row was written, so every Expression that Map handed out still resolves; the commit still mints a new `RevisionId`, keeping `every_write_and_only_a_write_mints_a_new_revision`'s contract that an empty write is a write. A commit that writes any row, including identical bytes, still re-derives those rows and mints a new Map identity (`rebuild_rows`' written-not-changed rule is unchanged). For 19: this makes the Map identity stable across no-write commits only; identical-byte commits still mint a new one, so 19 still needs its own content-derived key and this decision does not supply it. Covered by `a_commit_that_writes_no_cell_keeps_its_language_map` (model) and the allocation test above.
- *Properties.* `a_rebuilt_map_equals_the_map_a_full_build_would_have_made` and the Language Map property suites pass at `PROPTEST_CASES=32`; the full case count is the merge tier's. New unit tests pin row sharing (`a_rebuild_shares_every_row_it_does_not_rederive`, `rows_that_derive_nothing_share_one_derivation`).
- *Overlap.* This also delivers the sharing design `allocation-reduction/01` asks for; that ticket's own criteria (recorded counts above, margin-row comparison) are left for its owner to reconcile.
