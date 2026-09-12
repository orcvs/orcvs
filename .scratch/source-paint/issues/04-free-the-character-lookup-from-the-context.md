# 04 — Free the character lookup from the egui Context

**What to build:** Resolving which character a Cell shows stops needing an egui `Context`.

`GlyphTable::lay_out` takes a `&egui::Context` because it rasterises a galley per printable ASCII character. But `GlyphTable::character(cell)` at `console.rs:522-525` needs none of that — it is `cell.content().unwrap_or_else(|| self.blanks[blank_glyph_index(cell.glyph())])`, and `blanks` is `BLANK_GLYPHS.map(blank_character)`, a pure mapping from `Glyph` to the character a blank Cell spells.

Ticket `05` needs this. `Paint::derive` must take a `&RenderFrame` and nothing else, and it has to answer each Cell's character. If it needs a `GlyphTable`, it needs a `Context`, and the twenty-two Context-building tests that accept a blank Source to dodge harness cost keep dodging.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Resolving a `RenderCell` to the character it shows is reachable without an egui `Context`.
- [x] `GlyphTable` keeps only the galleys — the part that genuinely needs a `Context`.
- [x] `show_source` still resolves characters the same way and paints the same glyphs.
- [x] A test proves the lookup runs with no `Context` constructed.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package console --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package console --locked`.
