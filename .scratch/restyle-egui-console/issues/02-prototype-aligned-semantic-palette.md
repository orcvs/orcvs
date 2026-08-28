# 02 — Prototype-aligned semantic console palette

**What to build:** The console uses one coherent prototype-inspired palette: a charcoal page, near-black Source, subtle one-pixel Cell grid lines, muted ordinary Glyphs, teal Functions, soft-red Bang and diagnostic states, restrained teal selection and Cursor treatment, and a calm Number colour distinct from Functions, Notes, and ordinary Characters. Cell backgrounds stay transparent or near-black except for meaningful states, with no gradients, rounded tiles, shadows, animation, or decorative chrome.

**Blocked by:** 01 — Square, centred Source Grid viewport.

**Status:** ready-for-agent

- [ ] Semantic colours are derived from one theme boundary rather than scattered rendering literals.
- [ ] Glyph classification remains responsible for semantic colour choice while Grid rendering owns geometry.
- [ ] Numbers use a calm non-yellow colour distinct from Functions, Notes, and ordinary Characters.
- [ ] Cursor and Glyph classifications retain their existing behaviour.
