# 02 — Grid and Position round-trip properties

**What to build:** Encode the Grid laws that CONTEXT.md states. Generate Grid dimensions and
candidate coordinates, and check containment, the index round trip, and row coverage.

**Blocked by:** 01 — Add proptest for native targets.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `position(x, y)` returns `Some` exactly when `x` is inside the columns and `y` inside the rows.
- [ ] `position_at(index(p)) == Some(p)` for every Position the Grid mints.
- [ ] `index(position_at(i).unwrap()) == i` for every `i` below `count()`.
- [ ] `owns(p)` holds for every Position the Grid mints.
- [ ] `rows()` yields exactly `count()` Positions, each index appearing once.
- [ ] `offset_in_row(p, offset)` agrees with the column arithmetic at the right-hand edge: it
      returns `Some` exactly while `p` plus the offset stays inside `p`'s own row.
- [ ] `up`, `down`, `left`, and `right` always return a Position the Grid owns.
- [ ] Generated Grids include the one-column and one-row cases.

## Comments

The glossary sentence being encoded: "A Position can be obtained only from the Grid that contains it,
so a Position outside its Grid does not exist; the Grid converts between a Position and the index the
Source addresses Cells by."

`Grid` has two edge behaviours and the names do not distinguish them. `down(pos)` returns a
`Position` and clamps at the edge. `below(pos)` returns `Option<Position>` and gives `None`. Both are
reasonable, and a reader cannot tell which is which from the names. Cover both, and raise a rename if
the properties make the confusion concrete.

CONTEXT.md states that a Grid "has at least one column and one row". Generate from 1, and check what
`Grid::new(0, 0)` does before assuming it cannot happen.
