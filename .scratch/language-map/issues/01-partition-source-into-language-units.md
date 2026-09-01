# 01 — Partition Source into Language Units

**What to build:** Derive the row-local, non-overlapping Language Unit partition defined by ADR
0018 from one immutable Source revision.

**Blocked by:** orcvs-language-migration/01, orcvs-language-migration/02, orcvs-language-migration/04; lang-foundations/02; property-testing/07.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Complete Operand Literals, Functions, Bangs, and Activation Characters are recognized.
- [ ] Each unit records one Grid-minted anchor Position and complete Footprint.
- [ ] Recognition is left-to-right and non-overlapping (`***`, `<<<`, and `^^^^` match ADR 0018).
- [ ] An invalid character diagnoses and scanning resumes at the following Cell.
- [ ] No unit or Footprint crosses a row edge.
- [ ] Comment text from `##` through row end is excluded from Language Units and Expressions; one
      `#` is incomplete or invalid Source.
- [ ] Tests exercise the Language Map interface with rectangular Grids and Live Edit fragments.

## Comments

Partitioning establishes complete two-Cell Operand Literal identity, not a Number or Note type. The
containing Expression and its fixed Function signature assign exactly one of those types when they
consume the literal, as ADR 0021 requires.
