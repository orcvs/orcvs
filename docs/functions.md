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
| Interval Set | A chord or scale name | One of 39 names, each filling both Cells, for a list of semitones above a root. Chords: `M_ M7 M9 MD M6 69 +M m_ m7 mM m6 m9 mB mD o_ o7 h7 b9 7_ 9_ 13 s2 s4 +_ a9`. Scales: `MA mi do ph ly mx lo hm mm p5 p3 bl wt ch`. A one-character chord name takes the filler `_`, and `B` and `D` stand for 11 and 13. `M ` is half-written, not a name. |
| Voicing | A Voicing name | One of 8 names: `CL` close, `OP` open, `D2` `D3` `D5` `D6` drops, `SH` shell and `RL` rootless. |
| Untyped | A Number, a Note, `**`, or a nested Function's answer | Carried unchanged; whatever reads them at the destination decides what they mean. Empty Cells, or Cells that form no unit, diagnose. |
| MIDI domains | `00` to `0F` or `00` to `7F` | Numbers that a MIDI output Function also checks against a range, named in its entry. |

The same two Cells can be a Number in one operand and a Note in another: `C4` is Number `C4`
where a Number is expected. In the same way `69` and `13` are Interval Sets where an Interval Set
is expected and Numbers where a Number is, and `D2` is a Voicing, a Note or a Number by position.
An empty operand diagnoses, except a Play's `note`, which is optional (see **An empty note**).

**Answer or effect.** A Function either answers a value or performs an effect, never both. A
Function that answers a value writes it into its Output Portal, the two Cells one row south of its
anchor unless the entry names another place. A value Function can also stand in another Function's
operand, nested: `.+.^3C01` adds `01` to the `C4` that `.^3C` answers, read as the Number `C4`, and
answers `C5`. A Function that
performs an effect, such as playing a note, writes no answer and cannot be nested.

**Runs.** Most Functions take a Turn every Tick. Some are inert until a Bang reaches them. A root is
the Function that begins an Expression, as opposed to one nested in an operand. A Bang activates each root whose anchor is aligned with the `**` in one of four places: directly north or
south of it, with its anchor in the same column as the `**`'s first Cell, or in the two Cells
immediately before or after it on its row. A `**` that touches another Function's operand activates nothing: `!>0064C4**` does not
play, because the Cells before the `**` are the operand `C4`, not the anchor `!>`. Each entry says
which kind of Function it is.

**Clear.** Some value Functions answer a Clear rather than a value: they write two empty Cells. An
invalid Function writes nothing at all, so its last answer stays where it was. Nested, a Clear
leaves its parent invalid, except in a Play's `note`, where it leaves the note empty.

**Examples.** An example shows the Source before any Tick, then the Source after the Ticks named
above each later block, counted from Tick `0`.

## Arithmetic

Each takes two Numbers, `left` and `right`, answers a Number, and runs every Tick. Cells that do not
spell a Number, such as `G4`, diagnose. `C4` is read as the Number `C4`, not MIDI note 60; use
[`.v`](#v-convert-to-number) for a Note's MIDI number.

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

Each converts between a Note and its MIDI number, and runs every Tick.

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

`.^ number` answers the Note for a Number from `00` to `7F`. `80` to `FF` diagnoses and writes
nothing.

```orcvs
.^3C
```

After Tick `0`:

```orcvs tick 0
.^3C
C4
```

## Pitch

Each derives a Note or an Interval Set from the Notes and Interval Sets it reads, and runs every
Tick. A `note` or `root` that does not spell a Note, such as `3C`, is refused. Up, Down and Pick
answer a Note: an answer outside `00` to `7F` diagnoses and writes nothing, so a Note never wraps.
Stack answers an Interval Set: when no name holds the semitones it reaches, it diagnoses and writes
nothing.

### `#+` Up

`#+ note number` answers the Note `number` semitones above `note`.

```orcvs
#+C402
```

After Tick `0`:

```orcvs tick 0
#+C402
D4
```

`G9` is the highest Note, so going above it diagnoses and writes nothing.

```orcvs
#+G901
```

After Tick `0`:

```orcvs tick 0 diagnoses
#+G901
```

Nested in a Play's `note` operand, the Play sounds the raised Note. The nested Up writes its
own answer south of itself as well.

```orcvs
.=0101

!>007F#+C402
```

After Tick `0`:

```orcvs tick 0 plays 1
.=0101
**
!>007F#+C402
      D4
```

### `#-` Down

`#- note number` answers the Note `number` semitones below `note`.

```orcvs
#-D402
```

After Tick `0`:

```orcvs tick 0
#-D402
C4
```

`C/` is the lowest Note, so going below it diagnoses and writes nothing.

```orcvs
#-C/01
```

After Tick `0`:

```orcvs tick 0 diagnoses
#-C/01
```

### `#p` Pick

`#p root set index` answers the Note at position `index` of the Interval Set `set`, above `root`.
Index `00` is the root. An index past the end of the set wraps up an octave, so a set of seven
tones answers the root an octave up at index `07`.

```orcvs
#pC4MA04
```

After Tick `0`:

```orcvs tick 0
#pC4MA04
G4
```

The wrap adds exactly one octave per pass, so Pick rises through a set only while every entry is
less than an octave above the root. A set wider than an octave steps back down at the wrap: the
last entry of `MD` is `21` semitones, so index `05` answers `A5` and index `06` answers `C5`.

```orcvs
#pC4MD05

#pC4MD06
```

After Tick `0`:

```orcvs tick 0
#pC4MD05
A5
#pC4MD06
C5
```

An answer above `G9` diagnoses and writes nothing.

```orcvs
#pG9M_01
```

After Tick `0`:

```orcvs tick 0 diagnoses
#pG9M_01
```

### `#s` Stack

`#s set degree size` builds a chord on a scale degree. It takes `size` tones of `set`, starting at
position `degree` and stepping two positions at a time, wrapping up octaves as Pick does. It
subtracts the first tone and answers the Interval Set with exactly those semitones.

```orcvs
#sMA0004
```

After Tick `0`:

```orcvs tick 0
#sMA0004
M7
```

When no name holds exactly those semitones, Stack diagnoses and writes nothing. A major pentatonic
stacked from `C` reads the semitones `0 4 9`, which no name holds.

```orcvs
#sp50003
```

After Tick `0`:

```orcvs tick 0 diagnoses
#sp50003
```

Pick in a Chord Play's `root` and Stack in its `set` sound a chord on a scale degree. Here degree
`04` of C major is `G4`, and the chord built on it answers `7_`, so the Chord Play sounds a G
dominant seventh.

```orcvs
.=0101

!:0064#pC4MA04#sMA0404CL00
```

After Tick `0`:

```orcvs tick 0 plays 4
.=0101
**
!:0064#pC4MA04#sMA0404CL00
      G4      7_
```

## Tick

These read the Tick number or their own last answer, so they change over time. Each runs every
Tick.

### `~.` Clock

`~. length steps` answers which step a cycle is on: each step lasts `length` Ticks, and the cycle
holds `steps` steps, counted from `00`. It reads the Tick number, so a stopped and resumed
Clock carries on from the same step. A `length` or `steps` of `00` diagnoses.

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

`~* length steps` answers a Bang once every `length * steps` Ticks, starting at Tick `0`, and a
Clear on every other Tick. A `length` or `steps` of `00` diagnoses.

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

After Tick `0`, after Tick `1`, then after Tick `3`:

```orcvs tick 0
~%0308
**
```

```orcvs tick 1
~%0308
```

```orcvs tick 3
~%0308
**
```

### `~+` Increment

`~+ amount modulus` reads the Number in its own Output Portal, adds `amount`, and answers the result
modulo `modulus`. It reads back its last answer, so it counts up; empty Cells count as `00`, so it
starts from `00`. A `modulus` of `00`, or anything other than a Number in its Output Portal, diagnoses.

```orcvs
~+0104
```

After Tick `0`, then after Tick `3`, where it wraps:

```orcvs tick 0
~+0104
01
```

```orcvs tick 3
~+0104
00
```

### `~>` Interpolation

`~> amount target` reads the Number in its own Output Portal and moves it toward `target` by at most
`amount` each Tick, never past it. Empty Cells there count as `00`. An `amount` of `00` holds the
value. Anything other than a Number in its Output Portal diagnoses.

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
depends only on `seed`, the Tick number, and the Function's own Position, so the same Source
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

A Copy copies one two-Cell pair from its Input Portal to its Output Portal every Tick. It takes
whatever the pair holds, a value, a Bang answered that Tick or two empty Cells, and writes it over
the whole Output Portal. Part of a Function, or Cells that do not form one unit, diagnoses. A copied
`**` activates the roots aligned with its Output Portal. A `**` standing in the Source is a Bang
Function: it has taken its Turn by the time the Copy reads it, so the Copy reads its cleared Cells,
or `**` while a Halt locks it.

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

`=$ from to` copies the pair at the Position `from` to the Position `to`. Each Position is two
Numbers, column then row, counted from `00 00` at the Grid's top-left. It writes only the pair at
`to`, so the row below it changes only when `to` names that pair.

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
reads are ordinary Source rather than part of its Expression. A pair outside the Grid, cut short
by the row's edge, or holding Cells that form no unit, diagnoses. Empty Cells answer a Clear, and a
Function spelling is carried as it stands.

### `&>` Read East

`&> distance` reads the pair `distance` pairs east of its `distance` operand. `00` is the operand itself.

```orcvs
&>01C4
```

After Tick `0`:

```orcvs tick 0
&>01C4
C4
```

### `&<` Read West

`&< distance` reads the pair `distance` pairs west of its `distance` operand.

```orcvs
A4  &<03
```

After Tick `0`:

```orcvs tick 0
A4  &<03
    A4
```

### `&^` Read North

`&^ distance` reads the pair `distance` rows north of its `distance` operand, in the operand's columns.

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

`&v distance` reads the pair `distance` rows south of its `distance` operand, in the operand's columns.

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

`&$ at` reads the pair at a Position, two Numbers, column then row.

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

`@> distance value` writes `value` into the pair `distance` pairs east of its `distance` operand.

```orcvs
@>02C4
```

After Tick `0`:

```orcvs tick 0
@>02C4C4
```

### `@<` Write West

`@< distance value` writes `value` into the pair `distance` pairs west of its `distance` operand.

```orcvs
    @<03B4
```

After Tick `0`:

```orcvs tick 0
B4  @<03B4
```

### `@^` Write North

`@^ distance value` writes `value` `distance` rows north of its `distance` operand, in the operand's columns.

```orcvs

@^01G4
```

After Tick `0`:

```orcvs tick 0
  G4
@^01G4
```

### `@v` Write South

`@v distance value` writes `value` `distance` rows south of its `distance` operand, in the operand's columns.

```orcvs
@v01A4
```

After Tick `0`:

```orcvs tick 0
@v01A4
  A4
```

### `@$` Absolute Write

`@$ at value` writes `value` at a Position, two Numbers, column then row. A Read in its `value`, such as
`@$0003&$0005`, copies one pair to another; an empty source pair is a Clear, which diagnoses.

```orcvs
@$0002C4
```

After Tick `0`:

```orcvs tick 0
@$0002C4

C4
```

### `@t` Push

`@t index count value` writes `value` into one of `count` pairs on the row below, numbered from
directly under its anchor: pair `index % count`. A `count` of `00` diagnoses.

```orcvs
@t0103G4
```

After Tick `0`:

```orcvs tick 0
@t0103G4
  G4
```

## Bang

`*>` and its kin are Directional Bang Functions, and `>>` and its kin are Self-Banging Functions.

### `**` Bang

`**` where an Expression starts is the Bang Function. On its Turn it activates the roots aligned with it
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

After Tick `0`:

```orcvs tick 0 plays 1
!>0064C4
```

### `*>` Bang East

Inert until a Bang reaches it. A Bang reaching it emits a `>>` into the two Cells after it.

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

After Tick `0`, the Add did not run:

```orcvs tick 0

*!
.+0102
```

## MIDI output

These perform an effect: each sends one MIDI message when a Bang reaches it, except a Chord Play,
which sends one per tone, and writes nothing into the Source. Each is inert until a Bang reaches
it. An operand outside its range diagnoses and sends nothing. Choose a MIDI destination under
the Panel's **O** to hear them.

| Operand | Range |
|---|---|
| `channel` | `00` to `0F`: MIDI channels 1 to 16 (see **Built-in Synth**) |
| `velocity` | `00` to `7F`; `00` stops the note, and on `!%` stops the channel's Monophonic note |
| `controller`, `value` | `00` to `7F` |
| `lsb`, `msb` | `00` to `7F` |
| `length` | `00` to `FF` Ticks |
| `set` | an Interval Set name, such as `M7` or `MA` |
| `voicing` | a Voicing name: `CL`, `OP`, `D2`, `D3`, `D5`, `D6`, `SH` or `RL` |
| `inversion` | a Number, `00` to `FF`: how many times the lowest tone moves up an octave |

**An empty note.** The note of a Raw, Timed or Monophonic Play is optional. A Bang reaching a Play
whose note is empty sends nothing and reports nothing, so an empty step in a pattern is a quiet
step. It does not matter how the note got empty. You can leave it empty, a Function above it can
write two empty Cells into it, or a Function nested in it can answer a Clear. The Play's other
operands are still checked: an empty or out-of-range channel, velocity or length is diagnosed
whether the note is there or not. A half-written or mistyped note, such as `C ` or `ZZ`, is an
error: the Play is diagnosed and sends nothing. A Monophonic Play with no note leaves its channel's
note sounding until that note's own length ends. A Chord Play's root is not optional.

```orcvs
**
!>0064
```

After Tick `0`, nothing plays and nothing is diagnosed:

```orcvs tick 0

!>0064
```

### `!>` Raw Play

`!> channel velocity note` sends a Note On, with no Note Off of its own. At velocity `00` it sends a
Note Off for `note` instead. The `note` is optional: with it empty, a Bang sends nothing (see **An
empty note** above).

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

After Tick `0`:

```orcvs tick 0
!>0064C4
```

### `!~` Timed Play

`!~ channel velocity note length` sends a Note On, and Playback sends its Note Off `length` Ticks
later. A `length` of `00` sends nothing. The `note` is optional: with it empty, a Bang sends
nothing (see **An empty note** above).

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
Monophonic Play left sounding on the same channel, so a channel plays one of these at a time. At
velocity `00` it stops that note and plays nothing. The `note` is optional: with it empty, a Bang
sends nothing and stops nothing (see **An empty note** above).

```orcvs
**
!%0164G404
```

After Tick `0`:

```orcvs tick 0 plays 1

!%0164G404
```

### `!:` Raw Chord Play

`!: channel velocity root set voicing inversion` sends one Note On per tone, lowest first, with no
Note Off of its own. The tones are the `root` plus each semitone the `set` names. `C4M7` is C4, E4,
G4 and B4.

```orcvs
**
!:0064C4M7CL00
```

After Tick `0`, four Note Ons, C4 up to B4:

```orcvs tick 0 plays 4

!:0064C4M7CL00
```

The `voicing` and `inversion` then arrange the tones, in this order. The Voicing removes tones,
the inversion moves the lowest tone up an octave as many times as its Number says, and the Voicing
moves tones:

| Voicing | What it does |
|---|---|
| `CL` | Leaves the tones where the Interval Set puts them. |
| `OP` | Raises every second tone, the 2nd, 4th and so on, an octave. Needs 2 tones. |
| `D2` | Lowers the 2nd tone from the top an octave. Needs 2 tones. |
| `D3` | Lowers the 3rd tone from the top an octave. Needs 3 tones. |
| `D5` | Lowers the 2nd and 3rd from the top an octave. Needs 3 tones. |
| `D6` | Lowers the 2nd and 4th from the top an octave. Needs 4 tones. |
| `SH` | Keeps only the 1st, 2nd and 4th tones. Needs 4 tones. |
| `RL` | Removes the 1st tone, leaving the root to a bass voice. Needs 2 tones. |

The digit in a drop voicing is the sum of the positions it drops, counted from the top: `D5` drops
the 2nd and 3rd, and `D6` the 2nd and 4th.

An Interval Set with too few tones for its Voicing, or any tone outside `00` to `7F`, diagnoses the
whole chord and sends nothing.

`C4M7` dropped with `D5`, inverted once, plays G3, B3, E4 and C5:

```orcvs
**
!:0064C4M7D501
```

```orcvs tick 0 plays 4

!:0064C4M7D501
```

### `!#` Timed Chord Play

`!# channel velocity root set voicing inversion length` builds its tones as Raw Chord Play does and
sends each as a Timed Play: every tone stops `length` Ticks later, at its own Note Off. A `velocity`
of `00` stops every tone, and a `length` of `00` starts none.

```orcvs
**
!#0064C4M_CL0004
```

After Tick `0`, three Note Ons, each stopped four Ticks later:

```orcvs tick 0 plays 3

!#0064C4M_CL0004
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

## Built-in Synth

A page on the Orcvs site that frames the console offers the Built-in Synth as a MIDI destination,
so an example sounds with no MIDI device. It is listed after every MIDI port under the Panel's
**O**. On any other output, a channel plays whatever that device puts on it.

**Channels.** A `channel` operand counts from `00`, as a MIDI message does, and a DAW or a synth's
panel counts the same sixteen channels from 1: `00` to `0F` are MIDI channels 1 to 16, so `00` is
channel 1 and `09` is channel 10. The Built-in Synth plays tones on `00` to `07`, its acid voice on
`08`, and an 808-style kit on `09` to `0F`. Channel `09` is a drum channel, as it is on a General
MIDI module, so a drum part a DAW sends on channel 10 plays drums.

| Orcvs | DAW | Built-in Synth |
|---|---|---|
| `00` | 1 | Sine |
| `01` | 2 | Triangle |
| `02` | 3 | Saw |
| `03` | 4 | Square |
| `04` | 5 | FM, a soft bell |
| `05` | 6 | FM, a bright bass |
| `06` | 7 | AM, harmonicity 1.5 |
| `07` | 8 | AM, harmonicity 2 |
| `08` | 9 | Acid voice: one 303-style voice that slides between overlapping notes |
| `09` | 10 | The kit, on General MIDI's drum channel |
| `0A` to `0F` | 11 to 16 | The kit |

## Network output

These perform an effect: each sends one message over the network when a Bang reaches it, and
writes nothing into the Source. Each is inert until a Bang reaches it. Neither takes operands.
What each sends is its Payload: the Cells after its spelling through the next `!` on the row, which
closes it, or through the end of the row when no `!` follows. The Payload is never parsed, so it
can hold any character but `!`, and a Write into it changes what the next send carries.

Sends go to the host and ports set in the UDP/OSC section of the list under the Panel's **O**,
which default to `127.0.0.1`, UDP port `49161` and OSC port `49162`. A failed send is reported once,
until a different failure occurs or a send succeeds, and playback goes on. The web build cannot send UDP or
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
int32 read as hexadecimal, `f` for a float32 and `s` for a string. `!o/synth/freq ,fs 64.0 hi!` sends the float32 64.0
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

After Tick `0`:

```orcvs tick 0 diagnoses

!osynth 40!
```
