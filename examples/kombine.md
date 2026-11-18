# Kombine

`kombine.orcvs` plays three pitches against four velocities, so its loud
accent moves across a C major triad. Open it with **Help > Examples >
Kombine**, or as `?example=kombine`, and press Space to start Playback. The
first MIDI output is chosen for you; change it in the bottom Panel. The example
opens Untitled, so **File > Save** asks where to keep your edits.

It is the first entry in the [examples index](README.md) that is built to be
copied: its accent Track works on its own in another piece, and this guide
moves it into the [Tracker](tracker.md)'s closed hat.

```text
col  0         1
     01234567890123456789
  0
  1  || Kombine ||
  2  ~*0101
  3    !~00&$000B&$000701
  4
  5  || Pitch ||
  6  &t~.010303C4E4G4
  7
  8
  9  || Accent ||
 10  &t~.01040470305040
 11
```

Orca's *Kombine* builds each note from separate octave, note and velocity
Clocks. This one folds the octave into the Note and keeps two Tracks of
different lengths, so you can change one without the other.

## What each part does

| Cells | Function | Role |
| --- | --- | --- |
| (0, 2) `~*0101` | Every, division `01`, steps `01` | Bangs every Tick, starting at Tick 0, and writes `**` at (0, 3), which activates the Timed Play east of it. |
| (2, 3) `!~00&$000B&$000701` | Timed Play with two nested Absolute Reads | Channel `00` (MIDI channel 1), the Built-in Synth's sine voice. Its velocity is `&$000B`, the pair at (0, 11), and its note is `&$0007`, the pair at (0, 7). Length `01`: each Note stops one Tick after it starts, as the next one starts. Each nested Read also writes what it reads one row south, at (6, 4) and (12, 4). |
| (0, 6) `&t~.010303C4E4G4` | Track with a nested Clock | The Clock `~.0103` counts `00` to `02`, one step a Tick, and Track plays pair `index % 03` of `C4 E4 G4`. Track writes the Note at (0, 7), its Output Portal; the Clock writes its count at (2, 7). |
| (0, 10) `&t~.01040470305040` | Track with a nested Clock | The Clock `~.0104` counts `00` to `03`, and Track plays pair `index % 04` of the velocities `70 30 50 40` (112, 48, 80 and 64). Track writes the velocity at (0, 11), its Output Portal; the Clock writes its count at (2, 11). |

Leave rows 4, 7 and 11 empty: the Functions above them write there every Tick.
The Language Map diagnoses each Cell of the Notes and velocities after each
Track, as an unknown Function and as a character that starts no Language Unit.
A Track claims only its operands, so the Cells it reads are ordinary Source,
where a Note or Number standing on its own is invalid. They play as written.

## What you hear

One sine Note a Tick, each held for one Tick. At the default 120 BPM a Tick is
125 ms, a sixteenth note, so the twelve Ticks below last three beats before
they start again.

| Tick | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Note | C4 | E4 | G4 | C4 | E4 | G4 | C4 | E4 | G4 | C4 | E4 | G4 |
| Velocity | **70** | 30 | 50 | 40 | **70** | 30 | 50 | 40 | **70** | 30 | 50 | 40 |

The pitch Track wraps every three Ticks and the accent Track every four, so the
loud `70` lands on C4 at Tick 0, E4 at Tick 4 and G4 at Tick 8, and both start
together on C4 again at Tick 12.

## Three accent steps

Shorten the accent Track to three steps and the loud accent returns to C on
every pass. Make both edits, while it plays if you like; each is one Cell, and
each takes effect on the next Tick:

| Cell | Operand | Before | After |
| --- | --- | --- | --- |
| (7, 10) | the accent Clock's `steps` | `4` | `3` |
| (9, 10) | the accent Track's `count` | `4` | `3` |

Then clear the fourth velocity, `40` at (16, 10), which the Track no longer
reads: put the Cursor on (17, 10) and press Backspace twice.

```text
col  0         1
     01234567890123456789
 10  &t~.010303703050
```

Now the Clock counts `00` to `02` and the Track plays `70 30 50`. Both Tracks
wrap every three Ticks, so C4 plays at `70` on Ticks 0, 3, 6 and 9, E4 at
`30` and G4 at `50`.

Change both operands together. With only `count` at `03`, the Clock still
counts four steps and the Track plays `70 30 50 70`, the loud accent twice in a
row and drifting across the triad. With only `steps` at `03`, the accent
already stays on C, but `count` still names a fourth pair that never plays.

## The accent Region

The Region worth copying is the `Accent` Comment, the accent Track with its
nested Clock and its velocities, and the empty row its Functions write into:

```text
col  0         1
     012345678901234567
  9  || Accent ||           top-left (0, 9)
 10  &t~.01040470305040
 11  ooCC                   bottom-right (17, 11)
```

| Positions | What is there |
| --- | --- |
| (0, 9) to (17, 11) | The Region: three rows, eighteen columns. |
| (0, 11) and (1, 11), `oo` | Track's Output Portal: the velocity it plays this Tick. Read it with an Absolute Read, `&$` and this Position. |
| (2, 11) and (3, 11), `CC` | The nested Clock's Output Portal: its count. |
| (4, 11) to (17, 11) | Empty in the copy, so the Region keeps its shape when pasted. |

Track and Clock name no Position, so the Region works wherever it lands. Only
the Read of its Output Portal changes: in this file it is `&$000B`, column `00`
row `0B` (11). Wherever you paste the top-left at (x, y), the Output Portal is
at (x, y + 2).

## Moving the accents into the Tracker

The Tracker's closed hat, voice 2, plays `f2` on every second step on channel
`09` at velocity `77`. The transfer gives it the four accents.

1. Start from Kombine as shipped: if you made the three-step edit above, open
   **Help > Examples > Kombine** again first, so the Region holds four
   velocities and both operands read `04`. Then drag a Region from (0, 9) to
   (17, 11), or hold Shift with the arrows, and copy it. The copy holds whatever the Track and Clock last
   wrote in row 11; the Track writes over it on the first Tick.
2. Open **Help > Examples > Tracker**, put the Cursor at (2, 32), two rows
   below the last pattern row, and paste. The Region lands from (2, 32) to
   (19, 34), and its Output Portal is at (2, 34), `&$0222`.
3. Change the pasted Clock's `division` at (6, 33) from `01` to `02`: type
   `2` over the `1` at (7, 33). The hat
   plays every second Tick, so a Clock that steps every second Tick gives each
   hit the next velocity. At `01` the hits would land only on `70` and `50`.
4. Replace the hat Play's velocity `77` with `&$0222`. The Read is six Cells
   where `77` is two, so the rest of the Play would shift four columns east.
   Its nested `&v`, at (24, 7), would move to (28, 7), and its `distance`
   slot would leave (26, 7), the Cell its Add writes the pattern row into,
   for (30, 7). Move the Every and the Play four columns west instead, so the
   `&v` stays at (24, 7) and its slot at (26, 7):
   1. Drag a Region over the hat's Every, (16, 6) to (21, 6), cut it, put
      the Cursor on (12, 6) and paste.
   2. Drag a Region over the hat's Play, (18, 7) to (29, 7), and press
      Delete to empty it.
   3. Put the Cursor on (14, 7), type `!~09&$0222&v`, press the right arrow
      twice to leave the slot at (26, 7) empty, and type `01`. Space starts
      Playback rather than typing an empty Cell, so step over empty Cells
      with the arrows.

Rows 6 and 7 before:

```text
col  0         1         2         3         4         5         6
     0123456789012345678901234567890123456789012345678901234567890123456
  6                  ~*0101    .+08~.0110        ~*0101    .+08~.0110
  7                    !~0977&v  01                !~0677&v  01
```

The Tracker after the transfer, with the accent Region and the closed hat's
voice:

```text
col  0         1         2         3         4         5         6
     0123456789012345678901234567890123456789012345678901234567890123456
  6              ~*0101        .+08~.0110        ~*0101    .+08~.0110
  7                !~09&$0222&v  01                !~0677&v  01
 ...
 32    || Accent ||
 33    &t~.02040470305040
 34
```

Row 34 is drawn as it stands before the first Tick, after copying from a
Kombine that has not played. From the first Tick the Track writes the velocity
at (2, 34) and the Clock its count at (4, 34).

The Language Map diagnoses the pasted velocities, (12, 33) to (19, 33), as it
does in Kombine. They play as written.

| Positions | What is there |
| --- | --- |
| (2, 32) to (19, 34) | The accent Region in the Tracker. |
| (2, 34) | Its Output Portal, read by the hat's `&$0222`. |
| (12, 6) | The hat's Every, which bangs the Play at (14, 7). |
| (14, 7) `!~09&$0222&v  01` | The hat's Timed Play: channel `09`, velocity from (2, 34), note from its pattern column, length `01`. |

Press Space. The hat plays its eight hits a bar with these velocities, and the
loud `70` falls on every fourth hit:

| Step | 0 | 2 | 4 | 6 | 8 | 10 | 12 | 14 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Velocity | **70** | 30 | 50 | 40 | **70** | 30 | 50 | 40 |

The kick, snare and bassline keep velocity `77` and their own steps.

The accent Clock reads the absolute Tick rather than counting hits. Clear the
`f2` of step 2, at (26, 17), and step 2 rests while step 4 still plays `50` and
step 8 the loud `70`.
