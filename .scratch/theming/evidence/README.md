# Orcvs Light captures, for `.scratch/theming/issues/04`

These are the visual captures the issue owes the user before the light Theme's
colours are accepted and switching is exposed. **Nothing in the shipped console
selects this Theme**: `style::install` still registers the one resolved
Okabe–Ito style in both egui appearance slots, so these captures were made by a
temporary, uncommitted local patch (below), not by a control a viewer can reach.

| File | Viewport, logical points | What it shows |
|---|---|---|
| `light-wide.png` | 1200 × 700 | The wide console: page, Source ground, Cell grid lines, 8 × 8 Sector Seams, every glyph role, a multi-Cell Region under the lasso |
| `light-tall.png` | 700 × 1044 | The same Source in the tall layout. It was requested at 700 × 1200 and came out 156 points short, so the bottom of the tall layout is out of shot |
| `light-chrome.png` | 1200 × 700 | The Settings menu open: `selection.background` behind the open menu title with `text.active` on it, `panel.border`, the menu panel fill, and the two drag values' widget styling |
| `light-wide-protanopia.png` | 1200 × 700 | `light-wide.png` post-processed through the same Viénot, Brettel & Mollon (1999) transform `contrast::simulate` applies |
| `light-wide-deuteranopia.png` | 1200 × 700 | The same, for deuteranopia |
| `light-wide-tritanopia.png` | 1200 × 700 | The same, for tritanopia — the one this console does not gate, and the one to look at hardest |

- Theme: `orcvs-light`, exactly as `console/src/theme.rs::orcvs_light` and
  `console/src/theme.md` record it, including the user's 2026-09-23 re-pick of
  every Source glyph hue from the Okabe–Ito palette.
- Branch: `theming-04-light-theme`; the captures are of this branch's Theme
  values, made before the commit that records them.
- Platform: macOS (Darwin 25.4.0), native `eframe`/`glow` build,
  `console --features inspection`.
- Date: 2026-09-23.
- Each PNG is captured at `pixels_per_point: 2` — the logical viewport in
  the table above, at twice the pixel density, so an 11.5-point
  glyph's colour survives the capture. The earlier 1× captures did not: at one
  pixel per point every glyph is antialiased edge, and sampling one for its
  recorded colour finds nothing.
- Grid zoom: three steps in (`Cmd`/`Ctrl` `=`, three times), so the glyphs are
  legible at these viewport sizes. Border and Seam widths are in display points
  and do not scale with Grid zoom (ADR 0053), so they are at their recorded
  widths here.

## The Source in shot

```text
||ORCVS LIGHT THEME

.+0102

:#C4D4

:<07

:=040506

**

.+0Z
**

.+0304
06

:-0104
.+0203
```

It is laid out so that every painted state the contrast floor comes closest to
is on screen, which is what the first set of captures was missing. The blank
rows are not decoration: every root Function reserves the row south of its
anchor as an Output Portal, so an expression written directly beneath another
is painted in the Output Portal's colours instead of its own, and only a blank
row between them shows a role's own paint.

| Line | What it is there for |
|---|---|
| `\|\|…` | Comment |
| `.+0102` | Function, and two Valid Number operands on the Number tint |
| `:#C4D4` | Two Valid Note operands on the Note tint |
| `:<07` | Reverse's one Sequence operand, written: **Sequence, Invalid** — Diagnostic on the Sequence tint, 9.55:1 |
| `:=040506` | Replace's three operands: a Valid Number, then **Atom, Invalid** and **Sequence, Invalid** — the two states `04`'s first capture had no way to reach, because no other Function declares an Atom operand |
| `**` | Bang, on the bare Source ground |
| `.+0Z` | An unbindable Number operand: Diagnostic on the Number tint, with the second slot Pending (tinted, no glyph) |
| `**` under `.+0Z` | Bang inside an Output Portal Reservation: the one fact that keeps its own glyph colour and takes the Portal's background |
| `06` under `.+0304` | An Unclaimed pair inside a scalar Function's Reservation: the Output Portal foreground on the Portal tint alone |
| `.+0203` under `:-0104` | A Sequence-capable root's fitted Portal highlight over **another expression's Number operands**: the **doubled Portal-over-role tint**, `#D5DED9` under the Output Portal foreground at 5.75:1 |

The Region spans four empty Cells on two rows, so the lasso and
`region.background` are visible against the bare Source ground. Bang's own
Region state — `#90426F` on `#D8DDDB`, 4.78:1 — is the lowest of all 84 states;
the Region in shot is over empty ground rather than over the Bang, because the
lasso is what the Region capture is for and the ratio is in `theme.md`.

`source.sequence` `#0072B2` appears only as the `:<07` and `:=…06` tints. No
reachable state paints a glyph in it — `Token::Sequence` never binds, so every
written Sequence operand is Invalid and draws Diagnostic — which
`contrast::Role::glyph_channel` records as the reason Sequence is not one of the
glyph channels the colour-vision gate compares.

## Capture procedure

The Theme swap and the seeded Source are deliberately not committed: a shipped
seam that only a capture uses is exactly what the repository contract forbids.
**Both patches below were reverted before this branch's commit**, and neither
`console/src/console.rs` nor `console/src/main.rs` carries a line that exists
for a capture. To reproduce, on this branch:

1. In `Console::new`, replace `let theme = okabe_ito();` with
   `let theme = crate::theme::orcvs_light();`, and replace
   `let orcvs = Orcvs::with_source(start.source)?;` with a block that writes the
   Source above into `start.source` through `Source::set` — one
   `grid.position(col, row)` per non-space character — and then passes it to
   `Orcvs::with_source`. `Source::set` is already public API; no new one is
   needed, and nothing test-only is added to the shipped path.
2. `cargo build --package console --features inspection --locked`, then run the
   binary with `EGUI_INSPECTION=127.0.0.1:5719` and `HOME`/`XDG_DATA_HOME`
   pointed at a scratch directory, as `mise run inspect` does. Delete that
   directory first: eframe restores the stored Source over the seeded one
   otherwise, and the capture then shows a mixture of the two.
3. Attach `egui-mcp`, `resize` to each viewport above, press `Cmd`-`=` three
   times, click an empty Cell and `Shift`-`Arrow` to raise the Region, then
   `screenshot` with `pixels_per_point: 2`.
4. For the three simulated copies, post-process `light-wide.png` — linearize,
   apply the same Viénot, Brettel & Mollon matrices `contrast::simulate` uses,
   re-encode. They are the capture put through the transform, not a second
   capture.
5. Discard both patches.

On macOS the window must be visible for a frame to be captured; adding
`.with_always_on_top()` to the viewport builder in `console/src/main.rs` for
the duration of the capture is what made that reliable here. Do not send
`Escape` to the window through `egui-mcp` while capturing: it closes it.

## What the user is being asked

Whether these colours are right. Two questions are already answered:

- **Contrast.** `console/src/theme.md` records the report. All 84 measured
  states clear the 4.5:1 floor, the lowest at 4.78:1, and there are no accepted
  exceptions.
- **Colour vision.** `contrast::distinguish` gates protanopia and deuteranopia
  at ΔE00 5.0 and both built-ins clear it — Orcvs Light's worst red–green pair
  measures 7.19, the dark built-in's 6.65. Tritanopia is measured and not
  gated, and the reason is on this page's companion in `theme.md`: neither
  built-in separates every pair under it, and the Okabe–Ito palette makes no
  tritan claim. `light-wide-tritanopia.png` is what that costs, in the one
  place it can be looked at rather than read.

What is open is taste — the page, the Source ground, the Region and selection
washes, and how the seven darkened Okabe–Ito hues read on a near-white page.
Accepting them is what unblocks `04`'s remaining work: `set_style_of` per
appearance, the View menu's mode, the two Theme pickers, and the switching
acceptance tests.
