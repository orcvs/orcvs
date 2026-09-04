# 02 — Define Console in the glossary

**What to build:** Give `CONTEXT.md` an entry for the word four of its definitions already depend on,
and stop the `Snapshot` overload before it spreads.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `CONTEXT.md` gains a **Console** entry in the presentation group, alongside Cursor, Glyph,
      Marker and Render Frame.
- [ ] The entry says what a Console is in domain terms — the editing and presentation surface that
      draws one Orcvs's Source, owns the Cursor's blink, and offers its presentation options — and
      names no toolkit, module or type.
- [ ] Its `_Avoid_` line lists: shell, app, frontend, UI.
- [ ] The four definitions that already use the word are left unchanged in meaning.
- [ ] `restyle-egui-console/02`'s phrase "the snapshot test at `orcvs/src/render_frame.rs:250`" is
      corrected. That test asserts an empty Cell receives `Glyph::Space`; it is a unit test and not a
      snapshot of anything.
- [ ] `CONTEXT.md` gains no implementation detail: no crate name, no file path, no egui reference.

## Comments

**Reference Render** is deliberately not added to `CONTEXT.md`. It names a committed image captured
for comparison against a later capture, which is tooling rather than domain vocabulary, and
`docs/agents/domain.md` keeps the glossary free of implementation detail. It is recorded in this
effort's `spec.md` instead, and reserved now because `Snapshot` already means Source Snapshot in
`CONTEXT.md` and across a dozen ADRs.
