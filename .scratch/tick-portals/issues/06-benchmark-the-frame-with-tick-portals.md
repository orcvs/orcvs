# 06: Benchmark the Render Frame with Tick Portals

**What to build:** A benchmark of deriving the Render Frame with a full record
present (reads and writes, current and retained), beside the existing paint and
Source benchmarks, so CI measures the cost the record adds.

**Blocked by:** 03, 05

**Status:** ready-for-agent

- [ ] A benchmark derives a frame from a Source whose last Tick read and wrote
      across the Grid, current and retained.
- [ ] No performance claim is made beyond what the benchmark measures.
