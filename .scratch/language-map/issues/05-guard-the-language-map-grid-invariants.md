# 04 — Guard the Language Map Grid invariants

**What to build:** Make `LanguageMap`'s position and index lookups state the invariants they already
depend on, so a Position from a foreign Grid and a Language Unit / Atom count mismatch both fail
loudly instead of returning a plausible wrong answer.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `LanguageMap::glyph_at` asserts Grid ownership before indexing, matching its sibling
      `SourceRevision::content_at`.
- [ ] `parse_range` asserts that the Expression Unit count equals the parsed Atom count before the
      positional `zip` that assigns `LanguageUnitKind`.
- [ ] A test proves a foreign-Grid Position panics rather than reading another Grid's Glyph.
- [ ] Native, persistence, and WASM gates pass.

## Comments

Found by a CodeRabbit pass on branch `07-complete-the-numeric-function-family`. Neither finding is
in that branch's diff; both predate it, so they are recorded here rather than folded into that work.

`LanguageMap::glyph_at` (`orcvs/src/source/language_map.rs:242`) is public and takes a `Position`,
but indexes `self.glyphs` without calling `self.grid.assert_owns(position)`. `SourceRevision::content_at`
(`orcvs/src/source/mod.rs:38`) takes the same argument and does assert. The two sit either side of one
render loop and disagree about the same invariant: a Position from another Grid panics in one and
silently yields a wrong Glyph — or `None`, once the index falls outside the slice — in the other.

This is not a live defect. The only non-test caller, `RenderFrame` (`orcvs/src/render_frame.rs:93`),
walks `source.grid().rows()` and so can only pass positions the revision owns. The point is to close
the seam before a caller arrives that cannot make that promise, and to stop the pair of lookups
teaching two different rules. `.get()` keeps the read memory-safe either way, so this is a
correctness-of-contract change, not a soundness fix.

The second item is smaller. In `parse_range` (`orcvs/src/source/language_map.rs:291`), the
`expression_units.iter_mut().zip(expression.entries())` loop assigns each Unit its `LanguageUnitKind`
positionally. `zip` stops at the shorter side, so if the Unit partition and the parsed Atom sequence
ever disagree about how many entries an executable Expression has, the surplus Units keep whatever
kind they had and nothing reports it. A `debug_assert_eq!` on the two counts turns that into a test
failure at the point the assumption breaks.
