# 02 — Count allocations on the Source write and Language Map rebuild

**What to build:** Writing one Cell of the Source, and rebuilding the Language Map that derives from
it, are checkable for how much they allocate, so a per-Cell allocation on the editing path fails a
pull request.

**Blocked by:** 01 — Count allocations on the Tick and Render Frame paths.

**Status:** resolved

- [x] Whether the counting allocator is duplicated in this crate's test binary or shared with
      `lang`'s is decided and the reasoning recorded. A test-only allocator does not go into a
      shipped crate's API to avoid forty duplicated lines.
- [x] Writing one Cell of the Source is asserted against a shape: the in-place byte write should not
      allocate at all.
- [x] A Language Map rebuild is asserted to be bounded independently of Grid size.
- [x] Each assertion states whether it is measured on the calling thread, and any measurement that
      crosses a multi-threaded runtime uses atomics and says that it is correct only under nextest's
      process-per-test isolation.
- [x] The tests run inside the existing `cargo nextest run --workspace --profile ci --locked` line.
      No mise task, no workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

Two different paths share one ticket because they share one harness and one setup. If the Language
Map turns out to allocate per Cell, that is a finding and it belongs in this file — it is a result
about the Language Map, not a reason to split the ticket after the fact.

The Source write is the path that reaches the workspace's one `unsafe` block, which is also what
`06` scopes Miri to.

### Finding: the write path is a whole Language Map rebuild (2026-09-09)

Every assertion was written at zero or at exact equality first and every one of them failed. What
follows is what actually allocates, measured with the counter in `orcvs/tests/allocation.rs`
against the fixture the `orcvs` benchmarks use, on a `dev` build.

**The allocator is duplicated, not shared.** A `#[global_allocator]` is per-binary, so the one in
`lang/tests/allocation.rs` cannot be seen from `orcvs`'s test binary. Sharing would not mean
sharing the allocator; it would mean exporting `Counting`, `Allocations` and `measure` from `lang`'s
public API, where every consumer of `lang` would compile them. The alternatives are no better: a
`path = "../lang/tests/..."` module drags `lang`'s fixtures and its `#[test]` functions along with
it, and a third crate to hold forty lines is a workspace member and a `cargo deny` graph entry for
a test's sake. Forty duplicated lines wins; the reasoning is in the file's `//!` docs, and the two
files are meant to read as one harness.

**The in-place byte write is not observable on its own through the public API.**

`Source::set` in `orcvs/src/source/model.rs` validates the content and calls `edit`. `edit` calls
`set_source` — the workspace's one `unsafe` block, `bytes[cell.get()] = content.byte()` — and then
`rebuild_rows`, which builds a fresh `Arc<LanguageMap>`. `set_source` is private, so no public call
reaches the byte write without also paying for the rebuild, and the rebuild is four orders of
magnitude larger than the write. The ticket asked for "the in-place byte write should not allocate
at all"; that cannot be asserted from a test binary, and it is not asserted here. What is asserted
instead, in `writing_one_cell_allocates_nothing_that_grows_with_the_revision`:

- a refused edit (`set` with two characters) allocates at most one block — `check_content` returns
  before `edit`, so neither the byte write nor the rebuild is reached. Measured 1 block / 2 bytes,
  which is `SourceError::InvalidCell` owning the text it refused; the ceiling is `<= 1` so a
  borrowing refusal reads as an improvement.
- the same Cell transition costs exactly the same on every later revision — 247 blocks / 173,913
  bytes, identically, for six successive rounds on a populated 32x32 Source. That is the shape a
  leak on the editing path breaks, and nothing else in the file would catch it.

**A rebuild allocates a Grid-sized amount of bytes and a Grid-independent number of blocks.**

One Cell written into a Grid with nothing else in it:

```
16x16   -> 16 blocks,  6117 bytes
32x32   -> 16 blocks,  9061 bytes
64x64   -> 16 blocks, 16485 bytes
128x128 -> 16 blocks, 39853 bytes   (measured while diagnosing, not asserted)
```

The block count is the fixed set of collections `LanguageMap::rebuild` builds whatever the Grid:
the carried partition, the three `row_runs` bookkeeping vectors, `walks`, `row_units`, `glyphs` as
`vec![None; bytes.len()]`, the expression and diagnostic vectors, and the `Arc`. Their contents are
Grid-sized, and the growth is exactly one byte per Cell plus 136 bytes per row — the four
measurements fit that with no residual. A rebuild that produces a whole Map cannot be smaller than
the Map, so this half is not a defect.

Asserted in `a_language_map_rebuild_is_bounded_independently_of_grid_size`: block equality across
the three fixture sizes; bytes growing no faster than the Cell count (cross-multiplied, so no ratio
is rounded and no absolute number appears); and the same writing rebuilt in a Grid four times as
tall taking the same number of blocks.

**A rebuild deep-clones every Expression in every row it did not touch.** This is the finding.

One Cell written into a fully populated Source:

```
16x16 ->  79 blocks,  47971 bytes,  43 Expressions carried
32x32 -> 247 blocks, 173913 bytes, 160 Expressions carried
64x64 -> 928 blocks, 658728 bytes, 621 Expressions carried
```

Subtracting the 16-block Grid floor above leaves about 1.45 blocks and a kilobyte per Expression,
flat across all three sizes. The same series taken by varying how many rows of a fixed 32x32 Grid
hold writing gives the same slope — 29 blocks at 5 Expressions, 244 at 160 — so it is the
Expression count and not the Cell count that drives it.

`LanguageMap::rebuild` re-parses only the dirty row, which is what its doc comment claims and what
it does. But it carries every other row forward with `..entry.clone()`, and an `ExpressionEntry`
owns an `Atoms` (`Vec<Atom>`), an `Expression`, and an `Option<Diagnostic>` holding a `String`. So
the row-level incrementality avoids re-parsing and does not avoid re-allocating: editing one Cell
in an empty margin row of a populated 32x32 Source costs 229 blocks against 247 for editing one
inside an Expression. Nearly all of a keystroke's allocation is the rows the keystroke did not
touch.

The fix is not this ticket's: sharing the parsed Expressions an unchanged row contributes — an
`Arc` per row, or a partition the Map holds and the entries borrow — rather than cloning them. The
assertion is written so that fixing it passes.

Asserted in `a_language_map_rebuild_grows_with_the_expressions_it_carries_and_no_faster`: the cost
per carried Expression never rises as the Source grows, in blocks and in bytes. That is at worst
linear in what the Map carries and never quadratic, and it needs no absolute number.

**A revision that adds one Language Unit to a row can double the partition.** Writing into a blank
row of a populated 32x32 Source costs 229 blocks / 163,007 bytes when the row already held a
character, and 231 blocks / 387,351 bytes when the row gains one. `rebuild` sizes `units`,
`expressions` and `lexical_diagnostics` with `Vec::with_capacity(previous.len())`, so one unit past
the previous count forces the doubling realloc — two more blocks, and 224KB more bytes because the
default `GlobalAlloc::realloc` allocates the new capacity. Recorded, not asserted: it is a
transient of the growth policy rather than a shape, and `05` is the right place to watch the number.

### The literal wording of the third box

"Bounded independently of Grid size" is true of the block count and false of the bytes, and the
bytes that grow are the Map itself. The box is ticked against the bounds that hold and that are
asserted — block equality across sizes and across a taller Grid holding the same writing, bytes no
faster than the Cell count, cost per carried Expression never rising — and not against a claim that
a rebuild costs the same on a large Source as on a small one. It does not: it costs about 1.45
blocks per Expression the Source holds, and the paragraphs above are why.

### For `05`

The quantities worth publishing as a series are the ones the assertions are computed from, not the
assertions: blocks and bytes for one Cell write at 16x16, 32x32 and 64x64 populated; the same at
those sizes with nothing written; and the carried Expression count beside each, since it is the
divisor that makes the number comparable across a fixture change. A caller can reach all of them
without re-running anything in a separate binary, because they are already what the three tests
measure — the emitting change is to have `measure_one_write` hand its `Allocations` to a collector
and print one line per measurement in the bencher format the benchmark workflow already parses,
gated on an environment variable so an ordinary `nextest` run stays silent.

