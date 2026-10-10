//!
//! Allocation shapes for the two paths `lang` sits on: a Turn evaluates one
//! Function over resolved operands, and a Render Frame re-reads the Source
//! many times a second. Both run often enough that an allocation added to either is a
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
//! `orcvs/tests/allocation.rs` and `orcvs/src/lib.rs`'s `allocation` module
//! carry the same counting allocator, so a change to it belongs in all three.
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
//! # Publishing the numbers, not only asserting on them
//!
//! `.github/workflows/bench.yml` stores these measurements as a series on
//! `gh-pages` beside the criterion timings, so a creep too small for any one
//! assertion to catch is visible as a trend.
//!
//! The series is fed by the tests below rather than by a binary of its own. A
//! separate binary re-running the measured paths would let the published
//! number and the asserted number drift apart silently, which is the one thing
//! the series exists to prevent. So each test prints what it just measured and
//! then asserts over the same numbers, and a failed assertion fails the run
//! before anything is published.
//!
//! Printing is gated on `ORCVS_MEMORY_SERIES=1`, so an ordinary
//! `cargo nextest run --workspace` is silent and the assertions are the same
//! either way.
//!
//! The workflow collects the records with a bare `cargo test`, because the
//! benchmark job runs `mise-action` with `install: false` and so has no
//! `cargo-nextest`. That works for the reason the thread-local counters were
//! chosen: under `cargo test` every test in this binary runs on its own thread
//! of one shared process, which is exactly the case a `const`-initialised
//! thread-local is correct for.
//!
// Native-only, the way the property suites are: `System` and this counting are
// native concerns, and `--all-targets` on `wasm32-unknown-unknown` compiles
// this target too. The `cfg` matches the shape of the
// `[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` table in
// `lang/Cargo.toml`, so a WASM build sees an empty test binary rather than a
// global allocator it has no `System` for.
#![cfg(not(target_arch = "wasm32"))]

use lang::{
    Anchor, Atom, Error, Function, FunctionInputs, InterpretationError, Interpreter, Parser,
    PortalSource, Tick, TickInputs,
};
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
/// after `measure` has already answered.
fn publishing() -> bool {
    std::env::var("ORCVS_MEMORY_SERIES").is_ok_and(|value| value == "1")
}

/// Both halves of one measurement, as `<name> blocks` and `<name> bytes`.
///
/// One `customSmallerIsBetter` object per line — the shape
/// `benchmark-action/github-action-benchmark` reads. `name` is a literal here,
/// so it holds no character JSON would need escaped and no escaping is
/// written. A name that needed escaping would be a name that had changed, and
/// a changed name starts an empty series rather than continuing this one.
fn publish(name: &str, allocations: Allocations) {
    if !publishing() {
        return;
    }

    for (unit, value) in [("blocks", allocations.blocks), ("bytes", allocations.bytes)] {
        println!(
            r#"{SERIES_MARKER} {{"name": "{name} {unit}", "unit": "{unit}", "value": {value}}}"#
        );
    }
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

/// One call a Turn makes: a Function and the operands it resolved.
type Call = (Function, Vec<Atom>);

/// The calls a Source's Expressions make once their operands are resolved.
/// Rows that hold no Expression, and rows the Parser refuses, contribute
/// nothing, which is the point the assertions rest on. An Expression that
/// nests a Function is resolved by `orcvs` one call at a time, so only the
/// Expressions over Operand Literals are a call as written.
fn calls(rows: &[String]) -> Vec<Call> {
    rows.iter()
        .filter_map(|row| Parser::at(row, 0).try_parse().ok())
        .filter_map(|atoms| {
            let Some((Atom::Function(function), literals)) = atoms.split_first() else {
                return None;
            };
            let operands = literals
                .iter()
                .map(|literal| match literal {
                    Atom::Function(_) => None,
                    literal => Some(*literal),
                })
                .collect::<Option<Vec<_>>>()?;
            Some((*function, operands))
        })
        .collect()
}

/// Every call of an already-parsed Source, evaluated once through the
/// Interpreter entry point a Turn uses.
fn evaluate(calls: &[Call], inputs: TickInputs) -> usize {
    let mut evaluated = 0;
    for (function, operands) in calls {
        evaluated += usize::from(
            Interpreter::execute_function(
                black_box(*function),
                black_box(operands).iter().copied(),
                black_box(inputs).into(),
            )
            .is_ok(),
        );
    }
    evaluated
}

/// One Render Frame re-read: `analyze` per row, the permissive path a Source
/// mid-edit is read through.
fn reread(rows: &[String]) -> usize {
    let mut units = 0;
    for row in rows {
        units += Parser::at(black_box(row.as_str()), 0)
            .analyze()
            .expression()
            .len();
    }
    units
}

#[test]
fn evaluating_a_parsed_source_allocates_nothing_per_call_or_per_row() {
    let inputs = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));

    // A call over Atom operands allocates nothing. `Interpreter::execute_function`
    // holds its Operand Stack inline, sized to the widest operand list the
    // Function table declares, and every answer is one Atom or one Play
    // Command, so neither the stack nor the answer asks the allocator for a
    // block. Zero, for one pass and for four, rather than a ceiling per call:
    // a ceiling of one block per call would admit exactly the heap-backed
    // stack this rules out.
    let short = rows(SOURCE, 0);
    let short = calls(&short);
    assert!(!short.is_empty(), "the fixture must hold calls");

    // The same Source written out four times over: four times the Expressions,
    // in a Grid four times as tall.
    let mut repeated: Vec<&str> = Vec::new();
    for _ in 0..4 {
        repeated.extend_from_slice(SOURCE);
    }
    let long = rows(&repeated, 0);
    let long = calls(&long);
    assert_eq!(long.len(), short.len() * 4);

    // Warm up so a one-off initialisation on the first pass lands outside
    // every span below, where it would otherwise show up as an inequality.
    black_box(evaluate(&short, inputs));

    let (one, evaluated) = measure(|| evaluate(black_box(&short), inputs));
    black_box(evaluated);
    let (four, evaluated) = measure(|| evaluate(black_box(&long), inputs));
    black_box(evaluated);

    // Neither point is published: both are asserted to be zero, and a count
    // other than zero fails the measurement before anything is published, so a
    // published point could only ever be zero and would carry no trend.
    assert_eq!(
        one,
        Allocations::default(),
        "{} calls allocated",
        short.len()
    );
    assert_eq!(
        four,
        Allocations::default(),
        "{} calls allocated",
        long.len()
    );

    // Empty rows hold no Expression, so a taller Grid over the same writing
    // leaves exactly the calls it already had.
    //
    // Asserted over the calls, not over a fourth measurement. An empty row is
    // one the Parser refuses, so `calls` drops all 512 of them and a padded
    // Source yields a `Vec` element-wise identical to `short`. Measuring over
    // it would compare two runs over the very same input and could not fail
    // for any implementation of `Interpreter::execute_function` — it would
    // read as a fourth assertion while asserting nothing. The independence from empty rows that *is* worth
    // stating is this one, and it is the Source's property rather than the
    // Interpreter's. `re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty`
    // below states the Interpreter-side half, where `reread` does iterate the
    // padding and the equality has something to catch.
    let padded = rows(SOURCE, 512);
    let padded = calls(&padded);
    assert_eq!(
        padded, short,
        "512 empty rows changed the calls a Source makes"
    );
}

#[test]
fn re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty() {
    // A row whose analysis reports no error allocates nothing to read. The
    // Parser's pending stack is inline, and asking whether two Cells spell a
    // Function borrows them rather than building the
    // `SyntaxError::UnknownFunction` a refusal reports, so a literal operand
    // costs nothing to tell from a Function.
    //
    // Measured row by row, so no row's cost can hide inside another's: a
    // total over the fixture would let a clean row allocate what an
    // error-reporting row happens not to.
    //
    // A row that reports an error allocates at most the one error it reports,
    // and that error owns at most a copy of text written in the row, such as
    // the spelling a malformed operand's `TypeError::Number` carries. The
    // ceiling is derived from the row rather than from the measurement, so a
    // cheaper error still passes, and any allocation the Parser makes and
    // then discards on the way to the error exceeds it.
    let mut clean = 0;
    for row in SOURCE {
        // Warm up, for the same reason the call test does.
        black_box(Parser::at(row, 0).analyze());
        let (allocations, analysis) = measure(|| Parser::at(black_box(row), 0).analyze());
        if analysis.error().is_none() {
            clean += 1;
            assert_eq!(
                allocations,
                Allocations::default(),
                "reading {row:?}, which reports no error, allocated"
            );
        } else {
            assert!(
                allocations.blocks <= 1 && allocations.bytes <= row.len(),
                "reading {row:?} allocated {allocations:?}, more than the one error it reports"
            );
        }
    }
    assert!(clean > 0, "the fixture must hold rows that read cleanly");

    // A row that reports an error owns what it reports: a malformed operand's
    // `TypeError::Number` carries the operand's spelling. So the whole fixture
    // costs exactly what its error-reporting rows cost read alone, and empty
    // rows add nothing to that. A Source is as tall as the Grid, most of it
    // empty most of the time, and re-reading it costs only what is written in
    // it.
    let reported: Vec<String> = SOURCE
        .iter()
        .filter(|row| Parser::at(row, 0).analyze().error().is_some())
        .map(|row| row.to_string())
        .collect();
    assert!(
        !reported.is_empty(),
        "the fixture must hold a row that reports an error"
    );
    let few = rows(SOURCE, 4);
    let many = rows(SOURCE, 512);

    black_box(reread(&few));

    let (sparse, units) = measure(|| reread(black_box(&few)));
    black_box(units);
    let (empty, units) = measure(|| reread(black_box(&many)));
    black_box(units);
    let (errors, units) = measure(|| reread(black_box(&reported)));
    black_box(units);

    // One point of the measurements here: the others are asserted equal to it.
    publish("lang render frame re-read fixture", sparse);

    assert_eq!(
        sparse,
        errors,
        "re-reading the fixture allocated more than its {} error-reporting rows",
        reported.len()
    );
    assert_eq!(
        sparse,
        empty,
        "re-reading {} rows and {} rows of the same writing allocated differently",
        few.len(),
        many.len()
    );
}

/// One Turn through [`Interpreter::execute_function`], and what it allocated.
///
/// The answer is dropped outside the span, so only what the Turn asked for is
/// counted; the operands are built by the caller, outside it too.
fn turn<const N: usize>(function: Function, operands: [Atom; N]) -> Allocations {
    let inputs = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));
    // Warm up, for the reason the call test gives.
    black_box(Interpreter::execute_function(
        function,
        operands,
        inputs.into(),
    ))
    .expect("the warm-up Turn answers");
    let (allocations, answer) = measure(|| {
        Interpreter::execute_function(black_box(function), black_box(operands), inputs.into())
    });
    answer.expect("the measured Turn answers");
    allocations
}

#[test]
fn a_turn_over_atoms_allocates_nothing() {
    // The Operand Stack is inline and the answer is one Atom, so the Turn asks
    // the allocator for nothing. Asserted rather than published, for the
    // reason the call test gives.
    let add = turn(Function::Add, [Atom::Number(1), Atom::Number(2)]);

    assert_eq!(add, Allocations::default(), "`.+0102` allocated");
}

#[test]
fn a_copy_allocates_nothing_to_copy_or_refuse_its_portal_cells() {
    // A Copy reads the two Cells at its input Portal as a Function, a Number
    // or a Note, and refuses any other spelling with a `CopyInput` that owns
    // no text. Asking which of the three the Cells spell borrows them, so a
    // Note or an unreadable spelling costs nothing on the way through the
    // readings it is not, and the refusal a Turn reports is itself no block.
    //
    // `G4` is the Note: a Note spelled in hexadecimal digits, such as `C4`,
    // reads as a Number first and never reaches the Note reading.
    let tick = TickInputs::new(Tick::ZERO, Anchor::new(0, 0));
    let copy = |cells: &str| {
        Interpreter::execute_function(
            black_box(Function::CopyEast),
            [],
            FunctionInputs::with_portal_source(tick, PortalSource::from_cells(Some(cells))),
        )
    };

    for cells in [".+", "0A", "G4", "xx"] {
        // Warm up, for the reason the call test gives.
        let _ = black_box(copy(cells));
        let (allocations, answer) = measure(|| copy(black_box(cells)));
        if cells == "xx" {
            assert!(
                matches!(
                    answer,
                    Err(Error::Interpretation(InterpretationError::CopyInput {
                        function: Function::CopyEast
                    }))
                ),
                "{cells:?} was copied: {answer:?}"
            );
        } else {
            answer.expect("the Copy copies its Portal Cells");
        }
        assert_eq!(
            allocations,
            Allocations::default(),
            "a Copy over {cells:?} allocated"
        );
    }
}
