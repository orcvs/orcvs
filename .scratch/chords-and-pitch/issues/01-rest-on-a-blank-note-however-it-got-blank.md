# 01 — Rest on a blank note, however it got blank

Status: ready-for-agent

Blocked by: None — can start immediately

**What to build:** ADR 0104 for Raw, Timed and Monophonic Play. A fully blank note operand is a Rest
whether the Source was written blank, a Portal write emptied the slot, or a nested Function answered
a Clear. The Language Map treats such a Play as a root.

## Acceptance criteria

- [ ] Each of the three Plays rests, sending nothing and diagnosing nothing, by each of the three routes.
- [ ] A half-written note is still diagnosed by each route.
- [ ] A blank note beside a blank or invalid channel, velocity or length is diagnosed, as ADR 0075 says.
- [ ] The Language Map admits a Play whose only blank operand is its note as a root, and still excludes other blank operands.
- [ ] A Monophonic Play that rests leaves its channel's voice sounding to its own Note Off.
- [ ] The shipped example tests pass, and the Melody example no longer diagnoses its blank steps.
- [ ] Tests cover lang, Tick and Playback at the level `.scratch/chords-and-pitch/spec.md` asks.
- [ ] The Melody example's rest section and the `CONTEXT.md` entries for Rest and Clear match the behaviour.
