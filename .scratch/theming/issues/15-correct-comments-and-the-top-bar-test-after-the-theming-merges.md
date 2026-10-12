# 15 — Correct comments and the top-bar test after the theming merges

**What to build:** Fix the test that no longer opens the menus it claims to, and the comments and docs that describe behaviour `main` no longer has.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] *(Done by #129, which replaced the test.)* `no_picker_labels_appear_anywhere_in_the_top_bar` (`console/src/console/kittest_tests.rs:287`, menu loop at `:304`) opens `["File", "View", "Theme"]`. The Theme menu was removed by `09`, so it is silently skipped, and Settings is never opened. Open the menus that exist, and fail when a listed menu is missing, rather than skipping it.
- [x] *(Obsolete: 26167b95 replaced the `execute` floor with `execute_function` at 60 ns and new reasoning, `benches/floors.toml:47-59`.)* `benches/floors.toml:60-62`: 106 ns against a 70 ns baseline is about 151%, which trips the 150% alert. It passes only the 300% fail threshold. Correct the reasoning; the 100 ns floor is unchanged.
- [x] `benches/floors.toml:63` and `.scratch/paint-cell-cost/spec.md:3,17` name `Paint::derive_with_colours`, which #120 renamed to `derive_with_theme`.
- [x] *(Done: the comment is gone from `console/src`; `console/src/console/shapes.rs:94` now states the fixed display-point width.)* `console/src/console.rs:1515-1517` says the Grid lines and seams take the zoom scale. They stay a fixed display-point width.
- [x] `console/src/contrast.rs:539-541` says chrome has no reused composition because `03` has not landed. Delete it here if `11` has not already replaced it. *(Removed by `11` on `theming-11-chrome-contrast`, which replaced the hand-written layering with `contrast::chrome`.)*
- [x] *(Done by 85ae5b28: the `08` citations are removed, and `contrast.rs:1527` states the hand-kept `shipped` array in place.)* The `shipped_theme_gate` rustdoc (`contrast.rs:2226-2230`), and the `ContrastResult::accepted` rustdoc (`contrast.rs:633`), which says acceptances are recorded in `08`'s comments although `08` records none and both accepted lists are empty, cites a "known weakness recorded in `08`'s comments" that does not exist. Record the weakness (the hand-kept `[okabe_ito(), orcvs_light()]` array) somewhere real, or remove the citation.
- [x] *(Done by 85ae5b28: the `ba987f6` wording is gone from `style.rs`.)* `console/src/style.rs:2439` says the byte arrays were "copied verbatim from the `ba987f6` capture". The test's own doc (`:2405-2411`) says they were recaptured after the retune.
- [x] *(Moved to `syntax-highlighting/14`, which fixes the defect it describes.)* The public rustdoc on `RenderCell::output_portal` (`orcvs/src/render_frame.rs:83-85`) states the "each following written Cell pair" rule that `syntax-highlighting/12`'s review rejected. The shipped rule is the run of written Cells, stopping at the first blank Cell and clipped to the Reservation.
- [x] `console/src/theme_registry.rs:19-20` says `SelectedThemes` lists its Themes in the View menu's pickers. Selection is by `config.toml` and the console offers no picker (`console/src/theme_selection.rs:11`). Correct the module doc.
- [x] `cargo fmt --all -- --check`, clippy and nextest pass on `console` and `orcvs`. `cargo test --workspace --doc --locked` passes.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** Each line is a test, comment or doc that #119–#126 left describing behaviour `main` no longer has. They are one-line corrections with no shared code, grouped so one pull request clears them.

**2026-09-24 — rechecked against `origin/main` after #128-#130 merged.** The kittest line was done by #129. Every other line is still present on `main`; line references were refreshed. `contrast.rs:633` joins the `08`-citation line: it is a second reference to `08` comments that do not exist.

### Audit at cad296df — 2026-09-29

One content line is left: rename `Paint::derive_with_colours` to `derive_with_theme` in
`benches/floors.toml:63` and `.scratch/paint-cell-cost/spec.md:3,17`. The code has only
`derive_with_theme`, and no `derive_with_colours` remains in any crate. The gates line applies to
that change.

The other lines were checked:

- The floors-106 line is obsolete. 26167b95 deleted the `execute` floor, and `execute_function` at
  `max_ns = 60` carries its own reasoning.
- The zoom-scale comment no longer appears in `console/src`. I did not pin down which commit
  removed it. `shapes.rs:94` now says the stroke keeps its thickness at every Grid zoom.
- 85ae5b28 removed every `.scratch/theming/issues/08` citation from `contrast.rs`, restated the
  hand-kept `shipped` array at `contrast.rs:1527`, and removed the `ba987f6` wording from
  `style.rs`.

### Issue audit against d3fd1b27 — 2026-10-01

Two content lines are now left. The `derive_with_colours` rename above still applies. The
`theme_registry.rs:19-20` line is new: its module doc says `SelectedThemes` lists its Themes in the
View menu's pickers, but selection is by `config.toml` and the console offers no picker
(`theme_selection.rs:11`). The gates line applies to both.

### Resolved — 2026-10-10

Both content lines are done. `benches/floors.toml:63` and `.scratch/paint-cell-cost/spec.md:3,17`
name `Paint::derive_with_theme`. The `theme_registry.rs` module doc now says `SelectedThemes`
resolves the dark and light selections and gathers the registry's notices with its own, that the
selections come from `~/.orcvs/config.toml` on native and are the defaults on the web, and that
the console offers no Theme picker. Historical
`paint-cell-cost` issues, ADRs and the audit file keep the old name because they describe past
states. `cargo fmt --all -- --check`, clippy and nextest on `console` and `orcvs`,
`cargo test --workspace --doc --locked`, and `RUSTDOCFLAGS="-D warnings" cargo doc --package
console --no-deps --locked` pass.
