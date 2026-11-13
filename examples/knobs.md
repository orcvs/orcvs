# Knobs

`knobs.orcvs` turns three of the 303-style voice's knobs and the Built-in
Synth's eight effects over a sixteen-step acid bassline. Open it with
**Help > Examples > Knobs** and press Space to start Playback. Each knob's
value climbs a step at a time and glides back down when its Clock wraps; the
Sound Module on orcvs.com shows each effect move. The example opens Untitled,
so **File > Save** asks where to keep your edits.

```text
col  0         1         2         3         4         5
     0123456789012345678901234567890123456789012345678901
  0
  1    || bass ||
  2    ~*0101    .+04~.0110
  3      !~0964&>  01  ||  A1  A1  A1  A1A1    A2  A1G2  ||
  4
  5    || accent ||
  6    ~*0101    .+04~.0110
  7      !~097F&>  01  ||A1      C2        E2    C3    E2||
  8
  9    || slide ||
 10    ~*0101    .+04~.0110
 11      !~0964&>  02  ||    A2      G2      A1          ||
 12
 13
 14    || knobs ||
 15    ~*0101
 16      !c094A~>08~.0180    || CUT ||
 ...
 22      !c094B~>08~.0340    || DEC ||
 23
 24
 25    || effects ||
 26    ~*0101
 27      !c0001~>08~.0540        || MD ||
 ...
 48      !c0008~>08~.0840        || R2 ||
```

## The bassline

The bassline plays on channel `09`, the Built-in Synth's 303-style voice
(ADR 0099). Its three rows are the same voice of the Sequencer example, each
reading the same sixteen steps and playing them its own way. Each step holds a
Note in one row and is empty in the other two, where its Timed Play sends
nothing and reports nothing.

| Row | Timed Play | How its Notes play |
| --- | --- | --- |
| `bass` | `!~0964&>  01` | Velocity `64` (100), one Tick. |
| `accent` | `!~097F&>  01` | Velocity `7F` (127). From `78` up a Note is accented: louder, its filter opened further, and its sweep at its shortest. |
| `slide` | `!~0964&>  02` | Velocity `64`, two Ticks. The Note still sounds when the next step's Note starts, so the voice glides to it without striking it afresh. |

Move a Note from one row to another, keeping its step, to change how it plays.
A slide to the same Note strikes it again, because Playback stops a Note before
it plays it again.

| Cells | Function | Role |
| --- | --- | --- |
| (2, 2) `~*0101` | Delay | Writes `**` at (2, 3) every Tick, which activates the Timed Play at (4, 3). |
| (12, 2) `.+04~.0110` | Add, with a nested Clock | The Clock counts `00` to `0F`, one a Tick. The Add adds `04` and writes the sum into the Read's `n` slot at (12, 3). |
| (4, 3) `!~0964&>  01` | Timed Play with a nested Read east | Channel `09`, velocity `64`, length `01` Tick. Its note slot holds `&>`, which reads the step `n` pairs east of (12, 3). |
| (20, 3) to (51, 3) | Comment text | The sixteen steps, one Note to a pair. Type over any pair while it plays. |

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the bassline loops every bar.

## One knob

| Cells | Function | Role |
| --- | --- | --- |
| (2, 15) `~*0101` | Delay | Writes `**` at (2, 16) every Tick, which activates the Control Change at (4, 16). |
| (4, 16) `!c094A` | Control Change | Channel `09`, controller `4A` (74), the acid voice's cutoff. Its value slot holds the Interpolation. |
| (10, 16) `~>08` | Interpolation, nested | Moves its value toward the Clock's by at most `08` a Tick. It writes the value it sends at (10, 17). |
| (14, 16) `~.0180` | Clock, nested | Counts `00` to `7F`, one step a Tick, then wraps. |

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

Each effect knob is the same machine on channel `00`, sending controller `01`
to `08`, the Built-in Synth's effects (ADR 0088). Each effect is global, so the
channel picks nothing. `LN` and `EQ` sweep around their load values, `7F` and
`40`, so an Add puts a floor under the Clock:

| Cells | Function | Role |
| --- | --- | --- |
| (4, 33) `!c0003` | Control Change | Channel `00`, controller `03`, the `LN` effect. Its value slot holds the Interpolation. |
| (10, 33) `~>08` | Interpolation, nested | Moves its value toward the Add's by at most `08` a Tick. |
| (14, 33) `.+40` | Add, nested | Adds `40` to the Clock's count. |
| (18, 33) `~.0440` | Clock, nested | Counts `00` to `3F`, a step every 4 Ticks, then wraps. |

Each effect sweeps at its own rate, so the effects peak at different times:

| Controller | Effect | Clock | Sweep |
| --- | --- | --- | --- |
| `01` | `MD`, vibrato | `~.0540` | `00` to `3F`, a step every 5 Ticks |
| `02` | `OF`, how far into its course each sound starts | `~.0720` | `00` to `1F`, a step every 7 Ticks |
| `03` | `LN`, how long each sound decays | `.+40~.0440` | `40` to `7F`, a step every 4 Ticks |
| `04` | `DL`, delay | `~.0340` | `00` to `3F`, a step every 3 Ticks |
| `05` | `CH`, chorus | `~.0240` | `00` to `3F`, a step every 2 Ticks |
| `06` | `EQ`, tilt | `.+30~.0520` | `30` to `4F`, darker to brighter, a step every 5 Ticks |
| `07` | `R1`, small reverb | `~.0630` | `00` to `2F`, a step every 6 Ticks |
| `08` | `R2`, large reverb | `~.0840` | `00` to `3F`, a step every 8 Ticks |

Every value starts at `00`, so for the first second after you start Playback
`LN` and `EQ` glide up from below their sweeps: the sounds start short and dark.

The patch is made for the Built-in Synth. On another MIDI output, channel `09`
is just a channel: most synths read `47`, `4A` and `4B` as resonance, cutoff and
decay, and the accents and slides only change loudness and length. Controllers
`01` to `08` carry General MIDI's meanings instead: `01` is the modulation
wheel and `07` channel volume, so the sweep on `07` swells channel 1's volume
on most devices.

## Learn Orca

Learn Orca's knobs patch sends eight Control Changes whose values glide to one
of six presets chosen by hand, beside a tracker that makes the sound. This
example keeps its idea, knobs that move the sound and the effects, with a Clock
in place of the presets so the knobs move on their own, and its own bassline to
hear them on.
