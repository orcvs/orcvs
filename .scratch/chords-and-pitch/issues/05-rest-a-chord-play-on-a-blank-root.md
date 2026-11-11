# 05 — Rest a Chord Play on a blank root

Status: resolved

Blocked by: 01, 03

**What to build:** ADR 0104 for both Chord Plays. A fully blank root is a Rest by any route, and it
ignores other blank or cleared operands, so a wholly empty pattern row rests.

## Acceptance criteria

- [x] Both Chord Plays rest on a blank root by each of the three routes, sending nothing and diagnosing nothing.
- [x] A blank root with blank or cleared channel, velocity, set, voicing, inversion or length is still a Rest, and a Rest checks no operand's range. A root written in Source refuses an operand left blank in Source in the Language Map, and rests on the Tick a Portal empties it; see Resolved: the Language Map and a Portal-emptied root.
- [x] A half-written operand beside a blank root is diagnosed.
- [x] A blank Interval Set with a root present is diagnosed.
- [x] The Language Map admits a Chord Play with a blank root as a root.
- [x] `CONTEXT.md`'s Rest entry covers Chord Play.

## Resolved: the Language Map and a Portal-emptied root

ADR 0104 is amended to the narrowed rule, pending Toby's confirmation on PR #369. The Language Map
reads Source as written, so where Source holds a written root beside a blank operand it refuses the
row under ADR 0069's unwritten-operand rule. A Map refusal holds a Turn only while its operands hold
what Source held when the Map was built, so on the Tick a Portal empties that root the Turn runs and
rests, sending nothing and reporting nothing, and the Map rebuilt after it admits the row. Only the
Map's diagnostic before that Tick departs from the ADR's Decision. The alternative is for the Map to
admit a Chord Play with a written root and a blank operand and leave the refusal to the Turn, which
drops the Map's diagnostic for a blank Interval Set beside a root written in Source.
