# 02 — Define Console in the glossary

**What to build:** Give `CONTEXT.md` an entry for the word its presentation definitions already depend on,
and stop the `Snapshot` overload before it spreads.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `CONTEXT.md` gains a **Console** entry in the presentation block that runs from **Cursor**
      to **Panel**, beside **Render Frame**.
- [ ] The entry says what a Console is in domain terms — the editing and presentation surface that
      draws one Orcvs's Source, owns the Cursor's blink, and offers its presentation options — and
      names no toolkit, module or type.
- [ ] Its `_Avoid_` line lists: shell, app, frontend, UI.
- [ ] The definitions that already use the word are left unchanged in meaning.
- [x] `restyle-egui-console/02`'s phrase "the snapshot test at `orcvs/src/render_frame.rs:250`" is
      corrected. That test asserts an empty Cell receives `Glyph::Space`; it is a unit test and not a
      snapshot of anything.
- [ ] `CONTEXT.md` gains no implementation detail: no crate name, no file path, no egui reference.

## Comments

**Reference Render** is deliberately not added to `CONTEXT.md`. It names a committed image captured
for comparison against a later capture, which is tooling rather than domain vocabulary, and
`docs/agents/domain.md` keeps the glossary free of implementation detail. It is recorded in this
effort's `spec.md` instead, and reserved now because `Snapshot` already means Source Snapshot in
`CONTEXT.md` and across a dozen ADRs.

### Audit at cad296df — 2026-09-29

- `CONTEXT.md` has no **Glyph** or **Marker** entry, and the `Glyph` enum is gone from the code.
  The presentation block is Cursor, Region, Cursor Effect, Sector, Sector Seam, Render Frame,
  Paint, Theme, Source View, Pan, Zoom and Panel (`CONTEXT.md:283-329`). The first criterion now
  anchors the entry there.
- The word is used in about ten entries, not four (`CONTEXT.md:6` and most entries from `:284` to
  `:328`), so "four definitions" is dropped.
- `restyle-egui-console/02` no longer says "snapshot test". 00c0d2f7 removed the phrase, so that
  line is ticked. That ticket's Comment still cites `orcvs/src/render_frame.rs:250` for a
  `Glyph::Space` assertion that no longer exists; that is history in another ticket and is not
  this ticket's to fix.
- The **Console** entry itself is still missing.
