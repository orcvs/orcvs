# 21 — Deepen Render Frame ownership

**What to build:** Introduce one deep Render Frame module that derives a complete, immutable, row-structured visual snapshot from one coherent Source revision, the Grid, the Cursor, and marker/highlight configuration. Console should draw that snapshot with egui without reproducing Grid traversal, Source lookup, background Glyph precedence, selection, or Cursor visibility rules.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] One Render Frame is derived from a single coherent Source revision; a concurrent Tick cannot mix revisions within the frame.
- [x] The Render Frame is complete, immutable, and explicitly row-structured. Each visual Cell carries its Grid-minted Position, semantic Glyph, selected state, and Cursor visibility.
- [x] Occupied Cells use the Source-derived Glyph. Empty Cells resolve to Marker, Highlight, or Space using the existing precedence and exclusive marker-block rules.
- [x] Render Frame derivation is read-only. Console advances Cursor blink timing once before requesting the frame; deriving or drawing Cells does not mutate Cursor state.
- [x] Console owns egui-only presentation decisions such as fonts, colors, strokes, and widgets. No egui type crosses the Render Frame interface.
- [x] Clicking a Cell returns the Position already carried by the Render Frame to `App::select`; Console does not reconstruct coordinates or traverse the Grid itself.
- [x] SourceCommander supplies one coherent read of Cell content and Source-derived Glyphs without exposing a lock guard, parallel arrays, or repeated per-Cell locking.
- [x] Console no longer reads `App.grid`, `App.cursor`, or `App.opts` directly. Marker/highlight configuration reaches Render Frame derivation; egui-only configuration belongs to Console.
- [x] No formal presentation adapter interface or Render Frame cache is introduced. The current single egui adapter does not justify a new seam, and every repaint derives a fresh frame.
- [x] Visual outcome tests move behind the Render Frame interface: row order, occupied/background Glyph precedence, Marker/Highlight/Space resolution, selection, Cursor visibility, coherent revision, and marker-block edges.
- [x] Grid ownership and movement tests remain at the Grid interface; Source interpretation and occupied-Cell Glyph tests remain at the Source interface.
- [x] Event handling, Playback observation, MIDI menus, and the ADR-0001 Source/Playback and output adapter seams remain unchanged.

## Answer

App now returns a complete immutable Render Frame whose explicit rows contain every Grid-minted Position with its content, semantic Glyph, selection, and Cursor visibility. One coherent SourceCommander read captures the owning Grid, Cells, and Source-derived Glyphs under a single read lock; the Render Frame implementation then supplies Marker, Highlight, and Space Glyphs for empty Cells.

Console advances Cursor blink time once per repaint, draws only the returned frame with egui-owned fonts and styles, and returns clicked frame Positions to App. App's Grid, Cursor, and options are private, the old per-Cell rendering path is gone, and no presentation trait or frame cache was introduced.

## Notes

The current render path spreads one Render Frame across `console/src/console.rs`, `console/src/app.rs`, `console/src/source/`, `console/src/grid.rs`, `console/src/cursor.rs`, and `console/src/glyph.rs`. Console traverses the Grid and mutates Cursor blink state while App translates each Position to an index, acquires Source state, and supplies background Glyph fallbacks. Deleting `App::get` would merely move that coordination into Console, so its current interface is shallow.

The confirmed design gives callers leverage through one Render Frame interface and restores locality for traversal, visual precedence, selection, and snapshot consistency. The egui drawing implementation stays outside the module, but a trait is deliberately omitted until two real presentation adapters justify that seam.

No `CONTEXT.md` change is required: Render Frame already names exactly one repaint, and this ticket does not change its domain meaning. No ADR is required because the decision is local, reversible, and preserves the existing Playback decisions.

## Comments

**2026-08-27 — filed after architecture grilling (agent)**

The user confirmed the complete decision tree: semantic rather than egui values, explicit rows, coherent Source reads, read-only derivation, Position-preserving clicks, fresh frames without caching, direct frame assertions in tests, and strict exclusion of Playback/MIDI/event-handling refactors.

**2026-08-27 — implemented with TDD (agent)**

Focused Render Frame and coherent-read tests pass. `cargo test --workspace`, `cargo check --workspace`, `cargo fmt --all -- --check`, and `git diff --check` pass. Strict clippy remains blocked by seven pre-existing warnings in the unchanged `lang` crate.

**2026-08-27 — two-axis review resolved (agent)**

Standards review found no documented violations and flagged zero-capable highlight spacing plus an understated coherent-read name. Highlight spacing is now a validated positive whole-Cell type, and the read is named `SourceRevisionCells`. Spec review found the concurrent revision guarantee lacked a regression test; a synchronized Tick/render test now proves every frame contains either complete revision and never a mixture. The final audit also moved all egui color and stroke decisions from Glyph into Console.
