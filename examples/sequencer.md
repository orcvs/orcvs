# Orca sequencer

`sequencer.orcvs` is the sequencer from [Learn Orca](https://metasyn.srht.site/learn-orca/)
in Orcvs: ten voices, each reading a thirty-two-step row of Notes and playing it
on a MIDI channel. It ships playing the 808 beat. Open it with
**Help > Examples > Orca Sequencer** and press Space to start Playback. Space pauses and resumes
it, and Command Shift R (Ctrl Shift R outside macOS) rewinds every voice to step
0, as in Orca. The first MIDI output is chosen for you; change it in the bottom
Panel. The example opens Untitled, so **File > Save** asks where to keep your
edits.

```text
col  0         1         2         3         4         5         6         7         8
     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567
  0
  1    || SEQUENCER ||
  2                     ||0               1               2               3               ||
  3    ~*0101    .+04~.0120
  4      !~0877&>  01  ||C2      C2      C2      C2      C2      C2      C2      C2      ||
  5
  6    ~*0101    .+04~.0120
  7      !~0877&>  01  ||        D2              D2              D2              D2      ||
  8
  9    ~*0101    .+04~.0120
 10      !~0877&>  01  ||f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  f2  ||
 ...
 30    ~*0101    .+04~.0120
 31      !~0977&>  01  ||                                                                ||
```

The first three voices play the basic 4/4 808 beat over both bars, the beat the
808 example plays: the kick `C2` on every beat, the snare `D2` on beats 2 and 4,
and the closed hat `f2` on every eighth. They play on channel `08`, where the
Built-in Synth's kit picks each instrument from the Note's pitch class. Every
other voice ships empty, on its Learn Orca channel, `03` to `09`. Each voice's
steps are one `||` Comment, so the Notes in it are text the Reads select rather
than Functions. Step `s` of every voice is the pair at columns `20 + 2s` and
`21 + 2s`, and the Comment on row 2 numbers every eighth step, `0` to `3`, as
Orca's labels do. Keep each row's `||` at both ends.

## From Orca to Orcvs

Each voice is the same machine in both languages.

| Orca | Orcvs | In this patch |
| --- | --- | --- |
| `Cw` clock | `~.0120` Clock | Counts `00` to `1F`, one step a Tick. |
| `A` add | `.+` Add | Adds `04` to the count and writes it into the Read's `n` slot below. |
| `O` read, `T` track | `&>` Read east | Reads the Note `n` pairs east of its `n` slot, nested in the Timed Play's note slot. |
| `V` variable | nothing | Each voice reads its own row, so nothing is shared. |
| `H` over `*` | `~*0101` Delay | Bangs every Tick, which activates the Timed Play east of its `**`. |
| `:` MIDI | `!~` Timed Play | Plays the Note the Read returns on the voice's channel, velocity `77` (119, the byte Orca's default velocity `f` sends), for `01` Tick. |
| `#` comment | `\|\|` Comment | The title, the step numbers, and each voice's steps from end to end. The Reads see through it. |

## One voice

For voice 0, the kick, with its machine at rows 3 and 4:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 3) `~*0101` | Delay | Writes `**` at (2, 4) every Tick, which activates the Timed Play at (4, 4). |
| (12, 3) `.+04~.0120` | Add, with a nested Clock | Adds `04` to the step and writes the sum into the Read's `n` slot at (12, 4). The nested Clock also writes its count at (16, 4). |
| (4, 4) `!~0877&>  01` | Timed Play with a nested Read east | Channel `08` (MIDI channel 9 in most DAWs), velocity `77` (119), length `01` Tick. Its note slot holds `&>`, whose `n` operand at (12, 4) is the Add's sum. |
| (20, 4) to (83, 4) | Comment text | The voice's thirty-two steps, one Note to a pair. An empty step is a silent rest, as in Orca. |

`&> n` reads the pair `n` pairs east of its own `n` operand, so with the offset
`04` step 0 is the pair at column 20, past the length, the Clock's count and the
Comment's `||`. The nested Read also writes the Note it reads one row south of
itself, at (10, 5), which shows the step being played.

Voice `v` has its Delay at `(2, 3 + 3v)`, its Play one row down and two columns
east, and its steps on that row.

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the thirty-two steps are two 4/4 bars, as Orca's thirty-two frames are. Change
the tempo in the Panel's BPM field.

## Entering Notes

An Orcvs Note is its pitch letter and then its octave. Orca's `:` takes the
octave as an operand, `3` for every voice of this patch, and the note from the
step, so Orca's `C` in a step is `C4` here, MIDI note 60. A lowercase letter is
a sharp, as in Orca: Orca's `c` is `c4`, C♯4. A step holding half a Note, such
as `C ` while you type `C4`, is partial input: the Read is diagnosed and that
voice sends nothing that Tick.

## Where it differs from Orca

- Orca shares one count between its voices through the variables `a` and `b`.
  Each Orcvs voice has its own Clock instead, and every Clock reads the Tick
  number, so the voices stay on the same step.
- Orca reads half its voices' steps with `O` and half with `T`. Each Orcvs
  voice reads with `&>`.
- Learn Orca's sequencer ships every step empty. This one ships the 808 beat
  on its first three voices.
- Orca's tenth voice plays on channel `a`, and no voice plays on `9`. Here the
  beat's three voices share channel `08` and the others are on channels `03`
  to `09`.
- Orca's `H` halts the `*` below it, so the `*` stays and keeps banging. Orcvs's
  Halt `*!` locks only while a Bang activates it, and a locked `**` does not
  fire, so a Delay that bangs every Tick takes their place.
- An Orcvs step is two Cells wide, so each voice's steps take sixty-four
  columns where Orca's take thirty-two.
- The web console has no Rewind, because the browser keeps Command Shift R for
  a reload.
- Stopping sends All Notes Off, Reset All Controllers and a centred pitch bend
  on every channel, rather than a Note Off for each sounding note.
