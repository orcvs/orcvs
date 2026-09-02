# 02 — Derive Expressions, roots, and diagnostics

**What to build:** Extend the Language Map partition into row-confined Expressions with one root,
parsed language values, complete Footprints, and revision-consistent diagnostics.

**Blocked by:** 01 — Partition Source into Language Units; lang-foundations/07; lang-foundations/08.

**Status:** resolved

**Tags:** release/v1

- [x] Contiguous occupied Source is grouped into horizontal Expressions without row wrapping.
- [x] A valid Expression identifies its first Function as its root and retains nested Functions.
- [x] Literal-only, incomplete, invalid, and over-capacity Source produce the documented outcomes.
- [x] Diagnostics point to Grid Positions/Footprints from the same Source revision.
- [x] Operand-slot Glyph hints stop at row edges and are invalidated on edits.
- [x] Existing Source behavior tests survive through the Language Map interface.

## Comments

The current immutable Expression extent scan is migration input, not a second lasting seam.

## Answer

`LanguageMap` is now the public revision-derived seam for row-confined Expressions, contextual
Language Units, roots, complete Footprints, Glyph hints, and diagnostics. `Source::language_map`
exposes the map for its current revision; edits rebuild it atomically, preserving existing Source
execution and presentation behavior while invalidating stale hints and diagnostics.
