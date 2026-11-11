# 07 — Pick a tone and Stack a chord on a degree

Status: ready-for-agent

Blocked by: 02, 03

**What to build:** Pick `#p root set index`, which answers one Note, and Stack `#s set degree size`,
which answers an Interval Set name, as `.scratch/chords-and-pitch/spec.md` defines them.

## Acceptance criteria

- [ ] Pick answers the tone at `index`, and an index past the end of the Interval Set wraps up an octave.
- [ ] A Pick answer above `7F` diagnoses.
- [ ] Stack answers the name whose semitones match, for chords and scales.
- [ ] Stack diagnoses when no name fits, including a major pentatonic stacked from C.
- [ ] Each has arity, type, domain and boundary coverage, and works as a root and nested.
- [ ] Pick in root and Stack in set compose in one Chord Play to sound a chord on a scale degree.
- [ ] Both have Function reference rows with worked examples.
- [ ] `CONTEXT.md` defines Pick and Stack.
