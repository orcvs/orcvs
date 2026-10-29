# Function reference

Every Orcvs Function, with what it takes, what it does, and an example you can type into the
console. For a first session, start with [Your first result](first-result.md). For what each term
means, see [`CONTEXT.md`](../CONTEXT.md). The spellings and operands here come from the Function
table in `lang/src/atom.rs`, and `orcvs/tests/function_reference_doc.rs` runs every example below
against the language, so a change to a Function that this page does not follow fails a test.

The language is pre-release, so entries change with it.

## How to read an entry

**Spelling.** Every Function is two Cells, such as `.+`. Its operands follow in the same row with no
spaces, two Cells each: `.+0102` is Add with the operands `01` and `02`. The first Cell of the
spelling is the Function's anchor. The entry heading gives the spelling and a name.

**Operand types.**

| Type | Cells | Meaning |
|---|---|---|
| Number | `00` to `FF` | A byte in uppercase hexadecimal. `0a` and `G0` are not Numbers. |
| Note | Pitch letter then octave | Uppercase for a natural, lowercase for a sharp, then `/` for the octave below `0` or `0` to `9`: `C4`, `c4`, `C/`. A Note is MIDI note `00` to `7F`. |
| Untyped | Any two Cells | Carried unchanged; whatever reads them at the destination decides what they mean. |
| MIDI domains | `00` to `0F` or `00` to `7F` | Numbers that a MIDI Function also checks against a range, named in its entry. |

The same two Cells can be a Number in one operand and a Note in another: `C4` is Number `C4`
where a Number is expected.

**Answer or effect.** A Function either answers a value or performs an effect, never both. A
Function that answers a value writes it into its Output Portal, the two Cells one row south of its
anchor unless the entry names another place. A value Function can also stand in another Function's
operand, nested: `.+.^3C01` uses the Note `.^3C` answers as Add's first operand. A Function that
performs an effect, such as playing a note, writes no answer and cannot be nested.

**Runs.** Most Functions take a Turn every Tick. Some are inert until a Bang reaches them. A Bang
activates each root whose anchor is aligned with the `**` in one of four places: in the same two
columns one row north or one row south, or in the two Cells immediately before or after it on its
row. A `**` that touches another Function's operand activates nothing: `!>0064C4**` does not
play, because the Cells before the `**` are the operand `C4`, not the anchor `!>`. Each entry says
which kind of Function it is.

**Clear.** Some value Functions answer a Clear rather than a value: they write two empty Cells. An
invalid Function writes nothing at all, so its last answer stays where it was.

**Examples.** An example shows the Source before any Tick, then the Source after the Ticks named
above each later block, counted from Tick `0`. An example that plays MIDI also says how many
Play Commands its Tick sends.

## Arithmetic

Each takes two Numbers, `left` and `right`, answers a Number, and runs every Tick. A Note in either
operand is refused; convert it with [`.v`](#v-convert-to-number) first.

### `.+` Add

Answers `left + right`, wrapping past `FF` to `00`.

```orcvs
.+0102
```

After Tick `0`:

```orcvs tick 0
.+0102
03
```

### `.-` Subtract

Answers `left - right`, wrapping below `00` to `FF`.

```orcvs
.-0503
```

After Tick `0`:

```orcvs tick 0
.-0503
02
```

### `.|` Absolute Difference

Answers the distance between `left` and `right`, whichever is larger. It never wraps.

```orcvs
.|0307
```

After Tick `0`:

```orcvs tick 0
.|0307
04
```

### `.x` Multiply

Answers `left * right`, wrapped to a byte.

```orcvs
.x0304
```

After Tick `0`:

```orcvs tick 0
.x0304
0C
```

### `./` Divide

Answers `left / right`, rounded down. A `right` of `00` diagnoses and writes nothing.

```orcvs
./0902
```

After Tick `0`:

```orcvs tick 0
./0902
04
```

### `.%` Modulo

Answers the remainder of `left / right`. A `right` of `00` diagnoses and writes nothing.

```orcvs
.%0902
```

After Tick `0`:

```orcvs tick 0
.%0902
01
```

### `.<` Minimum

Answers the smaller of `left` and `right`.

```orcvs
.<0305
```

After Tick `0`:

```orcvs tick 0
.<0305
03
```

### `.>` Maximum

Answers the larger of `left` and `right`.

```orcvs
.>0305
```

After Tick `0`:

```orcvs tick 0
.>0305
05
```

### `.=` Equality

Answers a Bang, written as `**`, when `left` equals `right`, and a Clear when they differ. Its
Bang activates the roots aligned with the `**` it writes, so it can gate a Function that needs a
Bang.

```orcvs
.=0505  .=0506
```

After Tick `0`:

```orcvs tick 0
.=0505  .=0506
**
```

## Numeric conversion

The only way between a Number and a Note. Each runs every Tick.

### `.v` Convert to Number

`.v note` answers the MIDI note number of a Note.

```orcvs
.vC4
```

After Tick `0`:

```orcvs tick 0
.vC4
3C
```

### `.^` Convert to Note

`.^ value` answers the Note for a Number from `00` to `7F`. `80` to `FF` diagnoses and writes
nothing.

```orcvs
.^3C
```

After Tick `0`:

```orcvs tick 0
.^3C
C4
```

## Tick

These read the Tick number or their own last answer, so they change over time. Each runs every
Tick.

### `~.` Clock

`~. rate modulus` answers which step a cycle is on: each step lasts `rate` Ticks, and the cycle
holds `modulus` steps, counted from `00`. It reads the Tick number, so a stopped and resumed
Clock carries on from the same step. A `rate` or `modulus` of `00` diagnoses.

```orcvs
~.0204
```

After Tick `0`, then after Tick `2`:

```orcvs tick 0
~.0204
00
```

```orcvs tick 2
~.0204
01
```

### `~*` Delay

`~* rate modulus` answers a Bang once every `rate * modulus` Ticks, starting at Tick `0`, and a
Clear on every other Tick. A `rate` or `modulus` of `00` diagnoses.

```orcvs
~*0102
```

After Tick `0`, then after Tick `1`:

```orcvs tick 0
~*0102
**
```

```orcvs tick 1
~*0102
```

### `~%` Euclidean

`~% hits steps` answers a Bang on `hits` Ticks spread as evenly as possible across each cycle of
`steps` Ticks, and a Clear on the rest. `~%0308` gives the pattern `X..X..X.`. A `steps` of `00`,
or more `hits` than `steps`, diagnoses.

```orcvs
~%0308
```

After Tick `0`, then after Tick `1`:

```orcvs tick 0
~%0308
**
```

```orcvs tick 1
~%0308
```

### `~+` Increment

`~+ step modulus` reads the Number in its own Output Portal, adds `step`, and answers the result
modulo `modulus`. Empty Cells there count as `00`, so it counts up from wherever its last answer
left off. A `modulus` of `00`, or anything other than a Number in its Output Portal, diagnoses.

```orcvs
~+0104
```

After Tick `0`, then after Tick `4`:

```orcvs tick 0
~+0104
01
```

```orcvs tick 4
~+0104
01
```

### `~>` Interpolation

`~> rate target` reads the Number in its own Output Portal and moves it toward `target` by at most
`rate` each Tick, never past it. Empty Cells there count as `00`. A `rate` of `00` holds the
value.

```orcvs
~>0205
```

After Tick `0`, then after Tick `2`:

```orcvs tick 0
~>0205
02
```

```orcvs tick 2
~>0205
05
```

### `~?` Random

`~? seed minimum maximum` answers a Number from `minimum` to `maximum`, inclusive. The result
depends only on `seed`, the Tick number, and the Function's own position, so the same Source
replays the same values, and moving the Function changes its values. Reversed bounds give the same
range.

```orcvs
~?010010
```

After Tick `0`:

```orcvs tick 0
~?010010
02
```

## Copy

A Copy moves one two-Cell pair from its Input Portal to its Output Portal every Tick. It takes
whatever the pair holds, a value, a Bang or two empty Cells, and writes it over the whole
destination. Part of a Function, or Cells that do not form one unit, diagnoses. A copied `**`
activates the roots aligned with its destination.

### `=v` Copy South

Reads the pair one row north and writes it one row south.

```orcvs
C4
=v
```

After Tick `0`:

```orcvs tick 0
C4
=v
C4
```

### `=^` Copy North

Reads the pair one row south and writes it one row north.

```orcvs

=^
C4
```

After Tick `0`:

```orcvs tick 0
C4
=^
C4
```

### `=>` Copy East

Reads the pair two columns west, immediately before it, and writes it two columns east,
immediately after it.

```orcvs
C4=>
```

After Tick `0`:

```orcvs tick 0
C4=>C4
```

### `=<` Copy West

Reads the pair immediately after it and writes it immediately before it.

```orcvs
  =<C4
```

After Tick `0`:

```orcvs tick 0
C4=<C4
```

### `=$` Absolute Copy

`=$ source-column source-row destination-column destination-row` copies the pair at one position
to another, counting columns and rows from `00 00` at the Grid's top-left. It writes nothing south
of itself.

```orcvs
=$00020004

C4
```

After Tick `0`:

```orcvs tick 0
=$00020004

C4

C4
```

## Read

A Read reads one pair of Source and answers it, writing it one row south, every Tick. The Cells it
reads are ordinary Source rather than part of its Expression. A pair outside the Grid, or cut short
by the row's edge, diagnoses.

### `&>` Read East

`&> n` reads the pair `n` pairs east of its `n` operand. `00` is the operand itself.

```orcvs
&>01C4
```

After Tick `0`:

```orcvs tick 0
&>01C4
C4
```

### `&<` Read West

`&< n` reads the pair `n` pairs west of its `n` operand.

```orcvs
A4  &<03
```

After Tick `0`:

```orcvs tick 0
A4  &<03
    A4
```

### `&^` Read North

`&^ n` reads the pair `n` rows north of its `n` operand, in the operand's columns.

```orcvs
  E4
&^01
```

After Tick `0`:

```orcvs tick 0
  E4
&^01
E4
```

### `&v` Read South

`&v n` reads the pair `n` rows south of its `n` operand, in the operand's columns.

```orcvs
&v02

  G4
```

After Tick `0`:

```orcvs tick 0
&v02
G4
  G4
```

### `&$` Absolute Read

`&$ column row` reads the pair at a position.

```orcvs
&$0002

E4
```

After Tick `0`:

```orcvs tick 0
&$0002
E4
E4
```

### `&t` Track

`&t index count` reads one of the `count` pairs that follow its operands: the pair `index % count`,
counted from zero. A `count` of `00` diagnoses.

```orcvs
&t0103C4D4E4
```

After Tick `0`:

```orcvs tick 0
&t0103C4D4E4
D4
```

## Write

A Write writes its `value` operand into one pair of Source every Tick, and nowhere else. `value` is
Untyped: the Write carries its two Cells unchanged, and whatever reads them decides what they are.

### `@>` Write East

`@> n value` writes `value` into the pair `n` pairs east of its `n` operand.

```orcvs
@>02C4
```

After Tick `0`:

```orcvs tick 0
@>02C4C4
```

### `@<` Write West

`@< n value` writes `value` into the pair `n` pairs west of its `n` operand.

```orcvs
    @<03B4
```

After Tick `0`:

```orcvs tick 0
B4  @<03B4
```

### `@^` Write North

`@^ n value` writes `value` `n` rows north of its `n` operand, in the operand's columns.

```orcvs

@^01G4
```

After Tick `0`:

```orcvs tick 0
  G4
@^01G4
```

### `@v` Write South

`@v n value` writes `value` `n` rows south of its `n` operand, in the operand's columns.

```orcvs
@v01A4
```

After Tick `0`:

```orcvs tick 0
@v01A4
  A4
```

### `@$` Absolute Write

`@$ column row value` writes `value` at a position. A Read in its `value`, such as
`@$0003&$0005`, copies one pair to another.

```orcvs
@$0002C4
```

After Tick `0`:

```orcvs tick 0
@$0002C4

C4
```

### `@t` Push

`@t index count value` writes `value` into a lane of `count` pairs on the row below, starting
directly under its anchor, at the pair `index % count`.

```orcvs
@t0103G4
```

After Tick `0`:

```orcvs tick 0
@t0103G4
  G4
```

## Bang and movement

### `**` Bang

A standalone `**` where an Expression starts. On its Turn it activates the roots aligned with it
and empties its own two Cells, so it fires once. Functions that answer a Bang write `**`, which
then fires on the next Tick unless something writes over it first.

```orcvs
**
*v
```

After Tick `0`, the Bang has activated `*v`, which emits `vv` south:

```orcvs tick 0

*v
vv
```

A Bang activates a root on the row above it too, so this plays one note:

```orcvs
!>0064C4
**
```

```orcvs tick 0 plays 1
!>0064C4
```

### `*>` Bang East

Inert until a Bang reaches it. Each Bang it receives emits a `>>` into the two Cells after it.

```orcvs
**
*>
```

After Tick `0`:

```orcvs tick 0

*>>>
```

### `*<` Bang West

Inert until a Bang reaches it. Each Bang emits a `<<` into the two Cells before it.

```orcvs
  **
  *<
```

After Tick `0`:

```orcvs tick 0

<<*<
```

### `*^` Bang North

Inert until a Bang reaches it. Each Bang emits a `^^` into the two Cells one row north.

```orcvs

**
*^
```

After Tick `0`:

```orcvs tick 0

^^
*^
```

### `*v` Bang South

Inert until a Bang reaches it. Each Bang emits a `vv` into the two Cells one row south.

```orcvs
**
*v
```

After Tick `0`:

```orcvs tick 0

*v
vv
```

### `>>` Move East

Runs every Tick without a Bang, and moves itself one Cell east each Tick. When something blocks it,
or it reaches the Grid's edge, it turns into `**`, which fires on the next Tick.

```orcvs
>>
```

After Tick `0`, then after Tick `1`:

```orcvs tick 0
 >>
```

```orcvs tick 1
  >>
```

### `<<` Move West

Moves itself one Cell west each Tick, as `>>` moves east.

```orcvs
  <<
```

After Tick `0`:

```orcvs tick 0
 <<
```

### `^^` Move North

Moves itself one row north each Tick.

```orcvs


^^
```

After Tick `0`:

```orcvs tick 0

^^
```

### `vv` Move South

Moves itself one row south each Tick.

```orcvs
vv
```

After Tick `0`:

```orcvs tick 0

vv
```

### `*!` Halt

Inert until a Bang reaches it. When active, it stops the root directly south of it from taking its
Turn that Tick.

```orcvs
**
*!
.+0102
```

After Tick `0`, the Addition did not run:

```orcvs tick 0

*!
.+0102
```

## MIDI output

These perform an effect: each sends one MIDI message when a Bang reaches it, and writes nothing
into the Source. Each is inert until a Bang reaches it. An operand outside its range diagnoses and
sends nothing. Choose a MIDI destination under the Panel's **O** to hear them.

| Operand | Range |
|---|---|
| `channel` | `00` to `0F` |
| `velocity` | `00` to `7F`; `00` stops the note |
| `controller`, `value` | `00` to `7F` |
| `lsb`, `msb` | `00` to `7F` |
| `length` | `00` to `FF` Ticks |

### `!>` Play

`!> channel velocity note` sends a Note On, with no Note Off of its own.

```orcvs
**
!>0064C4
```

After Tick `0`:

```orcvs tick 0 plays 1

!>0064C4
```

A `**` after the last operand does not reach the anchor. It still empties itself, and nothing
plays:

```orcvs
!>0064C4**
```

```orcvs tick 0
!>0064C4
```

### `!~` Timed Play

`!~ channel velocity note length` sends a Note On, and Playback sends its Note Off `length` Ticks
later. A `length` of `00` sends nothing.

```orcvs
**
!~0064C404
```

After Tick `0`:

```orcvs tick 0 plays 1

!~0064C404
```

### `!%` Monophonic Play

`!% channel velocity note length` works like Timed Play, but first stops whichever note an earlier
Monophonic Play left sounding on the same channel, so a channel plays one of these at a time.

```orcvs
**
!%0164G404
```

After Tick `0`:

```orcvs tick 0 plays 1

!%0164G404
```

### `!c` Control Change

`!c channel controller value` sends a Control Change.

```orcvs
**
!c000740
```

After Tick `0`:

```orcvs tick 0 plays 1

!c000740
```

### `!b` Pitch Bend

`!b channel lsb msb` sends a Pitch Bend, low byte first. `!b000040` is the centre.

```orcvs
**
!b000040
```

After Tick `0`:

```orcvs tick 0 plays 1

!b000040
```

## Network output

These perform an effect: each sends one message over the network when a Bang reaches it, and
writes nothing into the Source. Each is inert until a Bang reaches it. Neither takes operands.
What each sends is its Payload: the Cells after its spelling through the next `!` on the row, which
closes it, or through the end of the row when no `!` follows. The Payload is never parsed, so it
can hold any character but `!`, and a Write into it changes what the next send carries.

Sends go to the host and ports set in the UDP/OSC section of the list under the Panel's **O**,
which default to `127.0.0.1`, UDP port `49161` and OSC port `49162`. A failed send is reported once,
until the failure changes or a send leaves, and playback goes on. The web build cannot send UDP or
OSC, so there each activation is reported instead.

### `!u` UDP

`!u` sends its Payload as one UDP datagram: one byte per Cell, a space for an empty Cell, and no
terminator. `!uhello!` sends `hello`. An empty Payload, `!u!`, sends nothing.

```orcvs
**
!uhello!
```

After Tick `0`:

```orcvs tick 0 plays 1

!uhello!
```

### `!o` OSC

`!o` sends its Payload as one OSC message. The Payload's words are separated by spaces. The first
is the address and starts with `/`. Without a type string, each later word is an argument whose
type comes from its shape:

| Word | Sends | Examples |
|---|---|---|
| One to eight uppercase hexadecimal digits | The int32 whose bits they spell | `40` sends 64, `1B8` sends 440, `FFFFFFFF` sends -1 |
| A decimal with a digit on each side of the point | A float32 | `0.5`, `-0.5`, `440.0` |
| Any other word | A string | `hi`, `ab`, `.5`, `1e3` |

`!o/synth/freq 40 7F!` sends `/synth/freq` with the int32s 64 and 127.

```orcvs
**
!o/synth/freq 40 7F!
```

After Tick `0`:

```orcvs tick 0 plays 1

!o/synth/freq 40 7F!
```

A word after the address that starts with `,` is a type string, one letter per argument: `i` for an
int32, `f` for a float32 and `s` for a string. `!o/synth/freq ,fs 64.0 hi!` sends the float32 64.0
and the string `hi`.

```orcvs
**
!o/synth/freq ,fs 64.0 hi!
```

After Tick `0`:

```orcvs tick 0 plays 1

!o/synth/freq ,fs 64.0 hi!
```

An address that does not start with `/`, a type letter other than `i`, `f` or `s`, a type string
whose length differs from the argument count, or a word that does not fit its type diagnoses and
sends nothing:

```orcvs
**
!osynth 40!
```

```orcvs tick 0 diagnoses

!osynth 40!
```
