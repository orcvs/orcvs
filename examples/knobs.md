# Knobs

`knobs.orcvs` is the knobs patch from [Learn Orca](https://metasyn.srht.site/learn-orca/)
in Orcvs, over a sixteen-step bassline: eight knobs, each setting one of the
Built-in Synth's effects every Tick, whose values glide to one of six presets.
Open it with **Help > Examples > Knobs** and press Space to start Playback.
Type a preset from `00` to `05` over the `00` at row 4 while it plays, and
every knob glides to its value in that preset; the Sound Module on orcvs.com
shows each effect move. The first MIDI output is chosen for you; change it in
the bottom Panel. The example opens Untitled, so **File > Save** asks where to
keep your edits.

```text
col  0         1         2         3         4         5
     0123456789012345678901234567890123456789012345678901
  0
  1    ~*0101    .+04~.0110
  2      !~0264&>  01  ||A1A1A2A1C2A1G2A1A1E2A1A2C3A1G2E2||
  3
  4                      ||00||
  5    ~*0101        .+04&^01
  6      !c0001~>08&>      ||3000107F2050||
  7
  8    ~*0101        .+04&^04
  9      !c0002~>08&>      ||000020000010||
 ...
 26    ~*0101        .+04&^16
 27      !c0008~>08&>      ||000000004040||
```

## The bassline

The bassline is one voice of the Sequencer example: a Delay, an Add with a
nested Clock, and a Timed Play whose nested Read east picks the step.

| Cells | Function | Role |
| --- | --- | --- |
| (2, 1) `~*0101` | Delay | Writes `**` at (2, 2) every Tick, which activates the Timed Play at (4, 2). |
| (12, 1) `.+04~.0110` | Add, with a nested Clock | The Clock counts `00` to `0F`, one a Tick. The Add adds `04` and writes the sum into the Read's `n` slot at (12, 2). |
| (4, 2) `!~0264&>  01` | Timed Play with a nested Read east | Channel `02`, the Built-in Synth's saw, velocity `64` (100), length `01` Tick. Its note slot holds `&>`, which reads the step `n` pairs east of (12, 2). |
| (20, 2) to (51, 2) | Comment text | The sixteen steps, one Note to a pair: `A1 A1 A2 A1 C2 A1 G2 A1 A1 E2 A1 A2 C3 A1 G2 E2`. Type over any pair while it plays. |

At the default 120 BPM a Tick is 125 ms, so each step is a sixteenth note and
the bassline loops every bar.

## The knobs

The preset and each knob's six presets are `||` Comments, so the Numbers in
them are text the Reads select rather than Functions. The preset is the pair at
column 22 of row 4. A knob's presets are the pairs from column 24: preset `00`
at column 24, preset `01` at column 26, and so on to preset `05` at column 34.
Preset `00` is selected as shipped. Keep each row's `||` at both ends.

Knob `k` sends Control Change `k` on channel `00`, which sets the Built-in
Synth's effect `k` (ADR 0088). Each effect is global, so the channel picks
nothing.

The patch is made for the Built-in Synth. On another MIDI output, controllers
`01` to `08` carry General MIDI's meanings instead: `01` is the modulation
wheel, `06` Data Entry, `07` channel volume and `08` balance. Preset `00` sends
`07` a `00`, which silences channel 1 on most devices.

| Knob | Controller | Effect | `00` | `01` | `02` | `03` | `04` | `05` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `01` | `MD` | `30` | `00` | `10` | `7F` | `20` | `50` |
| 2 | `02` | `OF` | `00` | `00` | `20` | `00` | `00` | `10` |
| 3 | `03` | `LN` | `7F` | `7F` | `40` | `7F` | `60` | `7F` |
| 4 | `04` | `DL` | `00` | `00` | `30` | `00` | `40` | `20` |
| 5 | `05` | `CH` | `00` | `00` | `00` | `40` | `00` | `30` |
| 6 | `06` | `EQ` | `40` | `40` | `30` | `40` | `60` | `40` |
| 7 | `07` | `R1` | `00` | `00` | `30` | `00` | `00` | `20` |
| 8 | `08` | `R2` | `00` | `00` | `00` | `00` | `40` | `40` |

Preset `00` adds vibrato to the synth's load values, and preset `01` is the
load values alone, the synth with no effects. Only `MD`, the vibrato, sounds
so far; each other effect shows its value in the Sound Module and changes
nothing until its sound lands.

## From Orca to Orcvs

Each knob is the same machine in both languages.

| Orca | Orcvs | In this patch |
| --- | --- | --- |
| `Q` query | `&^` Read north | Reads the preset at row 4, columns 22 and 23, `n` rows north of its `n` operand. |
| `J` jumper, `V` variable | `.+` Add and `&>` Read east | The Add points the Read at the knob's value in the selected preset, `04` pairs past the Read's `n` slot and on by one pair a preset. |
| `Z` lerp | `~>` Interpolation | Moves the knob's value toward that preset's value by at most `08` a Tick, and keeps it in the Cells below itself. |
| `D1` delay and its `*` | `~*0101` Delay | Bangs every Tick, which activates the Control Change east of its `**`. |
| `!` cc | `!c` Control Change | Sends the knob's value to its effect's controller on channel `00`. |

## One knob

For knob 1, with its machine at rows 5 and 6:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 5) `~*0101` | Delay | Writes `**` at (2, 6) every Tick, which activates the Control Change at (4, 6). |
| (16, 5) `.+04&^01` | Add, with a nested Read north | Adds `04` to the preset, which the Read reads `01` row north of its `n` operand, at (22, 4), and writes the sum into the Read east's `n` slot at (16, 6). The nested Read also writes the preset at (20, 6). |
| (4, 6) `!c0001` | Control Change | Channel `00` (MIDI channel 1 in most DAWs), controller `01`, the `MD` effect. Its value slot holds the Interpolation. |
| (10, 6) `~>08&>` | Interpolation, nested, with a nested Read east | Rate `08`. Its target is the pair the Read east reads, `n` pairs east of (16, 6). It writes the value it sends at (10, 7) and reads it back there on the next Tick. |
| (24, 6) to (35, 6) | Comment text | The knob's six presets. |

Knob `k`, from `0` to `7`, has its Delay at `(2, 5 + 3k)`, its Add fourteen
columns east of it, its Control Change one row down and two columns east of
the Delay, and controller `01 + k`. Its Read north's `n` is `1 + 3k`, the
distance from its row to row 4, so every knob reads the same preset.

Every value starts at `00`, so after you start Playback the knobs take up to
sixteen Ticks, two seconds at the default 120 BPM, to reach a preset. Until
then `LN` and `EQ` sit below their load values, so a sound's length and its
brightness rise over the first two seconds. Change the tempo in the Panel's
BPM field.

## Where it differs from Orca

- Learn Orca's knobs turn Enfer's effects. The Built-in Synth takes Enfer's
  place, so each knob sends the controller of one of its effects, `01` to
  `08`, rather than Orca's `!` controllers `41` to `48`, and the presets are
  values chosen for those effects rather than Orca's.
- Orca's patch makes no sound alone; Learn Orca plays it beside a tracker. The
  port carries its own bassline, so the effects have something to act on.
- Orca copies the selected preset's row out with `Q` and hands each value to
  its knob through a variable. Each Orcvs knob reads the preset itself and
  reads its own value from its row of presets.
- Orca's `Z` reads an empty rate as `0`, which holds its value where it is. The
  port gives the Interpolation a rate of `08`, so a new preset is heard within
  two seconds.
- Orca's patch is saved with preset `00`'s values already under each `Z`, so
  its knobs send them from the first frame. Each Orcvs knob starts at `00` and
  glides up to them.
- Stopping sends All Notes Off, Reset All Controllers and a centred pitch bend
  on every channel.
