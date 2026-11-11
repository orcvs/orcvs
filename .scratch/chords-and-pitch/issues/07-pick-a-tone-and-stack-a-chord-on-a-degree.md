# 07 — Pick a tone and Stack a chord on a degree

Status: resolved

Blocked by: 02, 03

**What to build:** Pick `#p root set index`, which answers one Note, and Stack `#s set degree size`,
which answers an Interval Set name, as `.scratch/chords-and-pitch/spec.md` defines them.

## Acceptance criteria

- [x] Pick answers the tone at `index`, and an index past the end of the Interval Set wraps up an octave.
- [x] A Pick answer above `7F` diagnoses.
- [x] Stack answers the name whose semitones match, for chords and scales.
- [x] Stack diagnoses when no name fits, including a major pentatonic stacked from C.
- [x] Each has arity, type, domain and boundary coverage, and works as a root and nested.
- [x] Pick in root and Stack in set compose in one Chord Play to sound a chord on a scale degree.
- [x] Both have Function reference rows with worked examples.
- [x] `CONTEXT.md` defines Pick and Stack.

## Decisions

- Pick's wrap is `root + set[index % n] + 12 * floor(index / n)`: one octave per pass through the
  set. Toby kept it after review showed that a set wider than an octave, such as `MD`, steps back
  down at the wrap (`#pC4MD05` answers A5, `#pC4MD06` answers C5). It is the same per-pass octave
  wrap that Tidal's `toScale'` and `scale`, Sonic Pi's `chord` with `num_octaves`, and
  SuperCollider's `performDegreeToKey` use.
