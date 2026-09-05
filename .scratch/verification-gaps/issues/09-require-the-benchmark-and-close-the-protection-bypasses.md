# 09 — Require the benchmark comparison and close the protection bypasses

**What to build:** The gates that guard `main` block a merge rather than advising one. The benchmark comparison fails on a threefold regression and blocks nothing, and a direct push to `main` skips every required context.

**Blocked by:** None — can start immediately. It changes repository settings, not code.

**Status:** ready-for-human

- [ ] The benchmark comparison is a required status context, or a recorded decision says why it stays advisory.
- [ ] Administrator enforcement and the review requirement are set deliberately, and the settings are recorded in this issue.
- [x] The intended protection is written down in the repository, so the intent survives the setting.

## Comments

Three contexts are required today. Neither benchmark job is among them, so the regression threshold produces a red check nobody has to clear. Administrator enforcement is off and no approving review is required, so a direct push to `main` bypasses all three — several pushes have, and some of those runs failed.

Making the benchmark required needs care. Its pull-request trigger is path-filtered, and a required context that is never triggered reports nothing and blocks a pull request indefinitely — unlike a job skipped by a condition, which reports as skipped and satisfies the context. A path-independent stub job is the usual answer, and choosing it deliberately is part of this ticket.

This one is a maintainer's to run: the settings need repository administration, and whether a single-maintainer repository wants a review requirement at all is a judgement rather than a defect.

## Progress — the written half is done, the settings half is the maintainer's

`docs/tooling.md` now records both bypasses in the tier description: that the benchmark comparison
fails on a threefold regression while not being a required status context, and that `main`'s
protection does not enforce against administrators, so a direct push skips every required context.
The intent is therefore in the repository whatever the settings say, and this file holds the decision.

The two remaining boxes need repository administration, which is not something an agent should reach
for. What is verified today:

```
required_status_checks.contexts   = ["full-gate", "macos", "wasm"]
required_status_checks.strict     = true
enforce_admins                    = false
required_approving_review_count   = 0
```

Neither `Benchmark` job is required.

Three notes for whoever runs this:

- Making the benchmark required needs a path-independent stub. `.github/workflows/bench.yml`'s
  pull-request trigger is path-filtered, and a required context that is never *triggered* reports
  nothing and blocks the pull request indefinitely — unlike a job skipped by an `if:` condition,
  which reports as `skipped` and satisfies the context. A stub job carrying the required context
  name, triggered unconditionally and skipping its body on the filtered paths, is the usual answer.
- `enforce_admins` and the review count are a judgement rather than a defect on a single-maintainer
  repository. The argument for turning admin enforcement on is that several direct pushes have
  bypassed all three contexts and some of those runs were red; the argument against a review
  requirement is that there is nobody else to give it.
- Issue 06 is the reason the third context now means something on every commit: before it, a push to
  `main` could have its run cancelled by the next merge and report `cancelled` rather than `failure`.

**Status stays `ready-for-human`.**
