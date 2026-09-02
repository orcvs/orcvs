# 01 — Partition Source into Language Units

**What to build:** Derive the row-local, non-overlapping Language Unit partition defined by ADR
0018 from one immutable Source revision.

**Blocked by:** orcvs-language-migration/01, orcvs-language-migration/02, orcvs-language-migration/04; lang-foundations/02; property-testing/07.

**Status:** resolved

**Tags:** release/v1

- [x] Complete Operand Literals, Functions (including Self-Banging Functions), and Bangs are recognized.
- [x] Each unit records one Grid-minted anchor Position and complete Footprint.
- [x] Recognition is left-to-right and non-overlapping (`***`, `<<<`, and `^^^^` match ADR 0018).
- [x] An invalid character diagnoses and scanning resumes at the following Cell.
- [x] No unit or Footprint crosses a row edge.
- [x] Comment text from `##` through row end is excluded from Language Units and Expressions; one
      `#` is incomplete or invalid Source.
- [x] Tests exercise the Language Map interface with rectangular Grids and Live Edit fragments.

## Comments

Partitioning establishes complete two-Cell Operand Literal identity, not a Number or Note type. The
containing Expression and its fixed Function signature assign exactly one of those types when they
consume the literal, as ADR 0021 requires.

Implemented the row-local partition as a derived `LanguageMap` view. Each recognized unit owns a
Grid-minted anchor and complete footprint; invalid-character diagnostics stay behind the partition
interface until the later diagnostics migration ticket moves Source consumers onto it. Comments now
terminate expression extents as well as unit recognition. All four Self-Banging Function spellings
parse and render canonically.
