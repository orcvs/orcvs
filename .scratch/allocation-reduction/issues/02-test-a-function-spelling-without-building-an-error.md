# 02 — Test a Function spelling without building an error

**What to build:** Asking whether two Cells spell a Function answers without allocating, so a Render
Frame stops building error text that nothing reads.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Reading a Source revision allocates nothing per literal operand. The path is
      `Parser::take_language_unit` -> `Parser::is_function_next` -> `parser::is_function` ->
      `Function::try_from(t).is_ok()`, whose refusal arm builds
      `SyntaxError::UnknownFunction(spelling.to_string())` in `lang/src/atom.rs` only for `is_ok()`
      to discard it.
- [ ] The predicate answers without constructing an `Error`. A borrowing test — matching the
      spelling directly, or a `Function::matches`-shaped predicate the `TryFrom` then reuses — is
      the shape; which one is the ticket's design decision and is recorded here.
- [ ] The same treatment is applied to `str_to_num`'s `TypeError::Number(s.to_string())` where it is
      reached as a test rather than as a reported failure. A malformed operand pays this twice
      today, once peeking and once converting.
- [ ] The error a genuine syntax failure reports is unchanged: same variant, same text, same
      Diagnostic. This ticket removes an allocation on the path that discards the error, not the
      error.
- [ ] `re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty` in
      `lang/tests/allocation.rs` is strengthened to zero once it is true, and its
      `FINDING (2026-09-09)` comment is replaced with a note saying what was corrected.
- [ ] Every existing `lang` test and property passes unchanged, including the parser suites and the
      `Function` conversion tests.
- [ ] `parse_source`, `parse` and `parse_invalid` are the benchmarks that cover this path. Note them
      for the comparison the benchmark workflow runs; do not run the comparison locally.

## Comments

What `memory-verification/01` measured, per row analysed:

```
".+0102"          -> 2 blocks, 4 bytes
"!>010AC4"        -> 3 blocks, 6 bytes
".+.+0101.-0A05"  -> 4 blocks, 8 bytes
".+01XY"          -> 3 blocks, 6 bytes
"**"              -> 0 blocks, 0 bytes
">>"              -> 0 blocks, 0 bytes
""                -> 0 blocks, 0 bytes
```

Two bytes per allocation is the tell: it is the operand's own two Cells, copied to the heap so that
`is_ok()` can throw them away. The rows that allocate nothing are the ones with no literal operand
to peek at, which is what identifies the call site.

This is the cheapest of the three findings to correct and the one on the most frequent path. A
Render Frame re-reads every row of the Source many times a second, and `analyze` is the permissive
path a Source mid-edit is read through — so this is paid for every operand of every row, on every
frame, including for Source that is malformed because someone is halfway through typing it.

The strengthening in the fifth box is the point of the ticket rather than an afterthought: this is
the one finding of the three where zero is reachable, and an assertion that says zero is worth more
than a ceiling that happens to be low.
