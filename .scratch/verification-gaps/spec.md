# Close the gaps in the verification gates

**Goal:** Make every check the repository owns run automatically, and make every gate that guards `main` actually execute for the commit it guards.

## Why

`ci-tiers` moved the expensive gates from the pull-request trigger to the merge trigger and recorded the consequence it accepted: "a WASM break is now found after merge, not before it." That consequence arrived. `shell/tests/wasm.rs`, the repository's only browser test target, stopped compiling when the Output Command seam landed, and `main` was red across three pushes before anyone looked. No pull-request gate compiled the file. `check_wasm` type-checked libraries only, and the one command that built the test target ran in a job the workflow skipped on pull requests.

An audit for the same failure shape found that this is not one gap but a class. A check can fail to run in four ways, and the repository has instances of all four.

A gate can be defined and never invoked. `mise run audit_deps` has no caller. `scripts/tests/roadmap.test.ts` is a passing ten-test suite that nothing executes. `shell/check.sh` is an entry point no automation reaches, whose contents a live assertion pins in place.

A gate can be invoked and skip. Every push to `main` shares one concurrency group, so a second merge cancels the first commit's run. `dd20cba6` landed with two of three jobs cancelled and was never re-run. A cancelled run is not a failed run, so nothing reported it.

A gate can run and not be required. The benchmark comparison fails on a threefold regression and blocks nothing. Administrator enforcement is off and no review is required, so a direct push to `main` skips every required context. Several have, and some were red.

A gate can compile the wrong targets. `--lib` type-checks no test target. That is why the browser suite rotted, and it is why the three persistence tests — which depend on a dev-dependency and therefore cannot appear in any library build — are neither run nor type-checked before a merge.

## Order

Issue 01 first: `main` is red until it lands. Issue 02 next, because the tooling contract exits at its first failure, so every later issue that adds an assertion needs it passing.

## Not in scope

Eleven stale remote branches still carry the `console/.github/workflows/` files that `pre-split-defects/07` deleted. `main` is clean and the files return only if an ancient branch is merged. Deleting remote branches is the maintainer's call and is recorded here rather than actioned.

macOS runs the pull-request tier and no merge-only gate. That is deliberate under the current job split and is not treated as a defect here.

## Issues

- `issues/01-repair-the-wasm-browser-test.md`
- `issues/02-refresh-the-stale-action-pins.md`
- `issues/03-run-the-repository-check-scripts.md`
- `issues/04-compile-the-wasm-test-targets-before-a-merge.md`
- `issues/05-compile-and-run-the-persistence-gate-before-a-merge.md`
- `issues/06-run-the-merge-tier-for-every-commit.md`
- `issues/07-run-the-dependency-audit-before-a-merge.md`
- `issues/08-compile-orcvs-test-targets-for-wasm32.md`
- `issues/09-require-the-benchmark-and-close-the-protection-bypasses.md`
- `issues/10-remove-the-verification-surface-that-proves-nothing.md`
- `issues/11-persist-the-interpreter-property-counterexamples.md`
- `issues/12-name-an-unsafe-review-gate-that-runs.md`
- `issues/13-bump-the-mise-tool-pins-automatically.md`
