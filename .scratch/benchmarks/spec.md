# Gate `lang` performance against its own history

**Goal:** Measure the `lang` parse and interpret paths with criterion, store the results as a series, and fail a merge when a benchmark blows up.

## Why

`AGENTS.md` requires "a reproducible benchmark or profile" behind any performance claim, and no benchmark exists. `lang` sits on two hot paths: every Tick interprets the Expressions in a Source Snapshot, and every Render Frame re-reads the Source. A regression in either is invisible to the current gates, which check correctness only.

## What this gate is

A wall-clock benchmark run on `ubuntu-latest`, compared by `benchmark-action/github-action-benchmark` against the previous stored result for `main`, with the series kept on the `gh-pages` branch and charted at `orcvs.github.io/orcvs/dev/bench`.

Both triggers are filtered to the paths that can move a measurement: `lang/**`, the root manifest and lockfile, `rust-toolchain.toml`, `mise.toml`, and the workflow itself. A pull request that touches none of them runs no benchmark, which is safe only while the benchmark jobs are not required status checks — `main` currently requires `full-gate`, `macos`, and `wasm`. Making a filtered job required would leave every unrelated pull request waiting forever for a check that never reports.

Filtering the push trigger as well keeps the series to one point per change that could have moved it. Each stored point carries the runner's own noise, and the comparison is against the previous point, so republishing an unchanged `lang` would let the baseline drift on nothing.

Two jobs share one measurement. The publishing job runs after a push to `main`, appends the result to the series, and holds `contents: write`. The pull-request job runs the same benchmarks, compares them against the last point `main` stored, and fails on the same threshold, but holds `contents: read` and supplies no token: on a public repository the action reads `gh-pages` anonymously, and the comparison and failure threshold need nothing more. Only `main` writes to the series, so a pull request cannot move the baseline it is judged against.

## What this gate is not

These limits are the point of the design, not defects in it.

It cannot see small regressions. Wall-clock timing on a shared GitHub runner varies by tens of percent between runs, so the thresholds are set where noise cannot reach them. A 10% regression passes.

It compares against the previous data point, not a fixed baseline. Ten commits that each cost 20% never trip the 300% fail threshold, and the branch arrives six times slower with a green history. The 150% comment threshold narrows that window; nothing in this action closes it.

It compares across runners. A pull request is measured on one hosted runner and compared against a number measured on another at another time, which is wider than the run-to-run spread on a single machine. The thresholds are set far enough out to absorb it.

Until `main` has published once there is nothing to compare against, and the action skips the check entirely rather than failing. A green pull-request benchmark job therefore does not prove a comparison happened.

It cannot be reproduced locally. `mise run bench` runs the same measurement, but the comparison lives in the action. This is the one exception to the local and CI equivalence `docs/tooling.md` promises, and the doc states it.

The upgrade path, if a real regression ever slips through, is `iai-callgrind`: instruction counts rather than wall clock, precise to about 1%. It is rejected today because it runs under Valgrind, which has no Apple Silicon support, so it would be a gate that cannot run on the maintainer's machine at all.

## Rules

The measurement command is identical locally and in CI: `cargo bench --package lang -- --output-format bencher`. The `--output-format bencher` flag is load-bearing. The action's `cargo` parser is one regex over `test <name> ... bench: <N> ns/iter (+/- <M>)`, and a line that does not match is skipped silently, so criterion's default output stores zero benchmarks and passes green.

The benchmark job is its own workflow. It needs `contents: write` to push the series, and `test.yml` runs on pull requests, so the write-scoped job stays out of it.

Benchmarks measure behaviour that exists, on Source drawn from the existing tests. A benchmark of an unimplemented path is a fiction with a number attached.

## Prerequisites

The orphan `gh-pages` branch exists on `origin` and GitHub Pages is enabled for it. Both are done.

## Issues

- `issues/01-add-criterion-and-the-lang-benchmarks.md`
- `issues/02-add-the-bench-profile-and-mise-task.md`
- `issues/03-gate-merges-on-the-benchmark-series.md`
- `issues/04-document-the-benchmark-tier.md`
