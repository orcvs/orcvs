# 01 — Thread Tick and Position into interpretation

**What to build:** Supply each root evaluation with the absolute Tick and its Language Map anchor
Position while keeping the Source Snapshot and Tick Plan deterministic.

**Blocked by:** language-map/03 — Move Source consumers behind the Language Map.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Playback begins at Tick `0` and increments one unsigned counter per executed Tick.
- [ ] Source interpretation receives Tick explicitly rather than reading wall time or global state.
- [ ] Function evaluation receives its Grid-minted anchor Position from the Language Map.
- [ ] Live Editing affects the next unsampled Tick without resetting absolute Tick.
- [ ] Identical Source Snapshot, Tick, and inputs produce identical Tick Plans.
- [ ] Each new Playback run restarts at Tick `0`, matching ADR 0012's first-Tick rule.

## Comments

Tick belongs to one Playback run; tests should pin restart behavior without adding another clock
seam.
