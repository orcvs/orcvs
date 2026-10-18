# 03: The Expression owns its Comment's text

**What to build:** The Parser records a Comment and its text through one
`Expression::add_comment`, so the `Token::Comment` entry and the text cannot
disagree, and `Expression::comment` answers the text as `lang::CommentText`.
`SourceAnalysis` carries no Comment text of its own, and the Language Map
builds a Comment unit from `Expression::comment` with no `expect`. See
`../spec.md` and ADR 0074.

**Blocked by:** 02

**Status:** resolved

- [x] `Expression::add_comment` records the Comment's Token and its text, and
      `Expression::comment` answers `Option<CommentText>`.
- [x] `SourceAnalysis::comment_text` and the Parser's and `SourceAnalysis`'s
      `comment_text` fields are gone.
- [x] `lang` owns `CommentText` with a public `range`; `LanguageUnitKind::Comment`
      carries it, and `orcvs` defines and re-exports no `CommentText` of its own.
- [x] The Language Map establishes a Comment unit exactly when
      `Expression::comment` answers, without an `expect`.
- [x] `Parser::try_parse` asks `Expression::comment` whether it holds a
      Comment, and keeps its debug assertion.
- [x] The Parser tests and the analysis property assert the text through
      `Expression::comment`.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-18: A refactor with no behaviour change. The `Token::Comment` entry
stays on the Expression for paint and `token_at`.
