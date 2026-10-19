# 02: Replay the Source to any Tick

**What to build:** The engine can reach any absolute Tick by replaying from
the base Source, silently, as issue 01's ADR decides. No command reaches it
yet. See `../spec.md`.

**Blocked by:** 01

**Status:** needs-triage

- [ ] A base Source captured before Tick `0` executes, and the record of live
      edits the ADR chooses.
- [ ] A replay that executes Ticks without sending MIDI or scheduling Note
      Offs, and commits the last replayed Tick's Tick Portals record.
- [ ] Tests: over every shipped example, a replay to Tick `N` reaches the
      Source, Tick Plan and Tick Portals record that running `N` Ticks
      reaches; a replay sends no MIDI; a live edit replays as the ADR says.

## Comments
