# 03 — Run the repository's own check scripts in the pull-request tier

**What to build:** Every check script the repository owns runs automatically. The tooling contract, the contract's own test suite, and the roadmap planner's test suite each execute in the pull-request tier, so drift in any of them fails a pull request instead of waiting for someone to run it by hand.

**Blocked by:** 02 — Refresh the tooling contract's stale action pins.

**Status:** in-review

- [x] The tooling contract runs in the pull-request tier.
- [x] The contract's own test suite runs in the pull-request tier.
- [x] The roadmap test suite runs in the pull-request tier, with the runtime it needs available to the gate.
- [x] `mise run check` still reaches all three, through the tier rather than by naming them again.
- [x] The contract pins each of these invocations.

## Comments

The contract was invoked only by `mise run check`, a local aggregate, and by no workflow. Its own test suite was invoked by nothing at all. That is why 02's break survived from 31 August unnoticed: a gate only a human reaches is a gate that drifts.

The roadmap planner is not merely a report. It throws on tracker inconsistency — an unnumbered tagged issue, a non-relative definition path, a tagged issue outside the release Gate's dependency closure — so its test suite guards invariants that agents edit constantly. The suite has drifted already: a completion log recorded eight tests and it now holds ten, so two were added to a suite nothing has ever executed automatically.

The roadmap gate needs a JavaScript runtime, and `mise.toml` currently pins cargo tools only. Adding that runtime to the pinned tooling is part of this ticket, not an aside.

An implementation of the first two acceptance criteria exists uncommitted in the working tree; the roadmap suite was not wired in.

## Resolution

`check_pull_request` runs four things nothing executed before: the contract, the contract's own test
suite, the roadmap suite, and the planner itself.

The planner run is the part the acceptance criteria did not name and the ticket's own reasoning
required. `node --test scripts/tests/roadmap.test.ts` drives `buildRoadmap` and `planRelease` over
`mkdtempSync` fixtures and never reads `.scratch/`, so the throws this ticket cites as the reason the
suite matters — an unnumbered tagged issue, a non-relative Definition path, a tagged issue outside
the release Gate's dependency closure — stay unreached by it. `node scripts/roadmap.ts > /dev/null`
is what runs them against the real tree. Both are pinned.

`mise.toml [tools]` gained `node = "22.23.1"`, matching the floor `package.json` declares for node's
TypeScript type stripping. `mise run check` reaches all four through `check_pull_request` rather than
naming them again, and `bash scripts/check-tooling-contract.sh` moved out of `check` for the same
reason.
