# 06 — Run the merge tier for every commit that reaches main

**What to build:** Every commit on `main` gets a merge-tier run that actually executes. Two mechanisms prevent that today: pushes to `main` share one concurrency group, so a later merge cancels an earlier commit's run, and a manual dispatch produces a green run that executes no merge-tier step.

**Blocked by:** 02 — Refresh the tooling contract's stale action pins.

**Status:** in-review

- [x] A push to `main` cannot cancel an in-progress run for an earlier commit on `main`.
- [x] A manual dispatch runs the merge tier.
- [x] Pull-request runs still cancel superseded runs for the same pull request.
- [x] The contract pins the concurrency group and the conditions on the merge-tier steps.

## Comments

The concurrency group interpolates the pull request number, which is empty on a push, so every push to `main` collapses into one group. This is not hypothetical: the PR 2 merge commit landed with two of its three jobs cancelled and was never re-run. A cancelled run reports `cancelled` rather than `failure`, so nothing alerted and nobody looked.

The benchmark workflow already sets `cancel-in-progress: false` on its publish job for exactly this reason. The test workflow does not.

The dispatch hole is the same failure reached from the other side. Both merge steps guard on the event being a push, so dispatching the workflow — the obvious way to re-verify a commit whose run was cancelled — reports three green jobs having run only the pull-request tier. The benchmark workflow's equivalent guard tests that the event is not a pull request, and so covers dispatch correctly.

## Resolution

Both halves are closed in `.github/workflows/test.yml`, and the concurrency fix is belt and braces
rather than one change:

```yaml
group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

`github.sha` replaces `github.ref` as the fallback, so each commit on `main` gets a group of its own;
`cancel-in-progress` is then confined to pull requests, so even a shared group could not cancel a
push. A pull request keeps the behaviour it had — its runs share the pull request number's group and
supersede each other.

The merge-tier steps guard on `github.event_name != 'pull_request'`, the shape
`.github/workflows/bench.yml` already used, so a manual dispatch runs them.

`scripts/tests/check-tooling-contract.sh` gained four scenarios rather than trusting the assertions
to be right: a group keyed back on `github.ref`, unconditional cancellation, the merge steps guarded
on `== 'push'` again, and a WASM job that stops running on pull requests. Each is rejected.
