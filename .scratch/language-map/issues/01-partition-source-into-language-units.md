# 01 — Partition Source into Language Units

**What to build:** Derive the row-local, non-overlapping Language Unit partition defined by ADR
0018 from one immutable Source revision.

**Blocked by:** `.scratch/orcvs-language-migration/issues/01-move-arithmetic-onto-the-dot-family.md`,
`.scratch/orcvs-language-migration/issues/02-free-the-bang-and-activation-spellings.md`, and
`.scratch/orcvs-language-migration/issues/04-select-a-disjoint-note-encoding.md`.

**Status:** ready-for-agent

- [ ] Complete Numbers, Notes, Functions, Bangs, and Activation Characters are recognized.
- [ ] Each unit records one Grid-minted anchor Position and complete Footprint.
- [ ] Recognition is left-to-right and non-overlapping (`***`, `<<<`, and `^^^^` match ADR 0018).
- [ ] An invalid character diagnoses and scanning resumes at the following Cell.
- [ ] No unit or Footprint crosses a row edge.
- [ ] Comment text from `#` through row end is excluded from Language Units and Expressions.
- [ ] Tests exercise the Language Map interface with rectangular Grids and Live Edit fragments.

## Comments

This ticket establishes lexical type identity. It must not consult operand context to distinguish a
Number from a Note.
