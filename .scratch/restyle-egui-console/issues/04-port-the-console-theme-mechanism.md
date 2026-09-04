# 04 — Port the console theme mechanism

**What to build:** Recover the per-theme style registration written on `feat/egui-theming` as a small
port onto the current crate, so the console derives egui's `Visuals` from a palette selected by theme
rather than from one hardcoded const.

**Blocked by:** 02 — Prototype-aligned console palette; 06 — Keep the restored theme preference at startup.

**Status:** ready-for-agent

- [ ] `PALETTE` becomes `DARK_PALETTE`. Its twenty-two values are unchanged and remain the ones `02`
      decides.
- [ ] `palette(theme: Theme) -> &'static ConsolePalette` selects the palette for a theme.
- [ ] `style(theme: Theme) -> Style` derives from `theme.default_visuals()` and overrides from the
      selected palette. The four prohibitions `02` names — no gradients, no rounded tiles, no
      shadows, no animation — hold for every theme it produces.
- [ ] `install(ctx: &Context)` registers a style for both `Theme::Dark` and `Theme::Light` through
      `set_style_of`, replacing the `set_style_of(Theme::Dark, ...)` and `set_theme(Theme::Dark)`
      pair in `Console::new`. Registering both and letting egui's own `ThemePreference` resolve is
      what makes the system preference work.
- [ ] `cell_visuals` and `sector_line` take the resolved palette rather than reading a const, and the
      six existing style tests follow.
- [ ] The render path resolves `palette(ctx.theme())` once per Render Frame.
- [ ] No theme switch is exposed in the View menu by this issue. The switch ships with `05`, when a
      light palette has been decided.
- [ ] `docs/research/egui-theming.md` is recovered from `feat/egui-theming` into `docs/research/`.
      It records the reasoning for this design and two egui constraints that `05` depends on.
- [ ] ADR 0030 is written, recording one claim: Glyph colour is a language concept, so the toolkit's
      theme model cannot define it. The other fifteen palette tokens are ordinary console
      presentation and are conceded to `Visuals`.
- [ ] The word "semantic" is removed from `console/src/theme.md` and from the test name
      `semantic_glyph_colours_are_distinct_and_bang_is_soft_red`. The repository has no definition
      for it, and `CONTEXT.md` lists "semantic Grid" and "semantic Source" under `_Avoid_`.
- [ ] `feat/egui-theming` is deleted or tagged as history in the same change, and the tag name is
      recorded in this issue's comments.
- [ ] `mise run check` passes.

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

Ordering behind `02` is deliberate. `02`'s acceptance lines cite `style.rs` by line number, and this
port moves every one of them.
