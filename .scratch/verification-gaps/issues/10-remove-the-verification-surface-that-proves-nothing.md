# 10 — Remove the verification surface that proves nothing

**What to build:** The test targets, helpers, and entry points that exist only to be compiled are gone, so what remains means what it says.

**Blocked by:** None — can start immediately.

**Status:** in-review

- [x] Shell's empty integration test target and the helper module it exists to pull in are removed, or given real tests.
- [x] Shell's placeholder unit test is removed, or given an assertion.
- [x] `shell/check.sh` is removed along with the contract assertions pinning its contents, or it is given a caller and corrected.
- [x] The spell-checker configuration is removed, or the tool is pinned and a gate runs it.
- [x] The contract passes with the assertions that referenced deleted files removed.

## Comments

The integration test target holds a module import and one commented-out test — the workspace's only commented-out test — and it is the sole reason the helper module beside it compiles, every symbol in that module silenced with an allow attribute. Shell therefore has no integration coverage at all, only in-crate unit tests, while the target list suggests otherwise.

The placeholder unit test runs on every native gate, initialises tracing, logs a word, and asserts nothing.

`shell/check.sh` re-runs what its own first line already ran: the aggregate it invokes reaches both later commands through the merge tier. No workflow, task, or script calls it, yet the contract asserts what those two redundant lines contain — a live assertion holding dead work in place.

The spell-checker configuration's entire call graph is three comment lines telling a human how to install and run a tool that appears in no pinned tooling and no task. It came from the template the console started from, alongside the other scaffolding this repository has been correcting.

Each of these is small. Together they are the reason a reader cannot tell, from the shape of the repository, which checks are real.

## Decision — removed rather than filled in

Each of the four was removed rather than given the thing that would have justified it, because
nothing in the repository wanted what they were placeholders for.

- `shell/tests/app_test.rs` and `shell/tests/common/mod.rs` are gone. The target held a module import
  and one commented-out test, and was the only reason the helper compiled. Writing real integration
  coverage for shell is a decision with its own scope; leaving a target list that suggests coverage
  exists is not the way to hold that decision open.
- `shell/src/lib.rs`'s `mod test` is gone. It initialised tracing, logged `"etc"`, and asserted
  nothing. `tracing-subscriber` stays a dependency — `shell/src/main.rs` still uses it.
- `shell/check.sh` is gone, with the two contract assertions that pinned its contents and the line in
  `scripts/tests/check-tooling-contract.sh` that copied it into every fixture. Its first line already
  reached both later commands through the merge tier.
- `shell/.typos.toml` is gone. Its whole call graph was three comment lines telling a human how to
  install a tool that appears in no pinned tooling and no task.

The one candidate deliberately left alone is `cargo check --package orcvs --lib --features
persistence --locked` in `test_persistence`, which the `--all-targets` clippy line beside it
subsumes. It matches the pattern but is not on this ticket's list, and issue 05 records why it stayed.
