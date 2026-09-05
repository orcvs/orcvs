# 07 — Decide whether a Grid still answers what fits in a row

**What to build:** Delete `Grid::fits` and the tests that are its only callers. The decision is
made: a Grid stops answering what fits in a row, because production code asks `offset_in_row`
instead.

**Blocked by:** None.

**Status:** resolved

**Tags:** release/v1

- [x] `Grid::fits` is gone from `orcvs/src/grid.rs`, and `test_grid_answers_whether_a_width_fits_in_the_row`
      goes with it.
- [x] No production call site changes, because there is none: `Portal::admit` and
      `LanguageMap::derive` already ask `offset_in_row`.
- [x] No behaviour changes; this is a question about what a Grid is asked, not about what it
      answers.

## Comments

Issue 02 replaced the partition's `grid.fits(anchor, 2)` guard with a row slice that simply has no
second byte to read at the row's edge — the row edge stopped needing a rule at all. That left
`fits` with its definition, its three tests in `orcvs/src/grid.rs`, and nothing calling it.

It is `pub`, so no lint fires and nothing forces the decision. That is exactly why it needs one
made deliberately: a Grid answering a question nobody asks is the kind of surface that quietly
grows a second, differently-shaped answer later. `spatial-tick-planning` adds producers that
write to Source and may well want to ask whether a result fits its row — `Portal::admit` currently
asks `offset_in_row` instead — so the honest options are a caller or a deletion, not a shrug.

### Decision: delete it

Taken during the `release/v1` issue alignment on 2026-09-04. `Grid::fits` has no production
caller. Its only callers are the six assertions inside its own test,
`test_grid_answers_whether_a_width_fits_in_the_row` at `orcvs/src/grid.rs:618-627`. The question
it answers is already asked elsewhere, in the form production code needs: `Portal::admit`
(`orcvs/src/source/portal.rs`) uses `offset_in_row` to reject a write that would cross the row
edge, and `LanguageMap::derive` (`orcvs/src/source/language_map.rs:380`) uses it to bound a Span.
`fits` is a second, differently-shaped answer to that one question, which is the surface this
effort exists to remove.

The two open options in the original statement are therefore settled as the deletion, and the
issue is `ready-for-agent` on that basis. The `release/v1` tag stays: `v1-release/03` names this
issue as a blocker, and `scripts/roadmap.ts` throws when a tagged open issue names a blocker that
is not itself open and tagged.

The paired change lives in `property-testing/02`: its sixth acceptance line named `fits(p, width)`
and now names `offset_in_row`. That ordering is encoded rather than described — `property-testing/02`
lists this issue as a blocker — so the deletion lands before the property suite starts and the two
issues cannot specify opposite things.

### Renamed by `sequence-values/04`, 2026-09-05

The caller this issue names is now `Portal::admit` in `orcvs/src/source/portal.rs`. It was
`SpanWrite::at` in `orcvs/src/source/tick.rs` when the decision above was taken; that constructor
was folded into the Portal and no longer exists under either name or location. The decision is
unaffected — the caller still asks `offset_in_row`, and `Grid::fits` still has none.

### Deleted, 2026-09-05

`Grid::fits` and its doc comment are gone from `orcvs/src/grid.rs`, and
`test_grid_answers_whether_a_width_fits_in_the_row` went with it. The `property` module's doc
comment names `offset_in_row` where it listed `fits` among the Grid suite the property-testing
effort will cover, so it agrees with `property-testing/02`'s sixth acceptance line rather than
pointing at a method that no longer exists. No production call site changed, because there was
none: `Portal::admit` and `LanguageMap::derive` still ask `offset_in_row`. The remaining
occurrences of the word are not references to the method: two expect-messages in `orcvs/`
(`source/portal.rs` and `source/tick.rs`), ordinary prose in `shell/` and `lang/`, and the
historical narrative in `.scratch/`. `docs/` has none.

Verification:

- `cargo fmt --all -- --check` — passed.
- `cargo check --package orcvs --all-targets --locked` — passed.
- `cargo clippy --package orcvs --all-targets --locked -- -D warnings` — passed.
- `cargo nextest run --package orcvs --locked` — passed, 218 tests.
- `cargo test --package orcvs --doc --locked` — passed, 8 doctests.
- `cargo check --workspace --all-targets --locked` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo nextest run --workspace --locked` — passed, 395 tests.
- `mise run check_wasm` — passed: WASM Clippy and both WASM application builds.
- `mise run check_pull_request` — passed.
- `mise run check_merge_native` — passed: `test_persistence`, `RUSTDOCFLAGS="-D warnings" cargo
  doc --workspace --no-deps`, and `cargo deny --locked check` (advisories, bans, licenses,
  sources all ok). This is the public-API risk gate CLAUDE.md names for removing a `pub fn`.
- `pnpm roadmap` — passed, reports `source-module-depth (8)` COMPLETE.

Not run: `mise run check` — it cannot pass on this repository state, and both blockers are
pre-existing on `main` rather than introduced here. `scripts/check-tooling-contract.sh`
requires `actions/checkout@<sha> # v4` while `.github/workflows/test.yml` pins `# v7.0.1`, and
`mise run test_wasm` fails to compile `shell/tests/wasm.rs:63`, which compares `OutputCommand`
with `PlayCommand`. Both were reproduced against `main` in a separate worktree; this branch
touches neither `.github/`, `scripts/`, nor `shell/`. Everything in `mise run check` other than
those two steps was run and passed, as listed above.

Run against `27adb0b`, the `main` this branch is rebased onto. The counts above are from a
private `CARGO_TARGET_DIR`: `~/.cargo/config.toml` points every worktree at one shared
`target-dir`, so a concurrent build in a sibling worktree can fail this one with types that do
not exist in this tree. Verification here is only trustworthy when the target directory is
isolated.

Risk: a `pub` method is removed from an internal, unpublished crate with no caller inside or
outside it; workspace compilation confirms that. No unsafe, dependency, feature, concurrency, or
performance change.
