# 03 — Show the Source View on the Overview

**What to build:** The Overview outlines the part of the Grid the Source View shows. The outline moves with every Pan — wheel, middle-drag, Alt-drag, and Cursor follow — shrinks as the performer Zooms in, is clipped to the Overview on an axis with nowhere to Pan, and is absent when neither axis can Pan. A new Theme key sets its colour and width. See `../spec.md` (Overlays, Theme).

**Blocked by:** 02 — Show the Overview.

**Status:** resolved

- [x] Built test-first at the Shape level: the outline is the Source View's visible region in Source points, offset by the margin and scaled to the Overview — not the visible Positions, which carry an extra Cell each side — at its predicted rect after a Pan and after a Zoom; clipped on an axis with nowhere to Pan; absent when neither axis can Pan.
- [x] The outline key is added to the Theme's style catalogue, the Theme schema and both built-in Themes within ADR 0053's chrome width bounds; a custom Theme that leaves it out inherits its parent's value, as any omitted key does, and one that sets its width out of bounds is refused.
- [x] Nothing on the Overview animates.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass; `mise run check_wasm` passes.

## Comments

Shipped in `console/src/console/overview.rs`. `Console::show_overview` now answers a `ShownOverview` — a painter clipped to the picture and the picture's rect — and `Console::ui` calls its `outline_source_view` after `show_source_panel`, so the outline is drawn from the Pan this Render Frame settled, as docked Diagnostics fills its reserved space after the Source. The outline is Diagnostics' `visible_source_region` (the console read back through the Source's origin, not the visible Positions), offset by the margin, clipped to the padded Source, and scaled from the padded Source at the snapped Cell to the 260-texel picture. It is not drawn when the console shows the whole padded Source on both axes, the same test the Source View's pointer uses for "nowhere to Pan". The stroke is `StrokeKind::Inside`, so a clipped edge stays on the picture.

The Theme key is the pair `overview.source_view` (colour) and `overview.source_view.width` (a chrome width, 0 to 2 points), matching every other stroke in the catalogue, which pairs a colour property with its `.width`. Okabe–Ito sets `#FFFFFF`, Orcvs Light `#000000`, both at width 1, so the outline is not the Cursor's ink. Both are in `style_catalogue!`, `schema.md`'s colour, width and chrome-mapping tables, `theme.md`'s built-in tables and width lists, and `examples/okabe-ito-copy.toml`. A custom Theme that leaves the key out inherits its parent's value, as every omitted `style` property does; a width over 2 is refused. The second criterion said a custom Theme missing the key "is refused as any missing key is"; no omitted `style` property is refused, since a custom document is sparse and inherits from its built-in, so the criterion now states that behaviour. The spec's user story 34 carries the same wording. The spec's "one new key" ships as the catalogue's colour property with its `.width` partner, the shape every stroke in the catalogue takes. The outline is painted after the Source, so ticket 05's Cursor and Region marks go in `show_overview`, under it, to keep the spec's overlay order.

Tests: four `console::tests` cases drive the shipped `Console`: the outline at the rect the Pan predicts at rest and after a wheel Pan, moved against the Pan; smaller after egui's zoom in, at its predicted rect; clipped to the picture across in a window wide enough for the padded Source; and absent in a window that holds all of it. The `theme_document` catalogue tests, the out-of-bound width cases, the inheritance test and both built-in pinning tests in `theme.rs` cover the key.
