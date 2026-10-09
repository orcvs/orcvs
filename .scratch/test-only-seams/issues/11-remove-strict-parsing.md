# 11 — Remove strict parsing that only tests reach

Status: ready-for-agent
Blocked by: None

**What to build:**

`Parser::try_parse` (`lang/src/parser.rs`) has no shipped caller. It is reached only from `lang`'s parser tests, `interpret_source` (`lang/src/lib.rs`, `#[cfg(test)]`), the `parse` bench (`lang/benches/lang.rs`), `lang/tests/allocation.rs` and the `orcvs` properties. `SyntaxError::CommentIsNotAValue` and `SyntaxError::UnexpectedTrailingContent` are raised only by `try_parse`. Both break the rule that no shipped function exists only for tests. Remove them and point their callers at `analyze()`, the path shipped code takes.

This carries ruling S1 from `.scratch/cell-tracker/issues/07-decide-blank-operand-edge-cases.md`, which does not depend on the Blank Answer that closed that ticket.

## Acceptance criteria

- [ ] `Parser::try_parse`, `SyntaxError::CommentIsNotAValue` and `SyntaxError::UnexpectedTrailingContent` are gone, along with anything else only they reach: the strict-parsing tests and properties, and the clause of `permissive_analysis_of_printable_ascii_reports_what_it_could_not_read` asserting that `analyze()` never raises `UnexpectedTrailingContent`.
- [ ] `lang/tests/allocation.rs`, `interpret_source` and the `orcvs` properties call `analyze()`. Where a caller relied on strict success, it asserts no diagnostics and the Atoms it expects.
- [ ] The `parse` bench measures `analyze()` on `NESTED` under a new name, such as `parse_nested`, and the commit message records that the `parse` series ends there. The benchmark action compares each benchmark against the previous point stored under the same name, so moving `parse` onto `analyze()` under its old name would report the change of path as a regression or an improvement. It is not redundant with `parse_invalid`, which runs `analyze()` on the malformed `INVALID` input, so it is renamed rather than dropped.
- [ ] Parser tests that asserted a strict-parse refusal assert the same refusal through `analyze()`'s diagnostics, or are removed where `analyze()` already covers them.
- [ ] Gates for `lang` and its dependants (`lang`, `orcvs`), plus `cargo test --workspace --doc --locked`.
