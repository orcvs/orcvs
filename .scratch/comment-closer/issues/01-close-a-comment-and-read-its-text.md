# 01: Close a Comment at the next `||` and let Reads select its text

**What to build:** The Parser ends a Comment at the first `||` after its
introducer, or at the row's end, and a Read, Track or Copy reads a pair of a
Comment's text as data. See `../spec.md` and ADR 0074.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] `||` closes a Comment; the Source after the closer parses.
- [x] `|||` is unclosed and `||||` is an empty closed Comment.
- [x] A Read, Track or Copy reads a pair inside a Comment's text, and refuses
      a pair touching either delimiter.
- [x] The Language Map partition property and its coverage guard handle
      several Comments per row, closed and unclosed.
- [x] The Function reference still parses without a diagnostic outside a
      result row.
- [x] ADR 0074 is written, ADR 0035's Status names it, and `CONTEXT.md`'s
      Comment and Expression entries match.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: Built on its own branch, split from the tracker examples PR, which
frames the Orca tracker's pattern rows in `|| … ||` once this lands.
