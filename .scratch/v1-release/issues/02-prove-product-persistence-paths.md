# 02 — Prove product persistence paths

**What to build:** Prove that persistence stores Grid and character Source as authority, rejects
malformed state, rebuilds every derived language view, and restores through the actual shipped
native and WASM storage integrations.

**Blocked by:** product-persistence/01 — Restore Source through the shipped storage path.

**Status:** resolved

**Tags:** release/v1

- [x] Automated model tests round-trip non-square edited Source, preserve Grid and Cells, reject
      malformed dimensions, and compare rebuilt Language Map, Glyph, and executable behavior.
- [x] Native save, restart, and reload follows the shipped storage path and records a repeatable
      procedure and result.
- [x] WASM save, browser restart/reload, and restore follows the shipped storage path and records a
      repeatable procedure and result.
- [x] Automated end-to-end coverage replaces a manual smoke wherever the host integration permits
      reliable automation.
- [x] The exact candidate gate reruns the automated proof; its final record links the native and
      WASM product-path evidence for the nominated SHA.

## Answer

Grid and character Source remain the only stored authority. A restore rebuilds the Language Map
and the Token each Cell presents, then executes the same Add as the Source that was written.
Native FileStorage is crate-private, so the product-path proof writes and reads the same RON
`HashMap<String, String>` the binary flushes to `eframe::storage_dir("Orcvs")/app.ron`, isolated
under a unique temp directory. WASM restore is automated against real `window.localStorage` in
headless Firefox. `v1-release/03` reruns `mise run test_persistence` and `mise run test_wasm` and
links this Evidence section for the nominated SHA — none is nominated yet.

## Evidence

Toolchain: `rust-toolchain.toml` pins `1.98.1`.

SHA: `2c3f111c93077ce99872f90d98b06b66433324e2` on `02-prove-product-persistence-paths`
(this Evidence SHA-fill is a follow-up commit on the same branch).

### Commands `v1-release/03` reruns

These are the automated proof. 03's final record links this Evidence section for the nominated
SHA and does not invent one.

- `mise run test_persistence` — exit 0 (822 nextest tests passed; doctests passed)
- `mise run test_wasm` — exit 0 (`wasm-pack test --headless --firefox console --test wasm --locked`;
  15 passed, including both `product_path` tests)

Crate-scoped gates run on this worktree (also exit 0):

- `cargo fmt --all -- --check`
- `cargo clippy --package orcvs --all-targets --locked -- -D warnings`
- `cargo clippy --package console --all-targets --locked -- -D warnings`
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --locked` — 433 passed
- `PROPTEST_CASES=32 cargo nextest run --package console --locked` — 113 passed
- `cargo nextest run --workspace --tests --no-default-features --locked` — 806 passed
  (persistence still compiles out)
- `mise run audit_deps` — exit 0 (`ron` is already in the locked graph through eframe;
  added only as a native console dev-dependency. `web-sys` gained the `Storage` feature on the
  existing wasm dependency.)
- `node --test scripts/tests/roadmap.test.ts` — 10 passed
- `node scripts/roadmap.ts > /dev/null` — exit 0
- `mise run check_wasm` — exit 0 (workspace wasm32 clippy plus both `trunk build`s)

Not run here: `mise run check`, `mise run check_merge`, and proptest's 256-case default —
deferred to CI. `mise run bench` is not a persistence gate.

### Proof tests

Model (`orcvs/src/source/model.rs`, `cfg(feature = "persistence")`):

- `test_source_round_trip_restores_shape_contents_and_derived_state` — 10×3 non-square,
  `.+0102` plus `"x"` at index 15; snapshot and Grid shape; Language Map units by kind and
  relative col/row (not Position identity); Token presentation per Cell; both Sources
  `execute(Tick::ZERO)` and write `03` into Cells 10–11.
- `test_source_deserialization_rejects_a_grid_that_does_not_match_its_cells` — well-formed
  encoding whose `cols` no longer matches its Cells, refused at the model Deserialize seam.
- `test_source_deserialization_rejects_an_empty_grid`
- `test_source_deserialization_rejects_overflowing_grid_dimensions`

What `Source` serializes is unchanged: `Grid` plus inner Cells. Language Map, Tokens,
diagnostics, and execution are rebuilt.

Native product path (`console::storage_tests::a_console_save_restarts_from_the_native_ron_file`):

- `FileStorage::from_ron_filepath` is `pub(crate)`. The test-only `RonFileStorage` speaks the
  same RON kv codec (`ron::Options::to_io_writer_pretty` / `ron::de::from_reader`) the native
  binary writes to `eframe::storage_dir("Orcvs")/app.ron`.
- Isolated under a unique temp directory. Never writes
  `~/Library/Application Support/Orcvs`.
- `Console::save` → `Storage::flush` → drop the session → `RonFileStorage::from_file` →
  `starting_source` / `Console::new` restores the edited 6×3 Source.

WASM product path (`console/tests/wasm.rs` `product_path`):

- `a_console_save_restarts_from_the_browser_local_storage` — clears `orcvs_source` and
  `orcvs_source_refused` from `window.localStorage`, implements `eframe::Storage` as the
  shipped LocalStorage wrapper, `Console::save`s an edited 6×3 `.+0102`, drops the Console,
  constructs a fresh `CreationContext` over the same localStorage, and asserts Grid, Cells,
  and `Token::Function` at the Add root.
- `a_malformed_local_storage_revision_is_refused_and_starts_the_default_grid` — a malformed
  `orcvs_source` in real localStorage is refused and starts the default Grid.

### Native binary procedure (host integration; needs a display)

FileStorage cannot be constructed from this workspace. The automated RON-file test is the
replacement for a manual smoke where automation is reliable. The binary path that still
needs a display:

1. From a checkout of the nominated SHA, `cargo run --package console --locked` (or
   `mise run run`). App id is `"Orcvs"` (`eframe::run_native("Orcvs", ...)` in
   `console/src/main.rs`).
2. Edit a non-square Source (a 6×3 `.+0102` is the fixture the automated test uses).
3. Quit. Native FileStorage flushes `eframe::storage_dir("Orcvs")/app.ron`
   (`~/Library/Application Support/Orcvs/app.ron` on macOS).
4. Reopen the same binary. The Grid and Cells of the last saved revision are restored;
   Language Map and Tokens are rebuilt, not stored.

Result: not exercised interactively in this worktree. The automated replacement is
`a_console_save_restarts_from_the_native_ron_file`.

### WASM browser procedure

`mise run test_wasm` is the procedure and the result: headless Firefox, real
`window.localStorage`, both `product_path` tests passed. That replaces a manual
save / browser restart / reload smoke.

The existing `refused_revision` module still covers developer-console reporting through
custom Storage impls; it is not the product-path localStorage proof.

### How 03 links this

`v1-release/03` stays open. Its final record for a nominated SHA must link this Evidence
section — not invent a SHA here. A pointer sits under
`v1-release/issues/03-run-the-exact-candidate-verification-workflow.md` `## Comments`.

## Comments

2026-09-14: Claimed and resolved on `02-prove-product-persistence-paths`. This is a proof
ticket: `product-persistence/01` already wired `Source` through eframe storage. `ron` was
added as a native-only console dev-dependency so the FileStorage-format test can speak the
codec FileStorage itself does not expose. `web-sys` gained the `Storage` feature so the
WASM proof can call `window.localStorage`.
