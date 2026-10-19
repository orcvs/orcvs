# 04: Bound a jump's cost with checkpoints

**What to build:** Source checkpoints every so many Ticks, so a jump replays
from the nearest one rather than from the base, at an interval a benchmark
chooses. See `../spec.md`, "Cost".

**Blocked by:** 02

**Status:** needs-triage

- [ ] A benchmark of replay cost over the shipped examples, at several Tick
      counts.
- [ ] Checkpoints at the interval it supports, and their memory bound.
- [ ] Tests: a jump from a checkpoint reaches what a replay from the base
      reaches.

## Comments
