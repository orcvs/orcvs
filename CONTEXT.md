# Orca Visual Synthesizer

Orcvs is a grid-based environment for composing and executing compact musical expressions. This glossary names the evolving pre-release language defined by the ADRs; it does not claim that every term's complete behavior is implemented yet.

**Orcvs (running instance)**:
One active console session, owning its Source and the Grid that shapes it, its Cursor, its Playback lifecycle, and its presentation options. Choosing an output device is console configuration rather than part of a running Orcvs. The unqualified system name Orcvs still names the environment and language as a whole.
_Avoid_: App, Session, Instance, Machine

## Language

**Source**:
The rectangular grid that holds the current Orcvs program as Cells.
_Avoid_: Document, buffer

**Grid**:
The fixed rectangular shape a Source occupies: its column and row counts, and the valid positions within them. The Grid is the shape; the Source is the contents. A Grid has at least one column and one row, and a position outside it does not exist.
_Avoid_: Canvas, matrix, bounds

**Position**:
The column and row of one Cell of a Grid. A Position can be obtained only from the Grid that contains it, so a Position outside its Grid does not exist; the Grid converts between a Position and the index the Source addresses Cells by.
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

**Atom**:
One parsed value or operation an Expression is made of: a Number, a Note, a Char, a Bang, a Self-Banging Function, a Function, or the absence marker. An Atom is what the Evaluator walks and what a Sequence holds. It is the parsed unit rather than the Cells that spell it, so the same two Source characters can be a Number in one operand position and a Note in another.
_Avoid_: Token, glyph, symbol, cell value

**Operand Literal**:
Two Source Cells interpreted as an Atom according to the typed operand position of the Function that consumes them. The characters have no Number or Note type outside that context, so a standalone operand literal is invalid.
_Avoid_: Typed Source Cell, intrinsically typed literal, contextual coercion

**Spatial Output**:
A Function output delivered through a Portal as literal Source encoding, interpreted in the receiving operand's context. It is distinct from a nested Function result, which retains its value's type.
_Avoid_: Implicit numeric conversion, typed spatial argument

**Pending Operand Encoding**:
The current characters of a spatially updated operand, awaiting interpretation when its receiving Function executes. They may be invalid for that operand's literal type and are not yet a decoded value.
_Avoid_: Incorrect typed value, inferred Number, failed decode

**Source Snapshot**:
The complete Source at the beginning of a Tick, including the accumulated output of preceding Ticks. It determines initial language state; dependencies supply current-Tick values before their consumers evaluate. Prior Bang display is not a new activation.
_Avoid_: Program state, runtime state

**Expression**:
A contiguous horizontal group of Cells established by parsing one Function and its operands, one standalone Bang or Self-Banging Function, or one Comment. Operand Cells may be empty or invalid; nested Functions extend the containing Expression, which never crosses a row edge. A Comment is the one Expression whose extent comes from its claim on the rest of its row rather than from an arity; an arity-determined claim reaches straight over a `||` inside it, which is then an operand Cell that fails to bind.
_Avoid_: Formula, statement

**Evaluator**:
The evaluator that turns a Function and its typed operands, or a complete Expression's Atoms, into one value or terminal effect. Its Operand Stack exists only for that evaluation; activation, spatial delivery and Source remain outside the Evaluator.
_Avoid_: Virtual machine, VM, interpreter loop, runtime

**Operand Stack**:
The stack of values one Expression is evaluated against. A literal Atom pushes one value onto it, and a Function pops the operands its signature declares and pushes one value in their place; an effect is never pushed onto it. It is created for one Expression and discarded when that Expression answers, so nothing carries from one Expression to the next or from one Tick to the next. ADR 0028 requires its depth to be proven sufficient for every Expression the parser accepts, or exhausting it to answer a diagnostic.
_Avoid_: Value stack, call stack, machine memory, register

**Function**:
A named Orcvs language operation evaluated within an Expression. A Function may adapt a capability found in Orca, but its syntax and behaviour follow Orcvs language rules rather than Orca compatibility. Per ADR 0028 every Function declares whether it answers a value the surrounding Expression can consume or performs an effect and answers nothing, never both and never neither, and per ADR 0029 that declaration is what the nesting guard, the activation gate, and Sequence membership each read rather than a spelling or a family prefix. The declaration is a property of the definition, settled before any Tick runs, and is not the Effect a Producer contributes to a Tick Plan. Terminal Output is one effect a Function may declare rather than the definition of effect, so a rule about having no Cell destination reads that narrower declaration while a rule about nothing consuming the answer reads the wider one. Per ADR 0036 a Function separately declares how wide its answer is — one Atom whatever its operands carry, as wide as its operands, or a Sequence whatever they carry — and that declaration is what lets a Tick reserve a result's Cells before any Function has evaluated. It is not the same declaration as pervasion: Equality extends over a Sequence operand and still answers one Atom.
_Avoid_: Operator, command

**Source Function**:
A Function whose result may depend on Cells outside its explicit operands or may change Cells beyond the ordinary result position. Its reads observe current-Tick values supplied by its scheduled dependencies, and its writes pass through Portals.
_Avoid_: Spatial operator, grid function

**Bang**:
A transient pulse Atom returned by a Function, whose output activates aligned neighboring roots during the same Tick. Its `**` spelling is a visual representation of that output, not an executable Function or a new activation on the following Tick. Manually entering `**` is a no-op. A `**` spelling rejected in a typed operand remains invalid syntax and cannot activate another root.
_Avoid_: Boolean, trigger flag

**Directional Bang Function**:
One of the four Source Functions `*^`, `*v`, `*<`, and `*>`. Like every ordinary Function it is inert until Bang activation, and each activation emits a Source-resident Self-Banging Function into the two Cells immediately outside its own two-Cell Span in the selected direction. The emitted Function first receives a turn from the following Source Snapshot and thereafter activates itself. That asymmetry is why both forms exist: a Directional Bang Function acts once per Bang it receives, while the Self-Banging Function it writes acts every Tick without one.
_Avoid_: Always Function, movement Function, automatic mode

**Self-Banging Function**:
One of the root-only Source Functions `^^`, `vv`, `<<`, and `>>`, emitted by the matching Directional Bang Function. In its scheduled evaluation it intrinsically receives Bang activation without creating a Source-resident `**`, then advances its complete two-Cell Span by one Cell in its retained direction. A successful move atomically clears the current Span and writes the Function's own spelling at the shifted destination. A blocked or out-of-Grid move instead changes its current Span to `**`. Complete aligned contact with an Expression root establishes Bang activation for that root in the dependency schedule. Contact with only part of another Language Unit is an alignment diagnostic: it still blocks the move and produces `**`, but does not activate the partially contacted unit. A Self-Banging Function is not an operand, runtime value, or Sequence member.
_Avoid_: Activation Character, Self-Activating Function, Arrow Function, moving Bang, projectile

**Halt Function**:
The Source Function `*!`. When active, it establishes a dependency that locks the Expression root directly south before that root can execute. Orcvs evaluates each Halt Function at most once per Tick, and a Halt Function suppressed by another Halt Function does not lock its own target.
_Avoid_: Stop Function, control phase, retroactive suppression

**Jump Function**:
One of the directional Address Functions `&^`, `&v`, `&<`, and `&>`. It copies exactly one aligned two-Cell Language Unit from the side opposite its direction to the far side of a consecutive chain with the same spelling. The chain head is the member adjacent to the input; only the head relays, while later members produce no effect. Horizontal members have touching Spans with anchors two columns apart; vertical members share an anchor column on adjacent rows. A gap, misalignment, or different spelling ends the chain. Empty aligned input clears the two-Cell destination. Partial or invalid input diagnoses and writes nothing. An ordinary output atomically overwrites its complete destination Span. A Bang output activates an Expression root without overwriting it, writes `**` into an empty destination, and diagnoses at an occupied non-root or out-of-Grid destination. Jump participates in the dependency schedule, and its Bang output can activate a root in the same Tick. A Jump does not transport a Sequence or part of a Language Unit.
_Avoid_: Jumper, Jymper, Sequence transport

**Number**:
An unsigned byte interpreted from an Operand Literal as exactly two uppercase hexadecimal Cells from `00` through `FF` when a Function requires a Number. General arithmetic wraps within this byte range; narrower domains such as MIDI parameters enforce their limits at their own boundaries.
_Avoid_: Base-36 value, decimal literal, single-glyph number

**Note**:
A pitched Atom carrying one MIDI note value from `00` through `7F`. In a Note operand position, its two-Cell Operand Literal uses an uppercase pitch letter for a natural or lowercase pitch letter for a sharp followed by its octave character: `/` for the octave below `0`, then `0` through `9`, such as `C/`, `C4`, or `c4`. The same Source characters may denote a Number in a Number operand position.
_Avoid_: Number, note-shaped Number, intrinsically typed literal

**Numeric Conversion Function**:
One of the numeric-family Functions `.v` and `.^`, whose family prefix fixes the numeric domain and whose directional suffix identifies the result type. Their Source literal signatures are monomorphic: `.v` consumes a Note literal and returns its underlying Number, while `.^` consumes a Number literal from `00` through `7F` and returns the corresponding Note, diagnosing `80` through `FF`. During evaluation, either Function also accepts an already-typed value of its result type as an identity; this supports composition and pervasive Sequence extension without making an overlapping Operand Literal ambiguous. Each is an Atomic Function over its one operand, so it extends pervasively across a Sequence operand, element by element and in order, and a member it cannot convert diagnoses the complete conversion rather than returning a shorter Sequence.
_Avoid_: Cast, implicit coercion, sticky Note

**Arithmetic Function**:
One of the nine numeric-family Functions Add `.+`, Subtract `.-`, Absolute Difference `.|`, Multiply `.x`, Divide `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and Equality `.=`, whose shared family prefix fixes the numeric domain and whose suffix names the operation. Every member declares two Number operands and returns a value. Add, Subtract, and Multiply wrap within the byte range; Absolute Difference answers the distance between its operands rather than a wrapped Subtraction, which is why it is a Function of its own; Minimum and Maximum return one operand unchanged; and Divide and Modulo answer the truncated quotient and the remainder. A Note, or any other Atom type, reaching a Number operand position diagnoses rather than being coerced, so the Numeric Conversion Functions are the only crossing between the two numeric types. Two members answer something other than plain arithmetic: Divide and Modulo each diagnose a zero divisor, under a diagnostic naming which of the two the Source wrote, and produce no value; and Equality returns a Bang for equal operands and the Absence Marker for unequal ones. Each is an Atomic Function over its two operands, so it extends pervasively across a Sequence operand, with Equality extending to find its comparison pairs and still returning one whole-value answer.
_Avoid_: Operator, infix arithmetic, boolean comparison, saturating arithmetic

**Clock Function**:
The Tick-family Function `~.`, which answers the step its cycle is at as a Number. Both operands are Numbers: `rate` is how many Ticks one step lasts and `modulus` is how many steps the cycle holds, so the cycle is `rate * modulus` Ticks long and the answer counts `00` through `modulus - 1` and then begins again. It reads the absolute Tick of the Playback run as the explicit interpretation input ADR 0012 makes it, rather than counting its own activations, which is what makes the same Source Snapshot interpreted at the same Tick answer the same step. The division and the modulus are taken in an integer wider than a byte before the step becomes a Number, because a Playback run's Tick outgrows a byte in seconds while the step it answers never does. A zero rate or a zero modulus is a cycle with no length: it diagnoses under a message naming the Function and the operand rather than reusing Division's or Modulo's, and the rate is answered first because that is signature order. It is an Atomic Function, so a Sequence operand extends it element by element and it answers a Sequence of steps.
_Avoid_: Counter, phase accumulator, tempo, bar position

**Delay Function**:
The Tick-family Function `~*`, which answers a Bang once per cycle and the Absence Marker on every other Tick. Its two Number operands are the rate and modulus the Clock Function takes, and it Bangs exactly when the absolute Tick is a multiple of `rate * modulus`, Tick `0` included: the first Tick of a Playback run is a Tick like any other, so a Delay fires as the run starts rather than one cycle into it. A modulus of `01` is therefore one Bang per `rate` Ticks and not one every Tick, which is what makes the two operands a rate and a step count rather than two spellings of one period. The cycle product is calculated in an integer wider than a byte because it is a length rather than a value the Source wrote — `~* 10 20` is a cycle of 512 Ticks, which a byte would fold to zero and then divide by. A zero rate or a zero modulus diagnoses exactly as it does for the Clock Function. It is a scalar exception to pervasive extension under ADR 0036: a Sequence at either operand position diagnoses rather than widening the operation, because a Sequence has no member that could stand where an element did not Bang, and every reduction of the elements to one answer would settle what layering two rhythms on one Cell means before any Source can spell it.
_Avoid_: Sleep, scheduled event, delay line, echo

**Euclidean Function**:
The Tick-family Function `~%`, which answers a Bang on the Ticks a Euclidean rhythm places an onset at and the Absence Marker on the rest. Both operands are Numbers: `hits` onsets distributed as evenly as whole numbers allow across a cycle of `steps` Ticks, so `~% 03 08` sounds the pattern `X..X..X.` without a pattern being written anywhere. ADR 0012 fixes the arithmetic exactly rather than leaving it to a distribution of choice, and the absolute Tick is reduced into the cycle before the phase offset is added: that is the same rhythm and it cannot overflow the Tick counter, and every term is evaluated in an integer wider than a byte. Validation asks for a positive step count before it compares hits to steps, so `~% 00 00` is a cycle with no positions rather than a pattern with no onsets and diagnoses instead of falling silent; more hits than steps diagnoses under a message naming both. With a positive step count, zero hits Bangs on no Tick and equal hits and steps Bang on every one, and neither is a case in the formula. Like the Delay Function it is a scalar exception under ADR 0036 and refuses a Sequence at either operand position.
_Avoid_: Euclidean algorithm, pattern table, rhythm string, step sequencer

**Absence Marker**:
The Atom an Expression answers when it leaves no value. It displays as `_` but has no Source encoding of its own, and it is not a language value: no Function takes it as an operand, it is refused as a Sequence member, and an Expression answering it plans no Cell write. Equality returns it for an unequal comparison, which is one reason that predicate returns one whole-value answer rather than an element-wise one: a Sequence has no position that could hold it. Delay `~*` and Euclidean `~%` return it on every Tick they do not Bang, and it is for the same want of a position that ADR 0036 has them refuse a Sequence operand rather than answer one. The empty Sequence that ADR 0007 defines plans no Cell write either, but it is a value holding no Atoms rather than the absence of a value, so the two agree on effect and differ in kind and each is handled on its own where a result becomes Source.
_Avoid_: Null, nil, empty value, empty Sequence, void

**Sequence**:
A flat ordered sequence of Atoms produced and consumed as one language value. Its members are Atoms of any kind other than a Self-Banging Function, a Function that answers an effect rather than a value, or the Absence Marker. Per ADR 0025 membership is checked at the single point every Sequence is constructed through, and per ADR 0029 that check asks a Function's declared kind rather than admitting the Function family. All three refusals are in force: `FunctionKind` distinguishes value from effect, with Terminal Output carried as one effect kind rather than being the definition of effect, so each effect Function declared later is refused there by its own definition. Atomic Functions extend pervasively across compatible Sequences, while Sequence-specific Functions transform the sequence itself.
_Avoid_: Pattern, Cell batch, write list, string

**Atomic Function**:
A stateless Function whose operands and result are single Atoms and which therefore extends pervasively across a Sequence operand. ADR 0007 calls this pervasive extension and ADRs 0011 through 0013 call it broadcasting; both names are current and name the same behaviour. An operation whose operands are all Atoms evaluates once and returns an ordinary Atom; an Atom operand repeats across every element of a Sequence operand; equal-length Sequence operands pair element-wise in order; and two non-scalar operands of different lengths diagnose as incompatible, including an empty Sequence against a non-empty one, rather than being padded or resized to fit. An empty Sequence operand is a legitimate width of no elements, so the result is the empty Sequence, and a scalar operand beside it is still checked because it is part of the operation whether or not any element repeats it. A shape, type, or evaluation failure diagnoses the complete operation and returns no partial Sequence, whichever element raises it. Every Function declares whether it extends or stays scalar, so an exception is stated rather than inherited: Delay `~*` and Euclidean `~%` stay scalar under ADR 0036 and are the only two that do today, Increment `~+` and Interpolation `~>` will stay scalar under ADR 0012 because element identity across Ticks would need hidden state, and the Terminal Output Functions extend under ADR 0030 even though they answer an effect rather than a value, because ADR 0028 bounds the kind of answer an instruction gives and not how much of it. Pervasion is therefore a property each Function declares and not one its family confers. Equality `.=` extends to find its comparison pairs and still returns one scalar under ADR 0011, which defers element-wise match positions and other comparison aggregations to Functions of their own. That whole-value answer is Equality's own and not a rule for every Function answering a pulse: a comparison is intrinsically a question about the whole set, whereas Delay `~*` and Euclidean `~%` refuse a Sequence operand altogether under ADR 0036, because an element that does not Bang has nothing to put at its position and every reduction of the elements to one answer would fix a meaning for layered rhythms that could not be relaxed later.
_Avoid_: Vectorised Function, Map Function, Sequence Function, Arithmetic Function

**Range Function**:
One of the monomorphic Sequence Functions `:-` and `:#`. Number Range `:-` returns an inclusive, unit-step Sequence between two Numbers; Note Range `:#` returns an inclusive chromatic Sequence between two Notes. Bound order selects ascending or descending output, and equal bounds return a singleton.
_Avoid_: Sequence generator, interval, polymorphic Range, mixed range

**Reverse Function**:
The Sequence Function `:<`. It reverses Atom order while preserving each Atom's complete encoding and type. A singleton and an empty Sequence remain unchanged.
_Avoid_: Character reversal, encoding reversal

**Concatenate Function**:
The Sequence Function `:&`. It promotes each Atom operand to a singleton Sequence and returns the left operand's Atoms followed by the right operand's Atoms. Output is flat, preserves Atom types and encodings, and treats an empty Sequence as the identity.
_Avoid_: Join Function, nested Sequence, append mutation

**Select Function**:
The Sequence Function `:?`. It uses a zero-based Number index modulo the length of a non-empty Sequence and returns the selected Atom with its type and encoding preserved. An empty Sequence or non-Number index diagnoses.
_Avoid_: Track Function, subsequence, broadcast selection

**Replace Function**:
The Sequence Function `:=`. It uses a zero-based Number index modulo the length of a non-empty Sequence and returns a new same-length Sequence with that Atom replaced. The replacement is one Atom and may have a different type. The input Sequence remains unchanged.
_Avoid_: Push Function, Sequence replacement operand, mutation

**Portal**:
One Cell destination resolved during a Tick. It carries an ordinary Atom or intact Sequence result, or one destination in a Source Function's validated write bundle; it is neither a language value nor persistent state.
_Avoid_: Port, address value, output coordinate

**Comment**:
The Language Unit the Parser establishes at the two-Cell introducer `||`, claiming every remaining Cell of its row. It records a Token and no Atom, which excludes it from evaluation: it answers no value, performs no effect, and is never scheduled. `||` opens a Comment only where a new Expression could start; inside a Function's arity-determined claim it is an operand Cell that fails to bind and diagnoses. One `|` alone is incomplete or invalid Source rather than a Comment.
_Avoid_: Comment Function, halted Expression

**Tick**:
One discrete musical-time step that interprets a Source Snapshot and atomically applies its Tick Plan.
_Avoid_: Cycle, frame

**Tick Plan**:
The complete deterministic outcome of interpreting one Source Snapshot at a particular Tick, including activation routing, Source writes, ordered Play Commands, and diagnostics.
_Avoid_: Play sequence, command batch

**Producer**:
An original Function with an anchor Position and an opportunity to contribute ordered Effects during a Tick. Bang is a result a Producer can emit, not a Producer of its own.
_Avoid_: Emitter, actor, source operator

**Turn**:
One original Producer's opportunity to emit Effects after its current-Tick data and activation dependencies have settled. A Producer takes at most one turn; Position breaks ties between independent Producers.
_Avoid_: Pass, visit, step

**Reservation**:
The Cells one computation's result may reach, settled while a Tick is scheduled and before any Function has evaluated. Per ADR 0036 it is derived from what each Function declares — how wide an answer it gives, and whether a Sequence operand widens that answer — rather than from a width no value has yet, so a computation whose answer cannot be a Sequence reserves the Cell pair a scalar result occupies while one whose answer can reserves its destination through the end of that destination's row. A Reservation orders Turns and decides nothing else: the write a Portal admits is always within it and is what decides whether a Cell was written, a computation suppressed, or a root activated. A computation a Reservation covers but the admitted write stopped short of therefore takes its Turn after that Producer and is then left standing.
_Avoid_: Footprint, allocation, reserved Span, write window

**Effect**:
One thing a Producer contributes to the Tick Plan: a Cell write, an activation delivery, a root lock, a diagnostic, or the ordered group of Play Commands one terminal root performs. Effects follow execution order and then the order their Producer emits them. A broadcast Terminal Output Function contributes one Effect carrying many commands rather than many Effects, because per ADR 0030 every command of one Expression shares that Expression's Position and Position alone can no longer order them: element index orders the commands within the Effect, and ADR 0020's order between Producers — dependency order, with Position breaking ties — is unchanged. A fault at any element emits no command at all, because the group is answered only once every element has produced its command: a partly sounded chord is never constructed rather than being refused after the fact, which is a property of the one construction point and not of the type. Cell writes validate their whole destination before any Cell of them is emitted, and resolve Cell-wise, so a later Effect wins each Cell it overlaps and leaves the rest of an earlier write standing.
_Avoid_: Action, mutation, command

**Playback**:
The time-driven process that requests a new Tick Plan for each Tick and dispatches its Play Commands. Playback does not parse Expressions or own Source interpretation.
_Avoid_: Player, sequencer

**Playback Engine**:
The module that owns Playback lifecycle and musical time and dispatches each Tick's ordered Play Commands exactly as supplied. It does not parse Source or interpret musical intent; when a Timed or Monophonic Play Command explicitly supplies a lifetime, it schedules the corresponding Note Off. One schedule holds both, keyed per channel and note for Timed Play and per channel alone for Monophonic Play, each claim carrying a generation token, so a stop retired by a replacement or by an explicit stop cannot cut a later note short. Beginning a run, stopping, disconnecting, and changing destination each clear the schedule. The schedule records only what an output adapter accepted, so a refused submission leaves it standing and the stop is delivered again at the next executed Tick. Stopping Playback, disconnecting an output adapter, changing destination, and tearing down after a refused delivery each trigger the same safety action, which returns every one of the sixteen MIDI channels to silence and to its defaults: All Notes Off, then Reset All Controllers, then an explicit centred Pitch Bend, in that order and channel by channel. Notes alone are not enough, because a Control Change can latch state and a Pitch Bend can deflect the wheel, and neither ends when the Source that wrote it stops running. A channel whose message the destination refuses does not end the action: the first refusal is the failure reported and every remaining channel is still attempted. Nothing is sent to release the individual notes the Playback Engine knows it started; All Notes Off is what silences them, and stopping, disconnecting, and changing destination discard the schedule rather than replay it.
_Avoid_: Runtime, audio engine, MIDI engine, sequencer

**Tick Grid**:
The deadlines one Playback run's Ticks are due at: every whole multiple of the Tick period from the deadline that run began on. Per ADR 0037 the grid holds until the run ends or its tempo is retuned, so a stall costs the Ticks it covered and does not move the ones after them, and the run resumes at the first deadline still ahead of the instant its clock woke. Retuning begins a new grid without beginning a new run, running it from the deadline the last executed Tick was due at rather than from the moment the retune arrived or the moment that Tick was seen.
_Avoid_: Schedule, timeline, beat clock

**Overrun**:
One Tick declined because it was observed a whole Tick period or more past the deadline it was due at. Per ADR 0037 it names that deadline, consumes no absolute Tick, and reports once per stall rather than once per deadline the stall covered: the deadlines a stopped clock never reached are skipped rather than delivered late so they can be declined in turn. An Overrun is a Playback diagnostic and carries no user-facing message.
_Avoid_: Dropped frame, xrun, missed tick

**Live Editing**:
Changing the Source while Playback continues. An edit affects the next Tick whose Source snapshot has not yet been taken.
_Avoid_: Hot reload, live coding

**Play Command**:
One interpreted MIDI instruction emitted by an active Terminal Output Function for delivery during a Tick. It is an explicit variant per spelling — a Raw Play note, a Timed Play note, a Monophonic Play note, a Control Change, and a Pitch Bend — carrying values already validated against their MIDI domains rather than assembled wire bytes, so the output adapter alone knows the protocol encoding. Every field is a type minted for the role it plays, so two operands that share a domain — a Control Change's controller and value, a Pitch Bend's LSB and MSB — are not assignable to one another. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list; velocity `00` explicitly stops a note using MIDI's zero-velocity convention.
_Avoid_: Performance command, MIDI event

**Performance**:
The ordered group of Play Commands one Terminal Output Function Expression performs, and what the language publishes to Tick planning in place of a single command. It has two shapes rather than one: a scalar Expression answers exactly one command and no group of one, and a Sequence operand widens it into many, ordered by element index per ADR 0030. A group of no commands is legitimate rather than a fault, because an empty Sequence operand is a real width of no elements and performs no MIDI output. Many effects remain one kind of answer: ADR 0028 bounds what kind of answer an instruction gives and not how much of it, so an Expression performing three commands has still answered no value and is still root-only.
_Avoid_: Chord, command batch, Play Command list, Tick Plan

**Output Command**:
One MIDI message the Playback Engine hands an output adapter. A Play Command says what the Source asked for; an Output Command says what is delivered, and the two differ wherever the Playback Engine owns the difference: a Timed or Monophonic Play's lifetime becomes a Note On in its Tick Plan order and a Note Off at the beginning of Tick `T + length`, and a Monophonic Play's replacement of the voice its channel was sounding becomes a Note Off of the note it found there, while a command with nothing to resolve is carried through unchanged. Control Change and Pitch Bend are the commands with nothing to resolve: neither carries a lifetime and neither owns a voice, so each has a variant of its own and reaches the adapter exactly as the Source wrote it. Every Output Command is one message the adapter assembles immediately, so an adapter is never handed a lifetime it would have to schedule.
_Avoid_: Wire command, output event, delivered Play Command

**Terminal Output Function**:
The family of `!`-spelled Functions that perform an effect and answer with no language value: Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`, Control Change `!c`, Pitch Bend `!b`, and in turn Application Command `!$`. Every member performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result. Activation gates evaluation itself, so an inactive terminal root reports no evaluation-time diagnostic either: ADR 0016's operand diagnostics are outcomes of evaluation, and an Expression that never evaluates has no outcome to report. Lexical and syntax diagnostics are unaffected and still fire regardless of activation. Each MIDI member emits a Play Command carrying operands already validated against their MIDI domains, so the output adapter alone assembles the wire message. Per ADR 0030 a member declared pervasive extends over a Sequence operand under ADR 0007's rules, so one Expression answers an ordered group of Play Commands, ordered by element index, while still answering no value; a scalar Expression answers exactly one command and no group of one. A domain or type fault at any element diagnoses the complete operation and emits no MIDI output at all, because a partly sounded chord could not be told from a chord written that way.
_Avoid_: Effect Function, side-effecting Function, output verb, action Function

**Play Function**:
The terminal `!> channel velocity note` Function, also called Raw Play, that interprets a hexadecimal Number channel `00`–`0F`, a hexadecimal Number velocity `00`–`7F`, and a Note as one raw Play Command. It performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result. It extends pervasively over a Sequence operand per ADR 0030, so `!> 00 7F :#C4E4` sounds a chord from one Expression with no new spelling: a scalar operand repeats across every element, equal-length Sequence operands pair element-wise, incompatible non-scalar lengths diagnose, and a fault at any element sounds nothing at all.
_Avoid_: Note output, MIDI Function

**Timed Play Function**:
The terminal `!~ channel velocity note length` Function. Channel is a Number `00`–`0F`, velocity is a Number `00`–`7F`, note is a Note, and length is a Number `00`–`FF`. Velocity `00` explicitly stops the specified note and schedules no expiry. Otherwise, length `00` emits no MIDI output, while a positive length starts the note in the current Tick and schedules Note Off at the beginning of Tick `T + length`. Its fixed arity distinguishes it from Raw Play without optional operands or overloading. It extends pervasively over a Sequence operand per ADR 0030 and schedules one Note Off per element at that element's own length, which needs nothing new of the Playback Engine because ownership is already keyed by channel and note.
_Avoid_: Play overload, optional-length Play

**Control Change Function**:
The terminal `!c channel controller value` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and controller and value `00`–`7F`, and sends them without scaling. Controller and value share that domain and are separate types regardless, one per role, so nothing that reads the command can put one where the other belongs; what remains, a transposition of the operand declaration itself, is held by a test whose three operand values all differ. It extends pervasively over a Sequence operand per ADR 0030, so `!c 01 :-0102 40` sweeps a bank of controllers from one Expression and a Sequence in the value position sends one controller a series. Invalid operands diagnose and emit no MIDI output, naming the role the Source wrote out of range rather than the domain the two roles share, and a fault at any element sends nothing at all.
_Avoid_: CC scaling, normalized controller value, optional value

**Pitch Bend Function**:
The terminal `!b channel lsb msb` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and each data byte `00`–`7F`, and sends the MIDI wire bytes without scaling. LSB precedes MSB on the wire, and the two halves are separate types for the reason a Control Change's controller and value are: a bend is never assembled into one fourteen-bit value anywhere in Orcvs, so nothing has to take it apart again. It extends pervasively over a Sequence operand per ADR 0030, and because the halves stay two operands rather than one assembled number, a Sequence in the LSB position sweeps the fine half against a held MSB. Invalid operands diagnose and emit no MIDI output, and a fault at any element sends nothing at all.
_Avoid_: PB scaling, normalized bend value, combined integer bend

**Monophonic Play Function**:
The terminal `!% channel velocity note length` Function with the same operand domains as Timed Play. The Playback Engine owns one Mono voice per MIDI channel, in the schedule that holds its Timed claims and cleared by the same lifecycle actions; ADR 0016 settles that seam, because an Output Command is one message carrying no lifetime and leaves an output adapter nothing to schedule with. The two ownerships differ only in their key — Timed Play's channel and note against Monophonic Play's channel alone — so neither can name the other's voice and each expires on its own claim. That independence is the engine's, not the wire's: a Note Off carries only channel and note, so a Mono note and a Timed note at the same pitch on one channel alias at the receiver, where the first stop delivered silences whichever is sounding. This is the aliasing ADR 0016 already leaves open between Mono and Raw notes rather than a separate one. Every command stops the prior Mono-owned note on that channel first, whatever note that was: the voice is the channel, and the Source need not remember what it last put there. Velocity `00` or length `00` then starts nothing, replacing the voice with silence, which is where it parts from Timed Play's length `00` no-op — a Timed command claims the note it names, so a note that never starts claims nothing, while a Monophonic command claims its channel whether or not it sounds. Otherwise, a positive length starts the replacement and schedules its Note Off at Tick `T + length`; a later replacement retires that expiry through its generation token. Raw Play and Timed Play notes do not enter Mono ownership. It extends over a Sequence operand per ADR 0030 like the rest of the family, and a Sequence through it does not sound a chord: each element replaces the one before it on that channel, so the last element is the note left sounding. That is a degenerate case rather than a forbidden one and needs no separate diagnostic.
_Avoid_: Global Mono voice, Play-wide voice stealing, implicit channel sharing

**Application Command Function**:
The terminal `!$` Function that sends a command to the Orcvs host application. It can invoke only commands that the application explicitly provides; it never invokes an operating-system shell or arbitrary executable. Its command value encoding is deferred until Orcvs defines a suitable text or message value.
_Avoid_: Host Command, shell command, process execution

**Cursor**:
The one Cell the console is editing: a Position, plus the blink state that draws it. The Cursor holds no dimensions and does no clamping of its own — the Grid answers where a move lands.
_Avoid_: Caret, pointer, insertion point

**Glyph**:
The classification that decides how a Cell is painted: Function, Note, Number, Bang, Comment or Char for a Cell the Source has parsed in its Expression context, and Marker, Highlight or Space for a Cell it has not. Comment reaches every Cell of a Comment's claim, empty Cells included, and paints them without standing anything in: an empty operand Cell shows the spelling its signature declares, and a Comment declares none. A Glyph is derived from the Source and typed Function operands, never stored as Cell content.
_Avoid_: Style, token, syntax highlight

**Marker**:
A purely visual Glyph the console draws at every marker-spacing interval of Cells in both axes, so distance across the Source can be read by eye. A Marker carries no content and belongs to no Expression; it appears only on a Cell the Source gives no Glyph of its own.
_Avoid_: Guide, gridline, ruler dot

**Render Frame**:
One repaint of the console, in which every Position the Grid yields is drawn once. Render Frames are driven by the UI many times a second, independently of musical time: a Render Frame reads the Source and never advances Playback, so it is not a Tick.
_Avoid_: Frame, Tick, refresh
