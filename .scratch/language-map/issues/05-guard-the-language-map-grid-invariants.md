# 05 — Guard the Language Map Grid invariants

**What to build:** Make `LanguageMap`'s position and index lookups state the invariants they already
depend on, so a Position from a foreign Grid and a Language Unit / Atom count mismatch both fail
loudly instead of returning a plausible wrong answer.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `LanguageMap::glyph_at` asserts Grid ownership before indexing, matching its sibling
      `SourceRevision::content_at`.
- [ ] `parse_range` asserts that the Expression Unit count equals the parsed Atom count before the
      positional `zip` that assigns `LanguageUnitKind`.
- [x] A test proves a foreign-Grid Position panics rather than reading another Grid's Glyph.
- [x] Native, persistence, and WASM gates pass.

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

## Answer

Both invariants hold. One was closed by other work, one is closed here, and the third acceptance
line describes code that no longer exists.

**`glyph_at` does refuse a foreign Position**, though not in the shape this ticket imagined. It
indexes `self.grid.index(position)`, and `Grid::index` opens with `self.assert_owns(pos)`
(`orcvs/src/grid.rs`), so the refusal is the Grid's own rather than a second assertion beside the
lookup. That is the better arrangement — one rule, stated where a Position is turned into an index —
but it left the contract resting on a callee's internals with nothing pinning it. A `glyph_at`
rewritten to index `self.glyphs` directly would drop the guard silently.

**So the test this ticket asked for is now written**, and it is the only code this resolution added:
`glyph_at_refuses_a_position_minted_by_another_grid` in `orcvs/tests/language_map.rs`. It uses two
Grids of the same shape on purpose — the coordinates are perfectly valid, and identity is the thing
under test.

**The second acceptance line is void.** `parse_range` and the
`expression_units.iter_mut().zip(expression.entries())` loop it named are both gone, deleted by ADR
0024 (`ccab028`): a unit's kind is established by the row partition when it recognises the spelling,
so there is no positional `zip` between the Unit partition and a parsed Atom sequence, and no count
for the two sides to disagree about. The failure mode this line guarded against is unreachable
rather than unguarded.

The ticket's own framing still stands: this was never a live defect, and the point was to stop two
lookups either side of one render loop teaching two different rules about the same argument. They
now teach the same rule, and a test says so.

Also fixed while here: this file's heading said "04", duplicating the ticket beside it.
