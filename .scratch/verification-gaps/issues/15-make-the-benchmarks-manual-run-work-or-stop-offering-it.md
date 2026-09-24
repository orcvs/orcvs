# 15 — Make the benchmark workflow's manual run work, or stop offering it

**What to build:** A `bench.yml` whose advertised triggers are the ones it can actually serve.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A manual run of the benchmark workflow either produces a comparison, or is not offered.
- [ ] Whichever way it resolves, the workflow's comment and `docs/tooling.md` say the same thing the file does.

## Comments

The `pull-request` job in `.github/workflows/bench.yml` is guarded by

```yaml
if: github.event_name == 'pull_request' || (github.event_name == 'workflow_dispatch' && github.ref != 'refs/heads/main')
```

and its comment reads "Measures pull requests and manual branch runs against the last point stored
by main." The manual half has never worked. Dispatched against `08-memory-verification`, run
`34299920158` got through checkout, the warm-up, and the benchmarks, and then failed:

```
##[error]No commit information is found in payload: {
  "ref": "refs/heads/08-memory-verification",
  "workflow": ".github/workflows/bench.yml",
  "inputs": null,
  ...
```

`benchmark-action/github-action-benchmark` labels each stored point with a commit, and it reads that
from `payload.head_commit` on a `push` or `payload.pull_request` on a `pull_request`. A
`workflow_dispatch` payload carries neither, so the action aborts before comparing anything. The job
spends the full warm-up and benchmark run — about twelve minutes — and then always errors. It is not
specific to this branch and it is not new: nothing about the guard or the action pin has changed.

The failure also takes the three memory-series steps with it, since they run after the timing
comparison in the same job and are skipped once it fails. A manual dispatch therefore cannot be used
to exercise the memory series either, which is how this was found.

There is a supported fix. The pinned action, `52576c9` (v1.22.1), takes a `ref` input described as
"optional Ref to use when finding commit", which exists for events whose payload has no commit.
Passing `ref: ${{ github.sha }}` should let it resolve the commit through the API. One thing to check
before relying on it: that lookup may require `github-token`, which this job deliberately does not
set — "the action reads gh-pages anonymously on a public repo, and comment-on-alert would require
one" — so confirm whether the anonymous path works before adding a token back.

The alternative is to drop `workflow_dispatch` from the job's `if:` and from the workflow's
triggers, which makes the file honest and costs nothing that works today. Either resolution is fine;
what should not survive is a trigger the workflow offers, spends twelve minutes on, and always fails.

If the trigger is kept, note that `scripts/check-tooling-contract.sh` pins much of this workflow's
shape and will need the same treatment as the rest.

**2026-09-24 — out of the release.** `v1-release/03` requires the nominated candidate to carry its own push-triggered Benchmark run, so release evidence does not depend on manual dispatch. This issue stays open as post-v1 work.
