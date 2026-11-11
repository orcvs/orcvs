# 04 — Drive a Chord Play from Source

Status: ready-for-agent

Blocked by: 03

**What to build:** Prove a Chord Play can be driven by Clocks and Reads, and that the nested tracker
and sequencer voice forms from ADR 0103 work. This is the first time the whole nested form runs in
the real interpreter, so a failure here reopens ADR 0103's width decision.

## Acceptance criteria

- [ ] Nested Reads and Track work in every Chord Play operand, including all six after channel, driven by a Clock across Ticks.
- [ ] Portal-written Cells mean what a Return means in root, set, voicing and inversion.
- [ ] `!~0877&v.+02~.011001` plays, Tick by Tick, what the tracker's current voice plays with the same pattern rows.
- [ ] The sequencer's nested voice plays, Tick by Tick, what its current voice plays.
- [ ] A chord tracker with root and set columns steps through a progression, with its nested Functions' Output Portal writes on the row south of the Play.
- [ ] The shipped tracker and sequencer examples are unchanged.
