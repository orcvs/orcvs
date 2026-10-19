# 01: Step one Tick back or forward, and jump to any Tick

**What to build:** While stopped, step forward or back one Tick, or jump to any
absolute Tick, by replaying from a base Source with MIDI silenced. See
`../spec.md`; its open questions are settled before this starts.

**Blocked by:** None (can start immediately once triaged)

**Status:** needs-triage

- [ ] Settle the spec's open questions and record them in an ADR, beside
      ADR 0076.
- [ ] A base Source captured at Tick `0`, and the record of live edits the ADR
      chooses.
- [ ] A replay that executes Ticks without sending MIDI or scheduling Note
      Offs, reaching the same Source as Playback reached.
- [ ] Commands for step forward, step back and jump to a Tick, with a stop's
      safety action on each.
- [ ] Checkpoints sized by a benchmark of replay cost over the shipped
      examples.
- [ ] Tests: a jump to Tick `N` matches running `N` Ticks; stepping back then
      forward returns the same Source; a replay sends no MIDI; a live edit
      replays as the ADR says.
- [ ] `CONTEXT.md` gains the navigation vocabulary.

## Comments
