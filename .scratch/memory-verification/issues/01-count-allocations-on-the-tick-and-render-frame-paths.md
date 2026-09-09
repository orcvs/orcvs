# 01 — Count allocations on the Tick and Render Frame paths

**What to build:** A Tick over an already-parsed Source, and a Render Frame re-reading it, are
checkable for how much they allocate, so an accidental allocation on either path fails a pull
request instead of hiding behind a cache hit in the criterion series.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A counting global allocator forwards every method to `System` and counts blocks and bytes,
      declared inside a `lang` integration-test binary so nothing shipped and no other test target
      can see it. No feature gate.
- [x] Counting is thread-local with `const`-initialised storage, so the counter cannot allocate from
      inside the allocation it is counting, and the tests stay correct under a bare `cargo test`.
- [x] `realloc` and `alloc_zeroed` are left to `GlobalAlloc`'s defaults, which route through this
      allocator's `alloc` and `dealloc` and are therefore counted, and the choice is stated.
- [x] The `unsafe impl` carries a SAFETY comment naming what makes the forwarding sound, and clears
      the workspace's `undocumented_unsafe_blocks` and `unsafe_op_in_unsafe_fn` denials.
- [x] The test binary is gated to native targets the way the property suites already are.
- [x] A Tick over an already-parsed Source is asserted against a shape, not a number: zero, or
      independent of the number of rows.
- [x] Parsing a Source is asserted to be independent of how many of its rows are empty.
- [x] The contract's dependency rationale is recorded, as the reasoning for taking no dependency:
      `dhat` 0.3.3 is from February 2024, ships a maintenance disclaimer, and pulls eight crates;
      `allocation-counter` has had no release since September 2023.
- [x] The tests run inside the existing `cargo nextest run --workspace --profile ci --locked` line.
      No mise task, no workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

This is the tracer bullet: the mechanism, the first real assertions, and the gate, in one change.
`02` and `03` inherit the allocator and the assertion convention from here, so the shape this
establishes matters more than the number of assertions it lands with.

If a path that should not allocate turns out to allocate, that is the finding. Record what it does
and why here before touching the assertion.

### Finding: neither path is allocation-free (2026-09-09)

Both assertions were written at "zero" first and neither held. What follows is what actually
allocates, measured with the counter in `lang/tests/allocation.rs` against the `SOURCE` fixture the
`lang` benchmarks use, on a `dev` build.

**A Tick allocates exactly one block per Expression, sized to that Expression's Atom count.**

```
tick over 11 Expressions   -> 11 blocks,   816 bytes
tick over 44 Expressions   -> 44 blocks,  3264 bytes
execute [Bang]                            -> 1 block,  24 bytes
execute [Add, 1, 2]                       -> 1 block,  72 bytes
execute [Add, Add, 1, 1, Subtract, 10, 5] -> 1 block, 168 bytes
```

`Interpreter::execute` calls `Context::new(inputs, atoms.len())`, which calls `Stack::new(limit)`,
which is `Vec::with_capacity(limit)` over a 24-byte `Value`. One heap block per Expression on the
path a Tick runs on a musical clock. A one-Atom Expression — a bare Bang — still pays for it.

This is the same shape the Parser's pending stack had before `05e4490` ("Avoid heap allocation for
common parser pending stacks") gave it an `ArrayVec` with heap overflow. The evaluation stack is a
candidate for exactly that treatment, and that is not this ticket's scope; the assertion is written
so that fixing it does **not** break the test.

Asserted instead of zero, in
`a_tick_over_an_already_parsed_source_allocates_per_expression_and_not_per_row`:

- `blocks <= expressions.len()` — a ceiling, so an inline evaluation stack driving this to zero
  reads as an improvement rather than a failure, while a second allocation per Expression fails.
- four times the Expressions cost exactly four times the blocks and the bytes — linear with a zero
  intercept, so nothing is allocated per Tick and nothing grows with the square of the Source.
- padding the Source with 512 empty rows changes nothing, because an empty row holds no
  Expression. That is the "independent of the number of rows" half, and it is true.

**A Render Frame re-read allocates one throwaway `String` per literal operand.**

```
analyze ".+0102"          -> 2 blocks, 4 bytes
analyze "!>010AC4"        -> 3 blocks, 6 bytes
analyze ".+.+0101.-0A05"  -> 4 blocks, 8 bytes
analyze ".+01XY"          -> 3 blocks, 6 bytes
analyze "**"              -> 0 blocks, 0 bytes
analyze ">>"              -> 0 blocks, 0 bytes
analyze ""                -> 0 blocks, 0 bytes
```

Two bytes per allocation is the tell: it is the operand's two Cells, copied to the heap and thrown
away. `Parser::take_language_unit` asks `self.is_function_next()` before reading each literal
operand; `parser::is_function` answers it with `Function::try_from(t).is_ok()`; and the refusal arm
of that conversion is `Err(SyntaxError::UnknownFunction(spelling.to_string()))` in `atom.rs`. The
`String` exists only so that `is_ok()` can discard it. A malformed operand pays twice — once for
the peek and once for `TypeError::Number(s.to_string())` in `str_to_num`.

So the Render Frame path allocates once per literal operand Cell pair, many times a second, to
build error text nobody reads. A borrowing or `&'static str` refusal in `Function::try_from`, or a
peek that does not go through `try_from` at all, would take it to zero. Out of scope here: the fix
is in `lang/src`, and this ticket is the measurement.

Asserted instead of zero, in
`re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty`:

- re-reading the fixture padded with 4 empty rows and with 512 empty rows allocates identically
  (24 blocks / 48 bytes both ways) — the empty rows really are free.
- a Source of nothing but 512 empty rows allocates zero, so the equality above is independence
  rather than two equally wasteful passes.

### Convention `02` and `03` inherit

- `Allocations { blocks, bytes }`, `snapshot()`, and `measure(f) -> (Allocations, T)`. `measure`
  hands the value back rather than dropping it inside, so deallocation stays outside the span.
- `black_box` around the inputs and the result of every measured closure, the way the benchmarks
  do, and a warm-up call before the first measured span so a one-off initialisation cannot show up
  as an inequality between two spans.
- Three assertion shapes, in order of preference: **zero**; **a ceiling** (`<= n` derived from the
  input, so a cheaper implementation still passes); **an equality between two measurements** of the
  same work at different input sizes. No absolute number appears in an assertion.
- Every relaxation from zero carries a `FINDING (date):` comment at the top of the test naming what
  allocates and where, and pointing at this file.

`02` decides for itself whether to duplicate the allocator into `orcvs/tests/allocation.rs` or
share it; nothing was added to `lang`'s public API for it, and a test-support module exported from
`lang` for the benefit of `orcvs`'s tests would compile into a dependency for a test's sake.
