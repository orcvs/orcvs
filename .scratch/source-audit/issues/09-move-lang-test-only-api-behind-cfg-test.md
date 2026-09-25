# 09 — Move lang test-only API behind cfg(test)

**What to build:** `lang` ships only what production calls. Stack's single-value pop and its maybe-Atom type are used only by tests; `Expression::take_tokens` has no caller; `TypeError::Function`, `TypeError::Bang`, `TypeError::String` and `ArgumentError::ExpectedFunction` are never constructed; `Atom::Char` and `to_atom_char` have no shipped producer; `Interpreter::execute` is called only by `lang`'s tests, `lang/benches/lang.rs` and `lang/tests/allocation.rs`; `tracing` is a normal dependency used only in tests; and commented-out derives and code linger (`atom.rs`, `interpreter.rs`).

`Token::Char` is not in scope: the parser never produces it, but `orcvs`'s `Source::token_at` answers it for leftover content (settled by typed-source-paint/04) and the console's style code reads it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Test-only Stack API compiles only under test, or tests use the shipped API.
- [x] Uncalled accessors and never-constructed error variants are deleted, or a shipped caller is named for each kept one.
- [x] `Atom::Char` and `to_atom_char` are deleted or given a declared producer; `Token::Char` stays as `orcvs`'s leftover-content token. Deleting `Atom::Char` reaches `orcvs` (`RenderError::Unrepresentable` in `source/encoding.rs`, `language_map.rs`, `render_frame.rs`, and their tests) and `lang`'s `sequence.rs` and `expression.rs`.
- [x] `Interpreter::execute` is narrowed to test and bench use, or a shipped caller is named (see 05).
- [x] `tracing` moves to dev-dependencies.
- [x] Commented-out code is removed.
- [x] Clippy and tests pass for `lang`, `orcvs` and `console`.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** `lang/` is unchanged since the baseline. Corrected the Char claim (`Token::Char` has a shipped producer in `orcvs`), named the variants, and added `Interpreter::execute`.

**2026-09-25 — resolved on `lang/narrow-api`.** `Stack::pop` and `MaybeAtom` are deleted and their tests use `pop_value`; `Expression::take_tokens`, `TypeError::{Function, Bang, Char, String}` and `ArgumentError::ExpectedFunction` are deleted. `Atom::Char` and `to_atom_char` are deleted; `Token::Char` stays as `orcvs`'s leftover-content token and refuses to decode. In `orcvs`, `RenderError::Unrepresentable` stays for `Encoding::literal` and as Tick planning's no-panic path, with tests restated over literal text plus a sweep proving every language value renders. `Interpreter::execute` is behind `cfg(any(test, feature = "test-execute"))`, enabled only by `lang`'s self dev-dependency (see 05). `tracing` is a dev-dependency. Commented-out code is gone.
