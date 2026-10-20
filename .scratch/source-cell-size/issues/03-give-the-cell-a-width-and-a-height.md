# 03: Give the Cell a width and a height

**What to build:** A prefactor with no visible change. The console's Cell becomes a width and a height everywhere it is read: the Cell geometry, the Source View's Pan bounds, margin, Cursor follow and edge scroll, the shapes step, the Cursor Effect, the Diagnostics readout and source reserve, and the default window. The Cell geometry is given the Cell size by its caller rather than reading a constant. The default stays 16 by 16 and the Glyph stays 11.5 points, so every existing behaviour test passes unchanged in what it asserts. See `../spec.md`.

**Blocked by:** 02 — Turn the Cursor Effect off by default.

**Status:** ready-for-agent

- [ ] Built test-first at the Cell geometry. A point still resolves to the Cell drawn under it across a sweep where width and height step independently, so reading one axis with the other's side fails. Each axis snaps to a whole physical pixel on its own at every Zoom step. A Cell whose width or height is zero or not finite answers no click.
- [ ] Pan margin, Cursor follow and edge scroll use the width across and the height down, which a console test with an unequal Cell proves.
- [ ] The Cursor Effect uses the width along horizontal edges and the height along vertical ones.
- [ ] No pixel of a 16 by 16 console changes. The existing console and kittest suites pass.
