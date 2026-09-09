# 08 — Bring the benchmark workflow inside its budget

**What to build:** A benchmark run whose cost matches the precision its thresholds can use. The
gate alerts at 150% and fails at 300%; it currently spends twelve and a half minutes buying ±2%.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `mise run bench` names a criterion measurement budget rather than taking the default one.
- [x] The warm-up run costs a fraction of the measured run instead of duplicating it.
- [x] The warm-up stays symmetric between the two jobs in `.github/workflows/bench.yml`.
- [x] `scripts/check-tooling-contract.sh` pins whatever the two commands become, and its comments
      say why each figure is what it is.
- [x] `docs/tooling.md` describes the run the workflow performs.
- [x] `bash scripts/check-tooling-contract.sh`, `bash scripts/tests/check-tooling-contract.sh`,
      `actionlint`, and `zizmor --offline .github/workflows` pass.

## Comments

### Where the time goes, 2026-09-09

Run `34321680905`, job `pull-request`, step by step:

```
setup (checkout, toolchain, cache, apt, mise)    40s
Warm up          (mise run bench > /dev/null)  5m24s
Run benchmarks   (mise run bench | tee)        5m11s
Measure allocations                            1m13s
                                              -------
                                              12m32s
```

Compilation is not the cost. The cache is warm and cargo reports
`Finished bench profile [optimized] target(s) in 0.11s` before the measured run. The 10m35s is
criterion executing 29 benchmarks twice — `lang` 89s for 9, `orcvs` 221s for 20, about ten seconds
each.

Ten seconds each is criterion's default budget, not the workspace's code: 3s warm-up plus 5s
measurement plus a 100,000-resample bootstrap, per benchmark, whatever is being measured. `parse`
reports 185 ns/iter, and the gate spends ten seconds establishing it.

That is the defect. `spec.md` already records what this gate can see: wall clock on a shared runner,
compared across runners against a single previous point, alerting at 150% and failing at 300%. It is
buying precision three orders of magnitude finer than the threshold that reads it.

### What was measured, 2026-09-09

criterion 0.8.2 accepts the budget flags alongside `--output-format bencher`. Measured locally on
the maintainer's machine (Apple Silicon), `lang` only, compile excluded:

```
default                                                                      4.9s/bench
--warm-up-time 1   --measurement-time 2 --sample-size 20                     4.9s/bench
--warm-up-time 0.5 --measurement-time 1 --sample-size 10 --nresamples 1000   1.9s/bench
```

The numbers hold across all three: `parse` 127/128, `parse_source` 465/475, `parse_records/63`
650/656 — under 2% drift, against a 150% alert threshold.

`--nresamples` matters as much as the time flags. Roughly 2.7s of each CI benchmark's ten seconds
sits outside the configured measurement budget, and the default 100,000-resample bootstrap is most
of it.

### Do not reach for `--quick`

It is the obvious flag and it silently disarms the gate. It drops the name from each line:

```
bench:         134 ns/iter (+/- 1)
```

The action's `cargo` parser is one regex over `test <name> ... bench: <N> ns/iter`, and a
non-matching line is skipped without an error, so a `--quick` run stores zero benchmarks and passes
green. That is the same failure `spec.md` records for `--output-format`, which is why the flag is
called load-bearing there. It was also no faster than the explicit flags: 17.3s against 17.0s over
the same nine benchmarks.

### The warm-up run has to stay symmetric

`bench.yml` runs `mise run bench` twice per job and discards the first, because a freshly compiled
criterion binary's first pass is contaminated — the comment records `parse_source` at 1,204 ns on a
first run against a settled 417 ns. The effect is real and it moves the *mean*, so the warm-up
cannot be dropped from one job and kept in the other: a pull request measured cold and compared
against a series measured warm would manufacture regressions out of nothing.

What the warm-up does not need is fidelity. Its output goes to `/dev/null`; its job is paging in the
binary and settling the CPU, not producing numbers, and it currently pays the full measured budget
to do it. A pass at `--measurement-time 0.1 --sample-size 10` touches every code path for a fraction
of the cost.

A differing measurement *budget* between the two jobs is safe in a way a differing warm-up is not:
it widens the variance of a reported mean without moving it. So if a tier split is wanted later —
faster on a pull request, fuller on the merge that publishes the point — it should be a shorter
budget, not a dropped warm-up.

### Subsetting is the weaker lever

Running fewer benchmarks on a pull request works mechanically: the action compares by name and
ignores a name it does not find. It costs more than it saves. Each size series exists so that
whole-map work shows as growth across the sizes rather than hiding inside one figure, and dropping
the large end of `source_execute_tick` removes exactly the reading the series was added for. The
budget change takes the same time out of the job without taking a measurement out of it.

### What this touches

`scripts/check-tooling-contract.sh` pins both commands verbatim:

- `mise.toml`'s `bench` task, pinned as
  `^run = .cargo bench --package lang --package orcvs --benches --locked -- --output-format bencher.$`
- `run: mise run bench > /dev/null`, asserted exactly once per job — which is also how "the memory
  series takes no warm-up run" is enforced, so the assertion has a second reason to exist and must
  keep serving it.

`docs/tooling.md` describes the tier in prose and states the local/CI equivalence the bench gate is
the one exception to. Whatever the command becomes, it stays identical locally and in CI:
`spec.md` requires that, and it is the reason the flags belong in the `mise` task rather than in the
workflow.

### Two neighbours

`verification-gaps/15` reports that the `workflow_dispatch` trigger burns the full twelve minutes and
then always fails, because the action finds no commit in a dispatch payload. The same pass is the
natural place to settle it.

`verification-gaps/09` holds whether this comparison becomes a required status check. A three-minute
job is a much easier thing to require than a twelve-minute one, so this issue moves that decision
without making it.

### What landed, 2026-09-09

`mise.toml` names the budget on `[tasks.bench]` and gains `[tasks.bench_warmup]` beside it, which
differs in one figure — `--measurement-time 0.1` against `1` — because its output goes to
`/dev/null`. Both jobs in `.github/workflows/bench.yml` call the warm-up task, and the comment on
each says why it is cheap and why it must stay identical in both.

Measured in this worktree, compile excluded:

```
mise run bench_warmup    30s
mise run bench           58s
                        ----
per job                  88s
```

Against the same machine at criterion's defaults, where the nine `lang` benchmarks alone took 4.9s
each, the two runs a job performs were about 284s. The saving is a little over threefold, and it
applies to both runs in both jobs. Scaled onto run 34321680905's 10m35s of criterion, a job should
land near four minutes rather than twelve and a half.

All 29 benchmarks still parse — the run emits 29 lines matching the action's
`test <name> ... bench: <N> ns/iter` regex, checked rather than assumed, since a run that parses to
nothing is the failure mode this gate has. The size series still reads: `source_execute_tick` goes
13,300 / 47,369 / 185,727 / 774,392 ns across 16x16 to 128x128, so the growth the series exists to
show survives the smaller sample. Reported spreads are all inside 2%, against thresholds of 150%
and 300%.

Two things beyond the ticket's list. The contract script gained an assertion refusing `--quick` in
both `mise.toml` and `bench.yml`, and `scripts/tests/check-tooling-contract.sh` gained four cases:
a defaulted budget, a warm-up given the measured budget, a job that skips the warm-up, and `--quick`
in either file. And criterion no longer prints its "Unable to complete 100 samples in 5.0s" warning
for the two 128x128 benchmarks, which it did on every CI run.

Neither neighbour was touched. `verification-gaps/15` (the `workflow_dispatch` trigger that always
fails) and `verification-gaps/09` (whether this comparison becomes a required status context) stay
open and are now cheaper to settle.

### The first comparison under this config crosses a baseline discontinuity

Recorded rather than corrected, and deliberately not a reason to change any figure above.

The point stored on `gh-pages` was produced under the old budget, where the discard run gave each
benchmark a 3s warm-up and a 5s measurement before the measured run gave it another 3s warm-up. The
first run under the new config is compared against that point. Counting both the discard run and the
measured run's own warm-up, total settling before the first measured sample falls from roughly 11s
per benchmark to roughly 1.1s.

Whether that moves a number is unmeasured. Nobody has reproduced the contamination locally: runs on
this machine read `parse_source` at its settled scale on a first pass after a fresh compile, and so
did the review's. The one documented case is the 1,204 ns against 417 ns the workflow comment
records, which is 2.9× — past the 150% alert and just under the 300% failure threshold. So the
effect is real enough to have been written down once, and its size on a hosted runner is not known.

What it is not is a permanent bias. It is a one-time step at the boundary: once `main` publishes a
point under the new config, every comparison after it is new against new, and the settling is
identical on both sides.

The experiment that settles it is this branch's own pull-request benchmark run. That job measures
the new config on a hosted runner and compares it against main's stored full-budget series, which is
exactly the crossing in question. It triggers here without arranging anything, because `mise.toml`
and `.github/workflows/bench.yml` are both in the workflow's path filter.

Two outcomes, and what each asks for:

- Within the runner's ordinary noise, no alert: the question is closed, and nothing changes.
- An alert on one or more benchmarks: raise `--warm-up-time` in both tasks — the two must move
  together, since the warm-up has to stay symmetric — and re-measure. The measurement budget and
  `--nresamples` are not implicated either way; settling is the only thing this crossing touches.
