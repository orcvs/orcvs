# 02 — Derive Expressions, roots, and diagnostics

**What to build:** Extend the Language Map partition into row-confined Expressions with one root,
parsed language values, complete Footprints, and revision-consistent diagnostics.

**Blocked by:** 01 — Partition Source into Language Units.

**Status:** ready-for-agent

- [ ] Contiguous occupied Source is grouped into horizontal Expressions without row wrapping.
- [ ] A valid Expression identifies its first Function as its root and retains nested Functions.
- [ ] Literal-only, incomplete, invalid, and over-capacity Source produce the documented outcomes.
- [ ] Diagnostics point to Grid Positions/Footprints from the same Source revision.
- [ ] Operand-slot Glyph hints stop at row edges and are invalidated on edits.
- [ ] Existing Source behavior tests survive through the Language Map interface.

## Comments

The current immutable Expression extent scan is migration input, not a second lasting seam.
