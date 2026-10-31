# The Minimap Pans over a cached picture

Status: accepted; its placement beside the Source View and its rejection of a floating overlay are amended by [ADR 0081](0081-the-minimap-floats-over-the-windows-corner.md). Amends [ADR 0045](0045-the-source-view-is-a-bounded-space.md)'s list of Pan gestures. Preserves [ADR 0040](0040-the-console-paints-from-a-value.md), [ADR 0052](0052-a-claim-answers-whether-its-slot-is-written.md) and [ADR 0053](0053-one-theme-styles-the-whole-console.md). [ADR 0084](0084-the-source-view-has-no-margin.md) removes the margin the picture below includes, so the picture is the Grid alone. Settled in `.scratch/minimap/`; the evidence is `docs/research/egui-minimap.md`.

The Source View shows a small part of a 256 by 256 Grid. The Minimap is a reduced picture of the whole Grid and its margin, one block of colour per Cell, docked beside the Source View with the Source View drawn on it as an outline.

## Decision

**The Minimap is a further Pan gesture.** A press on it centres the Source View there, and a drag carries the Source View with the pointer; a press inside the outline keeps its grab offset rather than jumping. It reaches the Source View through one pending-Pan request applied before the existing clamp, so it stops at the same margin as every other Pan. It never moves the Cursor or changes a Region, so the rule that a Pan alone does not chase the Cursor holds unchanged.

**The picture is cached in `console` across Render Frames, keyed by the Source's revision and the presented Theme and appearance.** It is a console-owned buffer of one colour per Cell, rebuilt only when that key changes and drawn as one nearest-filtered texture updated in place. The revision is read before the Render Frame, so a Tick committing between the two costs one extra rebuild and never a stale picture. The cache does not live on the Language Map or in `orcvs`: whole-Grid Paint work there runs on every revision rebuild, which is the cost ADR 0052 rejected ADR 0050 for. A console buffer rebuilt when the Minimap is drawn adds nothing to a revision.

**A retained texture is consistent with ADR 0040's one-frame `GlyphTable`.** The `GlyphTable` lives for one Render Frame because its galleys are relative to a font atlas egui may rebuild between passes. The Minimap's texture holds no galleys and draws no glyph: it is an owned image whose texels are the Theme's colours, so nothing it holds can go stale when the atlas does. Each Cell's colour is still decided as a value from a Render Frame and a Theme, with no `egui::Context`.

**One new Theme key styles the outline.** It carries a colour and a width, within ADR 0053's bounds for chrome widths, and is added to the style catalogue, the Theme schema and both built-ins. Cell colours, the Cursor and the Region reuse their existing keys. Borrowing the Cursor's or the Region's border would make one channel show two facts.

## Rejected alternatives

**Per-Cell Shapes or one Mesh.** egui keeps no tessellation across Render Frames, so 65,536 quads are rebuilt and uploaded every Render Frame, about 6.5 MiB, whether or not the Source changed. A texture uploads about 264 KiB, the Grid and its margin, only when it did.

**A paint callback.** It is backend-specific: glow and wgpu each need their own implementation, with `unsafe` GL in shipped code, to reinvent a texture upload.

**A floating overlay.** It covers Cells the performer is working on. A docked panel takes width instead.

**Caching on the Language Map.** See the second decision: it moves whole-Grid work into every revision rebuild.

## Consequences

- `CONTEXT.md` gains Minimap, and Pan names it as a way to Pan.
- During Playback the picture rebuilds once per Tick: one 65,536-Cell pass and a 264 KiB upload of the Grid and its margin. A console benchmark with a floor measures it; no cost is claimed before it lands.
- The Minimap carries no content and holds no Positions, so `orcvs` gains no API.
