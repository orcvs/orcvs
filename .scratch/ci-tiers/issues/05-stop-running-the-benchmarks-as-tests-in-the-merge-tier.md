# 05 — Stop running the benchmarks as tests in the merge tier

**What to build:** A merge tier that runs the persistence test suite and leaves measurement to the benchmark workflow.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The benchmark targets stop being executed by `test_persistence`'s `nextest` run.
- [x] The compile-and-link coverage `--all-targets` was there for is kept.
- [x] The tooling contract pins the filtered line, and rejects the unfiltered one.

## Comments

Four merge-queue runs — `34660432645`, `34668680539`, `34668705820`, `34672300753` — sat in
`mise run check_merge` until `timeout-minutes: 20` killed the `full-gate` job. Two different pull
requests, #69 and #75, so it was not one branch's regression.

The line was in `mise.toml`, in `test_persistence`:

```
cargo nextest run --workspace --all-targets --features persistence --profile ci --locked
```

`--all-targets` includes `--benches`, and `nextest` executes a criterion benchmark as a test.
Criterion then takes its own defaults, because none of the four budget flags `mise run bench`
passes reach it here: a 3s warm-up, a 5s measurement and a 100,000-resample bootstrap, per
benchmark, for all 29 of them. Measured locally on an idle machine, four of the `orcvs` benchmarks:

```
PASS [  13.881s] orcvs::bench/source source_execute_tick_edges/16x16
PASS [  14.067s] orcvs::bench/source source_execute_tick_edges/64x64
PASS [  14.099s] orcvs::bench/source source_execute_tick_edges/128x128
PASS [  14.636s] orcvs::bench/source source_execute_tick_edges/32x32
```

On a four-vCPU hosted runner, contending with each other four at a time, the same benchmarks took
235s each. `34668705820`'s log ends mid-suite:

```
SLOW [>180.000s] orcvs::bench/source source_execute_tick_edges/64x64
PASS [ 234.377s] orcvs::bench/source source_edit_rebuild_invalid/32x32
##[error]The operation was canceled.
```

Nothing read the measurement. `nextest` records pass or fail and discards every number;
`.github/workflows/bench.yml` is what measures these, budgeted, on its own trigger, and it was
green on every one of those commits.

The cost was not only the pathological case. On the last green push, `34669793731`, this invocation
reported 27.461s for 715 tests, while its sibling — the same suite without the benchmarks — reported
1.637s for 686. So about 26 of its 27.5 seconds were benchmarks on a good day, and the whole job on
a bad one.

The fix is `-E 'not kind(bench)'` on that line. `--all-targets` stays, so every target still compiles
and links under the feature; only the execution is filtered. `kind(bench)` names the target kind
rather than matching on a name, so a test binary that merely has "bench" in its name still runs. The
filter selects exactly the 29 benchmarks: `cargo nextest list` for the line reports 715 tests
unfiltered and 686 filtered, which is the count the non-benchmark workspace run reports. Run
locally, the filtered gate is 686 tests in 3.7s.

`scripts/check-tooling-contract.sh` pinned this line byte-identically and so had to move with it.
The pin now includes the `-E`, and was checked to have teeth: with the filter removed from
`mise.toml` the contract rejects, and restores to passing when it is put back.

Two things this does not fix, both filed elsewhere. A job killed by `timeout-minutes` reports
`cancelled`, not `failure`, which is the same signal a superseded run gives — that is
`verification-gaps/14`'s third checkbox, and it is why four timeouts went unnoticed. And the macOS
tier's cold-cache cost is `verification-gaps/14` proper.
