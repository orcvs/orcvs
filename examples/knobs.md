# Knobs

`knobs.orcvs` is the knobs patch from [Learn Orca](https://metasyn.srht.site/learn-orca/)
in Orcvs: eight knobs, each sending a MIDI Control Change every Tick, whose
values glide to one of six presets. Open it with **Help > Examples > Knobs** and
press Space to start Playback. Type a preset from `00` to `05` over the `00` at
the top while it plays, and every knob glides to its value in that preset. The
first MIDI output is chosen for you; change it in the bottom Panel. The example
opens Untitled, so **File > Save** asks where to keep your edits.

```text
col  0         1         2         3
     01234567890123456789012345678901234567
  0
  1                      ||00||
  2    ~*0101        .+04&^01
  3      !c0041~>04&>      ||00000F373737||
  4
  5    ~*0101        .+04&^04
  6      !c0042~>04&>      ||0B370F370F37||
 ...
 23    ~*0101        .+04&^16
 24      !c0048~>04&>      ||424242214D42||
```

The preset and each knob's six presets are `||` Comments, so the Numbers in
them are text the Reads select rather than Functions. The preset is the pair at
column 22 of row 1. A knob's presets are the pairs from column 24: preset `00`
at column 24, preset `01` at column 26, and so on to preset `05` at column 34.
As Learn Orca ships it, preset `00` is selected. Keep each row's `||` at both
ends.

| Knob | Controller | `00` | `01` | `02` | `03` | `04` | `05` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `41` | `00` | `00` | `0F` | `37` | `37` | `37` |
| 2 | `42` | `0B` | `37` | `0F` | `37` | `0F` | `37` |
| 3 | `43` | `00` | `00` | `08` | `00` | `04` | `00` |
| 4 | `44` | `00` | `21` | `13` | `21` | `13` | `21` |
| 5 | `45` | `00` | `0B` | `00` | `0B` | `3B` | `0B` |
| 6 | `46` | `00` | `16` | `37` | `16` | `37` | `16` |
| 7 | `47` | `00` | `16` | `37` | `16` | `37` | `16` |
| 8 | `48` | `42` | `42` | `42` | `21` | `4D` | `42` |

## From Orca to Orcvs

Each knob is the same machine in both languages.

| Orca | Orcvs | In this patch |
| --- | --- | --- |
| `Q` query | `&^` Read north | Reads the preset at the top of columns 22 and 23, `n` rows north of its `n` operand. |
| `J` jumper, `V` variable | `.+` Add and `&>` Read east | The Add points the Read at the knob's value in the selected preset, `04` pairs past the Read's `n` slot and on by one pair a preset. |
| `Z` lerp | `~>` Interpolation | Moves the knob's value toward that preset's value by at most `04` a Tick, and keeps it in the Cells below itself. |
| `D1` delay and its `*` | `~*0101` Delay | Bangs every Tick, which activates the Control Change east of its `**`. |
| `!` cc | `!c` Control Change | Sends the knob's value to its controller on channel `00`. |

## One knob

For knob 1, with its machine at rows 2 and 3:

| Cells | Function | Role |
| --- | --- | --- |
| (2, 2) `~*0101` | Delay | Writes `**` at (2, 3) every Tick, which activates the Control Change at (4, 3). |
| (16, 2) `.+04&^01` | Add, with a nested Read north | Adds `04` to the preset, which the Read reads `01` row north of its `n` operand, at (22, 1), and writes the sum into the Read east's `n` slot at (16, 3). The nested Read also writes the preset at (20, 3). |
| (4, 3) `!c0041` | Control Change | Channel `00` (MIDI channel 1 in most DAWs), controller `41`. Its value slot holds the Interpolation. |
| (10, 3) `~>04&>` | Interpolation, nested, with a nested Read east | Rate `04`. Its target is the pair the Read east reads, `n` pairs east of (16, 3). It writes the value it sends at (10, 4) and reads it back there on the next Tick. |
| (24, 3) to (35, 3) | Comment text | The knob's six presets. |

Knob `k`, from `0` to `7`, has its Delay at `(2, 2 + 3k)`, its Add fourteen
columns east of it, its Control Change one row down and two columns east of
the Delay, and controller `41 + k`. Its Read north's `n` is `1 + 3k`, the
distance from its row to row 1, so every knob reads the same preset.

Every value starts at `00`, so after you start Playback the knobs take up to
seventeen Ticks to reach preset `00`. At the default 120 BPM a Tick is
125 ms; change the tempo in the Panel's BPM field.

## Where it differs from Orca

- Orca's values are base 36 digits, `0` to `z`, and its `!` scales each to a
  MIDI value by `ceil(127 × value / 35)`. Orcvs sends a Number from `00` to
  `7F` as it is, so each preset here is Orca's digit already scaled: Orca's `3`
  is `0B`, `f` is `37` and `i` is `42`.
- Orca's `!` adds an offset of 64 to its knob, so knobs `1` to `8` are
  controllers `41` to `48`. Orcvs's `!c` names the controller itself.
- Orca copies the selected preset's row out with `Q` and hands each value to
  its knob through a variable. Each Orcvs knob reads the preset itself and
  reads its own value from its row of presets.
- Orca's `Z` reads an empty rate as `0`, which holds its value where it is. The
  port gives the Interpolation a rate of `04`, about Orca's step of one digit,
  so a new preset is heard.
- Orca's patch is saved with preset `00`'s values already under each `Z`, so
  its knobs send them from the first frame. Each Orcvs knob starts at `00` and
  glides up to them.
- Stopping sends All Notes Off, Reset All Controllers and a centred pitch bend
  on every channel.
