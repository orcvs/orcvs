# 06 — Run Miri deliberately against the Source model

**What to build:** Miri can be run against the tests that reach the workspace's one `unsafe` block,
by a command that exists and a job someone can trigger, without becoming a check every pull request
pays for.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A task runs Miri over the Source model tests, installing the nightly toolchain it needs rather
      than changing the pinned channel.
- [ ] The run is scoped by test filter, not by crate. The crate links ALSA through `midir` and a
      multi-threaded runtime, neither of which Miri can execute, but Miri interprets what runs, so a
      dependency no selected test calls never becomes a problem. The filter and this reasoning are
      recorded.
- [ ] The job is manually triggered and is not a required check. It does not run on pull requests.
- [ ] The contract wording is checked against `verification-gaps/12`, which decided the contract
      stops *requiring* Miri while saying it is the tool this gate would prefer and should be run
      deliberately. This issue provides the deliberate path and must not turn it back into a
      requirement.
- [ ] `actionlint`, `zizmor`, and `bash scripts/check-tooling-contract.sh` are run and their results
      recorded, since this touches a workflow and a mise task.

## Comments

This is the safety half of the effort, and it is small on purpose. The workspace holds exactly one
`unsafe` block — the in-place ASCII byte write in the Source model — and the clippy denials already
check that its invariant is written down. Miri is what checks whether the invariant holds.

The tests that reach it are the ones `02` also measures.

`nextest` is supported under Miri and gives each test its own interpreter context, which is what
makes leak detection at termination meaningful per test rather than per binary.
