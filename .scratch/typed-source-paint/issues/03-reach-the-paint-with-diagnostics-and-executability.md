# 03 — Reach the paint with diagnostics and executability

**What to build:** The two facts the typed structure already decides and the painter never sees — a
Cell covered by a parse diagnostic, and an Expression that will not execute — reach the Render
Frame, carried once per Expression rather than copied per Cell.

**Blocked by:** 01 — Carry the Token to the Render Frame and retire the Glyph classifier.

**Status:** ready-for-agent

- [ ] The Render Frame carries the Expression Spans it derived from, with each Span's `diagnostic`
      and `root`. `ExpressionEntry` already holds all three; this issue exposes them rather than
      recomputing anything.
- [ ] The facts are **not** copied onto every `RenderCell` the Span covers. A diagnostic covers a
      Span and an executability decision belongs to an Expression; per-Cell booleans would repeat
      at a finer granularity the duplication this effort removes. The console resolves a Cell to its
      Expression.
- [ ] Lexical diagnostics reach the same path. `LanguageMap` exposes them separately from
      Expression diagnostics and they also carry Spans, so a Cell can be inside one without being
      inside an Expression at all.
- [ ] A test asserts that a Source with a parse error paints differently from the same Source
      without one, and that an incomplete Function paints differently from a complete one. Both
      assertions fail against today's code, which is the point of the issue.
- [ ] No colour is chosen here. The issue hands `restyle-egui-console` a list of the
      classifications that now reach the paint and have no token: `Activation`, `Atom`, `Sequence`,
      diagnostic coverage, and inexecutability. That effort's `02` holds the decided record and this
      one does not edit it.
- [ ] Until those colours are decided, the new facts change no pixel. Shipping a provisional colour
      would put an undecided value into the record five issues settled.

## Comments

This is the half of the effort a user would actually notice. A console that cannot show you a parse
error or tell you which Expressions will run on the next Tick is withholding the two things the
typed structure knows and the Source does not say.

The seam question to settle first: `RenderFrame` is a value snapshot with no link back to the
`SourceRevision` it derived from, which is why `Glyph` was flattened onto each Cell in the first
place. Carrying Spans alongside the Cells keeps that property — the snapshot stays a value — without
reintroducing a per-Cell copy. Handing the console the `LanguageMap` instead would not; it would put
a live borrow across the console's whole paint, and the Render Frame exists to avoid exactly that.

`show_diagnostics` in `console.rs` is about Playback failures and is not this. Naming is going to
collide; pick the language-side name deliberately.
