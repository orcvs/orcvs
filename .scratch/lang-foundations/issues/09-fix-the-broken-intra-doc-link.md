# 09 — Fix the broken intra-doc link

**What to build:** `lang/src/atom.rs:821` links `[`Function::source_write`]`. No such method exists anywhere in the workspace — `Function::source_effect` is meant, and it is what the sentence is describing. The line reads:

> so a caller that means "is a Self-Banging Function" should ask [`Function::source_write`] instead.

This is the **only** rustdoc error in the workspace, and it fails the merge tier's rustdoc gate under `RUSTDOCFLAGS="-D warnings"`. The gate is therefore red on `main` today rather than on any one branch.

Change the link target. Check the sentence still says something true of `source_effect` while you are there: a Self-Banging Function declares `Some(SourceEffect { bundle: Advance, .. })`, and a Directional Bang Function declares `Some(.. Emit ..)`, so `source_effect` alone does not distinguish them either. If the advice needs sharpening, sharpen it; if the link was the whole error, fix the link and stop.

Found while reviewing the `function-replacement` effort. Unrelated to it, which is why it is filed here rather than there.

**Status:** resolved

- [x] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` succeeds.
- [x] The sentence carrying the link is true of the method it now names.

## Verification

`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, plus `cargo fmt --all -- --check` and `cargo clippy --package lang --all-targets --locked -- -D warnings`.

## Answer

The link now names `Function::source_effect`, and the advice needed the sharpening this issue
anticipated rather than the link alone. `source_effect` answers `Some(..)` for both groups, so
pointing a caller at it without saying what to read would have swapped one untrue sentence for
another. The effect table settles which half to read: every Self-Banging arm declares
`SourceBundle::Advance` and every Directional Bang arm `SourceBundle::Emit`, so the bundle is what
tells the groups apart and the sentence now says so.

`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` exits 0 across all three
crates, where it failed with "unresolved link to `Function::source_write`" and "could not document
`lang`" before. `cargo fmt --all -- --check`, `cargo clippy --package lang --all-targets --locked --
-D warnings`, `PROPTEST_CASES=32 cargo nextest run --package lang --locked` (223 passed) and
`cargo test --package lang --doc --locked` all pass.

Pushed straight to `main` rather than through a pull request. The gate this fixes is one of the
three required status checks, and it had been failing on `main` since before PR #71 — every merge
inherited a red `full-gate` and no pull request of its own could have turned it green any faster.
