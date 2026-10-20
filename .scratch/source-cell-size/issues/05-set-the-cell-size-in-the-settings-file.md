# 05: Set the Cell size in the settings file

**What to build:** A performer sets `width` and `height` under `[cell]` in `~/.orcvs/config.toml`, in points, and the console draws its Cells and Glyphs at that size. Each key is optional and bounded to 4 to 64 points. An out-of-range or mistyped value is reported in the usual settings notice and falls back to its own default, leaving the other key standing. A size that doesn't land on whole pixels snaps quietly. The web reads no file and uses the default. See `../spec.md`.

**Blocked by:** 04 — Draw 12 by 18 Cells with a matching Glyph.

**Status:** ready-for-agent

- [ ] The settings reader reads both keys, defaults each when it is absent, and falls back on its own for a bad value, as its existing tests do for other keys.
- [ ] A console started with a configured Cell presents its Grid, Pan bounds and Glyphs at that size.
- [ ] The settings file's documentation shows the `[cell]` section.
