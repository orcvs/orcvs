# 02 — Grid and Position round-trip properties

**What to build:** Encode the Grid laws that CONTEXT.md states. Generate Grid dimensions and
candidate coordinates, and check containment, the index round trip, and row coverage.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] `position(x, y)` returns `Some` exactly when `x` is inside the columns and `y` inside the rows.
- [x] `position_at(index(p)) == p` for every Position the Grid mints. `index` answers a
      `CellIndex` and `position_at` is total.
- [x] `index(position_at(c)) == c` for every `c` that `cell_index(i)` answers, for every `i`
      below `count()`.
- [x] `cell_index(i)` answers `None` exactly when `i` is at or above `count()`.
- [x] `owns(p)` holds for every Position the Grid mints.
- [x] `rows()` yields exactly `count()` Positions, each index appearing once.
- [x] `offset_in_row(p, offset)` agrees with the column arithmetic at the right-hand edge: it
      answers `Some(CellIndex)` exactly while `p` plus the offset stays inside `p`'s own row. It
      has no test today, and three production sites call it.
- [x] `up`, `down`, `left`, and `right` always return a Position the Grid owns.
- [x] Generated Grids include the one-column and one-row cases.

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

### What landed, 2026-09-09

Eight tests in `orcvs/src/grid.rs`'s `mod property`: seven properties covering the nine acceptance
lines — `owns` is folded into the others rather than stated alone, for the reason given below —
plus the guard that says the generator reaches the shapes the lines are about. Nothing in `Grid`
changed; this is a test-only ticket and every property passed as written.

The generated value is the shape alone, and each case sweeps every Position inside it. That is the
quantifier the glossary uses — "every Position the Grid mints" — and it is what the effort's wiring
seed said issue 02 owed: sampling one Position per case leaves the quantifier itself unchecked. The
sample buys the shape: a pull request draws 32 of the 4,096 dimension pairs and the merge tier 256,
draws rather than distinct pairs, since the weighted generator repeats the edge shapes on purpose.

That is also the answer to `spec.md`'s "an exhaustive loop proves those completely, and a random
sample does not", which the deleted seed's own comment raised against exactly this design. The rule
is about the domain a property covers rather than the value it draws, and the sweep separates the
two: with a whole shape swept per case these properties range over the (shape, Position) space —
some 4 x 10^6 pairs — and `an_offset_in_row_stays_inside_the_row_it_started_in` over the (shape,
Position, offset) space, some 10^8 triples. Neither is enumerable. What a sample could still lose
is an edge shape, and `dimensions` draws those by weight rather than by luck, which is what
`generated_grids_include_the_one_column_and_one_row_cases` exists to say. The `mod property` doc
comment now records this rather than presenting the sample as self-evidently the right instrument.

What each property encodes:

- `a_position_exists_exactly_where_the_grid_has_a_cell` — `position` is `Some` exactly inside the
  dimensions, and answers with the coordinates asked for. The sweep runs two past both dimensions
  so every case states the refusal too, and `usize::MAX` on each axis says the refusal is a
  comparison rather than a bound on how far outside a caller may ask. `origin` is checked here as
  the one Position handed out without coordinates.
- `every_position_the_grid_mints_round_trips_through_its_index` — `position_at(index(p)) == p` for
  every Cell of the shape, the index is below `count`, and `cell_index(index(p).get())` answers
  that same index, so the two ways of obtaining one cannot disagree. Sweeping the whole shape is
  what makes this injectivity rather than a round trip: a colliding `index` returns at most one of
  the two Positions that reached it.
- `every_index_the_grid_answers_round_trips_through_its_position` — the other direction, for every
  `i` below `count`.
- `an_index_exists_exactly_below_the_cell_count` — the half a round trip cannot see, since every
  index a round trip walks is one `cell_index` already answered. The Cell count is the drawn
  `cols * rows`, and `count` is checked against it rather than used as the bound.
- `rows_yields_every_cell_of_the_grid_once` — `rows` yields `rows` rows of `cols` Positions,
  `cols * rows` in total, each a Cell of the Grid, and their indices are exactly `0..cols * rows`.
  That one equality states no repeat, no omission, and no swapped axis at once.
- `an_offset_in_row_stays_inside_the_row_it_started_in` — every offset from every Cell, one and two
  past the last the row admits, so each case states both the last acceptance and the first refusal
  of every row; the answered index names the Cell `offset` along the *same* row.
- `every_move_lands_on_a_cell_of_the_grid` — the four moves land on a Cell of this Grid, change one
  axis, and are inverses of each other away from the edges.
- `generated_grids_include_the_one_column_and_one_row_cases` — the coverage guard, driving
  `TestRunner` directly at a pinned 256 cases so the draws can be counted across them, in the same
  shape and for the same reason as `lang::parser`'s
  `generated_source_covers_the_space_the_incomplete_hash_and_the_comment_introducer`.

`owns` has no property of its own. It is the wrong claim to state alone: `up`, `down`, `left` and
`right` build a Position from its fields rather than asking `position` for one, so a clamp that ran
one column past the last would mint a Position this Grid owns and cannot address, and `index` would
then hand the Source a Cell in the next row. Every property therefore checks the stronger
`is_a_cell_of` — re-mintable at the same coordinates — at every path that produces a Position:
`position`, `origin`, `position_at`, `rows`, `below`, and the four moves. The helper does call
`owns`, so acceptance line 5 is stated in its own words, but the two terms are not independent
checks: `Position` derives `PartialEq` over `grid_id`, so `position(x, y) == Some(pos)` already
implies `owns(pos)`. The equality is the claim; the `owns` call is the line's literal form.

### `Grid::new(0, 0)` panics, and cannot be reached another way

It asserts `cols > 0` first, so `Grid::new(0, 0)` panics with "cols must be greater than zero";
`Grid::new(4, 0)` panics on the row count. Both are `assert!` rather than `debug_assert!`, so a
release build refuses an empty Grid as well. `mod test`'s `test_grid_cannot_have_zero_cols` and
`test_grid_cannot_have_zero_rows` already pin the pair, and the `persistence`
`TryFrom<PersistedGrid>` refuses the same shape with an error rather than a panic, so a
deserialized Grid cannot arrive empty either. Generating from 1 is therefore not an assumption —
CONTEXT.md's "at least one column and one row" is enforced by construction, and no property here
needs to admit the empty shape. The `mod property` doc comment records that, so the next reader
does not have to re-derive it.

### The seed property is gone

`position_at_inverts_index_for_a_minted_position` proved the round trip for one drawn Position and
said in its own comment that proving it for every minted Position was this issue's. It is now
`every_position_the_grid_mints_round_trips_through_its_index`, so keeping the seed would have been
a second, weaker statement of the same law. What the seed existed to prove — that the native-only
proptest dependency and its `cfg` gate are real — the suite proves eight times over.

### Correction: `offset_in_row` has two production call sites, not three

The seventh acceptance line says three. There are two: `Portal::admit`
(`orcvs/src/source/portal.rs:129`) and `tick::Lookup::at` (`orcvs/src/source/tick.rs:182`). The
third was `LanguageMap::derive`, named in `source-module-depth/07`'s comments on 2026-09-04; the
Language Map now bounds a Span with a row slice that simply has no second byte at the row's edge,
so it asks nothing of the Grid. The line's substance is unaffected — the method had no test at all,
and both remaining callers hand it `width - 1` for a run of Cells they already hold, which makes
the row's last Cell the case that decides whether a write is refused.

### Every property was negative tested

Each law was broken in `Grid` in turn and the suite re-run, to check that the property states the
law rather than merely passing beside it. All eight mutations were caught, each by the property
that names the mutated law:

- `position` admitting one column too many — caught by
  `a_position_exists_exactly_where_the_grid_has_a_cell` (and by the offset property).
- `index` transposed to `x * rows + y` — caught by both round trips, `rows`, and the offset
  property; the rectangular arm of the generator is what makes this visible, since a transposed
  index agrees with a correct one on every square Grid.
- `cell_index` admitting `count` itself — caught by `an_index_exists_exactly_below_the_cell_count`
  alone, which is why it is stated separately from the round trip.
- `offset_in_row` computed as `cell_index(index(pos) + offset)`, the wrap onto the next row —
  caught by `an_offset_in_row_stays_inside_the_row_it_started_in` alone.
- `right` without its `min(cols - 1)` clamp — caught by `every_move_lands_on_a_cell_of_the_grid`,
  through `is_a_cell_of` rather than `owns`, which is the case that motivates the helper.
- `below` clamping like `down` — caught by `every_move_lands_on_a_cell_of_the_grid` (and by the
  existing example test, which pins the same pairing).
- `rows` yielding columns — caught by `rows_yields_every_cell_of_the_grid_once`.
- the generator's ranges raised from 1 to 2 — caught by
  `generated_grids_include_the_one_column_and_one_row_cases`, which is the whole reason that guard
  exists.

The counterexample files those runs wrote were deleted rather than committed: they are seeds for
mutations that no longer exist, not for a bug in the Grid. `orcvs/proptest-regressions/grid.txt` is
absent because no property fails, which is what issue 01 predicted.

### `down` versus `below` is now a ticket

Raised as `source-module-depth/10 — Name the Grid's two edge answers apart`, per this issue's own
instruction to raise a rename if the properties made the confusion concrete. They did: the
bottom-row branch of `every_move_lands_on_a_cell_of_the_grid` has to assert both behaviours and
comment which is which, because a reader meeting either line alone cannot tell whether the clamp is
the contract or the bug. Two things the properties turned up went into that ticket rather than
being fixed here — only the vertical axis has the `Option` form at all, and
`spatial-tick-planning/03` needs the out-of-Grid answer on all four directions. Nothing is renamed
on this branch.

### Verification

`PROPTEST_CASES=32` throughout, which is what the pull-request tier runs.

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package orcvs --all-targets --locked -- -D warnings` — passed.
- `cargo clippy --package shell --all-targets --locked -- -D warnings` — passed. `orcvs` means
  `orcvs` and `shell`, and the first version of this report named only the `shell` test run.
- `cargo nextest run --package orcvs --locked` — passed, 306 tests.
- `cargo nextest run --package shell --locked` — passed, 35 tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed. This and the two
  below are the pass CLAUDE.md asks for once, before opening a pull request.
- `cargo nextest run --workspace --locked` — passed, 530 tests.
- `cargo test --workspace --doc --locked` — passed.
- `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null` — passed,
  for the two `.scratch/` files this change touches.
- The suite's cost was measured rather than assumed, because the merge tier runs it at 256 cases
  and every case sweeps a whole shape: 11ms at one case, 17ms at 32, 34ms at 256, 723ms at 4,096.
  The scaling is also what says the sweeps run — a property whose body was optimised away would
  not get slower with more cases.

Not run: `mise run check`, `mise run check_merge`, `mise run test_wasm`, `mise run bench`, and
proptest's 256-case default — Deferred to CI. `mise run check_wasm` was not run either: the change
is one `#[cfg(all(test, not(target_arch = "wasm32")))]` module, which no WASM build compiles.

Risks: none. No production code changed, so no public API, unsafe, dependency, feature, or
performance risk. The one cost is test time, measured above.

### Review pass, 2026-09-09

Three reviewers ran against `main`: the repo's Standards and Spec axes, the built-in correctness
review, and the CodeRabbit CLI. CodeRabbit returned nothing. Fourteen findings survived reading the
code they cite; what changed:

**Two properties were bounded by the Grid's own answer about its size.** The worst was
`an_index_exists_exactly_below_the_cell_count`, which compared `cell_index(i).is_some()` against
`i < grid.count()`. `cell_index` *is* `idx < self.count()`, so both sides were one expression
compared with itself and the property could not fail for any `count`. Mutating `count` to
`cols * rows + 1` and re-running confirmed it: the property passed. It now takes the Cell count
from the drawn `cols * rows` and asserts `count` against it, and the same mutation fails it.
`every_index_the_grid_answers_round_trips_through_its_position` swept `0..grid.count()`, so a
`count` that under-reported by one made it skip the last Cell rather than fail — confirmed the same
way, and fixed the same way. `rows_yields_every_cell_of_the_grid_once` and the
`idx.get() < count` assertion in the forward round trip were moved to `cols * rows` for the reason
`every_position` already gave and these three did not follow.

This corrects the negative-testing record above in one place: the `cell_index` mutation it lists is
genuinely caught, but a `count` mutation was not caught by anything that named it. Both directions
are now caught by the property whose name claims them.

**The effort spec's exhaustiveness rule was raised and is now answered** rather than left to the
deleted seed's comment — see the paragraph added above, and the `mod property` doc comment.

**Four documentation defects**, all in claims about the work rather than in the work: the summary
line said "one per acceptance line" for nine lines and seven properties; the sample was described
as buying 32 distinct dimension pairs when the weighted generator repeats edge shapes on purpose;
`is_a_cell_of` was described as two independent checks when `Position`'s derived `PartialEq` over
`grid_id` makes the equality subsume `owns`; and the coverage guard's failure message named half
its own condition.

**Acceptance line 7 is restored to what it originally asked.** Amending it in place mutated the
record of what was commissioned; `docs/agents/issue-tracker.md` has corrections append under
`## Comments`, which is where the "two call sites, not three" correction already lived.

Three findings were not acted on. `test_grid_indices_cover_every_cell_exactly_once` and the
`cell_index(8)/cell_index(100)` assertions in `mod test` are subsumed by the new properties, but
deleting example tests is a different decision from deleting the seed property: the seed was a
weaker property beside a stronger one in the same register, and `source-module-depth/10` explicitly
requires `mod test` keep stating the `down`/`below` pairing. Threading `(grid, cols, rows)` through
a `Shape` struct and splitting `PAST_THE_END` into three constants are readability preferences that
would touch every property to settle a question no reviewer called a defect.
