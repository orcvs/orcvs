//!
//! Allocation shapes for the two paths an editing session sits on: writing one
//! Cell of the Source, and the Language Map rebuild every write pays for. A
//! keystroke happens as fast as a writer types and a Render Frame reads the
//! Map that the last keystroke left behind, so an allocation added per Cell
//! here is a regression the criterion series would hide behind a cache hit.
//!
//! # Why the allocator is duplicated rather than shared with `lang`
//!
//! `lang/tests/allocation.rs` holds the same `Counting` allocator, the same
//! `Allocations`, `snapshot` and `measure`. That duplication is deliberate.
//!
//! A `#[global_allocator]` is per-binary: the one in `lang`'s test binary
//! cannot be seen from this one, so sharing would mean not sharing the
//! allocator itself but exporting `Counting`, `Allocations` and `measure` from
//! `lang`'s public API — test-only support compiled into a shipped crate for
//! the benefit of another crate's tests, and visible to every consumer of
//! `lang`. The alternatives are no better: a `path = "../lang/tests/..."`
//! module would drag `lang`'s fixtures and its `#[test]` functions in with it,
//! and a third crate to hold forty lines is a workspace member and a
//! `cargo deny` graph entry for a test's sake.
//!
//! Forty duplicated lines against an API entry is the trade `01` left open and
//! this file settles: duplicate. The two files are meant to read as one
//! harness, so a change to the allocator or to `measure` belongs in both.
//!
//! # What the assertions say
//!
//! Shapes, never absolute numbers, in the order `01` fixed: zero; a ceiling
//! derived from the input, so a cheaper implementation still passes; an
//! equality between two measurements of the same work at different input
//! sizes.
//!
//! A failing assertion is a finding. Record what allocates and why in
//! `.scratch/memory-verification/issues/` before relaxing one.
//!
//! Everything measured here runs on the calling thread. `Source::set` is a
//! direct call, and `SourceCommander::set` only takes the write lock and
//! delegates on the same thread, so the thread-local counters below need no
//! synchronisation and these tests stay correct under a bare `cargo test`.
//! Nothing here crosses a multi-threaded runtime; anything that did would need
//! `AtomicUsize` with `Ordering::Relaxed` and would then be correct only under
//! nextest's process-per-test isolation.
//!
//! # Publishing the numbers, not only asserting on them
//!
//! `.github/workflows/bench.yml` stores these measurements as a series on
//! `gh-pages` beside the criterion timings, so a creep too small for any one
//! assertion to catch is visible as a trend. `lang/tests/allocation.rs` says
//! the whole of why, and the helpers below are the other half of the one
//! harness these two files are meant to read as.
//!
//! Two things about the records this file adds. The published quantities are
//! the ones the assertions are computed from rather than the assertions
//! themselves — one Cell write at each fixture size, with the Grid empty and
//! with it populated. And the Expressions a rebuild carries are published
//! beside each populated point, because that count is the divisor which makes
//! the two counts above it comparable at all: a change to the fixture moves
//! the allocation counts for a reason that is not a regression, and the
//! divisor moving beside them is what says so.
//!
// Native-only, the way the property suites and `lang/tests/allocation.rs` are:
// `System` and this counting are native concerns, and `--all-targets` on
// `wasm32-unknown-unknown` compiles this target too. The `cfg` matches the
// shape of the `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]`
// table in `orcvs/Cargo.toml`, so a WASM build sees an empty test binary
// rather than a global allocator it has no `System` for.
#![cfg(not(target_arch = "wasm32"))]

use orcvs::grid::{CellIndex, Grid};
use orcvs::source::Source;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;

thread_local! {
    // `const` initialisation is what makes these safe to touch from inside the
    // allocator: a lazily initialised thread-local allocates on first access,
    // which would be an allocation from within the allocation it is counting.
    // `Cell<usize>` has no drop glue, so no destructor is registered either.
    static BLOCKS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

/// `System`, plus a count of the blocks and bytes handed out.
///
/// `realloc` and `alloc_zeroed` are deliberately not overridden.
/// `GlobalAlloc`'s defaults implement both in terms of `Self::alloc` and
/// `Self::dealloc`, so they route back through the two methods below and are
/// counted without the counting being written twice.
struct Counting;

// SAFETY: `GlobalAlloc`'s contract is discharged entirely by `System`, which
// already upholds it. Every method here forwards to `System` with the very
// layout and pointer it was handed, adds nothing to either, and returns
// exactly what `System` returned — including a null pointer on failure, which
// the caller must already handle. So every block this allocator hands out was
// allocated by `System`, and every block it frees is passed to `System` with
// the layout it was allocated with, which is what makes deallocation sound.
// The counters are separate thread-local integers: they observe `layout.size()`
// and touch neither the pointer nor the layout, and their `const`-initialised
// storage cannot itself allocate and so cannot re-enter this allocator.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BLOCKS.with(|blocks| blocks.set(blocks.get() + 1));
        BYTES.with(|bytes| bytes.set(bytes.get() + layout.size()));
        // SAFETY: `layout` is the caller's, forwarded unchanged, and the
        // caller has already met `alloc`'s requirement that it be non-zero
        // sized. The result is returned unchanged, so `System` owns the block.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from this allocator, which means it came from
        // `System.alloc` above, and `layout` is the one it was allocated with,
        // because both are forwarded unchanged in both directions.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// What one span of work asked the allocator for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Allocations {
    blocks: usize,
    bytes: usize,
}

fn snapshot() -> Allocations {
    Allocations {
        blocks: BLOCKS.with(Cell::get),
        bytes: BYTES.with(Cell::get),
    }
}

/// Runs `f` and answers what it allocated, alongside whatever it produced.
///
/// The value is handed back rather than dropped inside, so a `Drop` that frees
/// falls outside the span — deallocation is not counted, but keeping the
/// result alive is what stops the optimiser deciding the work was pointless.
/// The helper itself allocates nothing: two `Cell` reads before and after.
fn measure<T>(f: impl FnOnce() -> T) -> (Allocations, T) {
    let before = snapshot();
    let value = f();
    let after = snapshot();

    (
        Allocations {
            blocks: after.blocks - before.blocks,
            bytes: after.bytes - before.bytes,
        },
        value,
    )
}

/// The prefix every published record carries.
///
/// `cargo test -- --nocapture` writes these records into the same stream as
/// the harness's own output, so the workflow step that turns them into JSON
/// picks them out by this marker. It is matched literally in
/// `.github/workflows/bench.yml`; it changes in both places or in neither.
const SERIES_MARKER: &str = "ORCVS-MEMORY-SERIES";

/// Whether this run publishes the series as well as asserting on it.
///
/// Reading the variable allocates, so every caller sits outside a measured
/// span. Nothing here is on a measured path in any case: publishing happens
/// after `measure_one_write` has already answered.
fn publishing() -> bool {
    std::env::var("ORCVS_MEMORY_SERIES").is_ok_and(|value| value == "1")
}

/// One `customSmallerIsBetter` record on its own line — the shape
/// `benchmark-action/github-action-benchmark` reads.
///
/// `name` is built from literals and decimal Grid dimensions, so it holds no
/// character JSON would need escaped and no escaping is written. A name that
/// needed escaping would be a name that had changed, and a changed name starts
/// an empty series rather than continuing this one.
fn record(name: &str, unit: &str, value: usize) {
    println!(r#"{SERIES_MARKER} {{"name": "{name}", "unit": "{unit}", "value": {value}}}"#);
}

/// Both halves of one measurement, as `<name> blocks` and `<name> bytes`.
fn publish(name: &str, allocations: Allocations) {
    if !publishing() {
        return;
    }

    record(&format!("{name} blocks"), "blocks", allocations.blocks);
    record(&format!("{name} bytes"), "bytes", allocations.bytes);
}

/// The Expressions a rebuild carried, published beside the counts it divides.
///
/// This is a fixture size rather than a cost, and it goes into the same
/// `customSmallerIsBetter` series as the counts, so the action scores it as if
/// smaller were better and alerts when it rises past `alert-threshold`. That is
/// the trade rather than an oversight. `github-action-benchmark` has no
/// per-metric threshold, so keeping the divisor unscored would mean a second
/// series, a second `memory.json`, and a second action step in each of the two
/// bench jobs — and it would put the divisor on a different chart from the
/// numbers it exists to make readable, which is the whole of why it is
/// published.
///
/// What it costs is bounded: `SIZES`, `EXPRESSIONS` and `source_text` are
/// constants, so this number moves only on a commit that deliberately edits the
/// fixture, and that commit moves the allocation counts beside it. An alert here
/// therefore means "the fixture changed", which is exactly what a reader needs
/// to know to read the alerts next to it — and `fail-on-alert: false` keeps it
/// from failing the job either way.
fn publish_carried(name: &str, expressions: usize) {
    if !publishing() {
        return;
    }

    record(
        &format!("{name} expressions carried"),
        "expressions",
        expressions,
    );
}

/// Representative Source shapes, as `orcvs/benches/source.rs` fixes them: a
/// console opens on 1000 Cells, which 32x32 stands for, and the two shapes
/// bracketing it each change the Cell count fourfold. Shared with the
/// benchmarks by duplication, because a bench target and a test target cannot
/// import one another.
const SIZES: &[(usize, usize)] = &[(16, 16), (32, 32), (64, 64)];

/// The Expression shapes an editing session actually holds. Duplicated from
/// `orcvs/benches/source.rs` for the same reason.
const EXPRESSIONS: &[&str] = &[
    ".+0102",
    ".x0201",
    "!>010AC4",
    ".-0A05",
    ".+01XY",
    "./0402",
    ".+.+0101.-0A05",
    "**",
    ".+01",
    ">>",
];

/// The Cell every edit here rewrites: an operand digit of the first Expression
/// on the second row, so an edit lands inside populated Source rather than in
/// an empty margin. The benchmarks edit the same Cell.
const EDIT_COLUMN: usize = 4;

/// What that Cell holds in the fixture, and what each unmeasured restore
/// writes back.
const RESTORED: &str = "0";

/// A different digit in the same operand position: still a valid Expression,
/// so the rebuild does the full parse an accepted edit pays for.
const EDITED_VALID: &str = "3";

/// The index `grid` mints for `idx`. A Cell is named by an index its Grid
/// minted, so a test states the number and the Grid answers with the Cell.
fn cell(grid: Grid, idx: usize) -> CellIndex {
    grid.cell_index(idx).expect("inside the Grid")
}

/// One Cell per Grid Position, as `orcvs/benches/source.rs` builds it.
fn source_text(cols: usize, rows: usize) -> String {
    let mut text = String::with_capacity(cols * rows);

    for row in 0..rows {
        let mut line = String::with_capacity(cols + EXPRESSIONS[0].len());
        let mut next = row;
        while line.len() < cols {
            line.push_str(EXPRESSIONS[next % EXPRESSIONS.len()]);
            line.push(' ');
            next += 1;
        }
        line.truncate(cols);
        text.push_str(&line);
    }

    text
}

/// The Source those Cells make, written one Cell at a time the way a session
/// writes one.
fn populated_source(cols: usize, rows: usize) -> Source {
    source_written_to(cols, rows, rows)
}

/// The same writing, laid into a Grid of `rows` rows but confined to the first
/// `written` of them. A Grid is as tall as it is whatever the writer has
/// filled in, so the unwritten rows below are the ordinary case rather than a
/// contrived one.
fn source_written_to(cols: usize, rows: usize, written: usize) -> Source {
    let grid = Grid::new(cols, rows);
    let mut source = Source::new(grid);

    for (idx, content) in source_text(cols, written).chars().enumerate() {
        if content != ' ' {
            source
                .set(cell(grid, idx), &content.to_string())
                .expect("fixture Source content is accepted");
        }
    }

    source
}

/// One accepted Cell edit: the whole public write path, byte write and
/// Language Map rebuild together.
fn write_one_cell(source: &mut Source, edited: CellIndex, content: &str) {
    black_box(source)
        .set(black_box(edited), black_box(content))
        .expect("the edited Cell is accepted");
}

/// What one write of `content` costs, with the restore that makes it a change
/// rather than a rewrite kept outside the measured span, and a warm-up round
/// before it so a one-off initialisation cannot land inside one.
fn measure_one_write(source: &mut Source, edited: CellIndex, content: &str) -> Allocations {
    write_one_cell(source, edited, content);
    write_one_cell(source, edited, RESTORED);
    let (write, ()) = measure(|| write_one_cell(source, edited, content));

    write
}

/// How many Expressions the current revision's Language Map holds. This is
/// what a rebuild carries, and the quantity the assertions below are bounded
/// against rather than the Cell count.
fn carried_expressions(source: &Source) -> usize {
    source.language_map().expressions().count()
}

#[test]
fn writing_one_cell_allocates_nothing_that_grows_with_the_revision() {
    // FINDING (2026-09-09): the in-place byte write cannot be measured on its
    // own from outside the crate, and the write path is not allocation-free.
    //
    // `Source::set` in `orcvs/src/source/model.rs` validates the content and
    // calls `edit`, and `edit` does two things: `set_source`, which is the
    // workspace's one `unsafe` block writing the byte in place, and then
    // `rebuild_rows`, which builds a fresh `Arc<LanguageMap>`. `set_source` is
    // private, so no public call reaches the byte write without also paying
    // for the rebuild, and one write of a populated 32x32 Source costs 247
    // blocks and 173,913 bytes — all of it the rebuild. See
    // `.scratch/memory-verification/issues/02-count-allocations-on-the-source-write-and-language-map-rebuild.md`.
    //
    // So this test asserts the two things that *are* observable through the
    // public API, and the two tests below bound the rebuild itself:
    //
    //   - a refused edit stops before either half, so it allocates at most the
    //     error text it hands back;
    //   - the same edit costs exactly the same on every later revision, so
    //     nothing the write path retains gets *more expensive* per revision.
    //
    // That second one is weaker than "the write path leaks nothing", and the
    // difference is worth stating because it is easy to read the equality as
    // the stronger claim. `measure` counts allocations and never
    // deallocations, so it sees cost per revision and not what is still held.
    // A retention of constant size — pushing each `Arc<LanguageMap>` onto an
    // undo history, say — allocates the same amount every round and passes
    // here untouched. What breaks the equality is a retention whose cost
    // *grows* with the revision count, and even then an amortised-doubling
    // growth reallocates on roughly one round in five. The wasm counterpart in
    // `console/tests/wasm.rs` is the assertion that watches retention itself,
    // because linear memory cannot shrink; on native there is no equivalent
    // and this file does not claim one.
    //
    // Measured on the calling thread. `Source::set` is a direct call, and
    // `SourceCommander::set` only takes the write lock and delegates on the
    // same thread.
    let (cols, rows) = (32, 32);
    let mut source = populated_source(cols, rows);
    let edited = cell(source.grid(), cols + EDIT_COLUMN);

    // Warm up, for the reason `measure_one_write` does.
    write_one_cell(&mut source, edited, EDITED_VALID);

    // A refused edit never reaches the byte write or the rebuild. The ceiling
    // is one block because `SourceError::InvalidCell` owns the content it was
    // refused; a borrowing refusal would be zero and must still pass.
    let (refused, outcome) = measure(|| source.set(black_box(edited), black_box("ab")));
    assert!(outcome.is_err(), "two characters are not one Cell");
    assert!(
        refused.blocks <= 1,
        "a refused edit allocated {refused:?}, past the error text it returns"
    );

    // The same transition, revision after revision. A per-revision cost that
    // grows — anything retained in the Source or its Map that gets dearer to
    // add — shows up as an inequality here, and nothing else in this file
    // would catch it. A retention of constant size does not; see above.
    let first = measure_one_write(&mut source, edited, EDITED_VALID);
    for round in 0..4 {
        let again = measure_one_write(&mut source, edited, EDITED_VALID);
        assert_eq!(
            again, first,
            "writing the same Cell allocated {again:?} on round {round} against {first:?} first"
        );
    }
}

#[test]
fn a_language_map_rebuild_is_bounded_independently_of_grid_size() {
    // FINDING (2026-09-09): in blocks, yes; in bytes, no, and the bytes are
    // the Grid itself rather than anything per-Cell that could be removed.
    //
    // `LanguageMap::rebuild` allocates a fixed set of collections whatever the
    // Grid: the carried partition, the per-row bookkeeping `row_runs` builds,
    // `glyphs` as `vec![None; bytes.len()]`, and the `Arc`. That set does not
    // grow in number with the Grid, so the block count below is equal at every
    // size. Their contents do: one Glyph slot per Cell and three row runs plus
    // a walk slot per row, which is 136 bytes per row and one byte per Cell.
    // A rebuild that produces a whole Map cannot be smaller than the Map.
    //
    // 2026-09-10: ADR 0035's branch replaced that set of collections. A Map
    // now holds one `DerivedRow` per row; `row_runs` and the flat carried
    // partition are gone, and `glyphs` is per-row rather than one
    // `vec![None; bytes.len()]`. A row the walk read no Source in allocates
    // nothing at all, so the equality asserted below still holds. The
    // collections named above no longer exist; the bound does.
    //
    // Measured on the calling thread.
    let mut measured = Vec::new();

    for &(cols, rows) in SIZES {
        // A Grid with nothing written in it but the Cell being edited, so
        // what is measured is the Grid itself and not what is written in it.
        // The writing is bounded in the test below.
        let mut source = source_written_to(cols, rows, 0);
        let edited = cell(source.grid(), cols + EDIT_COLUMN);
        let write = measure_one_write(&mut source, edited, "x");
        publish(&format!("orcvs write one cell {cols}x{rows} empty"), write);
        measured.push(((cols, rows), write));
    }

    let (_, first) = measured[0];
    for &((cols, rows), write) in &measured {
        assert_eq!(
            write.blocks, first.blocks,
            "a {cols}x{rows} rebuild took {} blocks against {} at the smallest size; all: {measured:?}",
            write.blocks, first.blocks
        );
    }

    // The bytes are not equal, and the shape asserted instead is that they
    // grow no faster than the Cell count. Cross-multiplied rather than
    // divided, so no ratio is rounded and no absolute number appears.
    for pair in measured.windows(2) {
        let [
            ((small_cols, small_rows), small),
            ((large_cols, large_rows), large),
        ] = pair
        else {
            unreachable!("windows(2) yields pairs")
        };
        let (small_cells, large_cells) = (small_cols * small_rows, large_cols * large_rows);
        assert!(
            large.bytes * small_cells <= small.bytes * large_cells,
            "a {large_cols}x{large_rows} rebuild took {} bytes over {large_cells} Cells, \
             against {} over {small_cells} — more than linear in the Cell count",
            large.bytes,
            small.bytes
        );
    }

    // The same writing in a taller Grid: the rows below hold nothing, so the
    // rebuild carries exactly the same Expressions and takes exactly the same
    // blocks to do it.
    let (cols, written) = (32, 8);
    let mut short = source_written_to(cols, written * 2, written);
    let mut tall = source_written_to(cols, written * 8, written);
    assert_eq!(carried_expressions(&short), carried_expressions(&tall));

    let edited = cell(short.grid(), cols + EDIT_COLUMN);
    let short = measure_one_write(&mut short, edited, EDITED_VALID);
    let edited = cell(tall.grid(), cols + EDIT_COLUMN);
    let tall = measure_one_write(&mut tall, edited, EDITED_VALID);

    assert_eq!(
        short.blocks, tall.blocks,
        "the same writing rebuilt in a taller Grid took {} blocks against {}",
        tall.blocks, short.blocks
    );
}

#[test]
fn a_language_map_rebuild_grows_with_the_expressions_it_carries_and_no_faster() {
    // FINDING (2026-09-09): a rebuild re-parses one row and deep-clones every
    // Expression in every other row. `LanguageMap::rebuild` carries an
    // unchanged row forward with `..entry.clone()`, and an `ExpressionEntry`
    // owns an `Atoms` — a `Vec<Atom>` — an `Expression`, and an optional
    // `Diagnostic` holding a `String`. So the row-level incrementality that
    // avoids re-parsing does not avoid re-allocating: one Cell edited in a
    // populated 32x32 Source costs 247 blocks and 173,913 bytes, about 1.4
    // blocks and a kilobyte for each of the 160 Expressions the Map carries,
    // and editing a Cell in an empty margin row costs nearly as much as
    // editing one inside an Expression.
    //
    // What is asserted is therefore the bound that holds — the cost per
    // carried Expression never rises as the Source grows, so the rebuild is at
    // worst linear in what it carries and never quadratic — rather than a
    // ceiling that would rot, or an independence from the Grid that this path
    // does not have. Making the carry cheap, by sharing the parsed Expressions
    // an unchanged row contributes instead of cloning them, would drive both
    // numbers down and must still pass.
    //
    // 2026-09-10: the carry is now `DerivedRow::for_revision`, which clones
    // the row and re-stamps each Expression with the new Map identity, so
    // `..entry.clone()` names a mechanism that is gone. The finding itself is
    // unchanged and if anything sharper: a carried row clones its Glyphs and
    // its diagnostics as well as its Expressions. The bound below is what was
    // re-measured, and it still holds.
    //
    // Measured on the calling thread.
    let mut measured = Vec::new();

    for &(cols, rows) in SIZES {
        let mut source = populated_source(cols, rows);
        let edited = cell(source.grid(), cols + EDIT_COLUMN);
        let write = measure_one_write(&mut source, edited, EDITED_VALID);
        let expressions = carried_expressions(&source);
        assert!(expressions > 0, "the fixture must hold Expressions");
        let name = format!("orcvs write one cell {cols}x{rows} populated");
        publish(&name, write);
        publish_carried(&name, expressions);
        measured.push(((cols, rows), expressions, write));
    }

    for pair in measured.windows(2) {
        let [
            (_, small_expressions, small),
            ((cols, rows), large_expressions, large),
        ] = pair
        else {
            unreachable!("windows(2) yields pairs")
        };
        assert!(
            large_expressions > small_expressions,
            "a larger Source must carry more Expressions"
        );
        // Cost per carried Expression, cross-multiplied so nothing is rounded
        // and no absolute number appears in the assertion.
        assert!(
            large.blocks * small_expressions <= small.blocks * large_expressions,
            "a {cols}x{rows} rebuild took {} blocks over {large_expressions} Expressions, \
             against {} over {small_expressions} — faster than linear in what it carries",
            large.blocks,
            small.blocks
        );
        assert!(
            large.bytes * small_expressions <= small.bytes * large_expressions,
            "a {cols}x{rows} rebuild took {} bytes over {large_expressions} Expressions, \
             against {} over {small_expressions} — faster than linear in what it carries",
            large.bytes,
            small.bytes
        );
    }
}
