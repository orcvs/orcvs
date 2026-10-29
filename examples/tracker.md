# Tracker

`tracker.orcvs` is the tracker from [Learn Orca](https://metasyn.srht.site/learn-orca/)
in Orcvs: eight voices, each with its own Clock, reading a sixteen-step column
of Notes and playing it on a MIDI channel. It ships playing the 808 beat. Open
it with **Help > Examples > Tracker** and press Space to start Playback. Space
pauses and resumes it, and Command Shift R (Ctrl Shift R outside macOS) rewinds
every voice to step 0, as in Orca. The first MIDI output is chosen for you;
change it in the bottom Panel. The example opens Untitled, so **File > Save**
asks where to keep your edits.

```text
col  0         1         2         3         4         5         6         7
     01234567890123456789012345678901234567890123456789012345678901234567890
  0
  1    || TRACKER ||
  2
  3                         ~*0101    .+0B~.0110        ~*0101    .+0B~.0110
  4                           !~0377&v  01                !~0777&v  01
  5
  6                  ~*0101    .+08~.0110        ~*0101    .+08~.0110
  7                    !~0877&v  01                !~0677&v  01
  8
  9           ~*0101    .+05~.0110        ~*0101    .+05~.0110
 10             !~0877&v  01                !~0577&v  01
 11
 12    ~*0101    .+02~.0110        ~*0101    .+02~.0110
 13      !~0877&v  01                !~0477&v  01
 14
 15            ||C2|   |  |   |f2|   |  |   |  |   |  |   |  |   |  ||
 16            ||  |   |  |   |  |   |  |   |  |   |  |   |  |   |  ||
 17            ||  |   |  |   |f2|   |  |   |  |   |  |   |  |   |  ||
 18            ||  |   |  |   |  |   |  |   |  |   |  |   |  |   |  ||
 19            ||C2|   |D2|   |f2|   |  |   |  |   |  |   |  |   |  ||
 ...
 30            ||  |   |  |   |  |   |  |   |  |   |  |   |  |   |  ||
```

The pattern holds the basic 4/4 808 beat, the beat the 808 example plays: the
kick `C2` on every beat, steps 0, 4, 8 and 12; the snare `D2` on beats 2 and 4,
steps 4 and 12; and the closed hat `f2` on every eighth, every even step. The
three voices that play it, in columns 12, 19 and 26, play on channel `08`, where
the Built-in Synth's kit picks each instrument from the Note's pitch class.
Every other voice ships empty, on its Learn Orca channel, `03` to `07`.

Each pattern row is one `||` Comment, so the Notes in it are text the Reads
select rather than Functions. A single `|` either side marks each voice's
step, and the Comment's own `||` marks the first voice's left side and the last
voice's right side. Type a Note between a step's marks to fill it in, and keep
each row's `||` at both ends.

## From Orca to Orcvs

Each voice is the same machine in both languages.

| Orca | Orcvs | In this patch |
| --- | --- | --- |
| `Cg` clock | `~.0110` Clock | Counts `00` to `0F`, one step a Tick. |
| `A` add | `.+` Add | Adds the voice's offset to the count and writes it into the Read's `n` slot below. |
| `Q` query | `&v` Read south | Reads the Note `n` rows south of its `n` slot, nested in the Timed Play's note slot. |
| `H` over `*` | `~*0101` Delay | Bangs every Tick, which activates the Timed Play east of its `**`. |
| `:` MIDI | `!~` Timed Play | Plays the Note the Read returns on the voice's channel, velocity `77` (119, the byte Orca's default velocity `f` sends), for `01` Tick. |
| `#` comment | `\|\|` Comment | The title, and each pattern row from end to end. The Reads see through it. |

The voices stand in two staircases of four, as in Orca: voices 0 to 3 from
column 2, and voices 4 to 7 from column 30. Each voice in a
staircase sits three rows above and seven columns east of the one before it, so
each voice's steps are five blank columns from the next. All eight read the
same pattern rows because each one's Add offset is the distance from its Read's
`n` slot to the first step, less one: `02`, `05`, `08` and `0B`, where Orca's
are `2` to `5`.

## One voice

For voice 0, the kick, with its machine at rows 12 and 13:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 12) `~*0101` | Delay | Writes `**` at (2, 13) every Tick, which activates the Timed Play at (4, 13). |
| (12, 12) `.+02~.0110` | Add, with a nested Clock | Adds `02` to the step and writes the sum into the Read's `n` slot at (12, 13). The nested Clock also writes its count at (16, 13). |
| (4, 13) `!~0877&v  01` | Timed Play with a nested Read south | Channel `08` (MIDI channel 9 in most DAWs), velocity `77` (119), length `01` Tick. Its note slot holds `&v`, whose `n` operand at (12, 13) is the Add's sum. |
| (12, 15) to (13, 30) | Comment text | The voice's sixteen steps, one Note per row. An empty row is a silent rest, as in Orca. |

`&v n` reads the pair `n` rows south of its own `n` operand, so with the offset
`02` step 0 is row 15. The nested Read also writes the Note it reads one row
south of itself, at (10, 14), which shows the step being played.

Voice `v` of the staircase starting at column `c` has its Delay at
`(c + 7v, 12 - 3v)`, its Play one row down and two columns east, and its steps
in columns `c + 7v + 10` and `c + 7v + 11`. That puts the columns at 12, 19,
26, 33, 40, 47, 54 and 61 for voices 0 to 7.

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the sixteen steps are one 4/4 bar, as an Orca frame is. Change the tempo in the
Panel's BPM field.

## Entering Notes

An Orcvs Note is its pitch letter and then its octave, and the octave is one
more than Orca's: Orca's `3C` is `C4`, MIDI note 60, and in general Orca's `nX`
is `X` followed by octave `n + 1`. A lowercase letter is a sharp, as in Orca:
Orca's `3c` is `c4`, C♯4. A step holding half a Note, such as `C ` while you
type `C4`, is partial input: the Read is diagnosed and that voice sends nothing
that Tick.

Each voice has its own Clock, as in Orca. Change a voice's modulus, for example
`~.0108`, to give it an eight-step loop against the others' sixteen.

## Where it differs from Orca

- Learn Orca's tracker ships one Note, `3C` on channel `1`, and its voices play
  channels `0` to `7`. This one ships the 808 beat on channel `08` in place of
  channels `00` to `02`.
- Orca's `H` halts the `*` below it, so the `*` stays and keeps banging. Orcvs's
  Halt `*!` locks only while a Bang activates it, and a locked `**` does not
  fire, so a Delay that bangs every Tick takes their place.
- An Orcvs Function is two Cells wide, so each voice's machine is wider than
  Orca's and the layout spreads further east and south.
- Orca frames each voice's step in its own `#..#`. Orcvs needs only one Comment
  per row to keep its Notes from being read as Functions, so each step is
  marked with a single `|` inside it, or by the Comment's `||` at either end.
- The web console has no Rewind, because the browser keeps Command Shift R for
  a reload.
- There is no Undo, and no way to run a single Tick by hand.
- Stopping sends All Notes Off, Reset All Controllers and a centred pitch bend
  on every channel, rather than a Note Off for each sounding note.
- Orca colours every operator's ports. Orcvs lights the Cells each Function
  read and wrote on the last Tick, so a voice's Read steps along its pattern as
  Orca's does, and marks the Function each Halt locked. The Delay that stands in
  for Orca's `H` over `*` locks nothing, so the tracker shows no locked Cells.
