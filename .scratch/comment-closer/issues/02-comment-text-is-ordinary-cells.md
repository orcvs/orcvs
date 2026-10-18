# 02: A Comment's text is ordinary Cells to every Function

**What to build:** Every coverage question meets a Comment through its two
delimiters only, so its text Cells are ordinary Cells to Read, Track, Copy,
Halt, movers and Writes. The Comment's Span stays its whole claim for paint,
Tokens and scheduling. See `../spec.md` and ADR 0074.

**Blocked by:** 01

**Status:** resolved

- [x] `LanguageUnit::occupied_cells` names a Comment's introducer and closer,
      and every other unit's Span, and each site feeding `coverage` uses it.
- [x] The Comment-specific arm in `Portal::language_unit` is gone; a pair a
      delimiter covers is still refused, including an introducer that is the
      whole Comment.
- [x] A Halt aimed at Comment text, blank or holding glyphs, locks nothing and
      does not diagnose; one aimed at a delimiter still diagnoses.
- [x] A mover enters blank Comment text and is refused by glyphs there without
      a contact diagnostic.
- [x] A Write into Comment text writes the Cells.
- [x] ADR 0074's Consequences and `CONTEXT.md`'s Comment entry match.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: Toby decided that a Comment's text Cells are ordinary Cells to
every Function and only the delimiters are occupied. A mover entering a pair
across a delimiter and the text now meets part of the Comment and diagnoses
the contact, as a pair across any unit's edge does.
