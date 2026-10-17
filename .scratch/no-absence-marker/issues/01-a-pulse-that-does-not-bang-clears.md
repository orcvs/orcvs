# 01: A pulse that does not Bang clears its Output Portal

**What to build:** Equality, Delay and Euclidean answer whether they Banged.
On a Tick they don't Bang they answer a clear, which writes two spaces at
their Output Portal whether the Function is a root or nested, and nothing in
the Source tick checks which Function answered. A nested pulse returns the
Cells its Output wrote. So on a Tick it doesn't Bang, its parent's operand is
unwritten, and the parent is diagnosed as invalid input. Toby confirmed this
as expected on 2026-10-17. See `../spec.md`, step 1.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [x] No pulse Function answers the Absence Marker; a Tick it does not
      Bang answers a clear.
- [x] A pulse that doesn't Bang clears its Output Portal, root or nested,
      with no Function check in the Source tick.
- [x] A Write whose value is an unequal Equality is diagnosed with the
      unwritten-operand message and writes nothing.
- [x] Every other existing test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-17: Equality, Delay and Euclidean answer `Interpretation::Clear` on a Turn they do not Bang, and the Source tick records it as `Answer::Clear`. `a_nested_equality_that_does_not_bang_leaves_its_parent_operand_unwritten` and the nested cases in `model.rs` now expect `expected a number, found "  "`.
