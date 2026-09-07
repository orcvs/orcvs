# 07 — Report what the Parser consumed

**What to build:** Make `Parser::analyze` report the Expression it built and how much Source it
consumed. An Expression has a fixed width, set by its root Function's arity, so Source after that
width is the next Expression's problem and not evidence that this one is wrong. `analyze` must stop
converting a `Complete` parse into `Invalid(UnexpectedTrailingContent)`.

This issue changes `lang` only. `LanguageMap::parse_span` restores the old verdict at the call site,
so the Language Map and the Tick behave exactly as they do today. Issue 08 removes that restoration.

**Blocked by:** None.

**Status:** resolved

**Tags:** release/v1

- [x] `SourceAnalysis` becomes a struct: `{ expression, status, consumed }`, where `AnalysisStatus`
      carries today's `Complete`, `Incomplete(Error)`, and `Invalid(Error)`.
- [x] `expression()`, `into_expression()`, and `error()` keep their signatures. `consumed()` and
      `is_complete()` join them.
- [x] `analyze` never constructs `SyntaxError::UnexpectedTrailingContent`.
- [x] `analyze` reports `Complete` for `.+0102Z`, with 6 consumed and `Z` unconsumed.
- [x] `consumed` is at least 1 for any non-empty input, asserted where it is established.
- [x] An unrecognized Function spelling reports 1 consumed, not 2, following ADR 0018.
- [x] A nested Function failure reports the same 1 consumed. State the rule once, so the
      `is_function_next` recursion (`lang/src/parser.rs:147`) inherits it.
- [x] `Incomplete` reports what was read, not what the layout claims. `.+01` reports 4 consumed while
      its layout claims 6.
- [x] `Parser::try_parse` keeps its strict contract and still fails on unconsumed Source.
- [x] `parse_span` re-applies the trailing-content diagnostic from `consumed`, so no `orcvs` test
      changes in this issue.

## The defect

The fixed-width parse is implemented and then discarded.

`Parser::take_language_unit` (`lang/src/parser.rs:118`) consumes exactly one Expression. It reads the
root Function, takes the arity from `Tokens::from(&f)`, takes exactly that many operands, and recurses
for a nested Function. What follows stays in `self.source`.

`Parser::analyze` (`lang/src/parser.rs:100`) then throws it away:

```rust
let mut status = self.take_language_unit()?;
if matches!(status, ParseStatus::Complete) && !self.source.is_empty() {
    let trailing = self.source.to_string();
    status = ParseStatus::Invalid(SyntaxError::UnexpectedTrailingContent(trailing).into());
}
```

A correct parse of `.+0102` with `Z` remaining becomes `Invalid`.

This contradicts ADR 0018, which landed 2026-08-31:

> The Language Map partitions each row left to right into non-overlapping complete Language Units. At
> each Position it recognizes a complete canonical encoding or reports that character as invalid, then
> resumes after the recognized Span or after the one invalid character. Thus `***` is Bang `**`
> followed by one invalid `*`.

`analyze` has the information the ADR's rule needs. It is the only place that knows where an
Expression ends, because it is the only place that knows the arity. It reports a verdict instead of a
boundary, so no caller can resume.

## Design

### The return shape

`SourceAnalysis` becomes a struct. The three current variants move into an `AnalysisStatus` enum held
by it, and `consumed: usize` joins them.

Two alternatives were considered. Returning the unconsumed remainder as a `&'a str` fails at the one
caller that matters: `analyze_span` (`orcvs/src/source/language_map.rs:450`) builds a local `String`
and returns the analysis, so a returned slice would borrow that local. Adding a `consumed` field to
each enum variant works, but `Complete(Expression)` must become a struct variant, every match arm
changes, and all three accessors repeat the match.

The struct is the smallest change because the accessors already hide the enum. One call site matches
it directly — `matches!(analysis, SourceAnalysis::Complete(_))` (`orcvs/src/source/language_map.rs:481`)
— and becomes `analysis.is_complete()`.

### Forward progress

Issue 08 makes the caller loop: parse, advance by `consumed`, parse again. `take_language_unit` can
consume nothing. When `next_token(2)` finds fewer than two characters left it returns `None` and
records `Incomplete(ExpectedFunction)` without reading anything (`lang/src/parser.rs:168`).

`***` is both ADR 0018's worked example and the case that hangs that loop. The first parse takes the
Bang and consumes 2. One `*` remains. The second parse consumes 0, and the loop never advances.

The Parser owns the invariant, so it can be asserted where it is established: `consumed` is at least
1 whenever the input is not empty. A one-character tail reports 1 and is diagnosed, as it is today.
Leaving the rule to callers would make issue 08's loop the only guard against an infinite loop, with
nothing stating why the guard is there.

### How far to advance after an unrecognized Function

ADR 0018 says to report one character as invalid and then resume. `walk_row` obeys this today: it
advances `column += 1` when `unit_kind` fails (`orcvs/src/source/language_map.rs:653`).

The Parser does not. `next_token(2)` consumes two characters before `Function::try_from` is tried, so
a failed Function consumes two (`lang/src/parser.rs:141`). Under issue 08 the Parser drives the walk,
and the difference misaligns everything after a bad character.

`Z.+0304` shows the cost. Advancing by one gives one invalid `Z`, and then `.+0304` parses at offset 1
and executes. Advancing by two tries `Z.` as a Function, fails, then tries `+0`, fails, and the
Function that is really there is never read. One mistyped character loses the whole row, and typing
one is an ordinary state to pass through.

So the error path at `lang/src/parser.rs:141` reports 1 while still recording one invalid Function
entry in the Expression. The record and the consumed length therefore disagree, deliberately: the
record describes what was attempted, and the length describes what to skip. Do not reconcile them.

## Regression history

`a8f544c` (2026-08-31, "Keep expressions bounded and diagnosable") added the first trailing-content
rejection, to `try_parse`. Before it, `try_parse` called `take_function()` and ignored anything after
the Expression, so `.+0102Z` evaluated as `.+0102`. Bounding the Expression was right. The commit
chose to reject the whole span where ADR 0018, dated the same day, says to resume after it.

`8e32100` (2026-09-02, "Separate source analysis from strict parsing") carried the same choice into
`analyze`, which is the path the Language Map uses. `ec59c35` and its successors then built Expression
derivation on top of it.

## Comments

Found while assessing an architecture review of the Tick scheduler. The review proposed to move the
scheduler's structural-stability rules behind the Language Map seam. That is the wrong target. Those
rules compensate for a boundary that no layer owns; issues 08 and 09 give it an owner and delete them.

The design walkthrough for issues 07 to 09, with diagrams of the current and proposed flows and of the
two cases above, is at https://claude.ai/code/artifact/ee649cf9-c4dd-46c3-a394-978176955494

Implemented. `SourceAnalysis` is now a struct over a private `AnalysisStatus`, and `analyze` reports
`consumed` alongside it. `analyze` no longer constructs `SyntaxError::UnexpectedTrailingContent`;
`Parser::try_parse` still does, which is now the whole difference between the two readings.

Two notes on the checklist:

The nested Function failure is structurally unreachable rather than merely untested.
`is_function_next` admits the recursion only for a spelling `Function::try_from` already accepted, so
the recursive `take_language_unit` cannot reach the `Err` arm that the one-Cell rule lives in. The
rule is still stated once, in that arm, so the recursion inherits it if the guard ever widens; the
test that covers it says so rather than asserting a case it cannot construct.

`consumed` is at least 1 for a non-empty Source because the `next_token(2)` miss now reads the tail
it declined and clears `self.source`, rather than leaving it for a caller that has no way to know it
is there. The invariant is a plain `assert!` in `analyze`: it runs once per Expression rather than
per Token, and issue 08's loop should not be the only thing standing between a stray `*` and a hang.

Review raised one defect and two smaller points, all fixed. The one-character skip was written as
`consumed -= 1`, which is one byte: `Parser::from` takes any `&mut str`, and `"é!"` would have
reported an offset inside the `é` for a caller to slice at. It now steps back by the length of the
first character, so `é` costs two and `Z` costs one. The non-ASCII test asserts every case lands on a
character boundary.

The invariant stays a release `assert!` rather than a `debug_assert!`. It is one branch per
Expression rather than per Token, and what it prevents is a running Tick hanging, which is not a
failure to leave to a debug build.

Review pass two found the same byte-vs-character mistake on the other path. When the second Cell
is only part of a character, `split_at_checked(2)` declines and the branch consumed the whole
remaining Source, so `€.+0304` reported the entire row and stepped over the Addition. It now consumes
one character there too, which is the skip the refused-spelling path already took. Both are pinned by
`source_that_is_not_ascii_is_refused_rather_than_panicking`, and each assertion was confirmed to fail
against the code it replaced.

Review pass three raised four more. Two were one defect: `consumed` was counted beside `self.source`
rather than derived from it, so the refused-Function arm rewound one and not the other, and the guard
in `analyze` fell back on the remainder — a proxy for the input it claims to be about. The Parser now
keeps the input's length and derives `consumed` from what is left, and the refused-Function arm
rewinds the Source itself. One record of a position cannot drift from a second that does not exist.
`SourceAnalysis::complete` checks its lower bound in release, as `analyze` does; a zero admitted
there hangs the same loop.

The fourth is real and stays: `standalone_run` reports the whole Span, so `**^^` is one four-Cell
Expression to the Language Map and a two-Cell Bang to the Parser. That disagreement is why the path
exists — a run of standalone Atoms is the shape the Parser cannot take whole — and reporting the
Parser's answer instead would diagnose `^^` as trailing content and change what a Tick plays, which
this issue forbids. Issue 08 deletes the path rather than reconciling the two. Recorded at the site.
