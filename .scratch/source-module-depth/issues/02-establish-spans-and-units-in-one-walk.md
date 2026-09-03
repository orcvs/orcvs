# 02 — Establish Spans and Language Units in one row walk

**What to build:** One pass over a row answers everything the row decides: which Language Units it
holds, which Expression Spans those fall into, and which characters diagnose. A Comment or a space
ends a run in exactly one place in the code, so the day the Comment rule changes it changes once.

**Blocked by:** 01 — Delete the standalone-run recognizer.

**Status:** resolved

**Tags:** release/v1

- [x] Language Units, Expression Spans and lexical diagnostics come from a single walk of a row.
- [x] The `##` Comment rule and the space rule each appear once.
- [x] A Span still covers Cells that produce no Language Unit — a lone unrecognised character is
      its own Span, as it is today.
- [x] Prospective Span lookup for one edited Cell still reads only the edited row.
- [x] Every existing partition, Span and diagnostic test passes unchanged.

## Comments

Two independently written rules currently agree by accident rather than by construction. The
partition's Comment check reads the byte after the current Cell — which at a row's last column is
the first byte of the *next* row — and is saved only by a `grid.fits` conjunct beside it. The Span
walk slices a buffer that is exactly one row, so it cannot straddle at all. Same answer today, two
different reasons.

ADR 0018 already says Expression construction operates on the partition and never reinterprets
overlapping character pairs. This makes that true.

Note the shape that does *not* work: deriving Spans by walking the units. A Span can cover Cells
that produce no unit, so the units alone cannot reconstruct the Spans. The walk has to emit both.
