# 09 — Fix the broken intra-doc link

**What to build:** `lang/src/atom.rs:821` links `[`Function::source_write`]`. No such method exists anywhere in the workspace — `Function::source_effect` is meant, and it is what the sentence is describing. The line reads:

> so a caller that means "is a Self-Banging Function" should ask [`Function::source_write`] instead.

This is the **only** rustdoc error in the workspace, and it fails the merge tier's rustdoc gate under `RUSTDOCFLAGS="-D warnings"`. The gate is therefore red on `main` today rather than on any one branch.

Change the link target. Check the sentence still says something true of `source_effect` while you are there: a Self-Banging Function declares `Some(SourceEffect { bundle: Advance, .. })`, and a Directional Bang Function declares `Some(.. Emit ..)`, so `source_effect` alone does not distinguish them either. If the advice needs sharpening, sharpen it; if the link was the whole error, fix the link and stop.

Found while reviewing the `function-replacement` effort. Unrelated to it, which is why it is filed here rather than there.

**Status:** ready-for-agent

- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked` succeeds.
- [ ] The sentence carrying the link is true of the method it now names.

## Verification

`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, plus `cargo fmt --all -- --check` and `cargo clippy --package lang --all-targets --locked -- -D warnings`.
