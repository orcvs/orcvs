# 03 — Derive the console's chrome from the Theme

**What to build:** egui's `Visuals`, the Cell grid line, and the Sector Seams take their values from the resolved Theme `06` defines, not from `PALETTE`. Until `04` completes and accepts the light definition, retain `02`'s shared presentation: register the same resolved Okabe–Ito style for both egui appearance slots and paint Source from that same Theme. Console colours, opacity, borders and border widths come from the resolved Theme.

**Blocked by:** 02 — Keep the restored theme preference at startup; 06 — Paint the Source from a named Theme.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `ConsolePalette` and `PALETTE` are gone. `style(theme: &Theme) -> Style` builds every `Visuals` field the console sets today from the Theme's named keys: `panel.background`, `panel.border`, `text`, `text.muted`, `input.background`, `selection.background`, `selection.border`, `selection.border.rest`, `error`, `warning`.
- [ ] Audit inherited toolkit colours as well as explicitly assigned fields: weak text, hyperlinks, code backgrounds and the text-edit caret must resolve from the Theme. Preserve current built-in values explicitly where appropriate; do not leave a second fixed palette hidden in `Visuals` defaults. Use the exact mappings in `../schema.md`.
- [ ] IME underline strokes derive from `input.cursor` and `input.cursor.width`: active uses the caret stroke and inactive retains the existing half-linear colour attenuation. These toolkit paths must not retain independent hardcoded colours.
- [ ] `Visuals::dark_mode` comes from the Theme's declared appearance.
- [ ] Chrome borders and border widths come from the resolved Theme. Use finite widths from 0 to 2 display points inclusive, with zero hiding a stroke. Preserve existing 1-point visible chrome borders and absent borders; use the exact key spellings and defaults in `../schema.md`.
- [ ] Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.
- [ ] Resolve the opaque application backdrop from `window.background` and the separate Grid surface from `grid.background`; reject a nonopaque window backdrop. Panel and Grid transparency composites over console surfaces without enabling desktop/window transparency.
- [ ] Preserve existing typography, spacing, square corners and the absence of shadows and gradients. These controls and font choice are not planned; avoid blocking future extensions without implementing them now.
- [ ] Until `04`, installation retains `02`'s shared presentation through `set_style_of`: register the same resolved Okabe–Ito style for both `egui::Theme::Dark` and `egui::Theme::Light`, and keep Source on that same Theme. Preserve the stored mode and Theme references without activating distinct selections; nothing calls `set_theme`. Distinct light/dark registration and live switching acceptance belong to `04`, not this issue.
- [ ] Cell grid lines and Sector Seams each take independently configurable colour, opacity and bounded width from the resolved Theme once per Render Frame. Grid line and border widths are measured in display points and retain the same visible thickness as Grid zoom changes; they do not scale with Cell size. Transparent colours can hide lines; existing square Cells, zoom and spacing remain unchanged.
- [ ] With the Okabe–Ito Theme, every chrome value is unchanged, and the existing style tests pass reading it from the Theme.
- [ ] Prepare dark and light Theme pickers, each listing only Themes of its appearance. Issue `04` exposes them together with the mode control after user review accepts the complete light Theme and Source and chrome both follow the selection.
- [ ] Interim tests show that restored settings and OS appearance changes retain the shared Source/chrome presentation and preserve stored preferences. Picker controls remain unexposed. `04` owns distinct-theme registration and switching tests, including startup restoration, mode changes and eventual loaded-Theme selection.
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

**2026-09-21 — Theme switching waits for coherent presentation.** The user confirmed that foundations may land separately, but switching is exposed only when both Source and chrome follow the selection. This issue therefore owns enabling the pickers. It does not resolve the separate question of which light Theme is available before `04`.

**2026-09-21 — visual restrictions become defaults.** The user confirmed custom font files alongside a Theme, referenced relative to it, and made corners, shadows and gradients Theme values. Today's built-in appearance remains the default. The former universal prohibition is superseded; animation remains separate from the Theme.

**2026-09-21 — font choice removed from planned scope.** The user clarified that font choice is a nice-to-have and not planned. The custom-font portion of the preceding comment is superseded. Keep the existing bundled fonts and avoid design choices that would prevent future font selection; do not implement speculative font support. Font sizes and the other confirmed appearance controls remain in scope.

**2026-09-21 — planned controls narrowed.** The user specified “colors, opacity, grid/cell colors, borders & widths” and “Need some control over the grid, but can be constrained.” This supersedes the broader appearance scope in earlier comments: font sizes, spacing, corners, shadows and gradients are not planned. Keep future extensions possible without building them now. The exact constrained Grid controls remain an open decision.

**2026-09-21 — constrained Grid controls confirmed.** Grid background and Cell fills expose colour and opacity. Cell grid lines and Sector Seams each expose independent colour, opacity and width. Cursor, Region and Diagnostic borders expose colour, opacity and width. Widths are bounded; transparent colours can hide lines. Existing square Cells, zoom and spacing remain unchanged. This settles the open scope question in the preceding comment; exact width units and bounds still need specification.

**2026-09-21 — light review precedes switching.** Prepare the picker implementation here, but expose switching under `04` only after the user reviews the complete light Theme, contrast results and visual captures. This refines the earlier picker ownership without making `03` depend on `04`.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.

**2026-09-22 — interim registration clarified after review.** This issue retains `02`'s one presentation in both egui appearance slots until `04` supplies and accepts the light definition. `04` owns distinct registration and switching acceptance; this avoids requiring its output as a prerequisite of `03`.
