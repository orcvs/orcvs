# Knobs

`knobs.orcvs` sweeps six of the Built-in Synth's effects over a sixteen-step
bassline. Open it with **Help > Examples > Knobs** and press Space to start
Playback. Each knob's value climbs a step at a time and glides back to `00`
when its Clock wraps; the Sound Module on orcvs.com shows each effect move.
The example opens Untitled, so **File > Save** asks where to keep your edits.

```text
col  0         1         2         3         4         5
     0123456789012345678901234567890123456789012345678901
  0
  1    || bass ||
  2    ~*0101    .+04~.0110
  3      !~0264&>  01  ||A1A1A2A1C2A1G2A1A1E2A1A2C3A1G2E2||
  4
  5
  6    || effects ||
  7    ~*0101
  8      !c0001~>08~.0180    || MD ||
  9
 10    ~*0101
 11      !c0002~>08~.0320    || OF ||
 ...
 23      !c0008~>08~.0440    || R2 ||
```

## The bassline

The bassline is one voice of the Sequencer example.

| Cells | Function | Role |
| --- | --- | --- |
| (2, 2) `~*0101` | Delay | Writes `**` at (2, 3) every Tick, which activates the Timed Play at (4, 3). |
| (12, 2) `.+04~.0110` | Add, with a nested Clock | The Clock counts `00` to `0F`, one a Tick. The Add adds `04` and writes the sum into the Read's `n` slot at (12, 3). |
| (4, 3) `!~0264&>  01` | Timed Play with a nested Read east | Channel `02`, the Built-in Synth's saw, velocity `64` (100), length `01` Tick. Its note slot holds `&>`, which reads the step `n` pairs east of (12, 3). |
| (20, 3) to (51, 3) | Comment text | The sixteen steps, one Note to a pair. Type over any pair while it plays. |

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the bassline loops every bar.

## One knob

| Cells | Function | Role |
| --- | --- | --- |
| (2, 7) `~*0101` | Delay | Writes `**` at (2, 8) every Tick, which activates the Control Change at (4, 8). |
| (4, 8) `!c0001` | Control Change | Channel `00`, controller `01`, the `MD` effect (ADR 0088). Its value slot holds the Interpolation. |
| (10, 8) `~>08` | Interpolation, nested | Moves its value toward the Clock's by at most `08` a Tick. It writes the value it sends at (10, 9). |
| (14, 8) `~.0180` | Clock, nested | Counts `00` to `7F`, one step a Tick, then wraps. |

Each knob's Clock sets how fast and how far its effect sweeps:

| Controller | Effect | Clock | Sweep |
| --- | --- | --- | --- |
| `01` | `MD` | `~.0180` | `00` to `7F`, a step a Tick |
| `02` | `OF` | `~.0320` | `00` to `1F`, a step every 3 Ticks |
| `04` | `DL` | `~.0240` | `00` to `3F`, a step every 2 Ticks |
| `05` | `CH` | `~.0340` | `00` to `3F`, a step every 3 Ticks |
| `07` | `R1` | `~.0230` | `00` to `2F`, a step every 2 Ticks |
| `08` | `R2` | `~.0440` | `00` to `3F`, a step every 4 Ticks |

`LN` and `EQ` keep their load values. Change a Clock's `rate` or `modulus`
while it plays to change its sweep. Only `MD`, the vibrato, sounds so far;
each other effect shows its value in the Sound Module and changes nothing until
its sound lands.

The patch is made for the Built-in Synth. On another MIDI output, controllers
`01` to `08` carry General MIDI's meanings instead: `01` is the modulation
wheel and `07` channel volume, so the sweep on `07` swells channel 1's volume
on most devices.

## Learn Orca

Learn Orca's knobs patch sends eight Control Changes whose values glide to one
of six presets chosen by hand, beside a tracker that makes the sound. This
example keeps its idea, knobs that move effects, with a Clock in place of the
presets so the knobs move on their own, and its own bassline to hear them on.
