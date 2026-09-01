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
