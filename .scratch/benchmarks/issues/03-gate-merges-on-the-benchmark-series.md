# 03 — Gate merges on the benchmark series

**What to build:** A workflow that runs the benchmarks on push to `main`, appends the results to the `gh-pages` series, and fails on a blowup.

**Blocked by:** 02 — Add the bench profile and the mise task.

**Status:** resolved

- [x] `.github/workflows/bench.yml` triggers on `push` to `main`, on `workflow_dispatch`, and on `pull_request`.
- [x] The `push` and `pull_request` triggers filter on the paths that can move a measurement; `workflow_dispatch` is unfiltered.
- [x] Workflow-level permissions are empty and each job declares its own.
- [x] The publishing job holds `contents: write`, runs only when the event is not a pull request, and pushes the series.
- [x] The pull-request job holds `contents: read`, supplies no token, sets `auto-push: false`, and fails on the same threshold.
- [x] The publishing job's `concurrency` sets `cancel-in-progress: false`, so a run is never killed while pushing the series; the pull-request job cancels superseded runs.
- [x] `test.yml` keeps `contents: read`.
- [x] A discarded warm-up run precedes the measured run.
- [x] The run step is `mise run bench | tee output.txt`.
- [x] `benchmark-action/github-action-benchmark` is pinned by commit SHA with a version comment, matching every other action in the repo.
- [x] It is configured with `tool: cargo`, `auto-push: true`, `github-token: ${{ secrets.GITHUB_TOKEN }}`, `alert-threshold: '150%'`, `comment-on-alert: true`, `fail-threshold: '300%'`, and `fail-on-alert: true`.
- [x] The `name` input is fixed and never changes, since the action keys the series by it.

## Comments

The action's README says outright not to run this workflow on pull requests, because it holds permission to modify contents. That is why it is a separate file rather than a job in `test.yml`, and why it pays its own compile instead of reusing the `full-gate` cache.

The first run has no previous data point. It stores the series and passes without comparing. That is expected, not a misconfiguration.

`auto-push` to the same repository works with `secrets.GITHUB_TOKEN`. The `action.yml` note about needing a personal access token applies to pushing to a different repository.

Measured on the first implementation: `parse_source` reported 1,204 ns (+/- 934) on the first run after a fresh compile and 417 ns (+/- 11, 7, 10) on the three settled runs that followed. A freshly compiled binary is the only thing CI ever measures, so without a discarded warm-up run the gate false-fails on its own noise.

The pull-request job is safe despite the action's warning about running on pull requests, because permissions are declared per job rather than per workflow: the pull-request job never receives write. It writes a local commit on the runner's `gh-pages` checkout that is never pushed, since the action pushes only when a token and `auto-push` are both given.

`comment-on-alert` must stay false there. The action throws `'comment-on-alert' input is set but 'github-token' input is not set` when it is enabled without a token. `summary-always` needs no token; it writes through `core.summary`.

When no previous point exists the action skips the comparison rather than failing, so the first runs pass without proving anything.

The mise step sets `install: false`. The first CI run spent ten of its twelve minutes compiling cargo-nextest, cargo-deny, trunk, and wasm-pack from source; `mise run bench` invokes none of them, only cargo from the toolchain step.

First CI measurement, against the settled local numbers: parse 143 ns (104 local), parse_invalid 62 ns (60), execute 15 ns (9), parse_source 435 ns (417). The action logged `Could not find data.js at dev/bench/data.js` and skipped the comparison, as expected before main has published.

The path lists are written out twice rather than shared through a YAML anchor. GitHub Actions does not support anchors in workflow files.

The filter is safe only while these jobs are not required status checks. `main` requires `full-gate`, `macos`, and `wasm`; adding a filtered job to that list would block every pull request that does not touch the filtered paths, because a skipped job reports no status at all.

### The gate's first real failure, and a deliberate re-baseline, 2026-09-05

`sequence-values/02` moved `execute` from 19 ns to 106 ns on the runner, a ratio of 5.58 against a
failure threshold of 3.00. The gate did exactly what it was built to do: it is the only check that
caught the regression, every correctness gate passed throughout, and the branch author had not run
`mise run bench`. Two fixes brought it to 70 ns, a ratio of 3.68, which still fails.

It was merged anyway. Recording why, and what that costs.

The gate is not a required status check — `main` requires `full-gate`, `macos`, and `wasm` — so no
override was needed and none was used. The remaining cost is understood rather than unexplained: it
is the broadcast seam, it is measured, and it is owned by ADR 0026 through the note in
`sequence-values/03`. Closing it means reshaping `Broadcast` so it no longer owns its operands,
which is the change most likely to make ADR 0026 harder to adopt later. A worse number now was
judged cheaper than a design that has to be undone.

**The cost of that choice is that this alert happens once.** After the merge, `main` publishes a
point near 70 ns and the series re-baselines. The next pull request compares against 70 ns, not
19 ns, and the gate will never mention this again. The number to restore is 19 ns on CI, or 12.4 ns
on the local rig the attribution was done on. Nothing enforces it. If the ADR 0026 revisit lands and
leaves `execute` at 70 ns, no automated check will say so.

Two things follow, neither of them built here:

- A deliberate baseline move is invisible to this gate afterwards. If that matters, the gate needs
  somewhere to record an accepted move and the figure it moved from — a floor for a named
  benchmark, checked separately from the ratio against the previous point. Worth an issue if the
  situation repeats; one occurrence is not yet evidence of a pattern.
- The ratio differs by machine. This branch measured 4.03x locally where CI measured 5.58x, and
  3.27x locally where CI measured 3.68x. A local measurement is not a prediction of the gate. Run
  the gate, or expect to be surprised by it.
