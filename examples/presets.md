# Presets

`presets.orcvs` plays an acid bassline under two controls, cutoff and
resonance, and recalls their settings from two presets. Edit one selector and
both controls move to the other preset, each at its own speed. Open it with
**Help > Examples > Presets**, or as `?example=presets`, and press Space to
start Playback. Choose the Built-in Synth, or any output, in the bottom
Panel. The example opens Untitled, so **File > Save** asks where to keep your
edits.

It is an entry in the [examples index](README.md) built to be copied: each
control works on its own in another piece, and this guide adds a third, decay,
while the bassline plays.

```text
col  0         1         2
     012345678901234567890123
  0
  1  || bass ||
  2  ~*0101
  3    !~0864&v.+~.01080201
  4        ||A10200||
  5          ||A1||
  6          ||A1||
  7          ||A2||
  8          ||A1||
  9          ||C2||
 10          ||A1||
 11          ||G2||
 12          ||E2||
 13
 14  || Preset ||
 15    .%0002
 16  ||00||
 17
 18  ~*0101  || CUT ||
 19    !c084A~>08&v.+02&$0210
 20        ||20  2002  00||
 21  ||preset 00   20||
 22  ||preset 01   60||
 23
 24  ~*0101  || RES ||
 25    !c0847~>05&v.+02&$0210
 26        ||20  2002  00||
 27  ||preset 00   20||
 28  ||preset 01   50||
```

The Language Map diagnoses nothing in the file, as it opens or on any Tick
after. The bassline's steps, the presets and every value a Function writes
are Comment text, which any Function reads as data and nothing parses, and
every other Cell belongs to a Function.

## The bassline

The bassline plays on channel `08` (MIDI channel 9), the Built-in Synth's
303-style acid voice, the channel the controls send on: eight steps, one a
Tick, each at velocity `64` (100) for one Tick.

| Cells | Function | Role |
| --- | --- | --- |
| (0, 2) `~*0101` | Every | Writes `**` at (0, 3) every Tick, which activates the Timed Play east of it, at (2, 3). |
| (2, 3) `!~0864` | Timed Play | Channel `08`, velocity `64`, length `01` Tick at (20, 3). Its note is the Read south. |
| (8, 3) `&v` | Read south, nested | The note: the pair its distance's number of rows south of (10, 3), in columns 10 and 11. |
| (10, 3) `.+` | Add, nested | The Read's distance: the Clock's step plus `02`. |
| (12, 3) `~.0108` | Clock, nested | Counts `00` to `07`, one a Tick. |
| (8, 4) to (13, 4) | Comment text | What the Read, the Add and the Clock write each Tick: the step's Note, the distance and the step, `A1`, `02` and `00` at Tick 0. |
| (10, 5) to (11, 12) | Comment text | The steps, `A1 A1 A2 A1 C2 A1 G2 E2`. Type over any step while it plays. |

Every velocity is below `78` (120), so no step is accented. The acid voice
gives an accented Note its shortest decay whatever the decay controller says,
so the bassline keeps every Note unaccented to let the third control below be
heard on all of them. At the default 120 BPM a Tick is 125 ms, so each step is
a sixteenth note and the bassline loops every half bar.

## The selector

| Cells | Function | Role |
| --- | --- | --- |
| (0, 14) `\|\| Preset \|\|` | Comment | The selector's label. |
| (2, 15) `.%0002` | Modulo | Its first operand, (4, 15) and (5, 15), is the selector, `00`. Its second, `02`, is the number of presets. It answers the selector modulo `02` and writes it at (2, 16). |
| (2, 16) and (3, 16) | Comment text | The preset in force, `00` or `01`. Every control reads it here. |

Type `1` over the `0` at (5, 15) and both controls turn toward preset `01`
on the next Tick. Type `0` there again and they turn back.

The selector wraps by the number of presets: `02` selects preset `00`, `03`
selects preset `01`, and so on up to `FF`, which selects `01`. A selector that
is not a Number is diagnosed on the Modulo as any Number operand is: `0G` as
`expected a number, found "0G"`, and a selector whose second Cell you empty
with Delete as `expected a number, found "0 "`, each beside its Cells'
`invalid Language Unit character` diagnoses. The Modulo then writes nothing,
so (2, 16) keeps the preset already in force and the controls carry on toward
it until you type a Number again.

## One control

Each control is a Control Change whose value is a Slew, five rows high:

| Cells | Function | Role |
| --- | --- | --- |
| (0, 18) `~*0101` | Every | Writes `**` at (0, 19) every Tick, which activates the Control Change east of it, at (2, 19). |
| (8, 18) `\|\| CUT \|\|` | Comment | The control's label. |
| (2, 19) `!c084A` | Control Change | Channel `08`, controller `4A` (74), the acid voice's cutoff. Its value is the Slew. |
| (8, 19) `~>08` | Slew, nested | Moves its value toward its target by at most `08` a Tick, never past it. It reads its value at (8, 20) and writes the new one there. |
| (12, 19) `&v` | Read south, nested | The Slew's target: the pair its distance's number of rows south of (14, 19), in columns 14 and 15. |
| (14, 19) `.+02` | Add, nested | The Read's distance: `02` plus the preset in force. |
| (18, 19) `&$0210` | Absolute Read, nested | Reads the preset in force at (2, 16), column `02`, row `10` (16). |
| (8, 20) to (19, 20) | Comment text | What the control writes each Tick, drawn below. |
| (14, 21) and (14, 22) | Comment text | The control's targets: `20` for preset `00` and `60` for preset `01`. |

Preset `00` puts the Read's distance at `02`, so it reads row 21, and preset
`01` at `03`, so it reads row 22.

Each nested Function writes its answer one row south of its first Cell, into
the control's Comment on row 20:

```text
col  0         1         2
     012345678901234567890123
 20        ||20  2002  00||
             SS  TTDD  PP
```

| Cells | Written by | Holds |
| --- | --- | --- |
| (8, 20) `SS` | the Slew | The value the control sends this Tick. The Slew reads it back on the next Tick and moves on from it. |
| (12, 20) `TT` | the Read south | The target it read. |
| (14, 20) `DD` | the Add | The distance it computed. |
| (18, 20) `PP` | the Absolute Read | The preset it read. |

The file ships each of them holding what Tick 0 writes under preset `00`, so
the Source looks the same before and after the first Tick. `RES`, from
(0, 24) to (23, 28), is the same control with controller `47` (71), its own
`amount` and its own targets.

## The presets

| Control | Controller | Slew `amount` | Preset `00` | Preset `01` | Ticks from one to the other |
| --- | --- | --- | --- | --- | --- |
| `CUT`, cutoff | `4A` (74) | `08` | `20` (32) | `60` (96) | 8 |
| `RES`, resonance | `47` (71) | `05` | `20` (32) | `50` (80) | 10 |

Preset `00` is darker and plainer, preset `01` brighter and sharper at the
cutoff. Each control sends on channel `08` on every Tick its Every bangs it,
which is every Tick, so the acid voice hears each control sixteen times a bar
whether it is moving or not, after the bassline's Note on the same Tick.

The file opens with preset `00` already in each control's value at (8, 20)
and (8, 26), so the first Tick sends `20` to both and they hold there. Type
`1` at (5, 15) before Tick 4 and the controls move from Tick 4:

| Tick | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `CUT` | 20 | 28 | 30 | 38 | 40 | 48 | 50 | 58 | **60** | 60 | 60 | 60 |
| `RES` | 20 | 25 | 2A | 2F | 34 | 39 | 3E | 43 | 48 | 4D | **50** | 50 |

Each control moves by at most its own `amount` a Tick and lands on its
target, so `RES`'s last step is `03`, from `4D` to `50`. Then each holds its
target for as long as the preset stays.

### Changing preset during a move

A new preset turns each control from wherever it is. Type `0` at (5, 15)
before Tick 13, after `CUT` has arrived at `60` and while `RES` is still at
`4D`:

| Tick | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `CUT` | 60 | 58 | 50 | 48 | 40 | 38 | 30 | 28 | **20** | 20 | 20 |
| `RES` | 4D | 48 | 43 | 3E | 39 | 34 | 2F | 2A | 25 | **20** | 20 |

`RES` turns back from `4D` without reaching `50`. A control takes as many
Ticks as its distance needs at its `amount`, so `CUT` takes 8 Ticks to cover
`40` and `RES` 9 to cover `2D`.

### Holding a control

An `amount` of `00` holds a control where it is. Type `0` over the `8` at
(11, 19), making `CUT`'s Slew `~>00`, and change preset: `RES` moves and
`CUT` keeps sending the value it holds. Type `8` there again and `CUT` moves
on toward the preset in force.

### How often a control moves

A Slew moves only on a Tick its Control Change is banged: unbanged, the
Control Change takes no Turn, and neither does the Slew inside it. Type `2`
over the `1` at (3, 18), making `CUT`'s Every `~*0201`, and `CUT` sends on
every second Tick and moves `08` each time it sends. It moves half as fast,
reaching preset `01` over 15 Ticks rather than 8, while `RES` and the bassline
keep their every-Tick cadence.

### The values

A Control Change sends values `00` to `7F` (0 to 127), so keep every target
in that range. A target from `80` up is still a Number, so the Slew moves
toward it, but once its value passes `7F` the Control Change is diagnosed,
`MIDI value 80 is outside the range 00–7F` for a value of `80`, and sends
nothing for that control until a target in range brings the value back. The
other controls and the bassline play on.

## Rewind and reopening

The controls' values live in the Source, in their Comments. Stopping and
Rewinding Playback, with the Panel's `R` or Command Shift R on native, changes
no Cell: the next run begins the bassline at its first step, and each control
carries on from the value its Comment holds toward the preset the selector
holds. To return to the file as it opens, preset `00` with both controls at
`20`, open it again from **Help > Examples > Presets**.

## The control Region

The Region worth copying is one control: its Every and label, its Control
Change, its Comment of written values and its two presets.

```text
col  0         1         2
     012345678901234567890123
 18  ~*0101  || CUT ||          top-left (0, 18)
 19    !c084A~>08&v.+02&$0210
 20        ||20  2002  00||
 21  ||preset 00   20||
 22  ||preset 01   60||         bottom-right (23, 22)
```

| Positions | What is there |
| --- | --- |
| (0, 18) to (23, 22) | The Region: five rows, twenty-four columns. |
| (2, 19) `!c084A` | The output: channel `08` at (4, 19) and controller `4A` at (6, 19). |
| (18, 19) `&$0210` | The one reference that leaves the Region: the preset in force at (2, 16), which every control shares. |
| (8, 20) to (19, 20) | The Cells the control writes while it runs, inside its Comment. |
| (14, 21) and (14, 22) | The targets, read relative to the Region. |

Everything else in the Region is relative to it, so it works wherever it
lands as long as `&$0210` still names the preset in force. Paste it anywhere
in this file and nothing changes. In another piece, copy the Preset block,
(0, 14) to (11, 16), as well. If its Modulo lands at (x, y), the preset in
force is at (x, y + 1), and each control's Absolute Read needs that Position,
column then row, each as a two-Cell hexadecimal Number.

## Adding decay

Decay is the acid voice's controller `4B` (75) on channel `08`: how long the
filter's sweep lasts after each Note starts. Add it as a third control, while
the bassline plays if you like:

1. Drag a Region from (0, 18) to (23, 22), or hold Shift with the arrows, and
   copy it.
2. Put the Cursor on (0, 30), two rows below the last row, and paste.
3. Type `DEC` over `CUT` at (11, 30), the label.
4. Type `B` over the `A` at (7, 31): controller `4B`.
5. Type `10` over `08` at (10, 31): the Slew's `amount`.
6. Type `1` over the `2` at (14, 33): preset `00`'s target, `10`.
7. Type `7` over the `6` at (14, 34): preset `01`'s target, `70`.

```text
col  0         1         2
     012345678901234567890123
 30  ~*0101  || DEC ||
 31    !c084B~>10&v.+02&$0210
 32        ||20  2002  00||
 33  ||preset 00   10||
 34  ||preset 01   70||
```

The copy brings `CUT`'s written values, so the decay control starts from the
value `CUT` held when you copied it and reaches its own target on the Ticks
after. Copied from the file as it opens, it starts at `20` and sends `10` on
its first Tick. The same selector drives it: type `1` at (5, 15) and decay
climbs `20 30 40 50 60 70`, one step a Tick, while `CUT` and `RES` move as
before. The bassline and the other two controls play exactly as they did.

Listen to the Notes' tails. At `10` the filter closes quickly after each Note
starts, a short pluck; at `70` it stays open across the Note. Every Note of
the bassline is at velocity `64`, below the accent, so every Note follows the
decay control. An accented Note, at velocity `78` or above, takes the
shortest decay whatever the control sends.

## On another output

On another MIDI output, channel `08` is just a channel: most synths read `4A`,
`47` and `4B` as cutoff, resonance and decay, as the acid voice does.
