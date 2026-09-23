# 11 — Validate chrome contrast through the painted composition

**What to build:** The contrast validator measures console chrome the way egui paints it, using the same composition the chrome's painting uses, and covers every chrome text-on-surface state a Theme can produce. `03` required that "painting and contrast validation must use the same composited backgrounds" and deferred it to `08`; `08` resolved having covered the Source only.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Chrome contrast states are composited by a function the chrome's painting also uses, not by the hand-written window → panel → input layering at `console/src/contrast.rs:539-548`. The comment there, which says chrome has no reused composition "yet" because `03` has not landed, goes.
- [x] The report covers text on `selection.background`, `text.active` on the hovered, active and open widget fills, and `error`, `warning` and `link` on the panel, in addition to the existing `text` and `text.muted` on panel and input. Any state left out is named in the report's scope, with its reason.
- [x] `shipped_theme_gate` holds Okabe–Ito and Orcvs Light to the same floor over the added states, or each failure is a pair-keyed accepted exception, recorded in `console/src/theme.md`.
- [x] Tests include a Theme built to fail on a chrome state the old report did not measure.
- [x] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** It owns `03`'s chrome half of the composited-contrast line. `07` is blocked by this issue: `07` shows the validator's report when a Theme loads, so the report should cover chrome first. `theming-04-switching` exposes Orcvs Light before this issue lands, so the light Theme's chrome states go unmeasured until it does.

**2026-09-23 — implemented on `theming-11-chrome-contrast`.** `contrast::chrome` replaces the
hand-written window → panel → input layering and its "no reused composition yet" comment. It
reads every colour out of `style::style` — the `Style` `style::install` registers for painting —
through the egui 0.36.2 accessors the widgets themselves paint from: `egui::Style::button_style`
(the function `Button::ui` calls for its fill and text colour), `Style::widget_style` (a
checkbox's label), `Visuals::text_color`/`weak_text_color`/`text_edit_bg_color`, and
`Visuals::selection` (what `paint_text_selection` recolours selected glyphs with). The open state
is measured the way `SubMenuButton::ui` paints it, `widgets.open` standing in for
`widgets.inactive`. Each fill is composited over its surface down to the opaque window backdrop
`Console::clear_color` clears to, with `Color32::blend`. No `style.rs` or `console.rs` change
was needed — no function had to be extracted, and no test-only seam was added. Egui's own
painter composites on the GPU; the shared *function* is `style` plus egui's accessors, and the
blend is the premultiplied source-over that painter applies.

The report now has 95 results: the 80 Source Grid states plus 15 chrome states — `text` on the
panel, the input, and the inactive and open widget fills; `text.muted` on panel and input;
`text.active` on the hovered and active widget fills and, frameless, on the panel; selected
text (`selection.border`, which egui paints selected glyphs in) on `selection.background` over
the panel and over the input; `error`, `warning` and `link` on the panel. One reading to note:
this line asks for `text.active` on the *open* fill, but egui paints an open widget in
`widgets.open.fg_stroke`, which the schema's Chrome mapping and `style` both set from `text`,
so the open state measures `text` — the pair actually painted. Left out, and named in
`ContrastReport::scope` (`validate_reports_the_floor_and_scope_in_the_returned_data` asserts
each): disabled widgets (egui fades them; WCAG 2.1 SC 1.4.3 exempts inactive components),
`code.background` (nothing paints a code span), a state's strong `bg_fill` (egui paints no text
on one), and a popup or window floating over another surface — the Source Grid or a panel —
(measured as one panel over the backdrop, identical while `panel.background` is opaque, as both
built-ins' are).

`shipped_theme_gate` holds both built-ins with no accepted exception. New figures — Okabe–Ito:
`text.active` on hovered/active fill 9.96:1, frameless 12.34:1; selected text 9.96:1 over panel
and input; `error`/`warning` 6.22:1; `link` 7.82:1; `text` on inactive/open fill 15.88:1. Orcvs
Light: `text.active` 5.81:1, frameless 6.64:1; selected text 5.81:1; `error`/`warning` 5.91:1;
`link` 5.47:1; `text` on inactive/open fill 9.94:1. Recorded in `console/src/theme.md`.

Tests built to fail on states the old report did not measure, each asserting every pre-`11`
state still passes: `text_active_matching_the_hovered_and_active_fill_fails_only_there`,
`selected_text_matching_selection_background_fails_only_there`,
`error_warning_and_link_matching_the_panel_each_fail_there`; plus
`the_open_widget_state_composites_its_fill_over_the_panel` for a translucent panel.
`cargo nextest run --package console --locked` (369 passed) and
`cargo nextest run --workspace --tests --no-default-features --locked` (1220 passed) pass.
