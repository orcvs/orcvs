# 01 — Write the source comment rule

**What to build:** A rule for what a source comment may carry, adopted from hyper's (tobyhede/hyper#276) and fitted to this repository's gates. The full text is `docs/agents/comments.md`; `AGENTS.md` carries a pointer bullet under Rust policy, and `rust-change` and `rust-review` apply it.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A comment states what the code does, why, and the invariant it keeps, in the present tense. A sentence that would not be true, and worth saying, had the code been written this way from the start is lineage, and goes to the commit message, an ADR or a ticket.
- [x] A citation stays only when it names a constraint the code cannot show and the cited document holds the reasoning.
- [x] The comment text a gate reads is set aside: `SAFETY:` comments (`clippy::undocumented_unsafe_blocks`), doctests, intra-doc links, and lint-suppression reasons.
- [x] Every example is drawn from real source.

## Inventory: comment text a gate reads

Counted by search, not by blanking comments and running the gates, so 02–05 should confirm by running each crate's gates after their sweep.

- `// SAFETY:` — 7: `lang/tests/allocation.rs` (3), `orcvs/tests/allocation.rs` (3), `orcvs/src/source/model.rs` (1).
- Fenced examples in `///` or `//!` — 44 fence lines across the three crates. Each is a doctest; the sweep does not delete one without treating it as a test change.
- Intra-doc links — throughout; `RUSTDOCFLAGS="-D warnings" cargo doc` in `check_pull_request` fails on a broken one.
- No test or script reads `.rs` comment text. `scripts/check-tooling-contract.sh` asserts manifest and asset lines, not comments.

## Sweep size

Lines carrying a lineage signal word, and lines citing `ADR NNNN` or `.scratch/`, by search. Signal words decide nothing on their own, so these are candidates, not findings.

| Scope | Signal words | Citations |
|---|---|---|
| `lang` | 15 | 234 |
| `orcvs` | 91 | 274 |
| `console` | 108 | 206 |
| `scripts/` | 10 | 5 |
