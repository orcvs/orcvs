# 04 — Assert wasm linear memory settles after warm-up

**What to build:** A long run of the web application in the browser suite leaves the wasm linear
memory at the size it reached after warm-up, so a leak on the web target is caught by the merge tier
instead of by a user's tab.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Linear-memory size is sampled in the browser regression suite, after a warm-up run and again
      after a long sequence of Source writes and Ticks, and the two are asserted equal.
- [ ] The warm-up length and the run length are chosen so the first sample is past allocator growth,
      and the choice is stated.
- [ ] The assertion states why monotonicity makes this a sound leak signal: freed memory returns to
      the allocator's free list rather than to the host, so linear memory never shrinks and any
      growth after warm-up is real growth.
- [ ] The test runs inside the existing headless browser task in the merge tier. No new task, no
      workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

Independent of every other issue in this effort. It needs no counting allocator, no dependency, and
nothing from `01`, because the measurement is a stable standard-library call on the wasm target.

It belongs in the merge tier rather than the pull-request tier for two reasons: the pull-request tier
does not run the browser suite at all, and the assertion wants a long run.

Neither Miri nor any sanitizer reaches wasm, so this is the only memory signal the web target gets.
