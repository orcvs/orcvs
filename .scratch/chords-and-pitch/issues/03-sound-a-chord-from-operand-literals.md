# 03 — Sound a chord from Operand Literals

Status: ready-for-agent

Blocked by: 02

**What to build:** ADR 0103's Raw Chord Play `!:` and Timed Chord Play `!#`, with Voicing and
inversion, played from Operand Literals. One Turn emits one ordinary Raw or Timed Play Command per
tone, lowest first, and the Evaluator's answer stays bounded and allocation-free.

## Acceptance criteria

- [ ] Each operand reaches the role its signature names, and each Chord Play requires exactly its declared operands.
- [ ] Channel, velocity and length take Play's domains, and an out-of-domain operand produces no Play Command at all.
- [ ] Every Voicing, alone and with inversion, gives the expected tones, applying removal, then inversion, then movement.
- [ ] Too few tones for a Voicing, or any tone outside `00`–`7F`, diagnoses the whole chord and sends nothing.
- [ ] A Bang activates a Chord Play once with its tones in order in the Tick Plan, and an unactivated one sends nothing.
- [ ] A Chord Play is invalid where a value is required.
- [ ] Playback: each Timed tone stops at its own Note Off, velocity `00` stops every tone, length `00` starts nothing, a repeated chord replaces the tones it repeats, and the safety action clears a sounding chord.
- [ ] Playback: a chord and a Monophonic Play on one channel keep separate ownership, and a Raw Chord Play delivers one Note On per tone.
- [ ] The allocation tests still assert no allocation, with a twelve-tone chord in the fixture.
- [ ] Both Functions have Function reference rows with worked examples, including what the digit in a drop voicing means.
- [ ] `CONTEXT.md` defines Chord, Inversion, Raw Chord Play and Timed Chord Play, and amends Terminal Output Function.
