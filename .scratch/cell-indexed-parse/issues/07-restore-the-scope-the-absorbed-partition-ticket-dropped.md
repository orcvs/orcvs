# 07: Restore the scope the absorbed partition ticket dropped

**What to build:** The parts of `language-map/08` that partitioning by parse requires but that
`cell-indexed-parse/01` and `02` do not carry. `08` is `wontfix` because its central change is
ticket `01`; these are the rest of it, and without them the second lexer survives, the language
change goes unrecorded, and the vocabulary keeps describing the partition that was deleted.

**Delete the second lexer.** The Map classifies a two-Cell spelling by calling the same conversion
functions the Parser calls, over the same characters, producing a strictly weaker context-free
answer. Once unit kinds come from the parse that established the Expression, that classifier has no
caller.

**Record the language change.** A space inside an Expression's arity-determined extent becomes an
unfilled operand rather than a terminator, so `.+01 02` is one Expression whose second operand is
` 0` and fails to bind, plus a leftover `2`. Today it is two things. This is a deliberate change and
needs a test and an ADR clause, not a silent consequence.

**Settle what a Cell holding plausible data but belonging to no Expression is.** A bare `0102` is
already an invalid Expression today; the kind that names it an operand literal buys exactly one
thing, which is that the console declines to paint it with a wrong Glyph. Nothing about meaning
changes. Decide whether that presentation survives and say so, rather than losing it by omission.

**Blocked by:** 01

**Status:** wontfix

**Tags:** release/v1

## Pre-delivery audit at `593613c` — 2026-09-08

The implementation statements in this audit describe commit `593613c`, before
live-typed-execution delivery. For the completed transferred work and current
implementation, see [delivery evidence](../../live-typed-execution/evidence.md).

Partially delivered in `593613c`: ADR 0033 records both space rules, adjacency,
one-Cell recovery, and row/Comment confinement; CONTEXT's Expression entry uses the parse.
`a_space_inside_an_expressions_claim_is_an_operand_cell_and_not_a_boundary` covers the internal-space rule using `.+  .-  `;
`build_separates_the_expressions_in_one_row` covers separated Functions. The consumed-width
restoration is gone. This is not full delivery: `name_units` still calls `unit_kind`, which
reinterprets spellings using Function/Activation conversion and Number/Note decoding.
Removing that second interpretation and deriving rendering from the Parser's semantic
product transfer to [live-typed-execution](../../live-typed-execution/spec.md), as do the
remaining exact-source regressions (`.+01 02`, `.+0102.+0304`, `.+0102Z`, `***`,
separated control/terminal input),
rebuild behavior, and an explicit decision/test for plausible data outside an Expression.
`record_expression` still has a standalone-literal Glyph exception; the current renderer's
raw-character/invalid-cell behavior must be assessed, not assumed to implement the old
presentation promise. The spelling-only unit requirement and mandatory CellRole glossary
entry are superseded by ADR 0034. The old parser-capacity edit guard is deliberately obsolete:
ADR 0033 has no 32-record cap, and `edits_accept_long_expressions` covers the replacement.
Preserve row confinement and complete long-expression ownership instead of reinstating it.

The original checklist below is retained as historical scope; this audit records
its disposition at `593613c`. The linked delivery evidence records subsequent delivery.

- [ ] The Map's two-Cell spelling classifier no longer calls the Parser's conversion functions, and
      unit kinds come from the parse that established the Expression.
- [ ] A unit kind still records the spelling and nothing else; no typed Atom is written back into the
      Map's units.
- [ ] The claimed-extent rule holds: `.+01 02` is one Expression whose second operand is ` 0` and
      fails to bind, plus a leftover `2`, and a test states it.
- [ ] The scanning rule holds: between Expressions the Parser skips empty Cells to find the next
      anchor, and the space in `.=0101 !>007FC4` is not diagnosed.
- [ ] `.+0102.+0304` is two Expressions, each with its own anchor and its own result destination.
- [ ] `.+0102Z` is a valid Expression and one invalid character, not one invalid Expression.
- [ ] `***` is a Bang and one invalid character, and the Bang Expression executes.
- [ ] A new ADR supersedes the two-stage kind mechanism, records what replaces it, and states both
      space rules — the one the partition ADR left out and the one this changes.
- [ ] CONTEXT.md's **Expression** entry no longer defines an Expression as a whitespace-delimited
      run, and CONTEXT.md gains an entry for the Cell-indexed classification.
- [ ] Whether a Cell holding plausible data outside any Expression keeps its distinct Glyph is
      decided and recorded; it is a presentation question and not a language one.
- [ ] The edit guard that refuses an edit exceeding parser capacity still holds.
- [ ] The temporary trailing-content restoration left by `language-map/07` is removed.

## Comments

Split out after review found that absorbing the partition ticket into `cell-indexed-parse/01` silently dropped eight
of its checkboxes. The spec's own user story about deleting the second lexer had no ticket at all.
