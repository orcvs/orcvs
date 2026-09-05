# 04 — Compile the WASM test targets before a merge

**What to build:** A browser test that no longer compiles fails the pull request that wrote it. The WASM tier compiles the browser test targets rather than libraries alone, and the WASM job runs that tier on every event instead of only after a merge. The browser run itself stays merge-only.

**Blocked by:** 01 — Repair the WASM browser test against the Output Command seam; 02 — Refresh the tooling contract's stale action pins.

**Status:** in-review

- [x] The WASM tier compiles shell's test targets for `wasm32-unknown-unknown`.
- [x] The WASM job runs the compile-and-build tier on a pull request.
- [x] The browser run remains merge-only, and the merge component narrows to it.
- [x] The contract pins the compile line, the job's pull-request step, and the number of steps carrying the push guard.
- [x] `docs/tooling.md` describes the tiers as they now are.

## Comments

`--lib` type-checks no test target. That single fact is why the browser suite could stop compiling and stay that way: the pull-request tier built libraries for the browser target, and the only command that built the test target ran behind the job's push guard.

Pinning the count of push-guarded steps is the part that keeps this closed. A job that quietly stops running on pull requests is exactly how the tier came to be skipped, and an assertion that merely finds the guard string somewhere cannot tell the difference.

The compile scope is shell rather than the workspace because `orcvs`'s test targets cannot compile for the browser target at all. Issue 08 removes that constraint.

Verified in both directions: the added command fails with the original error against the unrepaired suite and passes against the repaired one.

One hole stays open deliberately. Shell's browser test targets are compiled only with `persistence` enabled; the default-feature browser test path is compiled by nothing. It is green today, and a second compile line doubles the tier's clippy cost, so the decision is recorded here rather than taken silently.

An implementation exists uncommitted in the working tree, verified against `mise run check_wasm` end to end.

## Resolution

Issue 08 landed alongside this one, so the compile line ended up wider than this ticket described.
The two clippy invocations `check_wasm` carried — `--workspace --lib` and the `--package shell
--all-targets` workaround — collapsed into one `cargo clippy --workspace --all-targets --target
wasm32-unknown-unknown --features persistence --locked`. `--all-targets` expands to `--lib --bins
--tests --benches --examples`, so it covers every member's library exactly as the `--lib` line did.

That also relocates the hole this ticket recorded as deliberately open. The default-feature browser
test path is still compiled by nothing — the one compile line enables `persistence` — so the decision
stands as written. What changed is only its scope: the uncovered path is now every crate's rather
than shell's alone.

The guard on the merge-tier steps reads `github.event_name != 'pull_request'` rather than
`== 'push'`, which issue 06 changed for the dispatch hole. The contract's count assertion moved with
it and gained a negative assertion on the old string, so the guard cannot drift back.
