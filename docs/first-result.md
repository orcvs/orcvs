# Your first result

In about ten minutes you'll type a small sum, watch Orcvs work out the answer, change the sum
while it plays, and add a counter that keeps time with the beat. You need nothing but a web
browser.

## Before you start

On this site, press any example drawn as a grid of Cells, like the one under **Write an
Addition**. The console opens in the corner of the page with that example in it. To use the
whole window instead, choose **Console** in the sidebar. To run Orcvs on your own computer, see
[Run it on your desktop](#run-it-on-your-desktop).

The console is a Grid of Cells with the Panel along the bottom. The Panel shows the tempo under
**B**, the current Tick under **T**, the time played under **C**, and where sound goes under
**O**.

## Write an Addition

Press this example to open it in the console:

```text
.+0102
```

The Cursor, the outlined Cell, starts on the `.`. Typing a character fills the Cell under the
Cursor and steps the Cursor one Cell to the right, so you can click the console and type the six
characters over the top to see how it works.

`.+` is the Add Function. `01` and `02` are its two operands, each a two-Cell hexadecimal Number.
A Function and its operands together are an Expression. Nothing appears below it yet, because
nothing is playing.

## Start Playback

Click the console so it has the keyboard, then press **Space**, or press the **P** Cell at the
start of the Panel. Playback starts at Tick `0`, the Panel's **T** begins counting, and the first
Tick writes the answer into the two Cells directly below `.+`:

```text
.+0102
03
```

A Function that answers a value writes it one row below its first Cell, unless it names another
place. Each later Tick writes the same `03` again.

## Change an operand while it plays

Leave Playback running. Click the `2`, or move the Cursor onto it with the arrow keys, and type
`5`:

```text
.+0105
06
```

The next Tick reads what you typed, so the answer becomes `06` without stopping anything.

## Add a value that changes

Click the Cell three rows below the `.` of `.+`, and type:

```text
~.0804
```

`~.` is the Clock Function. Its first operand, `08`, is how many Ticks one step lasts; its second,
`04`, is how many steps make a cycle. The two Cells below it count `00`, `01`, `02`, `03` and back
to `00`. The Panel's **B** shows 120 BPM, and a beat is four Ticks, so Playback runs 8 Ticks a
second and the count moves about once a second. The Clock reads the Tick number, which kept
counting while you worked, so it may start partway through its cycle.

Partway through, the Grid looks like this:

```text
.+0105
06

~.0804
02
```

## Stop and rewind

Press **Space** or **P** again. Playback stops and holds its Tick: **T** and the Clock keep their
last values, and the next **Space** carries on from the following Tick. To begin again from Tick
`0`, press the **R** Cell beside **P** before you press **Space**.

## If something looks different

- **Nothing appears below the Expression.** Check that **T** is counting; if it isn't, click the
  console and press **Space**, or press **P**. Then check each operand is two Cells from `00` to
  `FF`, with uppercase letters. In `.+010a` the second operand is not a Number, so the console
  colours those Cells as an error and the Addition writes nothing.
- **An old answer stays after an edit.** A Function with an error writes nothing, so the answer
  from its last good Tick stays. Fix the operand and the next Tick replaces it.
- **Typing does nothing.** The console only hears the keyboard after you click it. Click the
  console, then type.
- **You can't type a space.** **Space** starts and stops Playback. To empty a Cell, put the Cursor
  on it and press **Backspace** or **Delete**.

## Next steps

- The [Function reference](functions.md) describes every Function, with an example to try.
- To hear something, use a Function that plays a note, such as `!>`, which the Function
  reference's MIDI output section describes. On this site the Panel's **O** shows Built-in Synth,
  which plays it in the browser; [its channels](functions.md#built-in-synth) pick the sound. On your desktop, choose a MIDI destination under **O**.
- In the full console, **Help > Function Reference** replaces what is on the Grid with one worked
  example of every Function. Play it to watch each answer being written.
- [`CONTEXT.md`](../CONTEXT.md), on GitHub, defines each term this walkthrough used: Source, Grid,
  Cell, Expression, Function, Number, Tick and Playback.

## Run it on your desktop

The desktop console adds saving and opening files and plays to MIDI devices. Build it from a
checkout of the repository, as the [README](../README.md#run-it) describes. On Linux, install the
ALSA development package first. Then, from the repository root, run:

```sh
cargo run
```

The first build takes several minutes. The console opens in its own window, with a menu bar above
the Grid. A first start opens an empty Grid; afterwards, choose **File > New** (Command N on
macOS, Ctrl N on Linux and Windows) for an empty one. Every step above works the same way, and
**Command Shift R** (Ctrl Shift R on Linux and Windows) also rewinds to Tick `0`.
