# 07 — Hold a named benchmark to a recorded floor

**What to build:** A check beside the ratio gate that fails when a named benchmark exceeds a figure
committed in the repository, so an accepted baseline move leaves an enforced floor behind instead of
only a prose note.

**Blocked by:** 03 — Gate merges on the benchmark series.

**Status:** ready-for-agent

- [ ] A committed file names each guarded benchmark and the figure it must not exceed, with the
      runner the figure was measured on.
- [ ] A check compares the run's result for each named benchmark against its figure and fails when
      the figure is exceeded.
- [ ] The check reads the same `mise run bench` output the ratio gate reads; it does not run the
      benchmarks a second time.
- [ ] Raising a figure is an edit to the committed file, visible in the diff a reviewer approves.
- [ ] A benchmark absent from the file is not guarded and does not fail the check.
- [ ] The check reports which benchmark exceeded which figure, and by how much.
- [ ] `mise run bench` and the scoped Rust gates pass.

## Comments

### Why this is not what issue 03 built, 2026-09-05

`benchmark-action/github-action-benchmark` compares each run against the previous point stored for
`main` and offers nothing else. It has `alert-threshold`, `fail-threshold`, `fail-on-alert`,
`comment-on-alert`, `save-data-file` and `external-data-json-path`, and no input, commit-message
flag, or label that accepts a known change. Its README documents no way to approve a regression or
reset a baseline, and no guidance on an intentional slowdown. The model is that the baseline moves
forward with the series, so merging *is* the acceptance, implicitly and silently.

That model has one defect, and `sequence-values/02` is the first change to hit it. That branch moved
`execute` from 19 ns to 106 ns on the runner, a ratio of 5.58 against a failure threshold of 3.00;
two fixes brought it to 70 ns and 3.68, which still fails. It was merged deliberately — the gate is
not a required check, the cost is understood, and closing it means reshaping a seam in a way that
would make ADR 0026 harder to adopt. The reasoning is recorded in `03` and in `sequence-values/03`.

The defect is what happens next. `main` republishes at about 70 ns, the next pull request compares
against 70 ns rather than 19 ns, and the gate never mentions the cost again. Nothing enforces a
return. If the ADR 0026 revisit lands and leaves `execute` where it is, no check says so. A prose
note is the only trace, and prose does not fail a build.

A floor is the smallest thing that fixes it. It answers a different question from the ratio: the
ratio asks "did this change make it worse", the floor asks "is it still as good as we agreed it
should be". Both are wanted. The ratio catches a sudden regression that no one intended; the floor
catches a slow drift, and a deliberate move that was meant to be temporary.

Two notes for whoever builds it:

- Keep it out of the action. The action owns the series and the ratio; this reads the same output
  and answers its own question. Putting a floor inside a tool that has no concept of one means
  fighting it.
- Record the runner beside each figure. This branch measured 4.03x locally where CI measured 5.58x,
  and 3.27x locally where CI measured 3.68x. A figure without the machine it came from is not
  reproducible, and a contributor comparing a local run against a CI floor will be misled.

Not in scope: changing the ratio gate's thresholds, and replacing the action. `bencherdev/bencher`
models baselines and thresholds per branch and would subsume both checks, but replacing a gate that
works on the evidence of one occurrence is not warranted.

### Second occurrence, and a worse one, 2026-09-20

The defect above predicted the mechanism from one case. Here is the second, found while opening
pull request #111.

The Benchmark run on `main` at `22ca2718` — the merge of #109, `syntax-highlighting` `01`–`10` —
**failed**, alerting on all ten `paint_derive` points and several `paint_background_runs` ones.
`paint_derive/fitted/256x256` went 629,884 to 3,232,981, a ratio of 5.13; the culled points moved
by about the same factor. `console/benches/paint.rs` is unchanged across those commits, so the
fixture did not move.

Then exactly what this issue describes happened. `main` republished at about 3.2M, and #106, #110
and #111 have all read green since, each compared against the elevated point rather than the 630k
it replaced. #111 measured 3,190,267 and the gate said success. The regression is now invisible to
the check that caught it, and the only trace is a red run in the history that nothing points at.

Two ways this occurrence differs from `sequence-values/02`, both of which sharpen the case:

- **It was not a deliberate acceptance.** `sequence-values/02` was merged knowingly, with the cost
  understood and recorded in `03` and `sequence-values/03`. This one appears to have merged without
  the failure being read: the ticket that landed it records no performance risk, and
  `syntax-highlighting/07` went on to ask, as an open follow-up, whether the paint bench had moved.
  It had already failed the gate by 5.13x when that question was written.
- **It exceeded the fail threshold, not just the alert one.** 5.13 against `fail-threshold: 300%`.
  A floor would have kept the old figure enforced; the ratio gate's own failure did not survive the
  merge.

So the floor this issue builds wants `paint_derive` among its first guarded names, and whoever
raises its figure has to do so in a diff. `paint-cell-cost/01` profiles the regression itself; this
issue is the mechanism that would have stopped it becoming the baseline.
