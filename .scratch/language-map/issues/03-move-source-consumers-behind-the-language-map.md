# 03 — Move Source consumers behind the Language Map

**What to build:** Make the Language Map the single Source-derived interface used by parsing,
interpretation, Glyph classification, diagnostics, and later spatial Tick planning.

**Blocked by:** 02 — Derive Expressions, roots, and diagnostics.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Source no longer owns parallel Expression-map, parsed-Atom, Glyph, and diagnostic protocols.
- [ ] Callers query units, roots, Footprints, and diagnostics without reconstructing spans.
- [ ] Render Frame still receives coherent Source Cells and semantic Glyphs from one revision.
- [ ] Persistence stores only Grid and character Source and rebuilds derived state on ingress.
- [ ] Source and Playback Engine seam from ADRs 0001–0002 remains unchanged.
- [ ] Native, persistence, and WASM gates pass.

## Comments

This is the foundation dependency for Sequence Portals and ADR 0020's row-major Tick pass.
