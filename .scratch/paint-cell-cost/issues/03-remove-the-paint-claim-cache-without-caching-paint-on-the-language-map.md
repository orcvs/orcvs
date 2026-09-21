# 03 — Remove the Paint claim cache without caching Paint on the Language Map

**What to build:** Remove the pointer-keyed Paint claim cache and its per-Cell Span walk by
answering whether a slot is written once per claim on the Render Frame. Preserve the merged Output
Portal implementation and restore Source revision work to its pre-`02` cadence: the Language Map
does not retain whole-Grid Source Paint or Output Portal highlight views.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Every Render Frame Cell covered by one parser claim carries the same once-derived written
      answer as the other Cells in that claim; an entirely blank operand slot is Pending and a
      partly or wholly written unbound operand slot is Invalid.
- [x] Paint contains no `HashMap<*const Claim, bool>`, pointer-keyed lookup, or per-Cell walk over a
      claim's Cell range.
- [x] The Language Map retains no whole-Grid Source Paint or fitted Output Portal highlight cache,
      and rebuilding a Source revision does not derive either view.
- [x] The fitted Output Portal remains the implementation merged through `syntax-highlighting/12`,
      including its first-blank-Cell rule and one-Cell-gutter regressions.
- [x] Existing Source Paint answers remain identical for Functions, Pending, Valid and Invalid
      operands, Bangs, Comments, unclaimed Cells, Output Portal precedence, and Cursor precedence.
- [x] Focused tests cover a multi-Cell claim, an empty pending slot, a partly written invalid slot,
      overlapping Expression ownership, and the unchanged Output Portal gutter behavior.
- [x] The scoped `orcvs` and `console` formatting, lint, test, feature-off, and documentation gates
      pass. Performance acceptance belongs to `04` rather than to a local benchmark number.

## Answer

The Render Frame now carries the parser's shared claim and one Boolean written answer for every
Cell it covers. Its derivation visits each unique claim once, asks whether any Cell in that slot is
written, and copies the answer across the slot. `RenderCell::source_paint` combines that scalar
with the claim without allocating or walking the claim again, so Paint reads one finished value
per visible Cell.

The failed whole-Grid Language Map Source Paint and fitted Output Portal caches are gone. Output
Portal fitting is again owned by `SourceRevision`, exactly where `syntax-highlighting/12` merged
it, including the first-blank-Cell rule and one-Cell-gutter tests.

This resolves the implementation and correctness slice only. `04` owns the authoritative CI
comparison and remains open; no benchmark acceptance is claimed here.

## Comments

**2026-09-21 — the Answer above describes a rejected implementation.** `ba99abc` put the `written`
answer on the Render Frame. It used a grid-sized side vector, a second list of unique Claims, and a
redistribution pass per frame. `04` rejected it because `source_render_frame` regressed at every
size. The replacement keeps every criterion above and changes where the answer is made. Each `Claim`
now carries `written`, read from the revision's bytes as `LanguageMap::claims_by_cell` builds the
Claim (ADR 0052, proposed). `RenderCell` again carries only the shared Claim, and
`RenderCell::source_paint` reads `claim.written`. The criteria stay checked because the semantics
did not change. Whether the new seam is accepted is `04`'s decision.
