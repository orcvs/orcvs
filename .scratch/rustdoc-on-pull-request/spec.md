# Run rustdoc on the pull-request tier

**Goal:** A rustdoc warning fails the pull request that introduced it, not the merge queue that
tried to land it.

## Why

The pull-request tier runs doctests. It does not generate rustdoc. Intra-doc lints — a public item
linking to a private one — are invisible until `check_merge` runs `cargo doc` with warnings denied.
PR #81 was green on the pull-request `full-gate` and red on the merge-group `full-gate` for that
reason.

The cost is not why the gate sat in the merge tier. On a warm Linux runner, after clippy has already
compiled the workspace, each `cargo doc --workspace --no-deps` pass is about a second. It was
bundled with the slow leftovers when the tiers were split, and then listed as merge-only without a
measured reason.

What the queue should still defer is behaviour: the browser suite, and persistence at proptest's
256-case default. Rustdoc is a compile-time lint over docs the pull-request tier already type-checks.

## Decision

Extract, do not copy. Both existing rustdoc invocations — default features and persistence — run in
`check_pull_request`. They leave the merge-only tasks. The merge queue still runs the pull-request
tier first, so a rustdoc warning never reaches `main` without failing a pull request.

## Verification

```sh
bash scripts/check-tooling-contract.sh
actionlint
zizmor --offline .github/workflows
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --features persistence --locked
node --test scripts/tests/roadmap.test.ts
node scripts/roadmap.ts > /dev/null
```
