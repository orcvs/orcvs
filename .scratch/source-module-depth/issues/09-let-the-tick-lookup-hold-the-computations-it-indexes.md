# 09 — Let the Tick Lookup hold the computations it indexes

**What to build:** `tick::Lookup` is a spatial index over a `&[Computation]` that keeps only
`usize` positions into that slice and never the slice itself. Every method that turns one of those
positions back into a `Computation` therefore takes the slice again, and nothing enforces that the
caller passes the same one `Lookup::new` walked. Give the index the computations once, so the
compiler states what the prose currently asks for.

**Blocked by:** None.

**Status:** resolved

- [x] A `Lookup` cannot be asked a question against computations other than the ones it indexed.
- [x] `Lookup::at` and `Lookup::root_at` no longer take a `nodes` parameter.
- [x] `Schedule` still hands its caller both the computations and the index, without a second
      owner of the computations and without copying them per query.
- [x] The existing suite passes unchanged; no Source that ticks today ticks differently.

## Comments

### Found during review of `deepen-portal-and-evaluator-seams`, 2026-09-09

`Lookup` (`orcvs/src/source/tick.rs`) stores `Claim { cells: Range<usize>, node: usize }` for its
three claim sets plus a `subtree_ends: Vec<usize>` indexed by node. All four hold positions into
the slice `Lookup::new(grid, nodes)` walked, and none holds the slice:

```rust
fn root_at(&self, grid: Grid, nodes: &[Computation], anchor: Position) -> Option<usize>
fn at<'a>(&'a self, grid: Grid, nodes: &'a [Computation], output: Position)
    -> Option<PortalRelationships<'a>>
```

The signature accepts any `&[Computation]`. A shorter slice panics on `nodes[index]`, or hands back
a `descendants()` range running past the end. A same-length slice with different contents does not
panic at all: `PortalRelationships::functions` computes `at_anchor` as
`self.nodes[index].anchor == self.output`, and that flag decides whether a write *replaces* a
Function or *suppresses* its whole subtree. A mismatched slice silently reverses that choice.

This is latent, not live — `plan_configured` and `schedule` each have exactly one `nodes` in scope.
It is worth writing down because the module already keeps a second array keyed by the same index:
`plan_configured`'s `functions: Vec<Function>` overlay holds replacements while `nodes` keeps the
original spellings. Two parallel views of one node set is the habit that makes passing the wrong
one plausible.

The shape is pre-existing — `root_at` has carried it since before that branch, and `at` inherited
the convention — so it is not a regression that branch introduced, and fixing one method without
the other would be worse than leaving both.

**The obstacle worth planning around.** The obvious change is a lifetime:

```rust
struct Lookup<'a> { nodes: &'a [Computation], /* ... */ }
```

but `Schedule` owns both halves:

```rust
struct Schedule {
    nodes: Vec<Computation>,
    lookup: Lookup,
    order: Vec<usize>,
    diagnostics: Vec<Diagnostic>,
}
```

so a borrowing `Lookup` makes `Schedule` self-referential — a struct owning a `Vec` alongside a
borrow into that same `Vec`. That is the whole cost of this ticket, and it is a design decision
rather than a mechanical edit. Three shapes to weigh:

- Split `Schedule` so the computations outlive the index and the borrow lives at the call site.
- Have `Lookup` own the computations and lend them out, inverting today's ownership.
- Hold them behind a shared pointer, which buys the least and costs an allocation.

The first keeps the current data layout and is probably the smallest, but it moves work into
`schedule`'s caller; confirm that before committing to it.

Pairs naturally with `06-measure-the-rebuild-path.md` if the chosen shape touches allocation on the
Tick path — `orcvs/benches/source.rs` is the arbiter for any claim that it does.

### Confirmed ownership design, 2026-09-09

Lookup owns the original computations it indexes and lends read-only access to scheduling and
execution. Moving the existing `Vec<Computation>` into Lookup keeps construction inside `schedule`
and avoids a self-referential Schedule, shared ownership, or copies per query. `Schedule` carries
the Lookup alongside execution order and diagnostics.

`PortalRelationships` borrows only Lookup rather than pairing it with a separate computation slice.
`potentially_active` likewise obtains the computations from Lookup. Replacement Functions and the
other changing execution state stay local to the Tick; the index retains the original parsed
structure. Existing behavior tests remain the test surface, with no new performance claim.

### Verification, 2026-09-09

Changed: `orcvs/src/source/tick.rs` — `Lookup` owns the `Vec<Computation>` its Claims and subtree
ranges index; `Lookup::at`, `Lookup::root_at`, and `potentially_active` lose their `nodes`
parameter; `Schedule` and `PortalRelationships` lose their computation fields.

Tests added or updated: none. The change is an ownership refactor the compiler enforces, so the
existing suite is the regression surface and no test file is touched.

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package orcvs --package shell --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --package shell --locked` — failed before
  compilation because sccache returned `Operation not permitted`, including on the escalated retry.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --package orcvs --package shell --locked` —
  passed all 334 existing tests with the compiler cache disabled; no tests changed.
- `node --test scripts/tests/roadmap.test.ts` — passed all 10 tests.
- `node scripts/roadmap.ts > /dev/null` — passed.
- `git diff --check` — passed; complete diff reviewed.

Not run: persistence, Linux, and WASM combinations — deferred to CI; no persistence, feature, or
platform-specific code changed. `mise run check`, `mise run check_merge`, `mise run test_wasm`,
`mise run bench`, and proptest's 256-case default — deferred to CI. Pre-PR workspace and doctest
gates subsequently passed as recorded below.

Risks: private ownership refactor; no public interface, unsafe, concurrency, dependency, or feature
changes. Scheduling and execution rules are unchanged. No performance improvement is claimed.

### Pre-PR verification, 2026-09-09

- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `RUSTC_WRAPPER= PROPTEST_CASES=32 cargo nextest run --workspace --locked` — all 523 tests passed.
- `RUSTC_WRAPPER= cargo test --workspace --doc --locked` — all 9 doctests passed.

The compiler cache remained disabled for these runs because of the sccache permission failure
recorded above. Tests are unchanged.

### Review follow-up: the Grid half of the same clump, 2026-09-09

A three-source review of the branch — the repository's Standards and Spec axes, the built-in
correctness review, and the CodeRabbit CLI — converged on one gap. Absorbing `nodes` closed half
of the hazard the ticket names and left the other half standing: `Lookup` held no `Grid`, yet every
`Claim.cells` and `subtree_ends` entry is a Cell number stated in the coordinates of the Grid that
built it, and `at` and `root_at` recomputed Cell numbers from a Grid the caller supplied.

`Grid::assert_owns` cannot refuse that pairing, because a caller offering a foreign Grid would also
be offering a Position that Grid genuinely owns. A `Lookup` built on a twenty-column Grid, queried
with a sixteen-column Grid's Position, would match `y * 16 + x` against claims stated as
`y * 20 + x` — no panic, wrong contacts, and the same silent `at_anchor` reversal the ticket exists
to eliminate, displaced one argument over. No caller does this today; neither did the `nodes`
mismatch the ticket was written about.

`Lookup` now owns the `Grid` as well. `at`, `root_at`, and `potentially_active` take no `Grid`, and
`PortalRelationships` reads it through the `Lookup` it already borrows rather than carrying a second
copy. `Grid` is `Copy` and three words wide, so this adds no allocation and no query-time cost.
Behaviour is unchanged and the compiler now refuses the pairing outright.

Not fixed: commit `fb1bf9a` carries a bare subject line with no rationale body and no
`Claude-Session:` trailer, unlike every preceding non-merge commit. Correcting it means rewriting
history already published to pull request #41, so it is left for the author to decide.

### Follow-up verification, 2026-09-09

Changed: `orcvs/src/source/tick.rs` — `Lookup` owns the `Grid` its Cell numbering is stated in;
`Lookup::at`, `Lookup::root_at`, and `potentially_active` lose their `Grid` parameter;
`PortalRelationships` drops its `Grid` copy. Plus the two prescribed completion-evidence lines and
the field comment's `///` form in this file and above.

Tests added or updated: none. Removing a parameter is a compile-time invariant with no runtime
behaviour to regress, so the existing suite is the regression surface.

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --package orcvs --package shell --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo nextest run --package orcvs --package shell --locked` — 334 tests
  passed, the same count and set as before the change.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `PROPTEST_CASES=32 cargo nextest run --workspace --locked` — all 523 tests passed.
- `cargo test --workspace --doc --locked` — all 9 doctests passed.
- `git diff --check` — passed; complete diff reviewed.

`RUSTC_WRAPPER=` was exported for every run above, because the sccache `Operation not permitted`
failure recorded earlier still stands.

Not run: persistence, Linux, and WASM combinations, `mise run check`, `mise run check_merge`,
`mise run test_wasm`, `mise run bench`, and proptest's 256-case default — deferred to CI. No
persistence, feature, platform, or benchmark input changed.

Risks: private ownership refactor; no public interface, unsafe, concurrency, dependency, or feature
changes. Scheduling and execution rules are unchanged. No performance claim.
