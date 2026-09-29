# 02 — Test a Function spelling without building an error

**What to build:** Asking whether two Cells spell a Function answers without allocating, so deriving a
Language Map row stops building error text that nothing reads.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Parsing a Source row allocates nothing for the Function-spelling predicate per literal
      operand. The path is
      `Parser::take_language_unit` -> `Parser::is_function_next` -> `parser::is_function` ->
      `Function::try_from(t).is_ok()`, whose refusal arm builds
      `SyntaxError::UnknownFunction(spelling.to_string())` in `lang/src/atom.rs` only for `is_ok()`
      to discard it.
- [x] The predicate answers without constructing an `Error`. A borrowing test — matching the
      spelling directly, or a `Function::matches`-shaped predicate the `TryFrom` then reuses — is
      the shape; which one is the ticket's design decision and is recorded here.
- [x] No numeric conversion error is built merely to test an operand. `take_token` calls
      `Token::decode` once; the former numeric peek is absent. A genuine numeric failure still
      constructs its reported `TypeError::Number` (verified 2026-09-29).
- [x] The error a genuine syntax failure reports is unchanged: same variant, same text, same
      Diagnostic. This ticket removes an allocation on the path that discards the error, not the
      error.
- [x] `re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty` in
      `lang/tests/allocation.rs` is strengthened to zero once it is true, and the `FINDING:`
      comment above it, which cites this ticket, is replaced with a present-tense statement of the
      invariant the test then asserts.
- [x] Every existing `lang` test and property passes unchanged, including the parser suites and the
      `Function` conversion tests.
- [x] `parse_source`, `parse` and `parse_invalid` are the benchmarks that cover this path. Note them
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

### Independent implementation audit — 2026-09-29

The Function predicate still allocates on refusal (`lang/src/parser.rs:333-339`,
`lang/src/atom.rs:793`). Numeric conversion is now one call from `take_token`
(`parser.rs:289-299`) through `Token::decode` (`lang/src/expression.rs:145`);
`str_to_num` has no production predicate caller. The numeric double-peek criterion is met.
The historical per-frame explanation above is obsolete: `RenderFrame::derive` reads the
revision's cached Language Map (`orcvs/src/render_frame.rs:135-173`), while row derivation
parses source (`orcvs/src/source/language_map.rs:766`). The remaining allocation belongs to
parsing/rebuilding changed rows and direct parser callers. The allocation test is a direct
parser fixture despite its render-frame wording. Status remains open.

### Implementation — 2026-09-29

**Design decision: a `Function::from_spelling`-shaped predicate, not a direct match at the call
site.** `define_functions!` now generates `pub(crate) fn Function::from_spelling(&str) ->
Option<Function>`, the one match over every spelling, and `TryFrom<&str>` answers through it,
building `SyntaxError::UnknownFunction(spelling.to_string())` only in its refusal arm. The
parser's `is_function` asks `from_spelling(..).is_some()`. A direct match in the parser would
have needed a second list of spellings (or a second macro expansion) that could drift from the
definitions; routing both through one generated table keeps a single source of truth, and the
`#[deny(unreachable_patterns)]` guard against two definitions sharing a spelling moves onto that
table. It is `pub(crate)`, so `lang`'s public API is unchanged. `functions::jump::copied_atom`
discarded the same error through `try_from(..).is_ok()`-shaped code and now uses the predicate
too.

The reported error is unchanged: `Parser::take_language_unit`'s Function slot still calls
`Function::try_from` and reports the same `SyntaxError::UnknownFunction` text and Diagnostic;
`retired_arithmetic_spellings_do_not_parse_as_functions` now also pins the `TryFrom` refusal and
the predicate's `None`, and the canonical-spelling test pins the predicate's `Some`.

**The allocation test, and why zero is per row.** Before the change the fixture measured 24
blocks / 48 bytes; after it, 1 block / 2 bytes. The one remaining block is not a discarded error:
it is the `TypeError::Number("XY")` that `.+01XY` genuinely reports, which the fourth criterion
keeps. So `re_reading_a_source_is_independent_of_how_many_of_its_rows_are_empty` now asserts zero
for every fixture row whose analysis reports no error (the row that failed first before the
change was `.+0102`, 2 blocks / 4 bytes), and that the whole fixture, padded with 4 or 512 empty
rows, costs exactly what its error-reporting rows cost read alone. The `FINDING:` comment is
replaced by the present-tense invariant. The `lang render frame re-read fixture` series keeps
publishing and moves from 24/48 to 1/2; the benchmark action treats a zero against a stored zero
as a ratio of 1, so a later zero point would not be undefined either.

Benchmarks covering the path, for the workflow's comparison: `parse_source`, `parse`,
`parse_invalid`. Not run locally.
