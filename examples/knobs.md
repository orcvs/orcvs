# Knobs

`knobs.orcvs` turns three of the 303-style voice's knobs and six of its
channel's effects over a sixteen-step acid bassline. The web console opens
it on a first visit, when it has no Source stored. It also opens with **Help >
Examples > Knobs**, or as `?example=knobs`. Press Space to start Playback.
Each knob's value climbs a step at a time and glides back down when its Clock
wraps; the Sound Module on orcvs.com shows each effect move on channel `08`. The example opens
Untitled, so **File > Save** asks where to keep your edits.

```text
col  0         1         2
     012345678901234567890123
  0
  1  || bass ||
  2  ~*0101
  3          .+0C~.0110
  4  !~0864&v0C01
  5
  6     || accent ||
  7     ~*0101
  8             .+07~.0110
  9     !~087F&v0701
 10
 11        || slide ||
 12        ~*0101
 13                .+02~.0110
 14        !~0864&v0202
 15
 16        ||  |A1|  ||
 17        ||A1|  |  ||
 18        ||  |  |A2||
 ...
 31        ||  |E2|  ||
 32
 33
 34  || knobs ||
 35  ~*0101  || CUT ||
 36    !c084A~>08~.0180
 ...
 42    !c084B~>08~.0340
 43
 44
 45  || effects ||
 46  ~*0101  || MD ||
 47    !c0801~>08~.0540
 ...
 62    !c085E~>08~.0340
```

The file is 24 columns wide, so a phone shows every line whole and scrolls it
only up and down.

## The bassline

The bassline plays on channel `08` (MIDI channel 9), the Built-in Synth's
303-style voice (ADR 0099). Three parts play it, `bass`, `accent` and `slide`,
each a machine like the Tracker example's voices that plays its Notes its own
way. Its sixteen steps run down rows 16 to 31, one step a row, and each step row
has a column for each part: `bass` in columns 8 and 9, `accent` in columns 11
and 12, and `slide` in columns 14 and 15. A step holds a Note in one column and
is empty in the other two, where that part's Timed Play sends nothing and
reports nothing.

| Part | Timed Play | Column | How its Notes play |
| --- | --- | --- | --- |
| `bass` | `!~0864&v0C01` | 8 | Velocity `64` (100), one Tick. |
| `accent` | `!~087F&v0701` | 11 | Velocity `7F` (127). From `78` up a Note is accented: louder, its filter opened further, and its sweep at its shortest. |
| `slide` | `!~0864&v0202` | 14 | Velocity `64`, two Ticks. The Note still sounds when the next step's Note starts, so the voice glides to it without striking it afresh. |

Move a Note from one column to another, keeping its row, to change how it
plays. A slide to the same Note strikes it again, because Playback stops a
Note before it plays it again.

Each part stands three columns east of the one above it, so its Read sits
above its own column of steps. For the `bass` part:

| Cells | Function | Role |
| --- | --- | --- |
| (0, 2) `~*0101` | Delay | Writes `**` at (0, 3) every Tick, which activates the Timed Play directly south of it, at (0, 4). |
| (8, 3) `.+0C~.0110` | Add, with a nested Clock | The Clock counts `00` to `0F`, one a Tick. The Add adds `0C` and writes the sum into the Read's `n` slot at (8, 4). The file ships that slot holding `0C`, the sum Tick 0 writes, so the Source opens with nothing diagnosed. |
| (0, 4) `!~0864&v0C01` | Timed Play with a nested Read south | Channel `08`, velocity `64`, length `01` Tick. Its note slot holds `&v`, which reads the step `n` rows south of (8, 4), in columns 8 and 9, and writes the Note it reads at (6, 5). |
| (8, 16) to (9, 31) | Comment text | The `bass` column of the sixteen steps, one Note to a row. Type over any step while it plays. |

The `accent` and `slide` parts are the same machine three and six columns
further east and five and ten rows further down, each with its own velocity
and length. Each Add's offset, `0C`, `07` and `02`, is the number of rows from
its Read's `n` slot to step 0 in row 16. Keep the rows between the parts and
the steps as they are: a row added or removed there moves the steps away from
the offsets.

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the bassline loops every bar.

## One knob

| Cells | Function | Role |
| --- | --- | --- |
| (0, 35) `~*0101` | Delay | Writes `**` at (0, 36) every Tick, which activates the Control Change at (2, 36). |
| (8, 35) `\|\| CUT \|\|` | Comment | The knob's label. |
| (2, 36) `!c084A` | Control Change | Channel `08`, controller `4A` (74), the acid voice's cutoff. Its value slot holds the Interpolation. |
| (8, 36) `~>08` | Interpolation, nested | Moves its value toward the Clock's by at most `08` a Tick. It writes the value it sends at (8, 37). |
| (12, 36) `~.0180` | Clock, nested | Counts `00` to `7F`, one step a Tick, then wraps. |

Each knob's Clock sets how fast and how far it turns:

| Controller | Knob | Clock | Sweep |
| --- | --- | --- | --- |
| `4A` (74) | `CUT`, cutoff | `~.0180` | `00` to `7F`, a step a Tick |
| `47` (71) | `RES`, resonance | `~.0280` | `00` to `7F`, a step every 2 Ticks |
| `4B` (75) | `DEC`, how long each Note's sweep lasts | `~.0340` | `00` to `3F`, a step every 3 Ticks |

The voice's other two controls keep their load values: `46` (70), the
waveform, a saw below `40` and a square from it, and `4F` (79), how far each
Note's sweep opens the filter. Add a knob for either, or change a Clock's
`rate` or `modulus` while it plays.

## The effects

Each effect knob is the same machine on channel `08`, sending one of the
Built-in Synth's effect controllers. Each channel has its own effects, so the
knobs send on the bassline's channel to change the bassline. On `08`, CC74
(`4A`) and CC75 (`4B`) are the voice's cutoff and decay, which the `CUT` and
`DEC` knobs already turn, so the tilt and the length have no knob here:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 50) `!c0855` | Control Change | Channel `08`, controller `55` (85), the `OF` effect. Its value slot holds the Interpolation. |
| (8, 50) `~>08` | Interpolation, nested | Moves its value toward the Clock's by at most `08` a Tick. |
| (12, 50) `~.0720` | Clock, nested | Counts `00` to `1F`, a step every 7 Ticks, then wraps. |

Each effect sweeps at its own rate, so the effects peak at different times:

| Controller | Effect | Clock | Sweep |
| --- | --- | --- | --- |
| `01` (1) | `MD`, vibrato | `~.0540` | `00` to `3F`, a step every 5 Ticks |
| `55` (85) | `OF`, how far into its course each sound starts | `~.0720` | `00` to `1F`, a step every 7 Ticks |
| `5B` (91) | `R1`, the small reverb's send | `~.0630` | `00` to `2F`, a step every 6 Ticks |
| `5C` (92) | `R2`, the large reverb's send | `~.0840` | `00` to `3F`, a step every 8 Ticks |
| `5D` (93) | `CH`, the chorus's send | `~.0240` | `00` to `3F`, a step every 2 Ticks |
| `5E` (94) | `DL`, the delay's send | `~.0340` | `00` to `3F`, a step every 3 Ticks |

Every value starts at `00`, the effects' load values, so each effect comes in
as its knob climbs.

The patch is made for the Built-in Synth. On another MIDI output, channel `08`
is just a channel: most synths read `47`, `4A` and `4B` as resonance, cutoff and
decay, and the accents and slides only change loudness and length. The effect
controllers carry their MIDI meanings there: `01` is the modulation wheel, `5B`
and `5D` are General MIDI's reverb and chorus sends, and `5E` is the delay send
on Roland GS modules, so most devices still add vibrato, reverb and chorus.

## Learn Orca

Learn Orca's knobs patch sends eight Control Changes whose values glide to one
of six presets chosen by hand, beside a tracker that makes the sound. This
example keeps its idea, knobs that move the sound and the effects, with a Clock
in place of the presets so the knobs move on their own, and its own bassline to
hear them on.
