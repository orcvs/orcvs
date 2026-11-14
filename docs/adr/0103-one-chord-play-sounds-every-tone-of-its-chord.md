# One Chord Play sounds every tone of its chord

Status: accepted. Amends the clause of [ADR 0028](0028-specify-the-orcvs-evaluation-machine.md) that a Terminal Output Function answers a Play Command, and the rule that each Expression performs at most one output, for the two Chord Play Functions only. Settled in `.scratch/chords-and-pitch/`. The drop Voicings `D5` and `D6` are renamed `23` and `24` by the 2026-11-14 amendment to [ADR 0105](0105-interval-set-and-voicing-are-literal-types.md), which names a drop for the positions it lowers rather than their sum; the order the tones are built in and the tones each Voicing needs stand.

A chord is several Play roots that one Bang activates ([ADR 0063](0063-a-list-is-cells-in-a-claim-not-a-value.md)). That still works, and it stays the way to give tones different channels, velocities or lengths. It cannot be driven as one thing: a progression written that way needs one Play and one Read for every tone of every chord, and changing a chord's type means editing several roots at once.

## Decision

**A Chord is a root Note then an Interval Set, as two operands.** `C4M7` is four Cells, two ordinary pairs, each carried and read like any other pair. A Chord is not a value: no Function answers one and no operand holds one.

**Chord Play has a Raw and a Timed variant.** Raw Chord Play is `!: channel velocity root set voicing inversion`. Timed Chord Play is `!# channel velocity root set voicing inversion length`. There is no Monophonic variant, because Monophonic Play owns one voice per channel and a chord needs several. Neither overloads Raw or Timed Play, because arity is fixed at parse.

**One Turn emits one Play Command per tone, lowest tone first.** Each is the ordinary Raw or Timed Play Command, so velocity `00`, length `00`, Note Off scheduling and the safety action apply tone by tone exactly as they do to a note. The Playback Engine gains no behaviour. The tones exist only inside the Turn: nothing is returned, nothing is written to Source, and no series of values passes between Functions, so ADR 0063 stands.

**The tones are built in a fixed order.** Voicing removes tones first: `SH` keeps the 1st, 2nd and 4th, and `RL` removes the 1st. Inversion then moves the lowest tone up an octave, as many times as its Number says. Voicing then moves tones: `OP` raises every second tone an octave, and `D2`, `D3`, `D5` and `D6` lower the tones they name, counted from the top. `CL` moves nothing. The digit in a drop voicing is the sum of the positions dropped, so `D5` drops the 2nd and 3rd and `D6` the 2nd and 4th.

**A chord that cannot be built whole is not sent.** A tone outside `00`–`7F` diagnoses the whole Chord Play, as does an Interval Set with too few tones for its voicing (`OP` and `D2` need 2, `D3` and `D5` need 3, `D6` and `SH` need 4, `RL` needs 2). No partial chord is ever sent.

**A Chord Play is diagnosed as one Expression.** Its Turn answers one chord, which the Tick expands into one Play Command per tone. A fault anywhere in building it, in an operand's domain or type, a voicing or a tone, raises one diagnostic for the Expression and emits no Play Command. The one-output rule's diagnostic side is unchanged: a Chord Play either sends its whole chord or diagnoses once.

**An operand that reads its value from Source holds a nested Read.** `&v.+02~.0110` costs 12 Cells where an Operand Literal costs 2, so each nested Read adds 10. A Timed Chord Play is 16 Cells with only Operand Literals, 36 with nested Reads in root and set, and 76 with nested Reads in all six operands after channel. A Raw Chord Play has no length, so it is 14, 34, and 64 with nested Reads in its five operands after channel. Width is accepted.

## Consequences

- The Evaluator's answer for a Terminal Output Function can be several Play Commands. A chord holds at most twelve tones, so the answer stays bounded and evaluation still allocates nothing.
- A patch built from the tracker or sequencer moves its Add and Clock into the Read's `n` operand. The Add-above-the-Play layout has room for one Read per Play, because two `n` operands of a Chord Play are 4 Cells apart and an Add with a nested Clock is 10 wide.
- The row directly south of a Chord Play holds its nested Functions' Output Portal writes, so a nested Read's first selected row is `02`.
- A Chord Play wider than the 28 columns between the tracker's two staircases reaches the second staircase's Play on the same row, so a chord tracker moves that staircase east.

## Considered options

- **Several roots only.** Needs nothing new, and cannot be driven as a progression without a Read per tone.
- **A Chord as one operand.** `C7` is already a Note, and an operand's width must be fixed at parse.
- **Overloading Raw and Timed Play.** Arity is fixed, and a chord's operand signature differs.
- **A Monophonic Chord Play.** Its one voice per channel contradicts sounding several tones.
- **Selecting a whole chord with one Read.** A Return is one two-Cell encoding ([ADR 0061](0061-a-nested-function-returns-to-the-slot-it-occupies.md)), so this would reopen that decision to save width that costs only layout.
