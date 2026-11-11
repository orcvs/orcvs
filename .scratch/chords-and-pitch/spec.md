# Chord Play and the Pitch family

Status: ready-for-agent

Settled in design on 2026-11-10. ADRs 0103, 0104 and 0105 record the decisions. Where
`docs/research/chords-and-patterns.md`, `docs/research/timing-operators.md` or
`.scratch/timing-and-chords/handoff.md` disagree with this spec, this spec is right.

## Problem Statement

A performer can only sound a chord by writing several Play roots that one Bang activates, with one
Read per tone if the notes come from a pattern. A progression is expensive to write and awkward to
change, because changing a chord's type means editing several roots. Nothing derives a Note from
another Note: transposing goes through Number and back, and there is no way to ask for a scale
degree or a chord built on one.

A Read or Track also behaves differently depending on layout. A nested Read of two empty Cells
rests, while the same Read standing above the note slot is diagnosed on every empty step.

## Solution

A Chord is a root Note and an Interval Set, two ordinary pairs of Cells a Clock and a Read can
drive. Raw Chord Play `!:` and Timed Chord Play `!#` sound one in a single Expression, with a
Voicing and an inversion. A new Pitch family derives Notes and Interval Sets: Pick `#p`, Up `#+`,
Down `#-` and Stack `#s`. A fully blank note or root is a Rest however it became blank.

A progression needs no new Function: a Clock drives Reads of a column of roots and a column of
Interval Sets.

## User Stories

### Sounding a chord

1. As a performer, I want to write a root and an Interval Set in one Expression, so that one Bang sounds the whole chord.
2. As a performer, I want Raw Chord Play `!: channel velocity root set voicing inversion`, so that I can sound a chord with no length.
3. As a performer, I want Timed Chord Play `!# channel velocity root set voicing inversion length`, so that every tone stops after the length I wrote.
4. As a performer, I want each tone to behave as its own Note Play, so that what I know about Raw and Timed Play stays true for chords.
5. As a performer, I want tones sent lowest first, so that the order is predictable.
6. As a performer, I want velocity `00` to stop every tone, so that a chord stops as a note does.
7. As a performer, I want length `00` on a Timed Chord Play to start no tone, so that it matches Timed Play.
8. As a performer, I want a repeated Timed Chord Play to replace the tones it repeats, so that overlapping chords behave as overlapping Timed Plays do.
9. As a performer, I want a tone outside `00`–`7F` to diagnose the whole Chord Play, so that I never hear part of a chord.
10. As a performer, I want a channel, velocity or length outside its domain to diagnose and send nothing, so that Chord Play's operands match Play's.
11. As a performer, I want a Chord Play to be invalid where a value is required, so that it follows every other Terminal Output Function.
12. As a performer, I want several Play roots one Bang activates to keep working, so that I can still give tones different channels, velocities or lengths.

### Interval Sets

13. As a performer, I want 25 chord names, so that I can write the chords I use without working out semitones.
14. As a performer, I want 14 scale names, so that Pick and Stack can use a scale.
15. As a performer, I want every name to fill both Cells, so that half-typed input is still recognisably partial.
16. As a performer, I want a scale name to be valid in a Chord Play, so that one type serves both and `C4MA` sounds all seven tones.
17. As a performer, I want an unknown name to diagnose, so that a typing mistake is reported.
18. As a performer, I want to Copy, Read, Track, Write and Push an Interval Set, so that I can drive it from a pattern.
19. As a performer, I want an Interval Set in Source Paint in its own operand colour, so that I can see which pairs are Interval Sets.

### Voicing and inversion

20. As a performer, I want `CL` to leave the tones where the Interval Set puts them, so that I have a neutral voicing.
21. As a performer, I want `OP` to raise every second tone an octave, so that I can open a chord up.
22. As a performer, I want `D2`, `D3`, `D5` and `D6`, so that I can use drop voicings, with the digit being the sum of the positions dropped.
23. As a performer, I want `SH` to keep the 1st, 2nd and 4th tones, so that I can write a shell.
24. As a performer, I want `RL` to remove the 1st tone, so that I can leave the root to a bass voice.
25. As a performer, I want an inversion Number that moves the lowest tone up an octave that many times, so that I can invert any chord.
26. As a performer, I want tones removed, then inverted, then moved, so that voicing and inversion compose the same way every time.
27. As a performer, I want an Interval Set with too few tones for its Voicing to diagnose the whole chord, so that a voicing never silently means something else.
28. As a performer, I want to Copy, Read and Track a Voicing, so that I can change it from a pattern.

### Driving a chord from Source

29. As a performer, I want nested Reads in root and set, so that a Clock can step through a progression.
30. As a performer, I want nested Reads in every operand after channel, so that a pattern can vary velocity, voicing, inversion and length by step.
31. As a performer, I want a Track in root or set, so that a one-row List can hold a progression.
32. As a performer, I want Portal-written Cells in root and set to mean what a Return means, so that layout does not change behaviour.
33. As a performer, I want the tracker and sequencer voices to work in the nested form, so that a chord patch can build on them.

### Rest

34. As a performer, I want a fully blank note to be a Rest on Raw, Timed and Monophonic Play however it got blank, so that layout does not decide whether I get a diagnostic.
35. As a performer, I want a fully blank root to be a Rest on both Chord Plays, so that an empty pattern step is silent.
36. As a performer, I want a blank root to ignore other blank or cleared operands, so that a wholly empty pattern row rests.
37. As a performer, I want a half-written pair to stay an error, even beside a blank root, so that half-typed input is reported.
38. As a performer, I want a blank Interval Set with a root present to stay an error, so that an incomplete chord is reported.
39. As a performer, I want a note Play with a blank note to still need its other operands to decode, so that ADR 0075 stands for Plays.
40. As a performer, I want a Rest to send nothing and diagnose nothing, so that resting costs nothing.

### Pitch family

41. As a performer, I want Pick `#p root set index` to answer one Note, so that one Function arpeggiates a chord and picks a scale degree.
42. As a performer, I want an index past the end of the Interval Set to wrap up one octave per pass, so that an arpeggio climbs through any set whose entries are all below `12`; a set wider than an octave steps back down at the wrap.
43. As a performer, I want Up `#+ note number` and Down `#- note number`, so that I can transpose a Note without converting through Number.
44. As a performer, I want a Pick, Up or Down answer outside `00`–`7F` to diagnose, so that a Note never wraps silently.
45. As a performer, I want Stack `#s set degree size` to answer the Interval Set name those tones form, so that I can build the chord on a scale degree.
46. As a performer, I want Pick and Stack to compose in one Chord Play, so that a root from Pick and a set from Stack give a chord on a degree.
47. As a performer, I want Stack to diagnose when no name fits, so that an unnameable stack is reported.
48. As a performer, I want Pitch Functions to nest and write through their Output Portal as any value Function does, so that nothing about them is special.

### Finding out how it works

49. As a performer, I want every new Function in the Function reference with a worked example, so that I can learn it the way I learn the others.
50. As a performer, I want the Function reference to say what the digit in a drop voicing means, so that `D5` and `D6` can be understood.
51. As a performer, I want the Melody example's explanation of rests to match the new behaviour, so that the example stays true.

## Implementation Decisions

- ADR 0103 decides Chord Play, ADR 0104 the unified Rest, and ADR 0105 the Interval Set and Voicing literals. The Pitch family is decided here.
- Spellings are final: `!:` Raw Chord Play, `!#` Timed Chord Play, `#` Pitch family, `#p` Pick, `#+` Up, `#-` Down, `#s` Stack.
- `.` stays the numeric family only. A Function deriving Notes or Interval Sets belongs to `#`. Chord Play stays in `!` with the other Terminal Output Functions.
- Pulse is not a value or operand type. A Pulse Function is one that answers a Bang or a Clear each Turn.
- The Function table gains six rows. Each declares its operands, and both Chord Plays declare the root as their rest operand.
- Atom gains Interval Set and Voicing. The literal decoding and the reading a Copy, Read or Track performs both learn them, so `M7` and `CL` are no longer refused.
- The Evaluator's answer for a Terminal Output Function can be several Play Commands, bounded at twelve tones, and evaluation still allocates nothing. Chord Play emits only ordinary Raw and Timed Play Commands, so the Playback Engine and the output adapter have no new behaviour.
- Tick execution's operand decoding treats a plain blank slot in the rest operand as a Rest, and a blank root on a Chord Play leaves other blank or cleared operands undiagnosed.
- The Language Map admits an Expression whose only blank operand is its rest operand, or a Chord Play with a blank root, as a root. Half-written operands stay excluded.
- Source Paint gains an operand colour for each new literal type.
- Pick answers `root + set[index % n] + 12 * floor(index / n)`, where `n` is the size of the Interval Set. Index `00` is the root, and every index from `00` to `FF` is valid.
- Stack counts positions as Pick does, from `00` at the first tone of `set`. It takes `size` tones at positions `degree`, `degree + 2`, and so on, wrapping up octaves as Pick does, and subtracts the first tone. It answers the Interval Set with exactly those semitones, and diagnoses if there is none. Every `degree` and `size` from `00` to `FF` is read: a degree past the end of the set wraps, and a size larger than the set's tone count takes wrapped tones. A `size` of `00`, or one larger than the longest name's twelve tones, matches no name and diagnoses.
- A Chord Play's inversion is a Number from `00` to `FF`. Each step moves the lowest tone up an octave, so an inversion larger than the tone count keeps moving tones and climbs further octaves: `C4M7` inverted `05` is E5, G5, B5 and C6. A tone that ends above `7F` diagnoses the whole chord.
- `CONTEXT.md`'s Rest, Clear and Pending entries state ADR 0104 for Raw, Timed and Monophonic Play. It gains Chord, Interval Set, Voicing, Inversion, Raw Chord Play, Timed Chord Play, Pitch Function, Pick, Up, Down and Stack as each is implemented, and Atom, Terminal Output Function, Rest, Clear, Pending and Return are amended for them then.
- The semitone tables below are normative. `IntervalSet` in `lang/src/atom.rs` declares the same values, and a test there decodes every name to them:

| Names | Semitones | Names | Semitones |
| --- | --- | --- | --- |
| `M_` | 0 4 7 | `9_` | 0 4 7 10 14 |
| `M7` | 0 4 7 11 | `13` | 0 4 7 10 14 21 |
| `M9` | 0 4 7 11 14 | `s2` | 0 2 7 |
| `MD` | 0 4 7 11 14 21 | `s4` | 0 5 7 |
| `M6` | 0 4 7 9 | `+_` | 0 4 8 |
| `69` | 0 4 7 9 14 | `a9` | 0 4 7 14 |
| `+M` | 0 4 8 11 | `MA` | 0 2 4 5 7 9 11 |
| `m_` | 0 3 7 | `mi` | 0 2 3 5 7 8 10 |
| `m7` | 0 3 7 10 | `do` | 0 2 3 5 7 9 10 |
| `mM` | 0 3 7 11 | `ph` | 0 1 3 5 7 8 10 |
| `m6` | 0 3 7 9 | `ly` | 0 2 4 6 7 9 11 |
| `m9` | 0 3 7 10 14 | `mx` | 0 2 4 5 7 9 10 |
| `mB` | 0 3 7 10 14 17 | `lo` | 0 1 3 5 6 8 10 |
| `mD` | 0 3 7 10 14 21 | `hm` | 0 2 3 5 7 8 11 |
| `o_` | 0 3 6 | `mm` | 0 2 3 5 7 9 11 |
| `o7` | 0 3 6 9 | `p5` | 0 2 4 7 9 |
| `h7` | 0 3 6 10 | `p3` | 0 3 5 7 10 |
| `b9` | 0 4 7 10 13 | `bl` | 0 3 5 6 7 10 |
| `7_` | 0 4 7 10 | `wt` | 0 2 4 6 8 10 |
| | | `ch` | 0 1 2 3 4 5 6 7 8 9 10 11 |

## Testing Decisions

A good test states behaviour a performer or the next layer can observe: what a Function answers,
what a Tick Plan holds, what reaches an output adapter. Full coverage is required at every level
below. No new seam is needed.

### Lang

Follows the coverage the existing Functions have beside the Function table.

- Every Chord Play operand is carried into the role its signature names, and Play Commands come out one per tone, lowest first.
- Each Chord Play requires exactly its declared operands.
- Channel, velocity and length take Play's domains, and an out-of-domain operand produces no Play Command at all.
- A Number in a Note operand and a Note in a Number operand are refused, and a Number or Note in an Interval Set or Voicing operand is refused unless it spells a valid name.
- All 39 Interval Set names decode to the table above, and all 8 Voicing names decode. Unknown and half-blank names are refused.
- Every Voicing, and each Voicing with inversion, gives the expected tones. Too few tones refuses. Removal, inversion and movement apply in that order.
- A tone outside `00`–`7F` refuses the whole chord.
- Pick, Up, Down and Stack each have arity, type, domain and boundary coverage, including wrapping and every diagnosis.
- The Copy reading admits Interval Set and Voicing and still refuses partial input.
- The allocation tests still assert that evaluating allocates nothing, with a twelve-tone chord in the fixture.

### Tick

Uses the existing Source-rows-to-Tick-Plan fixtures, alongside the rest, read, track, nesting and
play order tests.

- A Bang activates a Chord Play once and its tones appear in order in the Tick Plan, and an unactivated one sends nothing.
- Nested Reads and Track in every Chord Play operand, including all six after channel, driven by a Clock across Ticks.
- Portal-written Cells mean what a Return means in root, set, voicing and inversion.
- The nested tracker and sequencer voice forms from ADR 0103 play what their current forms play, Tick by Tick.
- Every Rest route: Source-blank, Portal-emptied and Clear, on all five Plays. A Rest sends nothing and diagnoses nothing.
- A half-written note, root or set is diagnosed in each route, and a blank set with a root present is diagnosed.
- A blank root ignores other blank or cleared operands but not a half-written one.
- A note Play with a blank note and a blank velocity is diagnosed.
- Pick, Up, Down and Stack as root and nested, including their Output Portal writes and Pick and Stack composed into one Chord Play.
- Interval Set and Voicing carried by Copy, Read, Track, Write and Push.
- The Language Map treats a blank note or root as a root, and marks Chord Play invalid where a value is required.
- The nesting property tests generate the two new literal types.
- The Function reference tests cover the six new rows, and the shipped example tests still pass with the unified Rest.

### Playback

Uses the existing Playback Engine schedule and shipped-example tests.

- Each tone of a Timed Chord Play starts in Tick Plan order and stops at its own Note Off.
- Velocity `00` stops every tone and schedules nothing. Length `00` starts nothing and leaves sounding notes standing.
- A repeated chord replaces the tones it repeats and retires their expiries, and a chord overlapping another by some tones releases only those.
- A Raw Chord Play delivers one Note On per tone.
- Stopping, rewinding and disconnecting while a chord sounds run the safety action and clear its schedule.
- A chord and a Monophonic Play on one channel keep separate ownership.

## Out of Scope

- Snapping a Note to a scale, and varying velocity or length across the tones of one Chord Play. Several Play roots remain the way to do the second.
- A Monophonic Chord Play.
- The timing work in `docs/research/timing-operators.md`, which has its own pass.
- Refactoring the shipped tracker and sequencer examples. Only a chord patch built on them takes the nested form.

## Further Notes

- A JavaScript prototype of these Functions was used in design. It is not in the repository and nothing here depends on it. Its semitone table and its voicing, inversion, Pick and Stack arithmetic agree with this spec. It differs in five ways:
  - It spells Raw Chord Play `!=`. The decided spelling is `!:`.
  - It rests only on a Clear in the root, as ADR 0075 does for a note. A root written blank in Source or emptied by a Portal is diagnosed there, which ADR 0104 reverses.
  - It checks channel and velocity ranges on a Rest. Under ADR 0104 a Rest checks no operand's range.
  - It never excuses a blank operand beside a blank root. A Source-blank operand goes to decoding and is diagnosed, and a cleared operand other than the root is excused only under its `whole` option. ADR 0104 excuses both blank and cleared operands beside a blank root.
  - It runs roots in Source order. The real scheduler orders them by dependency.
- The real interpreter has run three nested Reads in one Timed Play, with a Clock stepping through rows. The whole nested tracker and sequencer have not been run.
- Nothing yet shows Source or Tick diagnostics in the console, so a whole-chord diagnosis is silent to a performer.

## Comments

- ADR 0108 supersedes ADRs 0075 and 0104 and removes the Rest. User stories 34, 35, 36 and 40,
  and story 39's appeal to ADR 0075, describe the Rest and no longer hold: a blank note on a Raw,
  Timed or Monophonic Play plays nothing because the note is `Optional<Note>`, and a blank Chord
  Play root is a plain Note's refusal until a decision makes it optional. Ticket 05 waits on that
  decision.
