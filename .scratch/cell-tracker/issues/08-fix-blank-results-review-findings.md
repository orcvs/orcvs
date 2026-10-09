# 08 — Fix the ticket 03 review findings

Status: wontfix
Blocked by: None

**Where:** worktree `.worktrees/cell-tracker-blank-results`, branch `cell-tracker/blank-results` (stacked on `cell-tracker/sequence-retirement-and-return`). Review range: `git diff cell-tracker/sequence-retirement-and-return...cell-tracker/blank-results` (commit `8f65a8d7`).

**What to build:**

Resolve the review findings that do not depend on ticket 07's language decisions. Do not change whether an all-blank Expression runs or what a Jump copying blanks answers; ticket 09 owns those. Use TDD for the coverage gaps through the Source Tick interface.

## Acceptance criteria

Hard violations (CLAUDE.md Rust policy):

- [ ] `SyntaxError::BlankOperandIsNotAValue` (`lang/src/error.rs`) is reachable only from tests: `try_parse` (`lang/src/parser.rs`) has no shipped caller; its callers are `interpret_source` (`lang/src/lib.rs`, `#[cfg(test)]`) and proptests in `orcvs` (`tick.rs`, `language_map.rs`). Give the strict-parse refusal a test-only home or a shipped reason, and treat the existing `CommentIsNotAValue` gap the same way. ("Keep test-only inputs out of shipped code.")
- [ ] Wrap the `debug_assert!` message in `lang/src/parser.rs` ("an Expression that reported no error holds only Atoms, a Comment, or a blank operand") within 100 columns.

Coverage gaps (ticket 03 criteria):

- [ ] `a_blank_answer_through_a_portal_clears_timed_plays_note_rather_than_replaying_it`: in the nested case the child's Output Portal clears both Timed Play's note and length slots, so a blank length alone would explain the silence. Place the nested child so only the note slot is cleared, and assert the length slot keeps its characters.
- [ ] Add a test for a blank nested child under a Terminal Output parent, e.g. `!>007F.+  01`: no Play Command, no diagnostic, child's Output Portal cleared.

Judgement calls (Fowler smells; apply where they simplify, or record why not):

- [ ] `lang/src/expression.rs`: `blank: bool` beside `atom: Option<Atom>` encodes parsed / blank / refused. Replace with a slot enum, removing `add_blank` (near copy of `add_positioned`) and the `atom.is_some() || entry.is_blank()` pairs in `orcvs/src/source/tick.rs` and `language_map.rs`'s `schedules_as`.
- [ ] "Is a Comment or has a blank operand" is recomputed in `try_parse`, two lang proptests and the language_map proptest; name it once as an `Expression` method.
- [ ] Rename one of `Token::is_blank(spelling)` / `PositionedEntry::is_blank()`; drop the duplicate length check in `Parser::is_blank_next`.
- [ ] `orcvs/src/source/tick/execution.rs`: `deliver_output(answer: Answer, ..)` never sees `Blank`; take the Atom. Drop the `writes_cells()` guard in `deliver_blank` that repeats `project_value`'s.
- [ ] The relaxed parser property (`lang/src/parser.rs`, ~line 1510, `atoms().is_none() || nested_effect`) states the real invariant (every Token was read and the error alone blocks execution), uses `prop_assert!` like its neighbours, and lands as its own commit.

## Gates

From the worktree root, mise shims off PATH if untrusted:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --workspace --locked
cargo test --workspace --doc --locked
PROPTEST_CASES=32 cargo nextest run --workspace --tests --no-default-features --locked
cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings
```

Commit on `cell-tracker/blank-results` (no Co-Authored-By trailer; a hook refuses it). Do not push. `cell-tracker/track` must be rebased onto the result by ticket 10.

## Comments

Resolved by `49a141cc` and `c313b8f4` on `cell-tracker/blank-results` (address-code-review). J1 refuted (no invalid state is constructible); J2 deferred to S1, now ruled in 07 and carried by 09; J4 half refuted (`deliver_output` does receive `Blank`).

Superseded by ADR 0066 / PR #200; see 07. `49a141cc` and `c313b8f4` were never merged: #200 merged a rewritten `cell-tracker/blank-results`, on which `28e7b611` reverted the Blank Answer (`3ca55ddc`, `162e9c32`, `4b737ec3`).
