# 09 — Run the allocation steps after a ratio-gate failure

**What to build:** A timing ratio-gate failure (`fail-on-alert` at 300%) no longer skips the allocation measurement steps in `.github/workflows/bench.yml`. #124 left this open deliberately.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human — implemented, and every local gate passes; the second box needs one real CI run to prove, which only a push can give (see the 2026-09-23 implementation comment)

- [x] The allocation steps no longer rely on the timing step having fetched `gh-pages`. Today both set `skip-fetch-gh-pages: true` (`bench.yml:212`, `:402`). Either they fetch it themselves, or they run on a reliable signal that the fetch happened.
- [ ] After a ratio-gate failure the allocation steps run and report, and the job still fails.
- [x] "Check bench floors" stays the last step of each job, and `scripts/check-tooling-contract.sh` still asserts that.
- [x] `bash scripts/check-tooling-contract.sh`, `bash scripts/tests/check-tooling-contract.sh`, `actionlint` and `zizmor --offline .github/workflows` pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues,** from #124's "Not addressed" section. A plain `if: ${{ !cancelled() && steps.bench.outcome == 'success' }}` is not enough: the timing step's outcome cannot show whether it failed on the alert after fetching or before it, so the allocation publish could run against a `gh-pages` checkout that was never fetched.

**2026-09-23 — implemented on `benchmarks-09-allocation-after-ratio-failure`.** The allocation steps in both jobs of `.github/workflows/bench.yml` no longer consult the timing step at all. "Measure allocations" (`id: allocations`) runs on `!cancelled() && steps.bench.outcome == 'success'` — the criterion run, as the floor steps do. "Assemble the memory series" (`id: memory`) runs on `steps.allocations.outcome`. A new step, "Fetch gh-pages for the memory series" (`id: pages`), runs on `steps.memory.outcome` and does `git checkout --quiet --force --detach "$GITHUB_SHA"` then `git fetch origin +gh-pages:gh-pages`. The memory action runs on `steps.pages.outcome` and keeps `skip-fetch-gh-pages: true`, which now rests on a fetch that ran and succeeded in the same job seconds earlier. A failed fetch skips the publish rather than running it against an unfetched branch. A timing-step failure, on the alert or before its fetch, skips nothing. The job still fails, because nothing masks the failed timing step.

The fix follows the pinned action (`4322e572`, v1.22.2, `dist/src/write.js`). Unless `skip-fetch-gh-pages` is set, the action runs `git fetch <remote> gh-pages:gh-pages`, switches to the branch, and runs `git pull`. It then commits even without `auto-push`, pushes only with a token and `auto-push`, and runs `git checkout -` in a `finally`. `handleAlert` raises the ratio failure after that write. The obvious fix, letting the memory action fetch for itself, fails on pull requests: a plain fetch refuses to rewind the timing comparison's unpushed local commit. I reproduced this against a scratch remote. The unforced fetch printed `! [rejected] gh-pages -> gh-pages (non-fast-forward)`, and the forced one reset the branch to the remote. A simulated publish sequence (timing push, then forced fetch, then memory commit and push) stored both points. On `main` the force is a no-op after a timing push. After a failed timing push, it drops a point that never reached the series. The detaching checkout keeps the fetch off a checked-out `gh-pages`, where the timing action's `finally` could leave HEAD if a half-written `data.js` blocked it. The fetch is anonymous, as the pull-request job's reads already are. Pushes cannot race each other: the publishing job's concurrency group serialises runs, and within a job the two publishes happen one after the other.

"Check bench floors" stays last in both jobs. The reason has changed: the allocation steps now gate on the step before them rather than on the job's status, so a floor failure would not skip them anyway. Keeping the floor check last means the series does not depend on that.

`scripts/check-tooling-contract.sh` pins the following:

- the forced fetch and the detaching checkout, once per job;
- `fail-on-alert: true`, once per job, so a quoted gate cannot hide from the step check;
- the `steps.bench` guard, three times per job.

One awk pass over each job then enforces two rules. First, every step after a `fail-on-alert: true` step must carry `if: ${{ !cancelled() && steps.<id>.outcome == 'success' }}`, where `<id>` belongs to an earlier step in the same job and is not a gate. Second, the chain runs measure → bench, assemble → measure, fetch → assemble and publish → fetch, with each step's role found by what it runs. The floor-last check now uses `workflow_job_count`'s job-key pattern.

`scripts/tests/check-tooling-contract.sh` adds two scenarios. `allocation-after-ratio-gate` covers an unguarded step after the gate, a `success()` guard, the fetch moved above the criterion run, the chain's guards swapped, an unforced fetch, and a quoted `'true'` gate. `bench-floor-not-last` covers a step after the floor check; nothing tested that ordering before. Each of these scenarios asserts its specific message through a new `assert_rejected_with`. `docs/tooling.md` describes the fetch and the chain. The review (`code-review`, high) raised these points, and all of them are addressed above: the chain was pinned only by counts, the fetch could be placed above the criterion run, `success()` guards were accepted, a quoted gate went unseen, the fetch ran too early, HEAD could be stuck on `gh-pages`, the job-key regexes did not match, and the unforced-fetch test was not isolated.

Local gates passed: `bash scripts/check-tooling-contract.sh`, `bash scripts/tests/check-tooling-contract.sh`, `actionlint`, and `zizmor --offline .github/workflows` (no findings; 19 suppressed, the same as `HEAD`).

**Open:** the second box, where the allocation steps run and report after a real ratio-gate failure and the job still fails, has not been observed. It rests on the action's source and on GitHub's step-status rules. Proving it needs a real Benchmark run that trips the 300% gate, for example a throwaway pull request that slows a benchmark to more than three times its time. That run's log should show a red "Compare against main", then the three allocation steps, the fetch and the floor steps all running, and the job red.
