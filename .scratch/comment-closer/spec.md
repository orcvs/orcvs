# Comment closer: a Comment ends at the next `||`, and its text is data

Status: resolved

## Problem Statement

A `||` Comment claims every remaining Cell of its row (ADR 0035), and a Read
refuses a pair under a Comment as part of another Language Unit. Orca's `#`
toggles, so one row holds several comments and Source after a closing `#`
runs, and `Q` reads glyphs inside a comment. The Learn Orca tracker keeps its
Notes framed in `#..#` for that reason, and an Orcvs port could frame nothing:
its Notes stayed ordinary Source, and the Language Map reported each one as an
unknown Function.

## Decision

Toby asked for this on 2026-10-18 ("We need to make comments terminate then.
Core changes."). ADR 0074 records it.

1. A Comment claims from its introducer through the first `||` after it, or
   to the row's end when none follows. `|||` is unclosed; `||||` is an empty
   closed Comment. The row walk resumes after the closer.
2. A Read, Track or Copy selecting a pair of a Comment's text, between its
   delimiters, reads it as it reads a pair no Language Unit covers. A pair
   touching either delimiter is refused.

## Consequences

- Any Source with two `||` on one row now reads differently. The Function
  reference's six row-0 headers are each written `|| … ||`.
- A Comment's text is ordinary Cells to every Function, and only its two
  delimiters are occupied (issue 02): a Halt aimed at the text locks nothing,
  a mover is refused there only by glyphs, and a Write writes the Cells.
