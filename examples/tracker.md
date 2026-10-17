# Four-voice tracker

`tracker.orcvs` is Orca's tracker pattern in Orcvs: one Clock shared by four
voices, each reading its own column of Notes down a pattern grid and playing
them on its own MIDI channel. Open it with **File > Open** and press Space to
start Playback.

```text
col  0         1         2         3         4         5         6         7
     01234567890123456789012345678901234567890123456789012345678901234567890123456789
  0  ~.0208
  1
  2  ~*0102    .+01&$0001~*0102    .+01&$0001~*0102    .+01&$0001~*0102    .+01&$0001
  3    !~0064&v  04        !~0164&v  04        !~0264&v  04        !~0364&v  04
  4            C3                                      C5                  G5
  5                                E4                                      A5
  6            C3                                      E5
  7                                E4                  G5                  B5
  8            G2                                                          C6
  9                                D4                  E5
 10            A2                                      D5
 11            C3                  D4                                      B5
```

## From Orca to Orcvs

| Orca | Orcvs | In this patch |
| --- | --- | --- |
| `C` clock | `~.` Clock | `~.0208` at (0, 0) counts `00` to `07`, two Ticks per step, and writes the count at (0, 1). |
| `D` delay | `~*` Delay | `~*0102` bangs every second Tick, on the first Tick of each step. |
| `A` add | `.+` Add | `.+01` adds `01` to the step, for the reason below. |
| `O` read | `&v` Read south, `&$` absolute Read | `&$0001` reads the shared Clock's count at column `00`, row `01`. `&v` reads the voice's step from its column. |
| `:` MIDI | `!~` Timed Play | Plays the Note `&v` returns for four Ticks. |
| `*` bang | `**` | The Delay's `**` activates the Timed Play east of it. |

Orca has no absolute read, so an Orca patch either gives each voice its own
clock or reads one clock through offsets. Here every voice reads the one Clock
at (0, 1) with `&$`, so all four always play the same step.

## One voice

Voice `v` occupies columns `20v` to `20v + 19`. For the first voice (`v = 0`):

| Cells | Function | Role |
| --- | --- | --- |
| (0, 2) `~*0102` | Delay | Writes `**` at (0, 3), which activates the Timed Play at (2, 3). |
| (10, 2) `.+01&$0001` | Add, with a nested absolute Read | Adds `01` to the Clock's count and writes the sum one row south, into the Read's `n` slot at (10, 3). The nested Read also writes the count it read at (14, 3). |
| (2, 3) `!~0064&v  04` | Timed Play with a nested Read south | Channel `00` (channel 1 for voice 1, up to `03` for voice 4), velocity `64` (100), length `04` Ticks. Its note slot holds `&v`, whose `n` operand at (10, 3) is the Add's sum. |
| (10, 4) to (11, 11) | Ordinary Source | The voice's eight steps, one Note per row. An empty row is a rest. |

`&v n` reads the pair `n` rows south of its own `n` operand, in the operand's
columns. `n` of `00` is the operand itself, so the step column starts one row
below it, at row 4, and step `s` is `n = s + 1`. That is why each voice adds
`01` to the Clock: without it, step 0 would read the `n` slot and every other
step would read the row above the one you meant. The nested Read also writes
the Note it reads one row south of itself, at (8, 4), which shows the step being
played beside the column.

At the default 120 BPM a Tick is 125 ms, so each step lasts 250 ms (an eighth
note), a loop lasts one 4/4 bar, and each note sounds for 500 ms. Set your
instruments on MIDI channels 1 to 4; the shipped parts are a bass line, an
off-beat chord tone, a melody and a high counter-melody.

To keep the alignment when you change it:

- Each voice is a 20-column block; the next voice's Delay starts at column
  `20v`.
- The Add stands directly above the Read's `n` slot, at column `20v + 10`.
- The step column is the Read's `n` column, `20v + 10`, from row 4 down.
- A longer pattern needs a larger Clock modulus and a longer column.

## Rests and edits

An empty step is a rest. The Read returns the empty Cells, so the Timed Play's
note operand is unwritten and the Play is invalid: it sends no new note, is
diagnosed at its anchor, and does not stop a note an earlier step started. A
note whose four-Tick lifetime is still running sounds through the empty step
after it.

The steps are ordinary Source. Type over a Note while Playback runs and the
next step that reaches it plays the new Note; a step holding half a Note is
diagnosed by the Read and sends nothing. The Read takes its pair from working
Source at its Turn, after anything else that writes that pair in the same Tick
([ADR 0067](../docs/adr/0067-track-reads-a-portal-as-jump-does.md)).
