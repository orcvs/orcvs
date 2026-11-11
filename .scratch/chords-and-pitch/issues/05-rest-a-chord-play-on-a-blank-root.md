# 05 — Rest a Chord Play on a blank root

Status: resolved

Blocked by: 01, 03

**What to build:** ADR 0104 for both Chord Plays. A fully blank root is a Rest by any route, and it
ignores other blank or cleared operands, so a wholly empty pattern row rests.

## Acceptance criteria

- [x] Both Chord Plays rest on a blank root by each of the three routes, sending nothing and diagnosing nothing.
- [x] A blank root with blank or cleared channel, velocity, set, voicing, inversion or length is still a Rest, and a Rest checks no operand's range. A root written in Source refuses an operand left blank in Source until the root is blank in Source, even on the Tick a Portal empties it; see Open question.
- [x] A half-written operand beside a blank root is diagnosed.
- [x] A blank Interval Set with a root present is diagnosed.
- [x] The Language Map admits a Chord Play with a blank root as a root.
- [x] `CONTEXT.md`'s Rest entry covers Chord Play.

## Open question

ADR 0104 says a blank root rests however it got blank and excuses its other blank operands. The
Language Map decides from Source alone, so where Source holds a written root beside a blank operand
it refuses the row, and the refusal holds the Turn on the Tick a Portal empties that root. The Tick
is silent and reports nothing, and once the root is blank in Source the row rests, so only the Map's
diagnostic before that Tick departs from the ADR. Honouring the ADR there means the Map admits a
Chord Play with a written root and a blank operand and leaves the refusal to the Turn, which drops
the Map's diagnostic for a blank Interval Set beside a written root that story 38 relies on.
