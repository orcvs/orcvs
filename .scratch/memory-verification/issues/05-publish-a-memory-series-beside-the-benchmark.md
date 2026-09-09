# 05 — Publish a memory series beside the benchmark

**What to build:** The allocation counts from `01` and `02` are tracked over time next to the
existing timing series, so a slow creep that no single assertion catches is visible as a trend.

**Blocked by:** 01 — Count allocations on the Tick and Render Frame paths; 02 — Count allocations on the Source write and Language Map rebuild.

**Status:** ready-for-human — everything is written, and every local gate the change can reach
was run and passed. The two workflow steps themselves have never run in CI: nothing appends the
first point to the `memory` series until a push to `main` runs the publishing job.

- [x] The existing pinned `github-action-benchmark` is reused with `tool: customSmallerIsBetter` and
      its own `name`. No new action, no new pin. The action merges multiple named series into the one
      gh-pages branch, which is the documented pattern for exactly this.
- [x] The new series `name` is chosen once and documented as never changing, the way the existing
      series' name already is.
- [x] The measurement that feeds the series is the same one the assertions use. A separate binary
      that re-runs the measured paths is not acceptable, because the published number and the
      asserted number would drift apart silently.
- [x] The publishing job produces the JSON with cargo alone, so it keeps installing no other tooling.
- [x] The second publishing step sets `skip-fetch-gh-pages`, since the first step in the same job has
      already fetched and pushed the branch.
- [x] No warm-up run for the memory series, and the reason is stated: allocation counts are
      deterministic for a fixed input, so the warm-up the timing series needs would double the cost
      for no signal.
- [x] The series starts with alerts and a job summary and does not fail the workflow. Thresholds are
      set far tighter than the timing series' because the metric is deterministic.
- [x] The path filter is checked rather than assumed: it already covers the benchmarked crates and
      the manifests, so it likely needs no change.
- [x] `actionlint`, `zizmor`, and `bash scripts/check-tooling-contract.sh` are run and their results
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

### Built, and the shape it took (2026-09-09)

The emission lives in the tests that already assert, gated on `ORCVS_MEMORY_SERIES=1`. Each test
prints one `customSmallerIsBetter` object per measurement, on its own line, behind the literal
marker `ORCVS-MEMORY-SERIES`; the workflow greps the marker out of `cargo test -- --nocapture`,
sorts, joins with commas and wraps the result in an array. Twenty-one records, twenty-one series.

**The measurement command is `cargo test`, not `cargo nextest run`, and that is forced.** Both bench
jobs run `jdx/mise-action` with `install: false` precisely so the job installs no cargo tool beyond
the toolchain, so `cargo-nextest` is not present in the job that publishes. This works only because
`01` chose `const`-initialised thread-local counters: under a bare `cargo test` the tests of one
binary share a process and run on separate threads, which is the case a thread-local is correct for
and an `AtomicUsize` would not be. Verified for both crates —
`cargo test --package lang --test allocation --locked` and the same for `orcvs` both pass, as does
the two-package invocation the workflow runs.

**What is published, and what deliberately is not.** The quantities are the ones `02` named: one
Cell write at 16x16, 32x32 and 64x64 with the Grid empty and with it populated, blocks and bytes
each, and the carried Expression count beside every populated point — it is the divisor that says
whether a moved number is a regression or a moved fixture. `lang` contributes the Tick over the
fixture and over the fixture written four times, and one Render Frame re-read. Three measurements
are measured and not published: `with_empty_rows` and `empty` are each asserted *equal* to a point
already published, so they would store one number under two names, and `nothing` is asserted to be
zero — a zero point leaves the action's ratio against the previous one undefined.

**Every published value matches the numbers `01` and `02` recorded**, which is the check that the
emission reports the measurement rather than something beside it: 11 blocks / 816 bytes for the
Tick, 44 / 3264 for four times the Expressions, 24 / 48 for the re-read, 16 blocks at every empty
Grid size, and 79 / 47971 at 43 Expressions, 247 / 173913 at 160, 928 / 658728 at 621 populated.

**The path filter needed no change, and it was read rather than assumed.** Both triggers filter on
`lang/**` and `orcvs/**`, which contain `lang/tests/allocation.rs` and `orcvs/tests/allocation.rs`;
`Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` cover what they compile under; and
`.github/workflows/bench.yml` covers the emission and the thresholds themselves. The comment on both
copies of the filter now says so, so the next reader does not have to re-derive it.

**Thresholds and failure.** `110%`/`125%` against the timing series' `150%`/`300%`, with
`fail-on-alert: false` and `summary-always: true`. The `fail-threshold` is recorded now so that
turning failing on is one line rather than a second decision. `verification-gaps/09` still owns
whether this workflow blocks a merge; the memory series is not a required context and does not
become one here.

**JSON with coreutils only.** `grep`, `cut`, `sort`, `uniq`, `paste`, `printf` — no `jq`, no new
entry in `mise.toml`. The assembling step also rejects an empty record set and two records sharing a
series name, the second by reading each name back with `cut -d'"' -f4`; that works because the names
are built from literals and decimal Grid dimensions and hold no character JSON would escape, which
is stated where they are built.

**Pinned in `scripts/check-tooling-contract.sh`**, in the existing style, with every count derived
from `workflow_job_count` rather than from a literal so a third bench job cannot arrive with half of
this: the measurement command verbatim, that `cargo nextest run` appears nowhere in the workflow,
that `mise run bench > /dev/null` and the measurement each appear once per job (which is what pins
"no warm-up for the memory series"), the series names `memory` and `lang`, both `tool:` values,
`skip-fetch-gh-pages`, `fail-on-alert: false`, all four thresholds, the `printf`-assembled array,
the absence of `jq`, and four SHA-pinned uses of the one benchmark action. Each new assertion was
negative-tested against a mutated fixture copy and each rejects the mutation.

### Not done here

The series has no points yet, so there is nothing to judge stability against and `fail-on-alert`
stays `false`. Turning it on is the follow-up the spec already describes, and it wants a handful of
points on `main` first.
