# 05 — Compile and run the persistence gate before a merge

**What to build:** The persistence feature's tests compile and run on a pull request. They are the workspace's only serialisation round-trip coverage, and today they neither run nor type-check before a merge.

**Blocked by:** 02 — Refresh the tooling contract's stale action pins.

**Status:** in-review

- [x] The pull-request tier type-checks every target with `persistence` enabled.
- [x] The three persistence tests run in the pull-request tier.
- [x] Whatever remains genuinely merge-only stays there, and `docs/tooling.md` states which parts those are.
- [x] The contract pins the added invocation.

## Comments

This is the browser break's mechanism, one feature over, and it is worse than a tier choice. The tests live in a test-only module and depend on `serde_json`, a dev-dependency absent from the normal dependency graph, so no library build can reach them — including the library check inside `test_persistence` itself. Only an all-targets build with the feature enabled compiles them, and that runs behind the push guard.

Confirmed by listing the tests each gate would run: the pull-request tier's set is missing exactly the round-trip test and the two deserialisation rejections.

`docs/tooling.md` claims the delayed tier holds behaviour rather than compilation. That sentence is untrue for persistence until this lands, so the doc and the gate have to move together.

## Decision — `test_persistence` keeps its place in the merge tier

`check_pull_request` gained the two commands that reach the tests:

```
cargo clippy --workspace --all-targets --features persistence --locked -- -D warnings
cargo nextest run --workspace --all-targets --features persistence --profile ci --locked
```

`test_persistence` was not emptied to match, and the overlap is deliberate. `check_pull_request` sets
`PROPTEST_CASES` to 32; the merge tier leaves proptest's 256-case default in place. Moving the
persistence run out of the merge tier would have taken the only 256-case run of the workspace suite
with it, which is a coverage loss dressed up as a de-duplication. What stays merge-only is the
full-case run, the two rustdoc gates, and the browser suite, and `docs/tooling.md` now says so.

`cargo check --package orcvs --lib --features persistence --locked` stays in `test_persistence` even
though the `--all-targets` clippy line beside it subsumes it. It is the kind of line issue 10
removes, but issue 10 enumerates its removals and this is not among them; taking it out here would be
scope this ticket did not ask for.
