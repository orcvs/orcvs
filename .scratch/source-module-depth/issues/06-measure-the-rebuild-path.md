# 06 — Measure the per-keystroke rebuild path

**What to build:** Real numbers for the work done on every accepted keystroke, recorded where the
next person can find them, so the claims already made about that path are either evidenced or
withdrawn.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The Source edit and rebuild benchmarks are run across their Grid sizes on a machine quiet
      enough for the intervals to be trustworthy, and the load conditions are stated.
- [ ] Numbers are captured for the commit before the Language Map rework and for the current tip,
      so the difference is attributable.
- [ ] The result is recorded in this issue, including a result that shows no improvement.
- [ ] Any performance claim that the numbers do not support is corrected at its source.

## Comments

Three commits on this branch changed that path and deliberately made no performance claim, because
every run that afternoon returned intervals wide enough to report both a large gain and a large
loss for the same code. The repository contract asks for a reproducible benchmark before claiming a
path got cheaper, so this is standing evidence debt rather than new work.

Guidance from the attempts already made: quit browsers and other agents first, use Criterion's
saved baselines rather than its implicit last-run comparison, and treat any run whose interval
spans more than a few percent as unusable rather than averaging it.
