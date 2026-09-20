# 14 — Bring the macOS tier back inside its budget

**What to build:** A pull-request tier whose macOS job finishes with room to spare, and which reports a failure when it does not.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The macOS job's runtime is measured, and the dominant cost is named rather than guessed.
- [ ] That cost is reduced, or the budget is raised with a recorded reason for the new number.
- [ ] A job that exceeds its budget is distinguishable from one that was superseded.

## Comments

`08-memory-verification` landed on `main` as `f25bd09`. Its first pull-request run failed: the
`macos` job hit `timeout-minutes: 20` inside `mise run check_pull_request` and was killed. A re-run
of that job alone passed in 17m14s, and the pull request merged green.

Seventeen of twenty is not a pass with room in it. For comparison, the same tier on `main`:

```
main f131e5e   macos   2m28s    warm cache
main 72bb2cc   macos   8m25s    cold: the 1.98.0 -> 1.98.1 bump rebuilt everything
PR   8ef031d   macos   >20m     killed at the budget
PR   8ef031d   macos   17m14s   the re-run that merged
```

So the branch did not merely meet a cold cache. `72bb2cc` was the cold case and finished in 8m25s
with the same toolchain, and it saved the macOS cache that `8ef031d` then restored. Whatever the
extra nine minutes are, they are work the branch added, and they are now on `main` and paid by every
pull request.

Two candidates, and the point of the first checkbox is to measure rather than assume:

- `scripts/tests/check-tooling-contract.sh` runs roughly eighty scenarios, and every one of them
  calls `make_fixture`, which does a `git init` and then a full pass of
  `scripts/check-tooling-contract.sh` over the fixture. `08-memory-verification` added nine more
  fixtures to it. Most scenarios never ask git anything — only the `proptest-regressions`
  assertions need a work tree — so the `git init` may be avoidable for nearly all of them, and the
  fixture itself is a directory of copied files that could be built once and mutated per scenario.
- The branch added two integration test binaries, `lang/tests/allocation.rs` and
  `orcvs/tests/allocation.rs`. `check_pull_request` compiles all targets twice, once for default
  features and once for `persistence`, and runs `nextest` twice.

The third checkbox is separate from the first two and is the reason this is a verification gap
rather than a performance ticket. A job killed by `timeout-minutes` reports `cancelled`, not
`failure`. That is the same signal a superseded run gives, and this repository has already been
bitten by it once: `09-require-the-benchmark-and-close-the-protection-bypasses.md` and the
concurrency-group assertion in `scripts/check-tooling-contract.sh` both exist because `dd20cba6`
landed with two of three jobs cancelled and nothing alerted. The macOS timeout above was found by
reading the run, not by being told about it.

Raising `timeout-minutes` on its own would satisfy nobody: it buys time without saying what the
time is for, and it leaves the reporting gap in place.
