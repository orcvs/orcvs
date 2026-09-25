# 09 — Move lang test-only API behind cfg(test)

**What to build:** `lang` ships only what production calls. Stack's single-value pop and its maybe-Atom type are used only by tests; `Expression::take_tokens` has no caller; `TypeError::Function`, `TypeError::Bang`, `TypeError::String` and `ArgumentError::ExpectedFunction` are never constructed; `Atom::Char` and `to_atom_char` have no shipped producer; `Interpreter::execute` is called only by `lang`'s tests, `lang/benches/lang.rs` and `lang/tests/allocation.rs`; `tracing` is a normal dependency used only in tests; and commented-out derives and code linger (`atom.rs`, `interpreter.rs`).

`Token::Char` is not in scope: the parser never produces it, but `orcvs`'s `Source::token_at` answers it for leftover content (settled by typed-source-paint/04) and the console's style code reads it.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Test-only Stack API compiles only under test, or tests use the shipped API.
- [ ] Uncalled accessors and never-constructed error variants are deleted, or a shipped caller is named for each kept one.
- [ ] `Atom::Char` and `to_atom_char` are deleted or given a declared producer; `Token::Char` stays as `orcvs`'s leftover-content token. Deleting `Atom::Char` reaches `orcvs` (`RenderError::Unrepresentable` in `source/encoding.rs`, `language_map.rs`, `render_frame.rs`, and their tests) and `lang`'s `sequence.rs` and `expression.rs`.
- [ ] `Interpreter::execute` is narrowed to test and bench use, or a shipped caller is named (see 05).
- [ ] `tracing` moves to dev-dependencies.
- [ ] Commented-out code is removed.
- [ ] Clippy and tests pass for `lang`, `orcvs` and `console`.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `lang/` is unchanged since the baseline. Corrected the Char claim (`Token::Char` has a shipped producer in `orcvs`), named the variants, and added `Interpreter::execute`.
