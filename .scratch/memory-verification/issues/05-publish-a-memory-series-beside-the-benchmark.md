# 05 — Publish a memory series beside the benchmark

**What to build:** The allocation counts from `01` and `02` are tracked over time next to the
existing timing series, so a slow creep that no single assertion catches is visible as a trend.

**Blocked by:** 01 — Count allocations on the Tick and Render Frame paths; 02 — Count allocations on the Source write and Language Map rebuild.

**Status:** ready-for-agent

- [ ] The existing pinned `github-action-benchmark` is reused with `tool: customSmallerIsBetter` and
      its own `name`. No new action, no new pin. The action merges multiple named series into the one
      gh-pages branch, which is the documented pattern for exactly this.
- [ ] The new series `name` is chosen once and documented as never changing, the way the existing
      series' name already is.
- [ ] The measurement that feeds the series is the same one the assertions use. A separate binary
      that re-runs the measured paths is not acceptable, because the published number and the
      asserted number would drift apart silently.
- [ ] The publishing job produces the JSON with cargo alone, so it keeps installing no other tooling.
- [ ] The second publishing step sets `skip-fetch-gh-pages`, since the first step in the same job has
      already fetched and pushed the branch.
- [ ] No warm-up run for the memory series, and the reason is stated: allocation counts are
      deterministic for a fixed input, so the warm-up the timing series needs would double the cost
      for no signal.
- [ ] The series starts with alerts and a job summary and does not fail the workflow. Thresholds are
      set far tighter than the timing series' because the metric is deterministic.
- [ ] The path filter is checked rather than assumed: it already covers the benchmarked crates and
      the manifests, so it likely needs no change.
- [ ] `actionlint`, `zizmor`, and `bash scripts/check-tooling-contract.sh` are run and their results
      recorded, since this is the one issue in the effort that touches a workflow.

## Comments

Deliberately the smallest version of itself. `01` through `04` already fail loudly on a regression;
this issue buys the trend and nothing else.

It does not fail the workflow at first, and that is a decision rather than an omission. A
deterministic metric at a tight threshold fires on any real change, and the action offers no in-repo
way to accept a deliberate increase — the series lives on gh-pages, not in a file that can be edited
in the same pull request. Turn failing on once the series has enough points to show it is stable.

Whether the benchmark workflow blocks a merge belongs to `verification-gaps/09`. This issue stays out
of that question.
