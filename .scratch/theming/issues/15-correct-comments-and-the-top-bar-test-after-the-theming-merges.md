# 15 — Correct comments and the top-bar test after the theming merges

**What to build:** Fix the test that no longer opens the menus it claims to, and the comments and docs that describe behaviour `main` no longer has.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] *(Done on `theming-04-switching`, which replaces the test; drop this line once it merges.)* `no_picker_labels_appear_anywhere_in_the_top_bar` (`console/src/console/kittest_tests.rs:304`) opens `["File", "View", "Theme"]`. The Theme menu was removed by `09`, so it is silently skipped, and Settings is never opened. Open the menus that exist, and fail when a listed menu is missing, rather than skipping it.
- [ ] `benches/floors.toml:60-62`: 106 ns against a 70 ns baseline is about 151%, which trips the 150% alert. It passes only the 300% fail threshold. Correct the reasoning; the 100 ns floor is unchanged.
- [ ] `benches/floors.toml:70` and `.scratch/paint-cell-cost/spec.md` name `Paint::derive_with_colours`, which #120 renamed to `derive_with_theme`.
- [ ] `console/src/console.rs:1519-1521` says the Grid lines and seams take the zoom scale. They stay a fixed display-point width.
- [x] `console/src/contrast.rs:539-541` says chrome has no reused composition because `03` has not landed. Delete it here if `11` has not already replaced it. *(Removed by `11` on `theming-11-chrome-contrast`, which replaced the hand-written layering with `contrast::chrome`.)*
- [ ] The `shipped_theme_gate` rustdoc (around `contrast.rs:1865`) cites a "known weakness recorded in `08`'s comments" that does not exist. Record the weakness (the hand-kept `[okabe_ito(), orcvs_light()]` array) somewhere real, or remove the citation.
- [ ] `console/src/style.rs:2467` says the byte arrays were "copied verbatim from the `ba987f6` capture". The test's own doc (`:2436-2439`) says they were recaptured after the retune.
- [ ] The public rustdoc on `RenderCell::output_portal` (`orcvs/src/render_frame.rs:83-85`) states the "each following written Cell pair" rule that `syntax-highlighting/12`'s review rejected. The shipped rule is the run of written Cells, stopping at the first blank Cell and clipped to the Reservation.
- [ ] `cargo fmt --all -- --check`, clippy and nextest pass on `console` and `orcvs`. `cargo test --workspace --doc --locked` passes.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.**
