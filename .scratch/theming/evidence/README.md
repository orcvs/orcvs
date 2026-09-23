# Orcvs Light captures, for `.scratch/theming/issues/04`

These are the visual captures the issue owes the user before the light Theme's
colours are accepted and switching is exposed. **Nothing in the shipped console
selects this Theme**: `style::install` still registers the one resolved
Okabe–Ito style in both egui appearance slots, so these captures were made by a
temporary, uncommitted local patch (below), not by a control a viewer can reach.

| File | Viewport, logical points | What it shows |
|---|---|---|
| `light-wide.png` | 1200 × 700 | The wide console: page, Source ground, Cell grid lines, 8 × 8 Sector Seams, every glyph role, a multi-Cell Region under the lasso |
| `light-tall.png` | 700 × 1200 | The same Source in the tall layout |
| `light-chrome.png` | 1200 × 700 | The Settings menu open: `selection.background` behind the open menu title with `text.active` on it, `panel.border`, the menu panel fill, and the two drag values' widget styling |

- Theme: `orcvs-light`, exactly as `console/src/theme.rs::orcvs_light` and
  `console/src/theme.md` record it.
- Branch: `theming-04-light-theme`; the captures are of this branch's Theme
  values, made before the commit that records them.
- Platform: macOS (Darwin 25.4.0), native `eframe`/`glow` build,
  `console --features inspection`.
- Date: 2026-09-23.
- Grid zoom: three steps in from the default, so the glyphs are legible at
  these viewport sizes. Border and Seam widths are in display points and do not
  scale with Grid zoom (ADR 0053), so they are at their recorded widths here.

The Source in shot is five Expressions chosen to reach every painted role:

```text
||ORCVS LIGHT THEME
.+0102
:#C4D4
..3F
.+0Z
*
```

`||…` is a Comment; `.+0102` binds two Number operands; `:#C4D4` binds Note
operands; `..3F` and `.+0Z` leave an operand unbound, so those Cells paint
Diagnostic over their declared role's tint; `*` is a Bang. The Region spans
four empty Cells on two rows, so the lasso and `region.background` are visible
against the bare Source ground.

## Capture procedure

The Theme swap and the seeded Source are deliberately not committed: a shipped
seam that only a capture uses is exactly what the repository contract forbids.
To reproduce, on this branch:

1. In `Console::new`, replace `let theme = okabe_ito();` with
   `let theme = crate::theme::orcvs_light();`, and, after
   `let orcvs = Orcvs::with_source(start.source)?;`, feed the Source above
   through `orcvs.event_handler` as `InputEvent::Text` and
   `InputEvent::KeyPressed(InputKey::Arrow…)` events.
2. `cargo build --package console --features inspection --locked`, then run the
   binary with `EGUI_INSPECTION=127.0.0.1:5719` and `HOME`/`XDG_DATA_HOME`
   pointed at a scratch directory, as `mise run inspect` does.
3. Attach `egui-mcp`, `resize` to each viewport above, and `screenshot`.
4. Discard the patch.

On macOS the window must be visible for a frame to be captured; adding
`.with_always_on_top()` to the viewport builder in `console/src/main.rs` for
the duration of the capture is what made that reliable here.

## What the user is being asked

Whether these colours are right. The contrast question is already answered:
`console/src/theme.md` records the report, every one of the 84 measured states
clears the 4.5:1 floor, and there are no accepted exceptions. What is open is
taste — the page, the Source ground, the Region and selection washes, and the
five glyph hues. Accepting them is what unblocks `04`'s remaining work:
`set_style_of` per appearance, the View menu's mode, the two Theme pickers, and
the switching acceptance tests.
