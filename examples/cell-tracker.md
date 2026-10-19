# Cell tracker

`cell-tracker.orcvs` plays a repeating row of eight Notes that you edit where
they stand. Open it with **File > Open**, press Space to start Playback, and
type over any pair while it plays. The first MIDI output is chosen for you;
change it in the bottom Panel.

```text
col  0         1         2
     012345678901234567890123456789
  0            ~.0208
  1  ~*0102  &t  08C4D4  G4  A4C5
  2    !~0064  04
```

## What each part does

| Cells | Function | Role |
| --- | --- | --- |
| (10, 0) `~.0208` | Clock, rate `02`, modulus `08` | Counts `00` to `07`, holding each value for two Ticks. It writes the count one row south, into Track's `index` slot at (10, 1). |
| (0, 1) `~*0102` | Delay, rate `01`, modulus `02` | Bangs every second Tick, starting at Tick 0. It writes `**` one row south, at (0, 2). |
| (8, 1) `&t  08` | Track, `index` at (10, 1), `count` `08` at (12, 1) | Reads pair `index % count` from the eight pairs east of its last operand, and writes it one row south, into the Timed Play's note slot at (8, 2). |
| (14, 1) to (29, 1) | Ordinary Source | The pattern: pairs `00` to `07` at columns 14, 16, 18, 20, 22, 24, 26 and 28. As shipped they hold `C4 D4 __ G4 __ A4 C5 __`. |
| (2, 2) `!~0064  04` | Timed Play | MIDI channel `00` (channel 1), velocity `64` (100), its note slot at (8, 2) filled by Track, length `04` Ticks. The `**` at (0, 2) activates it. |

At the default 120 BPM a Tick is 125 ms, so each step lasts 250 ms (an eighth
note), a loop of eight steps lasts one 4/4 bar, and each note sounds for 500 ms
(a quarter note). Change the tempo in the Panel's BPM field.

To keep the alignment when you change it:

- The Clock's output must land on Track's `index` slot, so the Clock stands
  directly above it: column 10, one row up.
- Track's Output Portal must land on the Timed Play's note slot, so the Play's
  anchor is six columns west of Track and one row down.
- The Delay's `**` must sit immediately west of the Play's anchor.
- A longer pattern needs a larger `count` and a Clock modulus to match. Track
  counts its pairs from the end of `count`, so pair `n` starts at column
  `14 + 2n`.

## `index` and `count` are ordinary operands

Track has no hidden list and no state of its own. `index` and `count` are
Numbers like any other operand: a literal, a nested Function, or Cells a Portal
writes. Here the Clock writes `index`. You can type a literal over it to hold
one step, or replace `count` to shorten the loop while it plays.

Track reads its selected pair from working Source at its Turn
([ADR 0067](../docs/adr/0067-track-reads-a-portal-as-jump-does.md)). A write
this Tick to `index`, to `count` or to the selected pair is seen this Tick,
whether the writer stands before Track in the Grid or after it. An edit you type
reaches the next Tick that runs.

## How Track differs from Orca's `T`

Orca's `T` reads whatever its place in the Grid order finds, so a writer east of
it or below it is heard a frame late. Track's read waits instead: once `index`
and `count` settle, every writer of the selected pair that has not taken its
Turn goes first, and Track reads after it. If that writer waits on Track in
turn, the two form a same-Tick dependency cycle, which is diagnosed while the
rest of the Tick plays.

An empty pair is a rest. Track copies an empty pair's Cells as it copies any
others, so it clears the Timed Play's note slot. A Timed Play with an empty note
slot is invalid ([ADR 0069](../docs/adr/0069-an-unwritten-operand-is-invalid-input.md)):
it sends no new note and is diagnosed, and it does not stop a note an earlier
step started. In the shipped pattern D4 at step 1 keeps sounding through the
empty step 2, because its four-Tick lifetime has not ended.

A value Function placed between Track and the Play behaves differently. If
Track hands it an empty pair, it is invalid and writes nothing, so the note slot
keeps the last Note it wrote and the next Bang plays that Note again. Leave
Track writing straight into the note slot when you want empty pairs to rest.

## While you edit

A pair holding half a Note, such as `E ` while you type `E5`, is partial input.
If a Bang reaches the Play while Track selects that pair, Track diagnoses it and
writes nothing, so the note slot keeps the Note the previous step left there
and the Play sounds that Note again. The next Tick after you finish the Note
plays it. To avoid the repeat, type a whole Note at once with a paste, or edit
a pair the Clock is not on.
