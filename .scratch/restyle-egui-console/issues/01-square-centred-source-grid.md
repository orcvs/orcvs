# 01 — Square, centred Source Grid viewport

**What to build:** The console renders the largest square Source Grid viewport that fits the available console area. Cells remain square at every window size, and surplus rectangular space is centred as letterboxing without changing editing, Cursor, Playback, persistence, native, or WASM behaviour.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Cell width and height remain equal for wide, tall, and square available areas.
- [ ] The rendered Grid viewport remains square and centred in surplus space.
- [ ] Resizing cannot stretch one Cell axis independently of the other.
- [ ] Existing console interaction behaviour remains intact.
