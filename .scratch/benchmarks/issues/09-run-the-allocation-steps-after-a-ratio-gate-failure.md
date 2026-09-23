# 09 — Run the allocation steps after a ratio-gate failure

**What to build:** A timing ratio-gate failure (`fail-on-alert` at 300%) no longer skips the allocation measurement steps in `.github/workflows/bench.yml`. #124 left this open deliberately.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The allocation steps no longer rely on the timing step having fetched `gh-pages`. Today both set `skip-fetch-gh-pages: true` (`bench.yml:212`, `:402`). Either they fetch it themselves, or they run on a reliable signal that the fetch happened.
- [ ] After a ratio-gate failure the allocation steps run and report, and the job still fails.
- [ ] "Check bench floors" stays the last step of each job, and `scripts/check-tooling-contract.sh` still asserts that.
- [ ] `bash scripts/check-tooling-contract.sh`, `bash scripts/tests/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues,** from #124's "Not addressed" section. A plain `if: ${{ !cancelled() && steps.bench.outcome == 'success' }}` is not enough: the timing step's outcome cannot show whether it failed on the alert after fetching or before it, so the allocation publish could run against a `gh-pages` checkout that was never fetched.
