# 13 — Reorder a Tick incrementally as roots join

Status: needs-triage

**What to build:**

A Tick with dynamic writers still grows about 14x per doubling of the Grid's side in `source_execute_tick_dynamic_writers` (`orcvs/benches/source.rs`): 15.8 µs at 16x16 to 13.9 ms at 128x128, measured locally. Each rebuild of the order, on every root that joins the Tick and every Read that waits at its Turn, still costs O(N): building `Dependencies`, `cycle_closure` twice, `take_ready`, filtering `schedule.outgoing`, and building `incoming` in `writers_first` (`orcvs/src/source/tick/ordering.rs`). The number of rebuilds grows with the voices, so the Tick is quadratic in them. Reordering only what a joined root or a wait changes would remove the O(N) per rebuild.

## Acceptance criteria

- [ ] Decide whether the cost matters at the shipped 256x256 Grid with a realistic voice count, measured with the series above, before changing the scheduler.
- [ ] If it does: each rebuild costs time proportional to what the join or wait changed, the series shows sub-quadratic growth, and the order every existing ordering, write, read and push test pins is unchanged.
