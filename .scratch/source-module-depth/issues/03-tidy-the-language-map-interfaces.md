# 03 — Tidy the Language Map's interfaces

**What to build:** Three small corrections to types that describe more states than exist, or
recompute facts they already hold.

An Expression answers for its own Language Units without searching for them again. A Diagnostic's
anchor is a Position rather than an optional one, since every Diagnostic has an anchor. A planned
Cell write carries a typed Cell index from the moment it is planned to the moment it is applied.

**Blocked by:** None (can start immediately).

**Status:** resolved

**Tags:** release/v1

- [x] Asking an Expression for its Language Units does not repeat the search performed when the
      Expression was built.
- [x] A Diagnostic's anchor is not optional, and the relationship between an anchor and its Span is
      stated where the type is defined.
- [x] A planned write addresses its Cells with a typed index, so the unsafe mutation it reaches
      rests on the index's own guarantee rather than on a comment arguing the bound.
- [x] No public editing interface changes; that is issues 04 and 05.
- [x] The existing suite passes unchanged.

## Comments

Three findings merged because each is a few lines and none blocks the others; they are three
commits, not one.

The anchor correction is worth doing before `spatial-tick-planning` 03–05 land. Those add producers
whose anchor is demonstrably not their Span's first Cell — a Self-Banging Function writes elsewhere
than where it sits — and the two Diagnostic constructors currently disagree about whether the
anchor is derived from the Span or supplied independently. They coincide for every Source that
exists today, and nothing states it.
