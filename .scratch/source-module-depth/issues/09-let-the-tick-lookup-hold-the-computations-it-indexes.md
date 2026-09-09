# 09 — Let the Tick Lookup hold the computations it indexes

**What to build:** `tick::Lookup` is a spatial index over a `&[Computation]` that keeps only
`usize` positions into that slice and never the slice itself. Every method that turns one of those
positions back into a `Computation` therefore takes the slice again, and nothing enforces that the
caller passes the same one `Lookup::new` walked. Give the index the computations once, so the
compiler states what the prose currently asks for.

**Blocked by:** None.

**Status:** ready-for-agent

- [ ] A `Lookup` cannot be asked a question against computations other than the ones it indexed.
- [ ] `Lookup::at` and `Lookup::root_at` no longer take a `nodes` parameter.
- [ ] `Schedule` still hands its caller both the computations and the index, without a second
      owner of the computations and without copying them per query.
- [ ] The existing suite passes unchanged; no Source that ticks today ticks differently.

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
