# Orca tracker

`tracker.orcvs` is the tracker from [Learn Orca](https://metasyn.srht.site/learn-orca/)
in Orcvs: eight voices, each with its own Clock, reading a sixteen-step column of
Notes and playing it on its own MIDI channel. Open it with **File > Open** and
press Space to start Playback. Space pauses and resumes it, and Command Shift R
(Ctrl Shift R outside macOS) rewinds every voice to step 0, as in Orca. The
first MIDI output is chosen for you; change it in the bottom Panel.

```text
col  0         1         2         3         4         5         6         7
     01234567890123456789012345678901234567890123456789012345678901234567890
  0    || TRACKER ||
  1
  2                         ~*0101    .+0B~.0110        ~*0101    .+0B~.0110
  3                           !~0377&v  01                !~0777&v  01
  4
  5                  ~*0101    .+08~.0110        ~*0101    .+08~.0110
  6                    !~0277&v  01                !~0677&v  01
  7
  8           ~*0101    .+05~.0110        ~*0101    .+05~.0110
  9             !~0177&v  01                !~0577&v  01
 10
 11    ~*0101    .+02~.0110        ~*0101    .+02~.0110
 12      !~0077&v  01                !~0477&v  01
 13
 14          || |  |   |C4|   |  |   |  |   |  |   |  |   |  |   |  | ||
 15          || |  |   |  |   |  |   |  |   |  |   |  |   |  |   |  | ||
 ...
 29          || |  |   |  |   |  |   |  |   |  |   |  |   |  |   |  | ||
```

As Learn Orca ships it, the pattern holds one Note at step 0: Orca's `3C`, which
Orcvs spells `C4` (MIDI note 60), on channel `01` (MIDI channel 2 in most DAWs).
Each pattern row is one `||` Comment, so the Notes in it are text the Reads
select rather than Functions, and a single `|` either side marks each voice's
step. Type a Note between a step's marks to fill it in, and keep each row's `||`
at both ends.

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

The voices stand in two staircases of four, as in Orca: channels `00` to `03`
from column 2, and channels `04` to `07` from column 30. Each voice in a
staircase sits three rows above and seven columns east of the one before it, so
each voice's steps are five blank columns from the next. All eight read the
same pattern rows because each one's Add offset is the distance from its Read's
`n` slot to the first step, less one: `02`, `05`, `08` and `0B`, where Orca's
are `2` to `5`.

## One voice

For channel `00`, with its machine at rows 11 and 12:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 11) `~*0101` | Delay | Writes `**` at (2, 12) every Tick, which activates the Timed Play at (4, 12). |
| (12, 11) `.+02~.0110` | Add, with a nested Clock | Adds `02` to the step and writes the sum into the Read's `n` slot at (12, 12). The nested Clock also writes its count at (16, 12). |
| (4, 12) `!~0077&v  01` | Timed Play with a nested Read south | Channel `00` (MIDI channel 1 in most DAWs), velocity `77` (119), length `01` Tick. Its note slot holds `&v`, whose `n` operand at (12, 12) is the Add's sum. |
| (12, 14) to (13, 29) | Comment text | The voice's sixteen steps, one Note per row. An empty row is a silent rest, as in Orca. |

`&v n` reads the pair `n` rows south of its own `n` operand, so with the offset
`02` step 0 is row 14. The nested Read also writes the Note it reads one row
south of itself, at (10, 13), which shows the step being played.

Voice `v` of the staircase starting at column `c` has its Delay at
`(c + 7v, 11 - 3v)`, its Play one row down and two columns east, and its steps
in columns `c + 7v + 10` and `c + 7v + 11`. That puts the columns at 12, 19,
26, 33, 40, 47, 54 and 61 for channels `00` to `07`.

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

- Orca's `H` halts the `*` below it, so the `*` stays and keeps banging. Orcvs's
  Halt `*!` locks only while a Bang activates it, and a locked `**` does not
  fire, so a Delay that bangs every Tick takes their place.
- An Orcvs Function is two Cells wide, so each voice's machine is wider than
  Orca's and the layout spreads further east and south.
- Orca frames each voice's step in its own `#..#`. Orcvs needs only one Comment
  per row to keep its Notes from being read as Functions, so each step is
  marked with a single `|` inside it.
- The web console has no Rewind, because the browser keeps Command Shift R for
  a reload.
- There is no Undo, and no way to run a single Tick by hand.
- Stopping sends All Notes Off, Reset All Controllers and a centred pitch bend
  on every channel, rather than a Note Off for each sounding note.
- Orca colours every operator's ports. Orcvs lights the Cells each Function
  read and wrote on the last Tick, so a voice's Read steps along its pattern as
  Orca's does, and marks the Function each Halt locked. The Delay that stands in
  for Orca's `H` over `*` locks nothing, so the tracker shows no locked Cells.
