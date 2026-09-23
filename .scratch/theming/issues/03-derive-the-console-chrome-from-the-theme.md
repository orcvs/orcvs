# 03 — Derive the console's chrome from the Theme

**What to build:** egui's `Visuals`, the Cell grid line, and the Sector Seams take their values from the resolved Theme `06` defines, not from `PALETTE`. Until `04` completes and accepts the light definition, retain `02`'s shared presentation: register the same resolved Okabe–Ito style for both egui appearance slots and paint Source from that same Theme. Console colours, opacity, borders and border widths come from the resolved Theme.

**Blocked by:** 02 — Keep the restored theme preference at startup; 06 — Paint the Source from a named Theme.

**Status:** ready-for-human — three lines remain, one needing a human with push access to `orcvs/orcvs` and two deferred to `04`/`08` (see the 2026-09-22 review comment)

**Tags:** release/v1

- [x] `ConsolePalette` and `PALETTE` are gone. `style(theme: &Theme) -> Style` builds every `Visuals` field the console sets today from the Theme's named keys: `panel.background`, `panel.border`, `text`, `text.muted`, `input.background`, `selection.background`, `selection.border`, `selection.border.rest`, `error`, `warning`.
- [x] Audit inherited toolkit colours as well as explicitly assigned fields: weak text, hyperlinks, code backgrounds and the text-edit caret must resolve from the Theme. Preserve current built-in values explicitly where appropriate; do not leave a second fixed palette hidden in `Visuals` defaults. Use the exact mappings in `../schema.md`.
- [x] IME underline strokes derive from `input.cursor` and `input.cursor.width`: active uses the caret stroke and inactive retains the existing half-linear colour attenuation. These toolkit paths must not retain independent hardcoded colours.
- [x] `Visuals::dark_mode` comes from the Theme's declared appearance.
- [x] Chrome borders and border widths come from the resolved Theme. Use finite widths from 0 to 2 display points inclusive, with zero hiding a stroke. Preserve existing 1-point visible chrome borders and absent borders; use the exact key spellings and defaults in `../schema.md`.
- [ ] Transparency reveals the underlying console surface; the application window remains opaque. Desktop/window transparency is outside this effort. Painting and contrast validation must use the same composited backgrounds.
- [x] Resolve the opaque application backdrop from `window.background` and the separate Grid surface from `grid.background`; reject a nonopaque window backdrop. Panel and Grid transparency composites over console surfaces without enabling desktop/window transparency.
- [x] Preserve existing typography, spacing, square corners and the absence of shadows and gradients. These controls and font choice are not planned; avoid blocking future extensions without implementing them now.
- [x] Until `04`, installation retains `02`'s shared presentation through `set_style_of`: register the same resolved Okabe–Ito style for both `egui::Theme::Dark` and `egui::Theme::Light`, and keep Source on that same Theme. Preserve the stored mode and Theme references without activating distinct selections; nothing calls `set_theme`. Distinct light/dark registration and live switching acceptance belong to `04`, not this issue.
- [x] Cell grid lines and Sector Seams each take independently configurable colour, opacity and bounded width from the resolved Theme once per Render Frame. Grid line and border widths are measured in display points and retain the same visible thickness as Grid zoom changes; they do not scale with Cell size. Transparent colours can hide lines; existing square Cells, zoom and spacing remain unchanged.
- [x] With the Okabe–Ito Theme, every chrome value is unchanged, and the existing style tests pass reading it from the Theme.
- [ ] Prepare dark and light Theme pickers, each listing only Themes of its appearance. Issue `04` exposes them together with the mode control after user review accepts the complete light Theme and Source and chrome both follow the selection.
- [x] Interim tests show that restored settings and OS appearance changes retain the shared Source/chrome presentation and preserve stored preferences. Picker controls remain unexposed. `04` owns distinct-theme registration and switching tests, including startup restoration, mode changes and eventual loaded-Theme selection.
- [ ] `docs/research/egui-theming.md` is recovered from `feat/egui-theming` into `docs/research/`, and `feat/egui-theming` is deleted or tagged as history, with the tag name recorded in this issue's comments.
- [x] The word "semantic" is removed from `console/src/theme.md` and from the test name `semantic_glyph_colours_are_distinct_and_bang_is_soft_red`. `CONTEXT.md` lists "semantic Grid" and "semantic Source" under `_Avoid_`.
- [x] `cargo nextest run --package console --locked` passes.

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

**2026-09-22 — implemented, then rebased onto the revised named-property Theme format and re-scoped
to the interim registration above.** An earlier pass on this branch built `style(theme: &Theme)` and
an `install(ctx, settings: ThemeSettings { dark, light })` that registered a *distinct* style per egui
appearance slot — a design ADR 0053's base16-slot framing at the time seemed to invite. Rebasing onto
`theme-paint-source` (which had since replaced base16 slots with the one named-property format this
issue's acceptance lines now describe, and added the interim-registration line above) reversed that:
`ThemeSettings` is deleted outright, and `install(ctx, theme: &Theme)` now takes one `Theme` and
registers `style(theme)` for both `egui::Theme::Dark` and `egui::Theme::Light` through `set_style_of`,
always sharing one `Arc<Style>` — there is no settings pair left to diverge. `Console::new` passes
`okabe_ito()` directly, never the `dark_theme`/`light_theme` identities `Persistence` restores.
`style::tests::install_gives_each_slot_its_own_themes_style` and
`install_preserves_a_restored_preference_with_distinct_dark_and_light_themes` (the two tests that
exercised distinct registration) are gone; `install_shares_one_style_between_both_theme_slots` and
the new `shared_presentation_holds_across_every_resolved_egui_theme` replace them, the latter proving
the shared `Arc` holds across `ThemePreference::System`/`Dark`/`Light` alike, which is the "OS
appearance changes retain the shared presentation" half of the interim-tests line. `light_variant()`
and the tests that call `style()` directly with it (`dark_mode_follows_the_themes_declared_appearance`,
`the_four_prohibitions_hold_for_every_theme`, `chrome_colours_change_with_the_resolved_theme`) stay:
they test `style`'s own appearance-handling, not `install`'s registration, and nothing in the
re-scope touches that.

The toolkit-colour audit turned up one real bug, not just missing wiring: `visuals.widgets.hovered`/
`.active.fg_stroke` read `theme.selection_border`, but `.scratch/theming/schema.md`'s Chrome mapping
table gives that role its own key, `text.active`. Okabe–Ito's `text_active` and `selection_border`
happen to share one value (`#65E6BE`), which is exactly why the wrong key still looked right —
`hovered_and_active_foreground_reads_text_active_not_selection_border` retunes them apart to prove
the fix. `visuals.hyperlink_color`, `code_bg_color`, `text_cursor.stroke` and `ime_composition`'s two
underline strokes were egui's own inherited defaults, unread by `style` at all: each now reads
`link`/`code.background`/`input.cursor`+`.width` explicitly (the inactive IME underline keeps egui's
existing `Color32::linear_multiply(0.5)` half-linear attenuation over that same colour, and
`legacy_visuals` is left untouched — it is a platform default, not a themed property). Every chrome
border now reads its own bounded `ChromeWidth` instead of a literal `1.0`: `panel.border.width` for
the panel/window/noninteractive border, `widget.border.width` for hovered/active/open, `widget.
inactive.border.width` for the idle widget (replacing the literal `Stroke::NONE` it used to be — the
same absence today, since that width defaults to 0, but now a Theme value rather than a hardcoded
one), `selection.border.width` for text selection, `input.cursor.width` for the caret. All of Okabe–
Ito's own defaults for these already equal `1.0`/`0`/`2.0`, which is why the chrome is pixel-identical
and `okabe_ito_chrome_matches_the_decided_record` (extended to cover every new field) still passes
unchanged — `each_named_chrome_key_changes_style_when_retuned` and
`chrome_border_widths_come_from_the_theme` are what prove live wiring rather than coincidence.
`ChromeWidth::points`'s `expect(dead_code)` is removed: it is genuinely called now.

The opaque-backdrop line needed one small addition `resolve` (`theme.rs`, `06`'s function, still
unconsumed until `07`'s loader exists) didn't have: a `ThemeError::NonOpaqueWindowBackground { alpha
}` check on the resolved `window.background`, run once after every override is applied rather than
only when the document explicitly touches that key, so an inherited value is covered too even though
it is already opaque by construction. `clear_color` already read `theme.window_background` before
this issue (pre-existing, from `06`), and `source_panel_frame` already painted the Grid surface from
`theme.grid_background` distinctly, transparency and all (also pre-existing) — both audited and left
alone rather than duplicated. Cell grid line/Sector Seam colour, opacity and width were likewise
already themed once-per-Render-Frame and zoom-independent (`f05ab55`/`520fdff`, `theming/06` slice C
and its review, both landed before this issue started); nothing here repeats that work.

"Prepare dark and light Theme pickers... unexposed" became `crate::theme::themes_of_appearance`, a
pure filter over a caller-supplied `built_ins` slice (the same "no owned registry" shape `resolve`
already takes `built_ins` in), `#[cfg_attr(not(test), expect(dead_code, ...))]` like every other
piece of this Theme model still waiting on its caller. Nothing in `console.rs` calls it — no menu
widget was added — and `console::kittest_tests::the_theme_menu_offers_no_dark_or_light_picker` pins
that negative the same way `the_theme_menu_offers_no_reset_button` already pins the settings-removal
one.

`console/src/theme.md` (substantially rewritten by the rebase's own `9dc02d3`/`aa2fc42` for the
named-property format) said in two places that this issue hadn't landed yet: its opening paragraph
still read "The rest of the chrome still paints these values from compiled constants until `03`
derives it," and its property table simply had no rows for `text.active`, `text.muted`, `link`,
`code.background`, `input.cursor`, `window.background` or `widget.inactive.border`, nor any of the
five chrome width defaults — because none of that existed when those commits were written. Updated to
match what `style()` actually reads now, adding the missing rows and widths and rewriting the two
paragraphs (weak text, error/warning) that made now-false claims about what the console currently
does. `theme.md`'s own "Shipped Themes" section already correctly anticipated this issue's own state
("`03` prepares the dark and light pickers... so until then the console always resolves `okabe-ito`")
and needed no change.

**`docs/research/egui-theming.md` recovery is untouched by the rebase** — still the verbatim
`git show feat/egui-theming:docs/research/egui-theming.md` copy from before, unaffected by the
rescope since it is a dated snapshot, not live design. **`feat/egui-theming` is still not deleted or
tagged.** That remains a human decision for the reason the first pass gave: it is outward and
irreversible in a way a local commit is not, and the issue names no tag to use. Status moves to
`ready-for-human` (`docs/agents/issue-tracker.md`'s canonical set) rather than `resolved`, naming
exactly that one line as what is left, so a human with push access to `orcvs/orcvs` can tag (e.g.
`archive/egui-theming`) and delete it, or decide to keep it reachable a while longer, then tick the
line and record the choice here.

`style.rs`'s two contrast-floor tests and their shared `contrast` helper are still untouched, for the
same `theming/08` reason as before.

**2026-09-22 — review fixes, rebased onto `theme-paint-source` at `911282c` (a doc-comment-only
move of `okabe_ito`'s own doc, no further conflicts of substance).**

**Chrome and Source now share one `Theme` value, not two independent `okabe_ito()` calls.**
`Console::new` previously called `install(&cc.egui_ctx, &okabe_ito())` and separately set
`theme: okabe_ito()` in its own struct literal — two calls that happened to agree rather than one
value both consumers read. `Console::new` now resolves `let theme = okabe_ito();` once, passes
`&theme` to `install`, and moves `theme` into `Self`.

**The interim tests were genuinely weak, as flagged, and are replaced.** `style::tests::
shared_presentation_holds_across_every_resolved_egui_theme` only called `install` once and checked
`Arc::ptr_eq` immediately after — true unconditionally by construction (`install` never branches on
its argument), so it could not fail; deleted. Two new `console::tests`/`console::storage_tests`
integration tests replace it, each confirmed red under a real, reverted sabotage before being left
green: `chrome_and_source_stay_shared_across_an_os_appearance_change` runs two real frames through
`Console::ui` with different `RawInput::system_theme` values under `ThemePreference::System` and
checks both the installed `Style` (one shared `Arc` throughout) and `Console`'s own `theme` field
stay Okabe–Ito's; `a_restored_non_default_theme_identity_still_presents_okabe_ito_and_saves_back_
unchanged` restores `dark_theme`/`light_theme` as `"my-dark"`/`"my-light"` from `InMemoryStorage`,
asserts the console still presents Okabe–Ito (chrome and Source alike), then saves and asserts both
identities round-tripped unchanged. Confirming red: for the first, temporarily gave `install` a
`Theme` with a different `panel_background` than `self.theme` — the chrome-vs-`okabe_ito()` assertion
failed with the sabotaged colour on the left; for the second, temporarily hardcoded `Persistence::
save` to write `OKABE_ITO_IDENTITY` instead of `self.dark_theme` — the round-trip assertion failed
with `"okabe-ito"` on the left, `"my-dark"` on the right. Both reverted before committing.

**"Every chrome value is unchanged" is now a field-by-field pin, not a partial one.**
`okabe_ito_chrome_matches_the_decided_record` was missing exactly what review named:
`weak_text_color` (now asserted `== text.gamma_multiply(0.6)`, the formula the deleted
cross-check test carried), the inactive IME underline (asserted against `input_cursor.linear_
multiply(0.5)`), and every `widgets.open`/`widgets.inactive` fill and stroke. Extended to assert
every field `style()` sets — the practical equivalent of a whole-`Visuals` comparison against the
pre-`03` baseline, without the fragility of a giant literal `Visuals` construction, since a
regression now names the one field that moved. The test's own doc records the one intentional
byte-level difference from that baseline: `widgets.inactive.bg_stroke` was the literal
`Stroke::NONE`, is now `Stroke::new(0.0, theme.widget_inactive_border)` — a real colour at zero
width, not `TRANSPARENT` — and both paint nothing, so the visible chrome is unchanged even though
the value is not byte-identical to the old constant.

**Unticked: "Painting and contrast validation must use the same composited backgrounds."** It was
ticked but should not have been — this issue's own painting composites correctly (`source_panel_
frame` already tested against exactly this), but there is no general contrast-validation machinery
yet to hold to that rule; `theming/08`'s contrast validator delivers it, reusing the painter's own
compositing rather than a second implementation of it.

**Deleted `crate::theme::themes_of_appearance` and its `dead_code` suppression.** CLAUDE.md: "Keep
lint suppressions narrow and explain why the lint is inapplicable" — the suppression's own reason
("prepared for theming/04's picker UI, not yet called from console.rs") was true, but that makes the
lint *applicable*, not inapplicable: the function really was unused, on purpose, which is what
`dead_code` exists to flag. The picker filter's caller is `04`'s job, so the filter itself lands
with that caller rather than sitting unconsumed here. Unticked the "prepare dark and light Theme
pickers" line to match — this issue no longer prepares anything for it beyond the negative proof
below. `console/src/theme.md`'s "Shipped Themes" section, which the previous comment said "needed
no change," in fact still said "`03` prepares the dark and light pickers"; corrected to "`04`
prepares and exposes the dark and light pickers together."

**The no-picker kittest no longer depends on a "Theme" menu existing.** Renamed to `no_picker_
labels_appear_anywhere_in_the_top_bar`: it checks the bar's resting state, then opens every
top-level menu that currently exists (`File`, `View`, `Theme`) — skipping one that is not found
rather than failing — and checks each for the absence of picker labels. `theming/09` removes the
`Theme` menu once its remaining Cursor-effects sliders move elsewhere; this test survives that.

**Comments now describe current behaviour rather than narrating history**, per review: the
`Console::new` comment above `install` is unchanged from the first pass (already brief, already
pointing at `install`'s own doc for "the full interim rationale," so nothing there needed trimming);
`style.rs`'s inline chrome comments were tightened to lead with what each field *is* rather than what
it replaced; the `chrome_colours_change_with_the_resolved_theme` test's doc no longer opens with
"Ported from `feat/egui-theming`'s..." — it now states what the test proves directly.

**Small:** `style()` gained a local `bounded_stroke(width, colour)` helper for the seven repeated
`Stroke::new(theme.x_width.points(), theme.x)` call sites. `theme::tests::resolve_rejects_a_
nonopaque_window_background` and `..._a_fully_transparent_window_background` merged into one test
over both alpha values; `resolve_accepts_an_opaque_window_background` stays separate as the
distinct positive case.
