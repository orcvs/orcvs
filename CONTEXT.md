# Orca Visual Synthesizer

Orcvs is a grid-based environment for composing and executing compact musical expressions. Each entry gives a term's definition, its distinctions and the synonyms to avoid (`_Avoid_`). What the product does with a term is stated in the owning requirement, which `docs/requirements/README.md` routes by topic; per-Function spellings, operands and examples are in `docs/functions.md`.

## Language

**Source**:
The rectangular grid that holds the current Orcvs program as Cells.
_Avoid_: Document, buffer, canvas

**Source File**:
A Source written out as plain text: one line per row, one character per Cell, a space for an empty Cell. It states Cells and never a shape, so a short or missing line reads back as empty Cells.
_Avoid_: Document, save file, project

**Grid**:
The fixed rectangular shape a Source occupies: 256 columns by 256 rows, and the valid positions within them. The Grid is the shape; the Source is the contents. Every Grid has that one shape, and a position outside it does not exist.
_Avoid_: Canvas, matrix, bounds

**Position**:
The column and row of one Cell of a Grid, spelled in Source as two Numbers, column then row, from `00 00` at the top-left. A Position can be obtained only from the Grid that contains it, so a pair of Numbers past its Grid's extent is not a Position; the Grid converts between a Position and the index the Source addresses Cells by. The absolute Functions `&$`, `@$` and `=$` take each Position they address as two Number operands, column then row, counted in Cells.
_Avoid_: Coord, coordinate, point

**Cell**:
One position in the Source, containing exactly one printable single-byte ASCII character; a space represents an empty Cell.
_Avoid_: Character slot, text position

**Language Unit**:
One semantic value or operation recognized in a Source revision, such as a Function or Atom. A Language Unit has one anchor Position and a Span of one or more character Cells. Incomplete or invalid Source text does not form a Language Unit. A Comment is the one Language Unit that is neither a value nor an operation: it records a Token and no Atom, so it is established like any other and evaluated like none.
_Avoid_: Logical Cell, token Cell, glyph

**Span**:
The character Cells occupied by one Language Unit, Expression, or Diagnostic in a Source revision. A row is the whole horizontal extent there is, so a Span is a contiguous run within one row, named by its first and last Cell. Spatial behavior moves, replaces, or tests the complete Span while every Position remains a Position in the character Grid.
_Avoid_: Footprint, extent, range, Cell structure, semantic Grid, bounding box

**Language Map**:
The semantic view derived from the Parser's interpretation of one Source revision. It identifies Expressions, roots, typed operands, nested ownership, Language Units, anchor Positions and Spans without adding persistent program state or independently interpreting spellings.
_Avoid_: Overlay Grid, parsed Source state, semantic Source

**Expression**:
A contiguous horizontal group of Cells established by parsing one Function and its operands, one Bang Function or Self-Banging Function, or one Comment. Operand Cells may be empty or invalid; nested Functions extend the containing Expression, which never crosses a row edge. A Comment is the one Expression whose extent comes from its closer, or the end of its row, rather than from an arity; an arity-determined claim reaches straight over a `||` inside it, which is then an operand Cell that fails to bind.
_Avoid_: Formula, statement, Pending (as a Function's state), Pending Function, incomplete Expression

**Comment**:
The Language Unit the Parser establishes at the two-Cell introducer `||`, claiming every Cell through the next `||` on its row, which closes it, or every remaining Cell of the row when none follows. Its text, the Cells between the delimiters, is never parsed and is ordinary Cells to every Function: a Read, Track or Copy selecting a pair of it reads that pair as data, as it reads a pair no Language Unit covers; a Halt aimed at it locks nothing; a mover is refused there only by what the Cells hold; and a Write writes them. Only the two delimiters are the Comment's occupied Cells, so a pair touching either is refused. It records a Token and no Atom, which excludes it from evaluation: it answers no value, performs no effect, and is never scheduled. `||` opens a Comment only where a new Expression could start; inside a Function's arity-determined claim it is an operand Cell that fails to bind and diagnoses. One `|` alone is incomplete or invalid Source rather than a Comment.
_Avoid_: Comment Function, halted Expression

**Atom**:
One parsed value or operation an Expression is made of: a Number, a Note, an Interval Set, a Voicing, a Bang, or a Function. Every Atom has a Source encoding. A Self-Banging Function is not a kind of its own here: it is a Function, and the four spellings parse to one. An Atom is what an Expression's operands resolve to and what a value Function answers when it has one to write. A value Function's other successful answer, a Clear, is not an Atom. It is the parsed unit rather than the Cells that spell it, so the same two Source characters can be a Number in one operand position and a Note in another.
_Avoid_: Token, glyph, symbol, cell value

**Number**:
An unsigned byte interpreted from an Operand Literal as exactly two uppercase hexadecimal Cells from `00` through `FF` when a Function requires a Number. General arithmetic wraps within this byte range; narrower domains such as MIDI parameters enforce their limits at their own boundaries.
_Avoid_: Base-36 value, decimal literal, single-glyph number

**Note**:
A pitched Atom carrying one MIDI note value from `00` through `7F`. In a Note operand position, its two-Cell Operand Literal uses an uppercase pitch letter for a natural or lowercase pitch letter for a sharp followed by its octave character: `/` for the octave below `0`, then `0` through `9`, such as `C/`, `C4`, or `c4`. The same Source characters may denote a Number in a Number operand position.
_Avoid_: Number, note-shaped Number, intrinsically typed literal

**Interval Set**:
A two-Cell name for a chord quality or a scale, standing for its semitones above a root: one of 25 chords or 14 scales. Every name fills both Cells, so a half-blank pair is partial input. Triads are `MA` major, `mi` minor, `di` diminished and `+5` augmented; dominants are their bare number, `07`, `09` and `13`; the diminished seventh is `d7`; the major and natural minor scales take their mode names, `io` (Ionian) and `ae` (Aeolian), beside the other modes; the pentatonics are `pM` and `pm`. A scale name is valid wherever a chord name is. Nearly every name is one Cell from another (`M7`/`m7`, `07`/`09`), and a slip in one Cell is accepted as the other name rather than diagnosed, including pairs that mean very different things: `mM`/`mm`, `pM`/`pm`, `mi`/`mm`, `d7`/`07`, `io`/`lo` and `di`/`do`. No two names differ only by `o` against `0`. `69`, `13`, `07` and `09` are also Numbers and `d7` is also a Note: an Operand Literal has no type outside the operand position that receives it.
_Avoid_: Chord type, scale type, chord symbol

**Voicing**:
A two-Cell name for how a chord's tones are arranged once its Interval Set has placed them: `CL` close position, `OP` open position, `D2` drop 2, `D3` drop 3, `23` drop 2&3, `24` drop 2&4, `SH` shell and `RL` rootless. A drop is named for the positions it lowers, counted from the top; a drop of two positions names both and, in two Cells, has no room for the `D`. `D2` and `D3` are also Notes and Numbers and `23` and `24` are also Numbers, and the operand position receiving them decides. Inversion is a separate Number operand and not part of a Voicing.
_Avoid_: Chord shape, arrangement

**Operand Literal**:
Two Source Cells interpreted as an Atom according to the typed operand position of the Function that consumes them. The characters have no Number, Note, Interval Set or Voicing type outside that context. A Write's or Push's `value` takes no type: it carries the Language Unit its two Cells hold unchanged, and whatever reads them at the destination interprets them. A standalone operand literal, with no slot to type it, is invalid (LANG-130); whether that should change is listed under Open decisions in `docs/requirements/language.md`.
_Avoid_: Typed Source Cell, intrinsically typed literal, contextual coercion

**Defaulted Operand**:
An operand whose type gives a slot that holds no value a fixed Number, which the Function's row in the Function table states as the operand type's second parameter. A blank slot (LANG-320), whether written blank, emptied by a Portal write or cleared by a nested Function, binds exactly as that Number written there would, so the Function cannot tell the two apart, and a written value binds as the plain operand binds it. It sits beside the optional operand (LANG-300), whose blank binds to no value; every other operand still refuses a blank slot.
_Avoid_: Default argument, optional value, fallback

**Function**:
A named Orcvs language operation evaluated within an Expression. A Function may adapt a capability found in Orca, but its syntax and behaviour follow Orcvs language rules rather than Orca compatibility. Every Function declares whether it answers a value the surrounding Expression can consume or performs an effect and answers nothing, never both and never neither (EVAL-082), and that declaration is what the Parser's nesting refusal (LANG-250) and the activation gate (EVAL-050) each read rather than a spelling or a family prefix. The declaration is a property of the definition, settled before any Tick runs, and is not the Effect a Producer contributes to a Tick Plan. Terminal Output is one effect a Function may declare rather than the definition of effect, so a rule about having no Cell destination reads that narrower declaration while a rule about nothing consuming the answer reads the wider one. A value Function that evaluates answers either an Atom or a Clear (LANG-330), and both occupy the same reserved two-Cell pair, which is what lets a Tick reserve a result's Cells before any Function has evaluated. No Function accepts or answers a series of values.
_Avoid_: Operator, command

**Role**:
What one or more consecutive operands of a Function are to a reader, named on the Function's row in the Function table and read by Info: `.+` has the Roles `left` and `right`. A Position's two Numbers are one Role, so `=$` has the Roles `from` and `to` over its four operands. A signature in the Function reference spells its Function's Roles in order. A Role says what an operand is for and leaves its type to the signature.
_Avoid_: Argument name, parameter, operand name, role (unqualified: a Theme's colour role for a Cell and the part a Cell plays in a Portal are other things)

**Slot**:
The place one operand of a Function occupies in its Expression, in signature order: two Cells, or a nested Function standing in it and extending the Expression by its own operands. The operand type declared for a slot decides how its Cells are read and whether it may hold no value.
_Avoid_: Argument, field, cell pair

**Claim**:
The Cells a Function's arity takes for its Expression: its spelling and every slot its signature declares, extended by the slots of each nested Function (LANG-070). Cells a Function reads or writes outside its claim are ordinary Source. The Parser's Claim that a Render Frame carries and Paint reads is one slot's share of an Expression's claim, or its spelling's: those Cells, the declared Token, the bound Atom or none, and whether any of them is written (CONS-025, CONS-026). A Timed or Mono claim in Playback is note ownership, not Cells (PLAY-250).
_Avoid_: Arity span, footprint, lane

**Root**:
The outermost Function of a complete Expression, at the Expression's anchor: the computation that takes a Turn, that a Bang activates and that a Halt locks, while every Function nested in it takes its Turn through it. A Chord's root is a different thing: the Note its Interval Set stands above.
_Avoid_: Top-level Function, head, parent (a parent is any Function with a nested child)

**Return**:
The two-Cell Source encoding a nested Function hands to the inline operand it stands in. The receiving operand decodes it (LANG-260) by its declared literal type, exactly as it decodes characters a Portal wrote there, so the child's Atom type does not cross: a Note `C5` returned into an Addition operand is the Number `C5`. A Return is an additional delivery rather than a redirection: the nested Function also writes the same encoding through its own Output Portal, so its feedback stays in Source and the row below a nested Expression traces every step. A refused Output Portal write does not withdraw the Return, and a parent's failure does not undo the child's write. A nested Function that answers a Clear leaves the receiving operand unwritten, so the parent is invalid and its diagnostic names the child, unless the operand's type admits an unwritten value (LANG-300). A Function that answers no value has no Return, so the Parser refuses it wherever a value is required, from Source alone.
_Avoid_: Typed nested result, nested value, pass-through

**Spatial Output**:
A Function output delivered through a Portal as literal Source encoding, interpreted in the receiving operand's context. A nested Function's Return is the same encoding and the receiving operand interprets it the same way, so identical characters mean the same thing however they arrived.
_Avoid_: Implicit numeric conversion, typed spatial argument

**Clear**:
What a value Function answers when it has no Language Unit to write: two spaces at its Output Portal, as Orca's bang ports write `.`. Equality answers it for an unequal comparison, and Every `~*` and Euclidean `~%` on every Tick they do not Bang, so each of the three writes its Output Portal on every Turn and erases its last `**`, which would otherwise fire again as a Bang Function. A Copy, Read or Track answers it for two empty Cells, which it copies. The Cells a pulse clears are the ones it reserves for its Bang, so the clear adds no dependency. Nested, a Clear leaves the operand it stands in unwritten, so the parent is invalid and its diagnostic names the child that cleared it, unless the operand's type gives a blank a meaning, as a Play's optional note and a Defaulted Operand do. A Clear is not an Atom: no operand holds it. A Function with nothing to write this Tick, because it took no Turn or was invalid, answers nothing at all and leaves its last write standing.
_Avoid_: Absence Marker, Blank Answer, null, nil, empty value, void

**Atomic Function**:
A stateless value Function whose operands and result are single Atoms: the Arithmetic, Numeric Conversion and Pitch Functions, Clock and Random. An operation evaluates once and returns one Atom; a type, domain, or evaluation failure diagnoses and returns nothing.
Arithmetic Function names only the nine `.` arithmetic members, not every Atomic Function.
_Avoid_: Vectorised Function, Map Function

**Arithmetic Function**:
One of the nine numeric-family Functions Add `.+`, Subtract `.-`, Absolute Difference `.|`, Multiply `.x`, Divide `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and Equality `.=`, whose shared family prefix fixes the numeric domain and whose suffix names the operation. Every member declares two Number operands and returns a value. Add, Subtract, and Multiply wrap within the byte range; Absolute Difference answers the distance between its operands rather than a wrapped Subtraction, which is why it is a Function of its own; Minimum and Maximum return one operand unchanged; and Divide and Modulo answer the truncated quotient and the remainder. A Number operand reads its two Cells as a Number whatever type produced them: `C4` is the Number `C4`, and a spelling that is no Number, such as `G4`, diagnoses. The Numeric Conversion Functions are the only way to get a Note's MIDI number or the Note for a MIDI number. Two members answer something other than plain arithmetic: Divide and Modulo each diagnose a zero divisor, under a diagnostic naming which of the two the Source wrote, and produce no value; and Equality returns a Bang for equal operands and a Clear for unequal ones, writing `**` through its Output Portal for the first and two spaces for the second, root or nested.
_Avoid_: Operator, infix arithmetic, boolean comparison, saturating arithmetic

**Numeric Conversion Function**:
One of the numeric-family Functions `.v` and `.^`, whose family prefix fixes the numeric domain and whose directional suffix identifies the result type. Their Source literal signatures are monomorphic: `.v` consumes a Note literal and returns its underlying Number, while `.^` consumes a Number literal from `00` through `7F` and returns the corresponding Note, diagnosing `80` through `FF`. Each reads its operand only as that declared literal type, whether the characters were typed, written by a Portal, or returned by a nested Function, so conversions compose in opposite directions — `.v.^3C` and `.^.vC4` — while `.v.vC4` and `.^.^3C` diagnose, and `.+.^3C01` reads the returned `C4` as Number `C4`. A conversion never passes a value of its result type through.
_Avoid_: Cast, implicit coercion, sticky Note

**Pitch Function**:
A stateless value Function in the `#` family, which derives a Note or an Interval Set from the Notes and Interval Sets it reads and runs every Tick, root or nested, writing its answer through its Output Portal as any value Function does. The `.` family stays numeric, so a Function deriving Notes or Interval Sets belongs to `#`. Up, Down and Pick answer a Note: an answer outside `00`–`7F` diagnoses under a message naming the Function and the semitone it would have reached, and writes nothing, so a Note never wraps, as an Arithmetic Function does within a byte, and is never clamped to the end of the range. Stack answers an Interval Set: when no name holds the semitones it reaches, it diagnoses under a message naming the Function and its operands, and writes nothing.
_Avoid_: Transpose operator, numeric Function, wrapping pitch

**Up**:
The Pitch Function `#+ note semitones`, which answers `note` raised by `semitones` semitones. The note operand is a Note and the semitones operand a Number, so Up raises a Note without converting it through Number and back. An answer above `7F` diagnoses and writes nothing.
_Avoid_: Add, Transpose

**Down**:
The Pitch Function `#- note semitones`, which answers `note` lowered by `semitones` semitones. Its operands are Up's. An answer below `00` diagnoses and writes nothing.
_Avoid_: Subtract, Transpose

**Pick**:
The Pitch Function `#p root set index`, which answers the Note at position `index` of the Interval Set `set` above `root`: `root + set[index % n] + 12 * floor(index / n)`, where `n` is the size of the Interval Set. `index` counts from zero, so the one-based scale degree a musician counts is the index plus one: index `04` of C major is G, the 5th degree, V. An index past the end of the Interval Set therefore wraps up an octave, so one Function arpeggiates a chord and picks a scale degree. The wrap adds exactly one octave per pass, so the arpeggio rises only while every entry is below `12`: a set wider than an octave steps back down at the wrap, as `#pC4MD05` answers A5 and `#pC4MD06` answers C5. An answer above `7F` diagnoses and writes nothing.
_Avoid_: Arpeggiator, Degree (as the operand's name: `index` names it in Pick and Stack, and "degree" is kept for the one-based count)

**Stack**:
The Pitch Function `#s set index size`, which answers the Interval Set a chord on a scale degree forms. It takes `size` tones of `set` at positions `index`, `index + 2` and so on, wrapping up octaves as Pick does, subtracts the first tone, and answers the Interval Set with exactly those semitones. `index` counts from zero, as Pick's does, so index `04` of C major is G, the 5th degree, V, and Stack builds the chord on it. When no name holds exactly those semitones, as for a major pentatonic stacked from index `00`, it diagnoses and writes nothing.
_Avoid_: Chord lookup, Build chord

**Random Function**:
The Tick-family Function `~?`, which answers a Number selected inclusively between its normalized bounds. All three operands are Numbers: `seed` is the explicit stream identity, `minimum` and `maximum` are the inclusive range, so reversed bounds describe the same range and equal bounds return that value. It derives each result from the explicit seed, the absolute Tick of Playback, and the Function's own Position, rather than from activation history or an OS generator: identical Source Snapshots interpreted at the same Tick therefore produce identical Tick Plans, Functions at different Positions have independent reproducible streams, and moving one intentionally changes its stream. A Note at any operand diagnoses rather than converting.
_Avoid_: StdRng, entropy, activation count, dice, noise, non-deterministic random, hidden state

**Chord**:
A root Note and an Interval Set, written as two ordinary operand pairs, as in `C4M7`. A Chord is not a value (LANG-390): no Function answers one and no operand holds one. Raw and Timed Chord Play sound one in a single Expression.
_Avoid_: Chord value, chord operand

**Inversion**:
A Number operand of a Chord Play: how many times the lowest tone of the Chord moves up an octave. It applies after a Voicing removes tones and before a Voicing moves them, so voicing and inversion compose the same way every time. It is not part of a Voicing.
_Avoid_: Rotation

**Source Function**:
A Function whose result may depend on Cells outside its explicit operands or may change Cells beyond its Output Portal. Its value reads wait for current-Tick suppliers to settle. Placement tests occupancy in working Source at its Turn without waiting for a vacancy; its writes pass through Portals (EVAL-025).
_Avoid_: Spatial operator, grid function

**Copy Function**:
One of the directional Copy Functions `=^`, `=v`, `=<`, and `=>`, or the absolute Copy `=$ from to`, whose `from` and `to` are each a Position. A directional Copy is a Function that answers a value: it reads one aligned two-Cell Language Unit at the Portal opposite its output and writes that value through its output Portal. East and west displace two columns; north and south one row. Consecutive Copies compose through those Portals; when one output covers the next Function, the first write suppresses it. Empty aligned input is copied like any other: the Copy clears the two-Cell destination and, nested, returns the empty Cells, so its parent is invalid. Partial or invalid input diagnoses and writes nothing. An ordinary output atomically overwrites its complete destination Span. A Bang output is written as `**` over its complete destination like any other output, covering a root's anchor or data that stands there, and activates the roots aligned with that destination; an out-of-Grid destination diagnoses. A Copy participates in the dependency schedule, and its Bang output can activate a root in the same Tick. A Copy whose input is a Bang Function reads it after its Turn: the cleared Cells, or `**` while a Halt locks it. A Copy does not transport part of a Language Unit. The absolute Copy reads the pair at its source Position through a dynamic Input Portal and writes it to the pair at its destination Position through a dynamic Output Portal, in one Turn that reads and then writes, so a source and destination that overlap form no cycle.
_Avoid_: Jump, Jumper, Jymper

**Read Function**:
A Function that reads one Cell pair of Source and answers it: a directional Read `&^ &v &< &> distance`, the absolute Read `&$ at`, or Track `&t index count`. A directional Read reads the pair `distance` Portals from its `distance` operand in its arrow's direction: `00` is the operand itself, and each step is one pair east or west or one row north or south in the operand's columns, so `&>00C4` answers `00`, `&>01C4` answers `C4` and `&>02C4D4` answers `D4`. A nested `distance` counts from the slot it occupies, so in `&>.+0101C4` step `03` is `C4`. The absolute Read reads the pair at a Position. Every operand is a Number; each may be a literal, a nested Function or Cells a Portal writes, and an unwritten one makes the Read invalid. A Read reads its pair exactly as a Copy reads its Input Portal and writes what it reads through its Output Portal, one row south, and, nested, returns it to its parent. A Read claims only its operands: the Cells it reads are ordinary Source. Its Input Portal is dynamic, so its Turn waits for every writer of the selected pair that has not yet taken its Turn. A Read never waits on its own write: when its pair is its own Output Portal, it reads those Cells as they stand at its Turn, as Increment and Slew read their feedback at the same site as their Output Portal. A pair outside the Grid or cut short by the row edge diagnoses. One Read reads one Atom: Orca's Read and Query are Reads, and Konkat's hidden variable lookup is a Read of visible Source.
_Avoid_: Source Read, Read Operator, Query Operator, Konkat Operator

**Track Function**:
`&t index count`, a Read Function that answers a value. Its Input Portal is the two Cells `index % count` pairs east of its last operand, counted from zero, so `&t0103C4D4E4` reads `D4`. A nested Function supplying its count extends that operand through its own operands; if a write suppresses that Function, the count instead occupies the two Cells at its anchor and the selectable pairs begin immediately after them. This same rule applies when Track retries its Turn after waiting for a writer. Otherwise it reads, writes and waits as every Read Function does. A `count` of `00` diagnoses, and so does a pair past the row edge.
_Avoid_: Sequencer, list index, lookup table

**Write Function**:
A Function that writes its `value` operand into one Cell pair of Source: a directional Write `@^ @v @< @> distance value`, which counts its pair exactly as the Read with the same arrow does, or the absolute Write `@$ at value`, which writes at a Position. Its `value` takes no type: the Write carries the two Cells it holds, or the encoding a nested Function returns, unchanged, and the destination's reader interprets them. Its Output Portal is dynamic: it answers `value` at that pair and nowhere else, and, nested, returns `value` to its parent. A Read nested in a Write's `value` is Orca's Generator.
_Avoid_: Source Write, Generator Function

**Push Function**:
`@t index count value`, a Write Function that writes `value` into a lane of `count` pairs on the row below it, starting directly under its anchor: the pair `index % count` pairs east of its default Output Portal. So `@t0103C4` writes `C4` into Cells 2–3 of the row below. The lane is ordinary Source: Push claims only its operands.
_Avoid_: Generator (a Read nested in a Write), lane write

## Evaluation

**Source Snapshot**:
The complete Source at the beginning of a Tick, including the accumulated output of preceding Ticks. It is the whole of the initial language state: nothing about a Bang is remembered outside Source, so two identical Snapshots at the same absolute Tick plan the same Tick. Dependencies supply current-Tick values before their consumers evaluate.
_Avoid_: Program state, runtime state

**Tick**:
One discrete musical-time step that interprets a Source Snapshot and atomically applies its Tick Plan. Musically a Tick is a sixteenth note: four Ticks make a beat at the displayed BPM, so a 4/4 bar is 16 Ticks (`10`). A DAW's "tick" is a much finer subdivision of a beat, so the two are not the same unit. An error never stops a Tick: a same-Tick dependency cycle stops the remaining Turns in the Expressions it reaches, each diagnosed, while completed Turns retain their Effects and independent Expressions publish.
_Avoid_: Cycle, frame

**Absolute Tick**:
The number of a Tick counted from Tick `0`, an interpretation input supplied from outside Source (EVAL-060). Clock, Every, Euclidean and Random read it.
_Avoid_: Frame count, song position, step (a Clock's step is a count of its own)

**Tick Plan**:
The complete deterministic outcome of interpreting one Source Snapshot at a particular Tick, including activation routing, Source writes, ordered Play Commands, ordered Network Sends, and diagnostics.
_Avoid_: Play sequence, command batch

**Producer**:
An original Function with an anchor Position and an opportunity to contribute ordered Effects during a Tick. Bang is a result a Producer can emit, not a Producer of its own; the Bang Function, a standalone `**`, is a Producer like any other, and so is Halt.
_Avoid_: Emitter, actor, source operator

**Turn**:
One original Producer's opportunity to emit Effects after its current-Tick data and activation dependencies have settled. A Producer takes at most one turn; Position breaks ties between independent Producers.
_Avoid_: Pass, visit, step

**Activation**:
What lets a root take its Turn in a Tick: a Bang aligned with it, delivered during that Tick, or the root's own declaration that it needs none. Each Function declares where its activation comes from (EVAL-050), and a nested Function is activated with its root.
_Avoid_: Trigger, enable, gate

**Portal**:
One Cell destination resolved during a Tick. A Function may read through a Portal input, write through a Portal output, or both at the same site. When a Function does not name another Position, that Portal is one row south of the Function's anchor. The site is declared by the Function and placed by its anchor, so it is known from any Source revision before a Tick runs; what is resolved during a Tick is what the Portal carries. It carries an Atom, or one destination in a Source Function's validated write bundle; it is neither a language value nor persistent state.
_Avoid_: Port, address value, output coordinate, ordinary result

**Output Portal**:
The Portal through which a Function acts on the Source. Every Function has one. A static Output Portal is a fixed offset from the Function's anchor, known when the Source is parsed: one row south unless the Function names another, as each directional Copy names its own direction. A dynamic Output Portal is the pair the Function's operands select, known only at its Turn: every Write Function's, Push's and the absolute Copy's. A Function with a dynamic Output Portal takes its Turn before every computation that does not feed it, so they see its write in the same Tick; a Turn that read those Cells earlier in the Tick sees the write on the next Tick, as feedback. A Function that answers a value writes that answer there, whether it is a root or nested; Halt locks the root there instead; a Terminal Output Function writes nothing there, so its Output Portal is never addressed and has no offset, and a Source-writing Function's writes are its declared Source effect rather than an answer.
_Avoid_: Result Cell, output Cell, destination

**Input Portal**:
The Portal a Function declares it reads a Source input through, as an offset from its anchor. A directional Copy reads at the Portal opposite its Output Portal; Increment and Slew read at the same site as their Output Portal, which is how their feedback stays in Source, nested or not. An Input Portal is static or dynamic by when its position is known; its Cells are always read from working Source at the Function's Turn. The directional Copies', Increment's and Slew's Input Portals are static: their position is a fixed offset, known when the Source is parsed, so their writers are ordered before the Tick. A dynamic Input Portal's position is selected by its Function's operands, by a rule that Function's definition states, so its position is known only at its Turn and its writers are found then. Every Read Function's Input Portal is dynamic, and so is the absolute Copy's: Track's operands select a pair east of its last operand, a directional Read's count Portals from its `distance` operand, and the absolute forms address a Position. Most Functions declare none and take every input as an operand.
_Avoid_: Input Cell, source Cell, feedback register

**Pending Operand Encoding**:
The current characters of a spatially updated operand, awaiting interpretation when its receiving Function executes. They may be invalid for that operand's literal type and are not yet a decoded value.
_Avoid_: Incorrect typed value, inferred Number, failed decode

**Reservation**:
The Cells one computation's result may reach, settled while a Tick is scheduled and before any Function has evaluated. A value Function answers one Atom or a Clear (LANG-330), and either occupies the same two-Cell pair, so every value Function, root or nested, reserves that pair at each of its write Portals, and a Source-writing Function reserves the Spans its declared bundle names. The pair a value answer reserves at its Output Portal is its Output Portal Reservation, which the console highlights; a Source Bundle's Spans are not one. A Reservation orders Turns and decides nothing else: the write a Portal admits is what decides whether a Cell was written, a computation suppressed, or a root activated.
_Avoid_: Footprint, allocation, reserved Span, write window

**Dependency**:
An order between two Turns, known before the Turn it constrains. A Producer goes before every computation its Reservation, lock or Bang reaches, including one that reads Cells its Reservation covers, and a nested Function goes before the Function it answers. A root activated during a Tick brings its Dependencies with it. A cycle of Dependencies stops every Expression it reaches.
_Avoid_: Edge, ordering constraint

**Contact**:
A Function whose Cells a Reservation or an admitted write covers, together with every computation nested in it. A Contact that begins at the Function's anchor is where a Function Replacement lands. Scheduling and execution ask the same rule what a Contact means: before the Tick it can only order the producer first, and during the Tick it decides whether the Function is replaced, suppressed, or reached too late. A Bang reaching a root it aligns with is activation, not Contact.
_Avoid_: Overlap, hit, collision

**Effect**:
One thing a Producer contributes to the Tick Plan: a Cell write, an activation delivery, a root lock, a diagnostic, or the Play Command one terminal root performs. Effects follow execution order — dependency order, with Position breaking ties (EVAL-020, EVAL-021) — and then the order their Producer emits them. Cell writes validate their whole destination before any Cell of them is emitted, and resolve Cell-wise, so a later Effect wins each Cell it overlaps and leaves the rest of an earlier write standing.
_Avoid_: Action, mutation, command

**Source Bundle**:
The validated set of Source writes a Source-writing Function declares, stated by two facts: whether it clears the producer's own Span, and whether it writes a Function spelling at that Span displaced. An Advance does both, so its producer moves: a Self-Banging Function. An Emit only writes displaced, so its producer stays standing: a Directional Bang Function. A Fire only clears its own Span: the Bang Function. The writes are planned in that order, own Span first. What a refusal costs follows from the first fact: a producer that clears its own Span leaves `**` there when its displaced write is refused, and one that stays standing has no Cells of its own to say it with and is diagnosed instead. A Fire writes nothing displaced, so nothing of it can be refused.
_Avoid_: Write mode, movement type

**Function Replacement**:
A Cell write whose value is a Function Atom landing on the anchor Position of a computation already running in the same Tick, replacing which Function stands there for the remainder of that Tick. It is refused unless the incoming Function agrees with the running one on every fact the settled schedule was derived from: whether it answers a value, where its activation comes from, whether it can emit Bang, and the Source write it declares. Those four are held fixed because the schedule is fixed before any Function evaluates (EVAL-020), so a replacement differing on any of them would leave Turns ordered from edges that no longer describe what runs, or a write landing at Cells no dependency edge names. Every value Function reserves the same one Atom's Cell pair, so the width of a result is not among them. The comparison is against the Function the computation is running rather than the one the Parser found, which tells a second replacement at one anchor from the first. A refusal names which of the four differed. No Function answers a Function Atom today — ADR 0034 defers the Source operation that would — so the rule is stated and held ahead of the language that reaches it.
_Avoid_: Overwrite, mutation, hot swap, redefinition

**Bang**:
A transient pulse Atom returned by a Function, whose output activates aligned neighboring roots during the same Tick. Alignment is measured from the result's own two-Cell Span: a horizontally aligned root anchors two columns away with its Span touching the result's, and a vertically aligned root shares the result's anchor column one row north or south. A Function that answers Bang writes `**` and activates its aligned roots in its own Turn. The `**` it leaves is ordinary Source: on a later Tick it is a Bang Function, unless its producer's next write covers it first. In an untyped operand `**` is the Bang value; in a typed operand it remains invalid syntax and cannot activate another root. A Function that passes a pair through — a Copy, Read, Track, Write or Push — carries `**` as it carries any other pair: when its answer is a Bang it writes `**` over its complete destination, whatever stands there, and that write is its Bang output, activating the roots aligned with the destination. A root whose anchor the `**` covers is overwritten rather than activated: if it has not yet taken its Turn it takes none that Tick, and if it feeds the writer and so has already taken it, the `**` lands as feedback that the root meets on the next Tick. A `**` in a Write's or Push's untyped `value` is carried as a Bang in this way. A nested Function that answers Bang writes `**` through its own Output Portal and activates the roots aligned with that `**`, as a root does, and its Return is `**`.
_Avoid_: Boolean, trigger flag

**Bang Function**:
A standalone `**` where an Expression starts: a root-only Function with no operands that is intrinsically active, as Orca's `*` is an operator. Its Turn activates the roots aligned with its Span and clears its two Cells. Every other outcome is a rule every Function follows. A write that covers it before its Turn suppresses it, so a producer that bangs again or answers a Clear suppresses its own last `**`, and a partial overwrite suppresses the whole Bang Function. A producer that takes no Turn, or is invalid, writes nothing, so its last `**` fires once more. A Halt above it locks it, a reader of its Cells waits for it, and a mover meets it in Source order. Whether a `**` fires depends only on the Source Snapshot and the absolute Tick.
_Avoid_: Bang display, Bang glyph, typed Bang

**Missed Bang**:
A Bang delivered to a root that has already taken its Turn this Tick. It activates nothing there, and the root is diagnosed "Bang reached a root that has taken its Turn", whichever producer delivered it.
_Avoid_: Late Bang, stored Bang

**Directional Bang Function**:
One of the four Source Functions `*^`, `*v`, `*<`, and `*>`. Like every Function that performs an effect, other than a Self-Banging Function and the Bang Function, it is inert until Bang activation (EVAL-050), and each activation emits a Source-resident Self-Banging Function into the two Cells immediately outside its own two-Cell Span in the selected direction. The emitted Function first receives a turn from the following Source Snapshot and thereafter activates itself. That asymmetry is why both forms exist: a Directional Bang Function acts once per Bang it receives, while the Self-Banging Function it writes acts every Tick without one.
_Avoid_: Always Function, movement Function, automatic mode

**Self-Banging Function**:
One of the root-only Source Functions `^^`, `vv`, `<<`, and `>>`, emitted by the matching Directional Bang Function. In its scheduled evaluation it intrinsically receives Bang activation without creating a Source-resident `**`, then advances its complete two-Cell Span by one Cell in its retained direction. A successful move atomically clears the current Span and writes the Function's own spelling at the shifted destination. A blocked or out-of-Grid move instead changes its current Span to `**` and activates nothing that Tick; on the next Tick that `**` is a Bang Function and activates every root aligned with it. Contact with only part of another Language Unit is also diagnosed. A mover meets a Bang Function in Source order, as it meets another mover: one later in Source order enters the Cells the Bang Function cleared, and an earlier one finds them occupied. A Self-Banging Function is not an operand or runtime value.
_Avoid_: Activation Character, Self-Activating Function, Arrow Function, moving Bang, projectile

**Halt**:
The Source Function `*!`. When active, it locks the Expression root directly south. Its lock is an Effect of its Turn, which is ordered before its target's, and it holds only if a Bang that executed activated the Halt before that Turn. A Halt directly above the `**` that would activate it therefore takes its Turn inert: the `**` fires, and its Bang reaching the Halt is a Missed Bang. A Halt activated by another Bang locks a `**` below it, which stands without firing while it is locked. Orcvs evaluates each Halt at most once per Tick, and a Halt suppressed by another Halt does not lock its own target.
_Avoid_: Halt Function, Stop Function, control phase, retroactive suppression

**Clock Function**:
The Tick-family Function `~. division steps`, a step counter, which answers the step its cycle is at as a Number. Both operands are Numbers: `division` is how many Ticks one step lasts and `steps` is how many steps the cycle holds, so the cycle is `division * steps` Ticks long, and a higher `division` is slower, as on a clock divider. The answer counts `00` through `steps - 1` and then begins again. It reads the absolute Tick of Playback as the explicit interpretation input EVAL-060 makes it, rather than counting its own activations, which is what makes the same Source Snapshot interpreted at the same Tick answer the same step, and, because a stop holds the absolute Tick (PLAY-150), what resumes a stopped Clock on the step it stopped on. The division and the steps are taken in an integer wider than a byte before the step becomes a Number, because the absolute Tick outgrows a byte in seconds while the step it answers never does. A zero division or a zero steps is a cycle with no length: it diagnoses under a message naming the Function and the operand rather than reusing Division's or Modulo's, and the division is answered first because that is signature order.
_Avoid_: Phase accumulator, tempo, bar position, length (a Note's gate length), rate

**Every Function**:
The Tick-family Function `~* division steps`, a clock divider, which answers a Bang once per cycle and a Clear on every other Tick, root or nested. Its two Number operands are the division and steps the Clock Function takes, and it Bangs exactly when the absolute Tick is a multiple of `division * steps`, Tick `0` included: Tick `0` is a Tick like any other, so an Every fires as Playback starts from it rather than one cycle in. A `steps` of `01` is therefore one Bang per `division` Ticks and not one every Tick, which is what makes the two operands a division and a step count rather than two spellings of one period. Tidal's `every` applies a function to every nth cycle of a pattern; Every fires a pulse and transforms nothing. The cycle product is calculated in an integer wider than a byte because it is a length rather than a value the Source wrote — `~* 10 20` is a cycle of 512 Ticks, which a byte would fold to zero and then divide by. A zero division or a zero steps diagnoses exactly as it does for the Clock Function.
_Avoid_: Delay (the Sound Module's echo effect), sleep, scheduled event, delay line, echo

**Euclidean Function**:
The Tick-family Function `~%`, which answers a Bang on the Ticks a Euclidean rhythm places an onset at and a Clear on the rest, root or nested. Both operands are Numbers: `hits` onsets distributed as evenly as whole numbers allow across a cycle of `steps` Ticks, so `~% 03 08` sounds the pattern `X..X..X.` without a pattern being written anywhere. The patterns are Tidal's: each is the rotation Tidal's `bjorklund` gives, and one with an onset starts on it, so `~% 03 04` is `XXX.` as Tidal's `"bd(3,4)"` is, and a pattern copied from Tidal sounds the same (EVAL-063). The absolute Tick is reduced into the cycle first, so the Tick counter's size never reaches the pattern. Validation asks for a positive step count before it compares hits to steps, so `~% 00 00` is a cycle with no positions rather than a pattern with no onsets and diagnoses instead of falling silent; more hits than steps diagnoses under a message naming both. With a positive step count, zero hits Bangs on no Tick and equal hits and steps Bang on every one.
_Avoid_: Euclidean algorithm, pattern table, rhythm string, step sequencer

**Increment Function**:
The Tick-family Function `~+`, an accumulating counter that wraps, which answers `(previous + amount) % modulus` as a Number. Both operands are Numbers: `amount` is how far to advance and `modulus` is where the count wraps. Its Input Portal and Output Portal are the same site, one row south of the Function: the Number input reads that Portal in working Source at its Turn, including earlier same-Tick writes; empty Cells supply the initial Number `00`, and any other present Language Unit diagnoses. The addition and the modulus are taken in an integer wider than a byte before the answer becomes a Number, so `FF + 02` cannot wrap before the modulus is applied. A zero modulus diagnoses under a message naming the Function and the operand rather than reusing Division's or Modulo's.
_Avoid_: Counter (unqualified: Clock is the step counter, Increment the accumulating one), accumulator, hidden state, +=

**Slew Function**:
The Tick-family Function `~> amount target`, a slew limiter, which moves a Number toward a target by at most `amount` each Tick and never overshoots, so a jump in the target becomes a ramp. Both operands are Numbers: `amount` is the farthest one Tick may travel and `target` is the value it is moving toward. Its Input Portal and Output Portal are the same site, one row south of the Function: the Number input reads that Portal in working Source at its Turn, including earlier same-Tick writes; empty Cells supply the initial Number `00`, and any other present Language Unit diagnoses. When below the target it steps up by `amount` or lands on the target; when above it steps down the same way; when equal it returns the target unchanged. Each distance is calculated only in the branch whose subtraction is non-negative, and the remaining step is taken in an integer wider than a byte. An `amount` of `00` holds the current value.
_Avoid_: Interpolation, lerp, tween, easing, filter, glide

**Evaluator**:
The evaluator that turns a Function and its typed operands into one value or terminal effect. Its Operand Stack exists only for that evaluation; activation, spatial delivery and Source remain outside the Evaluator.
_Avoid_: Virtual machine, VM, interpreter loop, runtime

**Operand Stack**:
The stack of operand values one Function is evaluated against. It holds exactly the operands that Function's signature declares, in signature order, and the Function pops them from it; neither the Function's answer nor an effect is ever pushed onto it. An operand list of any other length is refused before the stack exists, so its depth never exceeds the widest signature any Function declares. It is created for one evaluation and discarded when that Function answers, so nothing carries from one evaluation to the next or from one Tick to the next. EVAL-081 bounds its depth by the Function table and has a push past that bound answer a diagnostic rather than panic.
_Avoid_: Value stack, call stack, machine memory, register, Stack (unqualified: Stack is a Pitch Function)

**Live Editing**:
Changing the Source while Playback continues. An edit affects the next Tick whose Source snapshot has not yet been taken. A Tick commits only against the revision it planned from (EVAL-090, EVAL-091), so an edit made while a Tick plans is never overwritten: that Tick takes its snapshot again, at the same absolute Tick, and publishes nothing from the plan it abandoned.
_Avoid_: Hot reload, live coding

**Tick Portals**:
The ordered record of every Portal access one committed Tick made, each entry naming the anchor, the Cells of its subtree and the Function of the Turn that made it as execution held them. It is an observation of a committed Tick, not a Portal: a Portal still does not survive its Tick (EVAL-030), and what survives is what each Turn did with one. A read is recorded once its Turn passes the writer wait, as the Cells it resolved or why none resolved; a resolved read says a site was read, not that it held a value. A write is recorded for each Output Portal site its Turn delivered to, as the Cells admitted or why the write was refused whole, and a Halt's lock and each Source Bundle part are recorded with what the Tick decided for them. It is planned with the Tick Plan (EVAL-100 to EVAL-106), committed or dropped with it, replaced by every committed Tick, and answered by a Source revision as `Current` when that commit left it or `Retained` when an edit has landed since; the Render Frame paints its resolved reads as Input Portal and its admitted writes as Output Portal, a retained record's only where the owner Function still stands at its anchor, no edit since the commit has written a Cell of the subtree the entry recorded and, for a place its operands selected, where no read recorded by a descendant of the owner read a Cell an edit has written since. It marks the Cells of the Function of each root a Halt's lock withheld as Locked, where the displayed revision still begins an Expression with a Function there, a retained record's while the Halt still stands at its anchor and no edit has written its Cells or the locked root's spelling, and the console paints Output Portal over Input Portal over Locked. It paints no Source Bundle part.
_Avoid_: Portal log, trace, Portal history

## Playback and outputs

**Playback**:
The time-driven process that requests a new Tick Plan for each Tick and dispatches its Play Commands. Playback does not parse Expressions or own Source interpretation.
_Avoid_: Player, sequencer

**Playback Engine**:
The module that owns Playback lifecycle and musical time and dispatches each Tick's ordered Play Commands and Network Sends exactly as supplied. Each destination keeps its own output policy (PLAY-390, PLAY-400): a Tick's MIDI is delivered before its Network Sends, Network Sends go on while no MIDI destination is connected, and stopping, rewinding, disconnecting and the safety action reach MIDI alone. It does not parse Source or interpret musical intent; when a Timed or Monophonic Play Command explicitly supplies a lifetime, it schedules the corresponding Note Off. One schedule holds both, keyed per channel and note for Timed Play and per channel alone for Monophonic Play, each claim carrying a generation token, so a stop retired by a replacement or by an explicit stop cannot cut a later note short. Beginning a run, stopping, rewinding a live run, disconnecting, and changing destination each clear the schedule. Stopping holds the absolute Tick (PLAY-150): the next run resumes at the Tick after the last one executed, and only a Rewind returns it to Tick `0`. What a refused submission does to the schedule is stated in PLAY-260 and in its entry under "Accepted but not built, or built differently" in `docs/requirements/playback.md`. Stopping Playback, rewinding a live run, disconnecting an output adapter, changing destination, and tearing down after a refused delivery each trigger the same safety action. Notes alone are not enough, because a Control Change can latch state and a Pitch Bend can deflect the wheel, and neither ends when the Source that wrote it stops running. A channel whose message the destination refuses does not end the action: the first refusal is the failure reported and every remaining channel is still attempted. Nothing is sent to release the individual notes the Playback Engine knows it started; All Notes Off is what silences them, and stopping, rewinding, disconnecting, and changing destination discard the schedule rather than replay it.
_Avoid_: Runtime, audio engine, MIDI engine, sequencer

**Safety Action**:
The fixed sequence of messages that returns every MIDI channel to silence and its defaults (PLAY-260); the one output the Playback Engine originates (PLAY-010).
_Avoid_: Panic, reset, all-notes-off

**Tick Grid**:
The deadlines one Playback run's Ticks are due at: every whole multiple of the Tick period from the deadline that run began on. The grid holds until the run ends or its tempo is retuned (PLAY-180), so a stall costs the Ticks it covered and does not move the ones after them, and the run resumes at the first deadline still ahead of the instant its clock woke. Retuning begins a new grid without beginning a new run, running it from the deadline the last executed Tick was due at rather than from the moment the retune arrived or the moment that Tick was seen.
_Avoid_: Schedule, timeline, beat clock

**Overrun**:
One Tick declined because it was observed a whole Tick period or more past the deadline it was due at. It names that deadline (PLAY-190), consumes no absolute Tick, and reports once per stall rather than once per deadline the stall covered: the deadlines a stopped clock never reached are skipped rather than delivered late so they can be declined in turn. An Overrun is a Playback diagnostic and carries no user-facing message.
_Avoid_: Dropped frame, xrun, missed tick

**Omitted Diagnostics**:
The per-class count of Playback diagnostics recorded since the last drain and not retained. A Playback Engine retains a bounded number of diagnostics between drains (PLAY-110 to PLAY-130), evicting Overruns first and clock failures last, and a drain that omitted anything ends with one summary of these counts. A count saturates rather than wrapping and is then a lower bound.
_Avoid_: Dropped diagnostics, lost events

**Rewind**:
Making Tick `0` the next Tick Playback executes, with Command Shift R on native as in Orca. A stopped Playback stays stopped (PLAY-160) and its next run begins at Tick `0`, with the Tick and Run Clock shown at zero; a live run is stopped, with the safety action and its schedule discarded, and the next begins at once at Tick `0` at the same period. A stop alone never rewinds: Space pauses and resumes, as Orca's does.
_Avoid_: reset (Reset All Controllers is part of the safety action), restart, pause as a state apart from stopped

**Run Clock**:
The wall-clock duration Playback has spent playing since Tick 0: held while Playback is stopped, counted on from there when a run resumes, and zero again after a Rewind.
_Avoid_: frame time, elapsed, timer

**Play Command**:
One interpreted MIDI instruction emitted by an active Terminal Output Function for delivery during a Tick. It is an explicit variant per spelling — a Raw Play note, a Timed Play note, a Monophonic Play note, a Control Change, and a Pitch Bend — carrying values already validated against their MIDI domains rather than assembled wire bytes, so the output adapter alone knows the protocol encoding. Every field is a type minted for the role it plays, so two operands that share a domain — a Control Change's controller and value, a Pitch Bend's LSB and MSB — are not assignable to one another. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list, beside the Tick's Network Sends; velocity `00` explicitly stops a note, and the Playback Engine delivers that stop as a Note Off.
_Avoid_: Performance command, MIDI event

**Network Send**:
One UDP datagram or one OSC message an activated UDP or OSC Function emits for delivery during a Tick, carrying its Payload already read and, for OSC, already typed. Network Sends are ordered within their Tick Plan and delivered as one list beside the Play Commands; a Network Send is never a Play Command, and nothing the Playback Engine does to MIDI, such as a lifetime or the safety action, reaches it.
_Avoid_: Network Play Command, packet, network event

**Output Command**:
One MIDI message the Playback Engine hands an output adapter. A Play Command says what the Source asked for; an Output Command says what is delivered, and the two differ wherever the Playback Engine owns the difference: a Timed or Monophonic Play's lifetime becomes a Note On in its Tick Plan order and a Note Off at the beginning of Tick `T + length`, and a Monophonic Play's replacement of the voice its channel was sounding becomes a Note Off of the note it found there, before the new Note On or, played legato, after it, and no Note Off at all when it is played legato at the pitch already sounding, and a Play's velocity `00` becomes a Note Off rather than a Note On, while a command with nothing to resolve is carried through unchanged. Every stop reaches the adapter as a Note Off, which the MIDI adapter encodes as `[0x80 | channel, note, 0x00]`: Orcvs has no release-velocity spelling, and the Playback Engine never hands an adapter a zero-velocity Note On. Control Change and Pitch Bend are the commands with nothing to resolve: neither carries a lifetime and neither owns a voice, so each has a variant of its own and reaches the adapter exactly as the Source wrote it. Every Output Command is one message the adapter assembles immediately, so an adapter is never handed a lifetime it would have to schedule.
_Avoid_: Wire command, output event, delivered Play Command

**Terminal Output Function**:
The family of `!`-spelled Functions that perform an effect and answer with no language value: Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`, Raw Chord Play `!:`, Timed Chord Play `!#`, Control Change `!c`, Pitch Bend `!b`, UDP `!u`, OSC `!o`, and in turn Application Command `!$`. Every member performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result. Activation gates evaluation itself, so an inactive terminal root reports no evaluation-time diagnostic either: ADR 0016's operand diagnostics are outcomes of evaluation, and an Expression that never evaluates has no outcome to report. Lexical and syntax diagnostics are unaffected and still fire regardless of activation. Each MIDI member emits Play Commands carrying operands already validated against their MIDI domains, so the output adapter alone assembles the wire message, and the UDP and OSC Functions each emit one Network Send of their Payload. A Chord Play emits one Play Command per tone (PLAY-290), lowest first, and every other Expression performs at most one output: a domain, type or Payload fault diagnoses and emits nothing. Several Terminal Output roots that one Bang activates remain the way to give tones different channels, velocities or lengths.
_Avoid_: Effect Function, side-effecting Function, output verb, action Function

**Raw Play**:
The Terminal Output Function `!> channel velocity note`, which interprets a hexadecimal Number channel `00`–`0F`, a hexadecimal Number velocity `00`–`7F`, and a Note as one raw Play Command. The note of a Raw, Timed or Monophonic Play is optional (LANG-300): a Play whose note is blank, however the note got blank, whether written blank, emptied by a Portal write or cleared by a nested Function, sends nothing and is not diagnosed. A half-written or mistyped note is still invalid and diagnosed, every other operand binds and is diagnosed exactly as when a note is present, and the Play keeps its Turn in Tick order. A note an earlier Turn started sounds to its own Note Off.
_Avoid_: Play Function (unqualified), Note output, MIDI Function, Rest, silent Play

**Timed Play**:
The terminal `!~ channel velocity note length` Function. Channel is a Number `00`–`0F`, velocity is a Number `00`–`7F`, note is a Note, and length is a Number `00`–`FF`. Velocity `00` explicitly stops the specified note and schedules no expiry. Otherwise, length `00` emits no MIDI output, while a positive length starts the note in the current Tick and schedules Note Off at the beginning of Tick `T + length`. Its fixed arity distinguishes it from Raw Play without overloading or a variable operand count.
_Avoid_: Play overload, optional-length Play

**Monophonic Play**:
The terminal `!% channel velocity note length legato` Function, whose first four operands take Timed Play's domains and whose `legato` is a Number that may be blank, where blank means `00` (a Defaulted Operand, LANG-320). The Playback Engine owns one Mono voice per MIDI channel, in the schedule that holds its Timed claims and cleared by the same lifecycle actions; PLAY-230 and PLAY-250 settle that seam, because an Output Command is one message carrying no lifetime and leaves an output adapter nothing to schedule with. The two ownerships differ only in their key — Timed Play's channel and note against Monophonic Play's channel alone — so neither can name the other's voice and each expires on its own claim. That independence is the engine's, not the wire's: a Note Off carries only channel and note, so a Mono note and a Timed note at the same pitch on one channel alias at the receiver, where the first stop delivered silences whichever is sounding. This is the aliasing PLAY-250 states for notes of the three Play spellings at one pitch rather than a separate one. Every command replaces the prior Mono-owned claim on that channel, whatever note it held: the voice is the channel, and the Source need not remember what it last put there. A `legato` of `00` or blank retriggers, stopping the prior note before the replacement starts; any other `legato` starts the replacement first and stops the prior note after it, so a mono synth slides, and a replacement at the pitch already sounding sends its Note On with no Note Off, so that note sounds on to the replacement's end. A sliding replacement on the Tick the prior note's stop is due takes that stop over, so a line whose lengths equal its steps still slides. Velocity `00` or length `00` starts nothing whatever the `legato`, replacing the voice with silence, which is where it parts from Timed Play's length `00` no-op — a Timed command claims the note it names, so a note that never starts claims nothing, while a Monophonic command claims its channel whether or not it sounds. Otherwise, a positive length starts the replacement and schedules its Note Off at Tick `T + length`; a later replacement retires that expiry through its generation token. Raw Play and Timed Play notes do not enter Mono ownership.
_Avoid_: Global Mono voice, Play-wide voice stealing, implicit channel sharing

**Raw Chord Play**:
The terminal `!: channel velocity root set voicing inversion` Function. One Turn emits one ordinary Raw Play Command per tone, lowest first. The Voicing removes tones, the Inversion moves the lowest up an octave as many times as its Number says, and the Voicing then moves tones, so the order is fixed. An Interval Set with too few tones for its Voicing, or any tone outside `00`–`7F`, diagnoses the whole Chord Play and sends nothing, so no partial chord is ever sent. The tones exist only inside the Turn: nothing is written to Source.
_Avoid_: Chord Function, arpeggio

**Timed Chord Play**:
The terminal `!# channel velocity root set voicing inversion length` Function, built as Raw Chord Play is but emitting one ordinary Timed Play Command per tone. Each tone is then its own Timed Play to the Playback Engine: velocity `00` stops every tone, length `00` starts none and leaves what is sounding, each tone stops at its own Note Off, and a repeated Timed Chord Play replaces the tones it repeats. A Timed Chord Play's tones and a Monophonic Play on one channel keep separate ownership, as a Timed Play and a Monophonic Play do. There is no Monophonic Chord Play: a voice per channel cannot hold several tones.
_Avoid_: Chord length, timed arpeggio, Monophonic Chord Play

**Control Change Function**:
The terminal `!c channel controller value` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and controller and value `00`–`7F`, and sends them without scaling. Controller and value share that domain and are separate types regardless, one per role, so nothing that reads the command can put one where the other belongs. Invalid operands diagnose and emit no MIDI output, naming the role the Source wrote out of range rather than the domain the two roles share.
_Avoid_: CC scaling, normalized controller value, optional value

**Pitch Bend Function**:
The terminal `!b channel lsb msb` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and each data byte `00`–`7F`, and sends the MIDI wire bytes without scaling. LSB precedes MSB on the wire, and the two halves are separate types for the reason a Control Change's controller and value are: a bend is never assembled into one fourteen-bit value anywhere in Orcvs, so nothing has to take it apart again. Invalid operands diagnose and emit no MIDI output.
_Avoid_: PB scaling, normalized bend value, combined integer bend

**Application Command Function**:
The terminal `!$` Function that sends a command to the Orcvs host application. It can invoke only commands that the application explicitly provides; it never invokes an operating-system shell or arbitrary executable.
_Avoid_: Host Command, shell command, process execution

**UDP Function**:
The terminal `!u` Function. When activated it sends its Payload as one UDP datagram to the console's UDP destination: one byte per Cell, a space for an empty Cell, and no terminator (PLAY-320). An empty Payload sends nothing. In a build that cannot send UDP, an activated `!u` diagnoses instead.
_Avoid_: String output, UDP message value

**OSC Function**:
The terminal `!o` Function. When activated it sends its Payload as one OSC message to the console's OSC destination (PLAY-330). The Payload's first word is the address and starts with `/`; an optional next word starting with `,` is a type string of `i`, `f` and `s`; the remaining words are the arguments. Without a type string, one to eight uppercase hexadecimal digits send an int32 whose bits they spell, so `1B8` sends 440, a decimal with a point sends a float32, and any other word sends a string. A malformed address, type string or argument diagnoses and sends nothing. It sends no bundle and no timetag.
_Avoid_: OSC path glyph, OSC value

**Payload**:
The Cells a UDP or OSC Function sends: every Cell from the end of its spelling through the next `!` on its row, which closes it, or through the row's end when none follows. A Payload is never parsed and cannot contain `!`. Only the spelling and the closer are occupied Cells, so the Payload's own Cells are ordinary Cells that a Write writes and a Read reads, and its Function reads them at its Turn after their writers. Its extent is fixed when the Source is parsed and holds for the whole Tick, so a `!` a Write puts inside it is sent as a character until the next parse.
_Avoid_: Text value, message value, string literal

**Built-in Synth**:
The synth a page that frames the console offers it as a MIDI destination, so an example sounds with no MIDI device: tone channels, a 303-style acid channel and a synthesised drum kit, each channel with its own effects. Its channels, controllers and timing are stated in `docs/requirements/playback.md` (PLAY-520 to PLAY-620), and how a page offers it in `docs/requirements/site.md` (SITE-040 to SITE-049).
_Avoid_: internal synth, sound module, Pilot, Enfer

## Console

**Orcvs (running instance)**:
One active console session, owning its Source and the Grid that shapes it, its Cursor, its Playback lifecycle, and its presentation options. Choosing an output device is console configuration rather than part of a running Orcvs. The unqualified system name Orcvs still names the environment and language as a whole.
_Avoid_: App, Session, Machine

**Render Frame**:
One repaint of the console, in which every Position the Grid yields is drawn once. Render Frames are driven by the UI many times a second, independently of musical time: a Render Frame reads the Source and never advances Playback, so it is not a Tick.
_Avoid_: Frame, Tick, refresh

**Cursor**:
The one Cell the console is editing: a Position drawn with a still outline on its Cell's edges. The Cursor holds no dimensions and does no clamping of its own — the Grid answers where a move lands.
_Avoid_: Caret, pointer, insertion point

**Region**:
A rectangle of Positions within the Grid, spanned from an anchor Cell to a live end, with the Cursor on one of its Cells. The Cursor stays one Cell, at the live end except after command `A` spans the whole Grid around it; the Region is what holds the extent.
_Avoid_: Selection, range, block, marquee

**Source View**:
The region of the Source's space the console shows, at the Source's own Cell size in points; Zoom changes how many physical pixels a point is, never that size. It shows no further than the Grid's edges: the Grid is bounded, so its presentation is too.
_Avoid_: Canvas, camera, viewport, document, scroll position

**Pan**:
Moving the Source View across the Source, as far as the Grid's edges and no further. An axis on which the whole Source already shows has nowhere to Pan. A Pan is made by wheel, middle-drag, Alt-drag, a finger's drag, or a press or drag on the Minimap.
_Avoid_: Scroll, drag

**Zoom**:
Enlarging or shrinking the whole console, Source Grid included, in fixed steps.
_Avoid_: Scale, pinch, fit

**Minimap**:
A reduced picture of the whole Grid, one mark per Cell in the Theme's colours, floating over the Source's bottom-right corner, above the bottom panel. It shows where the Source View is, and a press or drag on it Pans the Source View there. It carries no content, holds no Positions, and moves no Cursor. The outline it draws is the Source View, as the Minimap shows it.
_Avoid_: Overview, map, thumbnail, navigator, viewport, scrollbar

**Sector**:
One block of Cells the console draws the Source Grid divided into, bounded by Sector Seams: the Sector Seam spacing wide and high, and cut short at the Grid's right and bottom edges. Tab and Shift Tab step the Cursor along its row a Sector at a time. A Sector is console presentation and navigation: it carries no content and no language meaning, and no Expression, Function or Evaluator reads it.
_Avoid_: Block, tile, chunk, page

**Sector Seam**:
The solid line the console draws along every sector boundary of the Source Grid, at the configured interval in both axes. It is composited over the line each Cell edge it lies on shows, except the Cursor or Region outline's, and carries no content, occupies no Cell, and belongs to no Expression.
_Avoid_: Marker, guide, gridline, ruler dot

**Paint**:
The per-Cell decision of how the Positions a console draws of one Render Frame are drawn: their background, border, foreground, sector seams and the character shown, and the line each Cell edge shows. Two Cells share the edge between them, so a Paint answers one line for it: the highest-ranked fact either Cell carries on that side, or the Grid line where neither carries one. A Paint is derived from a Render Frame and the range of Positions the viewport reaches, and carries no geometry; where a Cell sits and how wide a line is drawn belong to the step that turns a Paint into what is shown. A Pending Operand is how Paint shows an operand slot none of whose Cells is written: in its declared Token's colour rather than as a fault (CONS-027). It is a fact about one slot, not a state of the Function.
_Avoid_: Shapes, draw list, painter

**Theme**:
The shared appearance of the Source Grid and the console around it: colours, opacity, Grid and Cell colours, borders and border widths. A Theme changes appearance while preserving Grid structure and input behaviour. One Theme styles the whole console at a time: the Source Grid and the console around it are never themed separately. The console's settings choose a Theme by name and hold none of its values: one Theme for dark appearance and one for light, and whether the console follows the operating system's appearance or holds one of the two. Every Theme is either dark or light, and only a dark Theme can be chosen for dark appearance. A custom Theme is authored and changed in a file outside Orcvs, and stands beside the built-in Themes as an equal; the console selects Themes and does not edit them. A Theme a viewer makes starts from exactly one built-in Theme and states only how it differs; it never starts from another Theme a viewer made. It shares its parent's dark or light appearance. A Theme decides how presentation looks, never how much it moves.
_Avoid_: Palette, Visuals, chrome palette, colour settings, override

**Panel**:
The console's performer-facing telemetry surface for one running Orcvs. It is static: it does not move.
_Avoid_: HUD, toolbar, dock, egui Panel

**Readout**:
One labelled value on the Panel.
_Avoid_: widget, field, metric, status item

**Info**:
The Panel's two lines beneath its Readouts describing what the Cursor's Cell belongs to: on the first, the Cells of each of its parts as the Source paints them, the Cursor's outline included; on the second, beneath each, the Function's name, a Role, or what its answer is; or the diagnostic that covers the Cell. It is shown or hidden from View, shown by default, and while shown it keeps its rows whether or not it has anything to say.
_Avoid_: Cursor Line, status line, tooltip, hint, inspector

**BPM**:
The performer's display of the Tick Grid's period, in beats per minute.
_Avoid_: speed, clock rate

**Toast**:
One standing failure, from MIDI, UDP/OSC or the playback engine, floating in the window's top-right corner beneath the menu bar until it fades after the configured seconds or is dismissed.
_Avoid_: notification, popup, alert, status message

## Site

**Linked Console**:
A console whose page address opens it on a particular Source, by naming an example, carrying Source text, or asking for the Small Console, whether or not that address could be read. A Linked Console is a view of what its link names rather than the performer's own work.
_Avoid_: Embedded console, shared console, guest mode

**Small Console**:
The Linked Console a site page frames beside its prose, which takes its Source from the page by message rather than from its address.
_Avoid_: Embed, mini console, widget

**Sound Module**:
The row of the site's page below the console that shows the Built-in Synth: sixteen channels that show in the Function role's colours while a note sounds, and the eight effects of the chosen channel, which follows the channel of each effect Control Change it hears until the reader chooses one by clicking it or from the keyboard, each a column of a two-Cell magnitude, a two-Cell value and a two-Cell name, set by that channel's Control Changes or by the reader typing into the values, which sends them on that channel (SITE-033 to SITE-038). It is drawn as Cells, as the site draws a Source.
_Avoid_: effects panel, mixer, control panel
