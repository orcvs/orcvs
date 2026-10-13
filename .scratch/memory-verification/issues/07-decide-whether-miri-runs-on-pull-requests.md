# 07 — Decide whether Miri runs on pull requests

**What to build:** A recorded decision on whether the Source model Miri run stays manual-only or
becomes a check on pull requests (or the merge queue), made against its measured cost.

**Blocked by:** 06 — Run Miri deliberately against the Source model.

**Status:** ready-for-human

- [ ] The decision is recorded here, and it is weighed against `verification-gaps/12`, which made
      Miri deliberate and non-gating because it ships on nightly only.
- [ ] If Miri gains a trigger, the workflow, `mise.toml` and `scripts/check-tooling-contract.sh`
      (which currently pins `workflow_dispatch` as the only trigger) change together, and the job's
      timeout is set from measured runs rather than the 90-minute ceiling.
- [ ] If it stays manual, the trigger for dispatching it is still the one `docs/tooling.md` names.

## Comments

`06`'s second dispatch measured the cost: 6m 32s per job on `ubuntu-latest`, of which 4m 05s is
the 80 selected tests. Run in parallel with the other pull-request jobs, it lengthens the wall
clock only if it finishes last.
