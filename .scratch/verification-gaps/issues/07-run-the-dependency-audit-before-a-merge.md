# 07 — Run the dependency audit before a merge

**What to build:** Advisory, licence, and source auditing runs on a pull request. Today the audit runs only after a merge, and `mise run audit_deps` — the task that also inspects the feature-resolved dependency tree — has no caller at all.

**Blocked by:** 02 — Refresh the tooling contract's stale action pins.

**Status:** in-review

- [x] The dependency audit runs in the pull-request tier.
- [x] `mise run audit_deps` has an automatic caller, or its contents are folded into a task that has one.
- [x] `docs/tooling.md` states where dependency auditing runs.
- [x] The contract pins the invocation.

## Comments

Dependabot opens weekly grouped cargo bumps. Those are precisely the pull requests an audit exists for, and they merge today with no advisory, licence, or source check — the audit runs only once the change is already on `main`.

The contract currently pins the body of `audit_deps` while nothing executes it: a guarantee that an unrun task is spelled correctly. Either the task gains a caller or its two lines move somewhere that has one, but the contract should not keep asserting the shape of dead work.

The feature-resolved dependency tree is the half with no coverage at all. The audit half is at least duplicated inline in the merge tier; the tree inspection runs only when a human types it.

## Decision — the task gains a caller rather than being folded away

`check_pull_request` runs `mise run audit_deps`, so both halves of the task now execute: the advisory,
licence, and source check, and the feature-resolved tree that had no coverage at all. The contract
asserts the caller, so the task cannot go back to being a shape nothing runs.

`check_merge_native` keeps its own `cargo deny --locked check`. It is now redundant — `full-gate`
runs `check_pull_request` on a push as well as on a pull request — but `cargo deny` costs seconds,
and each tier reading correctly on its own is worth more than the removal. Taking it out would also
mean rewriting the two contract scenarios that distinguish the locked invocation in the check task
from the one in the audit task, which is churn for no gate.
