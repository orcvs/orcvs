# Your first result

This walkthrough takes you from a running console to a Source that Playback changes in front of
you. You write one Addition, watch the first Tick write its answer, change an operand while
Playback runs, and add a Clock whose answer counts. It needs no MIDI device.

## Before you start

Start the console from the repository root, as the [README](../README.md#run-it) describes. On
Linux, install the ALSA development package first. Then run:

```sh
cargo run
```

The first build takes several minutes. The console opens as a desktop window: a Grid of Cells,
a menu bar above it, and the Panel along the bottom. The Panel shows the tempo under **B**, the
current Tick under **T**, the time played under **C**, and the MIDI output under **O**.

## Open an empty Source

A first start opens an empty Source. Otherwise choose **File > New**: Command N on macOS, Ctrl N
on Linux and Windows. If the Source holds anything, the console asks first, and **Discard**
empties it.

The Grid is now empty. The Cursor, the outlined Cell, is at the top-left Cell, column `00` row
`00`. The console shows no coordinates; this walkthrough counts columns and rows from that Cell.

## Write an Addition

Type these six characters:

```text
.+0102
```

Each character fills the Cell under the Cursor, and the Cursor steps one Cell east. `.+` is the Add
Function; `01` and `02` are its two operands, each a two-Cell hexadecimal Number. A Function and
its operands together are an Expression. Nothing appears below the Expression yet, because no Tick
has run.

## Start Playback

Press **Space**. Playback starts at Tick `0`, the Panel's **T** begins counting, and that first
Tick writes the answer into the two Cells directly below `.+`:

```text
.+0102
03
```

A Function that answers a value writes it one row south of its first Cell unless it names another
place. Each later Tick writes the same `03` again.

## Change an operand while it plays

Leave Playback running. The Cursor is just past the last `2`, at column `06`. Press **Left Arrow**
once to put it on the `2`, then type `5`:

```text
.+0105
06
```

The next Tick reads the edited Source, so the answer becomes `06` without stopping anything. The
Cursor is back at column `06`.

## Add a value that changes

Press **Down Arrow** three times and **Left Arrow** six times, so the Cursor is at column `00`,
row `03`. Type:

```text
~.0804
```

`~.` is the Clock Function. Its first operand, `08`, is how many Ticks one step lasts; its second,
`04`, is how many steps the cycle holds. The two Cells below it, on row `04`, now count `00`, `01`,
`02`, `03` and back to `00`. The Panel's **B** shows 120 BPM, and a beat is four Ticks, so
Playback runs 8 Ticks a second and the count moves about once a second. The Clock reads the Tick
number, which kept counting while you worked, so it may start part way through its cycle.

For example, part way through:

```text
.+0105
06

~.0804
02
```

## Stop and rewind

Press **Space** again. Playback stops and holds its Tick: **T** and the Clock keep their last
values, and the next **Space** carries on from the following Tick. To begin again from Tick `0`,
press **Command Shift R** on macOS, or **Ctrl Shift R** on Linux and Windows, before you press
**Space**.

## If the result differs

- **Nothing appears below the Expression.** Check that **T** is counting: **Space** starts and
  stops Playback. Then check each operand is two Cells from `00` to `FF`, with uppercase letters.
  In `.+010a` the second operand is not a Number, so the console draws those Cells in its
  diagnostic colour and the Addition writes nothing.
- **An old answer stays after an edit.** A Function that is invalid writes nothing, so the answer
  its last valid Tick wrote is left standing. Correct the operand and the next Tick replaces it.
- **You cannot type a space.** **Space** controls Playback. To empty a Cell, put the Cursor on it
  and press **Backspace** or **Delete**.

## Next steps

- **Help > Function Reference** replaces the running Source with one worked example of every
  Function. Playing it shows each result being written.
- [`CONTEXT.md`](../CONTEXT.md) defines each term this walkthrough used: Source, Grid, Cell,
  Expression, Function, Number, Tick, Playback.
- Sound needs a MIDI destination chosen under the Panel's **O** and a Function that sends MIDI,
  such as `!>`. The Function Reference's MIDI column shows each one.
