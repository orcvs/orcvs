//!
//! Allocation shapes for the two paths `lang` sits on: a Tick interprets an
//! already-parsed Source, and a Render Frame re-reads the Source many times a
//! second. Both run often enough that an allocation added to either is a
//! regression the criterion series would hide behind a cache hit.
//!
//! # Why there is no dependency here
//!
//! The counting allocator below is written rather than taken from a crate, and
//! that absence is the dependency decision the repository contract asks to be
//! recorded.
//!
//! `dhat` is the standard choice and its testing mode is the right shape, but
//! 0.3.3 dates from February 2024, it carries an explicit maintenance
//! disclaimer from its own author, and it pulls eight crates into the graph
//! `cargo deny` runs over. `allocation-counter` has exactly the right API and
//! no runtime dependencies, but nothing has been released since September
//! 2023. The mechanism both implement is a `GlobalAlloc` that forwards to
//! `System` and increments a counter — the forty lines below.
//!
//! So the trade taken is one small `unsafe impl` whose whole invariant is
//! "forwards to `System` unchanged", instead of a stale dependency. The
//! contract prefers safe Rust; this is the narrowest unsafe scope that buys
//! anything, and it lives in a test binary that nothing shipped links.
//!
//! `dhat` remains the right tool for *diagnosing* a failure here, because its
//! per-callsite attribution says which allocation appeared. Add it to a branch
//! for that and do not commit it — counting says an assertion broke, and
//! attribution says why.
//!
//! # What the assertions say
//!
//! Shapes, never absolute numbers. An absolute count rots on a compiler
//! release or a dependency bump, and the fix is always to edit the number,
//! which teaches everyone to edit the number. Each assertion here is either
//! zero, a ceiling that stays true if the path gets cheaper, or an equality
//! between two measurements of the same work at different input sizes.
//!
//! A failing assertion is a finding. Record what allocates and why in
//! `.scratch/memory-verification/issues/` before relaxing one.
//!
//! # Where the allocator lives
//!
//! An integration test is its own binary, so the `#[global_allocator]` here
//! reaches this file and nothing else — not the shipped Console, not another
//! test target. That is why no feature gate is needed and why the contract's
//! feature combinations are untouched.
//!
// Native-only, the way the property suites are: `System` and this counting are
// native concerns, and `--all-targets` on `wasm32-unknown-unknown` compiles
// this target too. The `cfg` matches the shape of the
// `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` table in
// `lang/Cargo.toml`, so a WASM build sees an empty test binary rather than a
// global allocator it has no `System` for.
#![cfg(not(target_arch = "wasm32"))]

use lang::{Anchor, Atom, Interpreter, Parser, Tick, TickInputs};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;

thread_local! {
    // `const` initialisation is what makes these safe to touch from inside the
    // allocator: a lazily initialised thread-local allocates on first access,
    // which would be an allocation from within the allocation it is counting.
    // `Cell<usize>` has no drop glue, so no destructor is registered either.
    //
    // Thread-local rather than atomic, following `allocation-counter`'s
    // design. Everything measured in this file runs on the calling thread, so
    // the counters need no synchronisation and the tests stay correct under a
    // bare `cargo test`, where tests share a process. Work measured across a
    // multi-threaded runtime would need `AtomicUsize` instead and would then
    // be correct only under nextest's process-per-test isolation; a test in
    // that position has to say so.
    static BLOCKS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

/// `System`, plus a count of the blocks and bytes handed out.
///
/// `realloc` and `alloc_zeroed` are deliberately not overridden.
/// `GlobalAlloc`'s defaults implement both in terms of `Self::alloc` and
/// `Self::dealloc`, so they route back through the two methods below and are
/// counted without the counting being written twice. Overriding them to reach
/// `System`'s own `mremap` and calloc paths would be faster and would put the
/// counter in four places; a test binary does not need the faster path.
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

/// One Source revision, row by row, as the `lang` benchmarks fix it: complete
/// Expressions, a Bang, an Activation Character, malformed text, and empty
/// rows. Shared with `lang/benches/lang.rs` by duplication, because a bench
/// target and a test target cannot import one another.
const SOURCE: &[&str] = &[
    ".+0102",
    ".x0201",
    "",
    "!>010AC4",
    ".-0A05",
    "**",
    "./0402",
    ">>",
    "",
    ".+.+0101.-0A05",
    ".+01XY",
    "",
    ".x0F10",
    ".-0201",
    "./0100",
    "",
];

/// A Source's rows, padded with `empty_rows` further empty ones. A Grid is as
/// tall as it is whatever the writer has filled in, so the padded rows are the
/// ordinary case rather than a contrived one.
fn rows(source: &[&str], empty_rows: usize) -> Vec<String> {
    let mut rows: Vec<String> = source.iter().map(|row| row.to_string()).collect();
    rows.extend(std::iter::repeat_n(String::new(), empty_rows));
    rows
}

/// The Expressions a Source holds, already parsed — what a Tick runs over.
/// Rows that hold no Expression, and rows the Parser refuses, contribute
/// nothing, which is the point the Tick assertions rest on.
fn expressions(rows: &[String]) -> Vec<Vec<Atom>> {
    rows.iter()
        .filter_map(|row| {
            let mut row = row.clone();
            Parser::from(row.as_mut_str()).try_parse().ok()
        })
        .collect()
}

/// One Tick: every Expression of an already-parsed Source, interpreted once.
fn tick(expressions: &[Vec<Atom>], inputs: TickInputs) -> usize {
    let mut interpreted = 0;
    for atoms in expressions {
        interpreted +=
            usize::from(Interpreter::execute(black_box(atoms), black_box(inputs)).is_ok());
    }
    interpreted
}

/// One Render Frame re-read: `analyze` per row, the permissive path a Source
/// mid-edit is read through.
fn reread(rows: &mut [String]) -> usize {
    let mut units = 0;
    for row in rows.iter_mut() {
        units += Parser::from(black_box(row.as_mut_str()))
            .analyze()
            .expression()
            .len();
    }
    units
}

#[test]
fn a_tick_over_an_already_parsed_source_allocates_per_expression_and_not_per_row() {
    let inputs = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));

    // FINDING (2026-09-09): a Tick is not allocation-free. `Interpreter::execute`
    // builds its evaluation stack through `Stack::new(atoms.len())`, which is a
    // `Vec::with_capacity`, so every Expression costs exactly one block sized
    // to its Atom count. That is the shape asserted below: a ceiling of one
    // block per Expression, nothing per row, and no super-linear term. The
    // ceiling is `<=` rather than `==` on purpose — giving the evaluation stack
    // inline storage the way the Parser's pending stack already has would drive
    // this to zero, and an improvement must not read as a failure.
    let short = rows(SOURCE, 0);
    let short = expressions(&short);
    assert!(!short.is_empty(), "the fixture must hold Expressions");

    // The same Source written out four times over: four times the Expressions,
    // in a Grid four times as tall.
    let mut repeated: Vec<&str> = Vec::new();
    for _ in 0..4 {
        repeated.extend_from_slice(SOURCE);
    }
    let long = rows(&repeated, 0);
    let long = expressions(&long);
    assert_eq!(long.len(), short.len() * 4);

    // Warm up so a one-off initialisation on the first Tick lands outside
    // every span below, where it would otherwise show up as an inequality.
    black_box(tick(&short, inputs));

    let (one, interpreted) = measure(|| tick(black_box(&short), inputs));
    black_box(interpreted);
    let (four, interpreted) = measure(|| tick(black_box(&long), inputs));
    black_box(interpreted);

    // At most one block per Expression, and never one per Atom.
    assert!(
        one.blocks <= short.len(),
        "a Tick over {} Expressions took {} blocks",
        short.len(),
        one.blocks
    );

    // Linear with a zero intercept: four times the Expressions cost four times
    // as much, so nothing is allocated per Tick and nothing grows with the
    // square of the Source.
    assert_eq!(
        four.blocks,
        one.blocks * 4,
        "a Tick over {} Expressions took {} blocks, over {} it took {}",
        short.len(),
        one.blocks,
        long.len(),
        four.blocks
    );
    assert_eq!(four.bytes, one.bytes * 4);

    // Empty rows hold no Expression, so a taller Grid over the same writing
    // costs a Tick nothing at all.
    let padded = rows(SOURCE, 512);
    let padded = expressions(&padded);
    let (with_empty_rows, interpreted) = measure(|| tick(black_box(&padded), inputs));
    black_box(interpreted);
    assert_eq!(with_empty_rows, one);
}

#[test]
fn re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty() {
    // FINDING (2026-09-09): a Render Frame re-read is not allocation-free
    // either, and the reason is a discarded error rather than any parser
    // state. `Parser::is_function_next` calls `is_function`, which asks
    // `Function::try_from(spelling).is_ok()`; the refusal path builds
    // `SyntaxError::UnknownFunction(spelling.to_string())`, heap-allocating
    // the two Cells it was handed only for `is_ok()` to throw the error away.
    // That is one small allocation per literal operand read, on the path a
    // Render Frame runs many times a second. It is recorded in
    // `.scratch/memory-verification/issues/01-count-allocations-on-the-tick-and-render-frame-paths.md`,
    // and it is why this test asserts independence from the empty rows rather
    // than zero.
    //
    // What is asserted is what the Grid actually varies: a Source is as tall
    // as the Grid, most of it empty most of the time, and re-reading it must
    // cost only what is written in it.
    let mut few = rows(SOURCE, 4);
    let mut many = rows(SOURCE, 512);

    // Warm up, for the same reason the Tick test does.
    black_box(reread(&mut few));

    let (sparse, units) = measure(|| reread(black_box(&mut few)));
    black_box(units);
    let (empty, units) = measure(|| reread(black_box(&mut many)));
    black_box(units);

    assert_eq!(
        sparse,
        empty,
        "re-reading {} rows and {} rows of the same writing allocated differently",
        few.len(),
        many.len()
    );

    // The stronger half of the same statement: a row with nothing written in
    // it costs nothing, so the equality above is independence rather than two
    // equally wasteful passes.
    let mut blank = rows(&[], 512);
    let (nothing, units) = measure(|| reread(black_box(&mut blank)));
    black_box(units);
    assert_eq!(nothing, Allocations::default());
}
