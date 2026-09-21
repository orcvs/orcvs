# 03 — Derive the console's chrome from the Theme

**What to build:** egui's `Visuals`, the Cell grid line, and the Sector Seams take their values from the resolved Theme `06` defines, not from `PALETTE`. Register a style for both of egui's themes, from the dark and light Themes the settings name. After this, nothing the console draws is a compiled constant.

**Blocked by:** 02 — Keep the restored theme preference at startup; 06 — Paint the Source from a scheme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `ConsolePalette` and `PALETTE` are gone. `style(theme: &Theme) -> Style` builds every `Visuals` field the console sets today from the Theme's named keys: `panel.background`, `panel.border`, `text`, `text.muted`, `input.background`, `selection.background`, `selection.border`, `selection.border.rest`, `error`, `warning`.
- [ ] `Visuals::dark_mode` comes from the Theme's declared appearance.
- [ ] The four prohibitions `restyle-egui-console/02` names hold for every Theme `style()` produces: no gradients, no rounded tiles, no shadows, no animation.
- [ ] `install(ctx, settings)` registers `style()` of the dark Theme for `egui::Theme::Dark` and of the light Theme for `egui::Theme::Light` through `set_style_of`, and egui's own resolution of the mode picks between them. Nothing calls `set_theme`.
- [ ] `grid.border` and `sector.seam` are read from the resolved Theme once per Render Frame, like every other key.
- [ ] With the Okabe–Ito Theme, every chrome value is unchanged, and the existing style tests pass reading it from the Theme.
- [ ] `docs/research/egui-theming.md` is recovered from `feat/egui-theming` into `docs/research/`, and `feat/egui-theming` is deleted or tagged as history, with the tag name recorded in this issue's comments.
- [ ] The word "semantic" is removed from `console/src/theme.md` and from the test name `semantic_glyph_colours_are_distinct_and_bang_is_soft_red`. `CONTEXT.md` lists "semantic Grid" and "semantic Source" under `_Avoid_`.
- [ ] `cargo nextest run --package console --locked` passes.

## Comments

**This is a port, not a merge, and the distinction is load-bearing.** Merging `feat/egui-theming`
would bring 107 files and +9888/-6173, of which 71 are new — including `console/src/app.rs`,
`grid.rs`, `playback.rs`, `render_frame.rs` and `source/`. That branch predates `crate-boundaries`,
so those files are the pre-split monolith and merging would reintroduce a second copy of the `orcvs`
crate. It would also drag in that branch's `rust-toolchain.toml`, `scripts/check-tooling-contract.sh`,
`scripts/roadmap.ts` and `package.json`. The merge base is 2024-11-12.

What is actually wanted is 241 added lines in one file. Take them by hand.

The branch also carries three tests worth taking: that each egui theme selects a matching palette and
style, that canvas colours change with the resolved theme, and that installing styles preserves the
existing theme preference.

Ordering behind `restyle-egui-console/02` is deliberate. Its acceptance lines cite `style.rs`
by line number, and this port moves every one of them.

**2026-09-21 — re-scoped by ADR 0053.**

ADR 0051, which this issue cited, conceded chrome to egui's `Visuals` as a second, fixed palette per theme. ADR 0053 corrects that: one Theme styles the whole console, and `Visuals` are built from it. So this issue no longer ports `feat/egui-theming`'s `DARK_PALETTE`/`LIGHT_PALETTE` mechanism. It derives chrome from the Theme `06` builds. The comments above about not merging that branch still hold. Of what the branch carries, only `docs/research/egui-theming.md` and the three tests' intent are worth taking. `restyle-egui-console/02` is no longer a blocker: its values survive as the Okabe–Ito Theme's, and `theme.md` records which key each one maps to.
