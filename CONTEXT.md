# Orca Visual Synthesizer

Orcvs is a grid-based environment for composing and executing compact musical expressions. This glossary names the evolving pre-release language defined by the ADRs; it does not claim that every term's complete behavior is implemented yet.

**Orcvs (running instance)**:
One active console session, owning its Source and the Grid that shapes it, its Cursor, its Playback lifecycle, and its presentation options. Choosing an output device is console configuration rather than part of a running Orcvs. The unqualified system name Orcvs still names the environment and language as a whole.
_Avoid_: App, Session, Instance, Machine

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
The column and row of one Cell of a Grid, spelled in Source as two Numbers, column then row, from `00 00` at the top-left. A Position can be obtained only from the Grid that contains it, so a pair of Numbers past its Grid's extent is not a Position; the Grid converts between a Position and the index the Source addresses Cells by.
_Avoid_: Coord, coordinate, point

**Cell**:
One position in the Source, containing exactly one printable single-byte ASCII character; a space represents an empty Cell.
_Avoid_: Character slot, text position

**Language Unit**:
One semantic value or operation recognized in a Source revision, such as a Function or Atom. A Language Unit has one anchor Position and a Span of one or more character Cells. Incomplete or invalid Source text does not form a Language Unit. A Comment and a written List Item are the Language Units that are neither a value nor an operation: each records a Token and no Atom, so it is established like any other and evaluated like none.
_Avoid_: Logical Cell, token Cell, glyph

**Span**:
The character Cells occupied by one Language Unit, Expression, or Diagnostic in a Source revision. A row is the whole horizontal extent there is, so a Span is a contiguous run within one row, named by its first and last Cell. Spatial behavior moves, replaces, or tests the complete Span while every Position remains a Position in the character Grid.
_Avoid_: Footprint, extent, range, Cell structure, semantic Grid, bounding box

**Language Map**:
The semantic view derived from the Parser's interpretation of one Source revision. It identifies Expressions, roots, typed operands, nested ownership, Language Units, anchor Positions and Spans without adding persistent program state or independently interpreting spellings.
_Avoid_: Overlay Grid, parsed Source state, semantic Source

**Atom**:
One parsed value or operation an Expression is made of: a Number, a Note, a Bang, a Function, or the absence marker. A Self-Banging Function is not a kind of its own here: it is a Function, and the four spellings parse to one. An Atom is what an Expression's operands resolve to and what a value Function answers. It is the parsed unit rather than the Cells that spell it, so the same two Source characters can be a Number in one operand position and a Note in another.
_Avoid_: Token, glyph, symbol, cell value

**Operand Literal**:
Two Source Cells interpreted as an Atom according to the typed operand position of the Function that consumes them. The characters have no Number or Note type outside that context, so a standalone operand literal is invalid.
_Avoid_: Typed Source Cell, intrinsically typed literal, contextual coercion

**Spatial Output**:
A Function output delivered through a Portal as literal Source encoding, interpreted in the receiving operand's context. A nested Function's Return is the same encoding and the receiving operand interprets it the same way, so identical characters mean the same thing however they arrived.
_Avoid_: Implicit numeric conversion, typed spatial argument

**Return**:
The two-Cell Source encoding a nested Function hands to the inline operand it stands in. Per ADR 0061 the receiving operand decodes it by its declared literal type, exactly as it decodes characters a Portal wrote there, so the child's Atom type does not cross: a Note `C5` returned into an Addition operand is the Number `C5`. A Return is an additional delivery rather than a redirection: the nested Function also writes the same encoding through its own Output Portal, so its feedback stays in Source and the row below a nested Expression traces every step. A refused Output Portal write does not withdraw the Return, and a parent's failure does not undo the child's write. A Blank Answer returns two blank Cells, and the parent gives the Blank Answer in turn. An answer with no Source encoding, the Absence Marker, returns nothing and the parent diagnoses. A Function that answers no value has no Return, so the Parser refuses it wherever a value is required, from Source alone.
_Avoid_: Typed nested result, nested value, pass-through

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
The evaluator that turns a Function and its typed operands into one value or terminal effect. Its Operand Stack exists only for that evaluation; activation, spatial delivery and Source remain outside the Evaluator.
_Avoid_: Virtual machine, VM, interpreter loop, runtime

**Operand Stack**:
The stack of operand values one Function is evaluated against. It holds exactly the operands that Function's signature declares, in signature order, and the Function pops them from it; neither the Function's answer nor an effect is ever pushed onto it. An operand list of any other length is refused before the stack exists, so its depth never exceeds the widest signature any Function declares. It is created for one evaluation and discarded when that Function answers, so nothing carries from one evaluation to the next or from one Tick to the next. ADR 0028 bounds its depth by the Function table and has a push past that bound answer a diagnostic rather than panic.
_Avoid_: Value stack, call stack, machine memory, register

**Function**:
A named Orcvs language operation evaluated within an Expression. A Function may adapt a capability found in Orca, but its syntax and behaviour follow Orcvs language rules rather than Orca compatibility. Per ADR 0028 every Function declares whether it answers a value the surrounding Expression can consume or performs an effect and answers nothing, never both and never neither, and per ADR 0029 that declaration is what the Parser's nesting refusal and the activation gate each read rather than a spelling or a family prefix. The declaration is a property of the definition, settled before any Tick runs, and is not the Effect a Producer contributes to a Tick Plan. Terminal Output is one effect a Function may declare rather than the definition of effect, so a rule about having no Cell destination reads that narrower declaration while a rule about nothing consuming the answer reads the wider one. Per ADR 0063 every value Function answers exactly one Atom, which is what lets a Tick reserve a result's Cells before any Function has evaluated, and no Function accepts or answers a series of values.
_Avoid_: Operator, command

**Source Function**:
A Function whose result may depend on Cells outside its explicit operands or may change Cells beyond its Output Portal. Its value reads wait for current-Tick suppliers to settle. Placement tests occupancy in working Source at its Turn without waiting for a vacancy; its writes pass through Portals (ADR 0060).
_Avoid_: Spatial operator, grid function

**Bang**:
A transient pulse Atom returned by a Function, whose output activates aligned neighboring roots during the same Tick. Alignment is measured from the result's own two-Cell Span: a horizontally aligned root anchors two columns away with its Span touching the result's, and a vertically aligned root shares the result's anchor column one row north or south. Its `**` spelling is a visual representation of that output, not an executable Function or a new activation on the following Tick. Manually entering `**` is a no-op. A `**` spelling rejected in a typed operand remains invalid syntax and cannot activate another root. A nested Function that answers Bang writes `**` through its own Output Portal and activates the roots aligned with that display, as a root does, and its Return is `**`.
_Avoid_: Boolean, trigger flag

**Directional Bang Function**:
One of the four Source Functions `*^`, `*v`, `*<`, and `*>`. Like every ordinary Function it is inert until Bang activation, and each activation emits a Source-resident Self-Banging Function into the two Cells immediately outside its own two-Cell Span in the selected direction. The emitted Function first receives a turn from the following Source Snapshot and thereafter activates itself. That asymmetry is why both forms exist: a Directional Bang Function acts once per Bang it receives, while the Self-Banging Function it writes acts every Tick without one.
_Avoid_: Always Function, movement Function, automatic mode

**Self-Banging Function**:
One of the root-only Source Functions `^^`, `vv`, `<<`, and `>>`, emitted by the matching Directional Bang Function. In its scheduled evaluation it intrinsically receives Bang activation without creating a Source-resident `**`, then advances its complete two-Cell Span by one Cell in its retained direction. A successful move atomically clears the current Span and writes the Function's own spelling at the shifted destination. A blocked or out-of-Grid move instead changes its current Span to `**`. Complete aligned contact with an Expression root establishes Bang activation for that root in the dependency schedule. Contact with only part of another Language Unit is an alignment diagnostic: it still blocks the move and produces `**`, but does not activate the partially contacted unit. A Self-Banging Function is not an operand or runtime value.
_Avoid_: Activation Character, Self-Activating Function, Arrow Function, moving Bang, projectile

**Halt Function**:
The Source Function `*!`. When active, it establishes a dependency that locks the Expression root directly south before that root can execute. Orcvs evaluates each Halt Function at most once per Tick, and a Halt Function suppressed by another Halt Function does not lock its own target.
_Avoid_: Stop Function, control phase, retroactive suppression

**Jump Function**:
One of the directional Address Functions `&^`, `&v`, `&<`, and `&>`. It is a Function that answers a value: it reads one aligned two-Cell Language Unit at the Portal opposite its output and writes that value through its output Portal. East and west displace two columns; north and south one row. Consecutive Jumps compose through those Portals; when one output covers the next Function, the first write suppresses it. Empty aligned input clears the two-Cell destination: the Jump gives the Blank Answer, and nested it returns blank. Partial or invalid input diagnoses and writes nothing. An ordinary output atomically overwrites its complete destination Span. A Bang output activates an Expression root without overwriting it, writes `**` into an empty destination, and diagnoses at an occupied non-root or out-of-Grid destination. Jump participates in the dependency schedule, and its Bang output can activate a root in the same Tick. A Jump does not transport part of a Language Unit.
_Avoid_: Jumper, Jymper

**List**:
Per ADR 0063, a horizontal series of two-Cell Items inside the claim of the Function that reads it, directly east of that Function's operands. Its length is the Function's last operand, a count the Parser reads as a literal Number before any Function evaluates, so a nested Function cannot supply it and a count of `00` claims nothing and is refused. The Parser claims one Item per counted position, so an Item spelling a Function, a Comment introducer or a Bang is data and never opens anything; a count the row cannot hold is refused rather than reaching past the claim. An Item is untyped: the Parser never decodes it, and the operand that receives a copy decodes it, so a malformed Item diagnoses where it is read. A blank Item is a rest. A List is Source, not a value: it never passes between Functions, and a performer edits it in place. Source Paint identifies each Item as part of its Function's claim.
_Avoid_: Sequence, array, pattern value, step table

**Track Function**:
The List Function `@t index count`, followed by its List. It answers the Item at `index % count` every Tick without Bang activation, so any index source, such as a Clock above the index, drives a List of any length. The count the Tick's Source Snapshot parsed fixes the claim, the wrap and the zero check for the whole Tick; a same-Tick write to the count takes effect when the next Tick parses it. Every Item of that claim is a current-Tick read, so each Item's producers take their Turns before Track, whichever side of it they stand in Grid order, and Track reads the index and the selected Item's characters from working Source after they settle; partial, competing, absent and failed writes leave Track the Cells that survive, and a cycle through any Item read stops Track and the Expressions that depend on it, never the rest of the Tick. The selected characters are copied whole through the Output Portal and, nested, returned to the parent, which decodes them as it would any written Cells. A blank Item gives the Blank Answer. A Track copying `**` writes characters and activates nothing in that Tick.
_Avoid_: Sequencer, Select, tracker editor, playhead

**Number**:
An unsigned byte interpreted from an Operand Literal as exactly two uppercase hexadecimal Cells from `00` through `FF` when a Function requires a Number. General arithmetic wraps within this byte range; narrower domains such as MIDI parameters enforce their limits at their own boundaries.
_Avoid_: Base-36 value, decimal literal, single-glyph number

**Note**:
A pitched Atom carrying one MIDI note value from `00` through `7F`. In a Note operand position, its two-Cell Operand Literal uses an uppercase pitch letter for a natural or lowercase pitch letter for a sharp followed by its octave character: `/` for the octave below `0`, then `0` through `9`, such as `C/`, `C4`, or `c4`. The same Source characters may denote a Number in a Number operand position.
_Avoid_: Number, note-shaped Number, intrinsically typed literal

**Numeric Conversion Function**:
One of the numeric-family Functions `.v` and `.^`, whose family prefix fixes the numeric domain and whose directional suffix identifies the result type. Their Source literal signatures are monomorphic: `.v` consumes a Note literal and returns its underlying Number, while `.^` consumes a Number literal from `00` through `7F` and returns the corresponding Note, diagnosing `80` through `FF`. Each reads its operand only as that declared literal type, whether the characters were typed, written by a Portal, or returned by a nested Function, so conversions compose in opposite directions — `.v.^3C` and `.^.vC4` — while `.v.vC4` and `.^.^3C` diagnose, and `.+.^3C01` reads the returned `C4` as Number `C4`. A conversion never passes a value of its result type through.
_Avoid_: Cast, implicit coercion, sticky Note

**Arithmetic Function**:
One of the nine numeric-family Functions Add `.+`, Subtract `.-`, Absolute Difference `.|`, Multiply `.x`, Divide `./`, Modulo `.%`, Minimum `.<`, Maximum `.>`, and Equality `.=`, whose shared family prefix fixes the numeric domain and whose suffix names the operation. Every member declares two Number operands and returns a value. Add, Subtract, and Multiply wrap within the byte range; Absolute Difference answers the distance between its operands rather than a wrapped Subtraction, which is why it is a Function of its own; Minimum and Maximum return one operand unchanged; and Divide and Modulo answer the truncated quotient and the remainder. A Note, or any other Atom type, reaching a Number operand position diagnoses rather than being coerced, so the Numeric Conversion Functions are the only crossing between the two numeric types. Two members answer something other than plain arithmetic: Divide and Modulo each diagnose a zero divisor, under a diagnostic naming which of the two the Source wrote, and produce no value; and Equality returns a Bang for equal operands and the Absence Marker for unequal ones.
_Avoid_: Operator, infix arithmetic, boolean comparison, saturating arithmetic

**Clock Function**:
The Tick-family Function `~.`, which answers the step its cycle is at as a Number. Both operands are Numbers: `rate` is how many Ticks one step lasts and `modulus` is how many steps the cycle holds, so the cycle is `rate * modulus` Ticks long and the answer counts `00` through `modulus - 1` and then begins again. It reads the absolute Tick of the Playback run as the explicit interpretation input ADR 0012 makes it, rather than counting its own activations, which is what makes the same Source Snapshot interpreted at the same Tick answer the same step. The division and the modulus are taken in an integer wider than a byte before the step becomes a Number, because a Playback run's Tick outgrows a byte in seconds while the step it answers never does. A zero rate or a zero modulus is a cycle with no length: it diagnoses under a message naming the Function and the operand rather than reusing Division's or Modulo's, and the rate is answered first because that is signature order.
_Avoid_: Counter, phase accumulator, tempo, bar position

**Delay Function**:
The Tick-family Function `~*`, which answers a Bang once per cycle and the Absence Marker on every other Tick. Its two Number operands are the rate and modulus the Clock Function takes, and it Bangs exactly when the absolute Tick is a multiple of `rate * modulus`, Tick `0` included: the first Tick of a Playback run is a Tick like any other, so a Delay fires as the run starts rather than one cycle into it. A modulus of `01` is therefore one Bang per `rate` Ticks and not one every Tick, which is what makes the two operands a rate and a step count rather than two spellings of one period. The cycle product is calculated in an integer wider than a byte because it is a length rather than a value the Source wrote — `~* 10 20` is a cycle of 512 Ticks, which a byte would fold to zero and then divide by. A zero rate or a zero modulus diagnoses exactly as it does for the Clock Function.
_Avoid_: Sleep, scheduled event, delay line, echo

**Euclidean Function**:
The Tick-family Function `~%`, which answers a Bang on the Ticks a Euclidean rhythm places an onset at and the Absence Marker on the rest. Both operands are Numbers: `hits` onsets distributed as evenly as whole numbers allow across a cycle of `steps` Ticks, so `~% 03 08` sounds the pattern `X..X..X.` without a pattern being written anywhere. ADR 0012 fixes the arithmetic exactly rather than leaving it to a distribution of choice, and the absolute Tick is reduced into the cycle before the phase offset is added: that is the same rhythm and it cannot overflow the Tick counter, and every term is evaluated in an integer wider than a byte. Validation asks for a positive step count before it compares hits to steps, so `~% 00 00` is a cycle with no positions rather than a pattern with no onsets and diagnoses instead of falling silent; more hits than steps diagnoses under a message naming both. With a positive step count, zero hits Bangs on no Tick and equal hits and steps Bang on every one, and neither is a case in the formula.
_Avoid_: Euclidean algorithm, pattern table, rhythm string, step sequencer

**Increment Function**:
The Tick-family Function `~+`, which answers `(previous + step) % modulus` as a Number. Both operands are Numbers: `step` is how far to advance and `modulus` is where the count wraps. Its Input Portal and Output Portal are the same site, one row south of the Function: the Number input reads that Portal in working Source at its Turn, including earlier same-Tick writes; empty Cells supply the initial Number `00`, and any other present Language Unit diagnoses. The addition and the modulus are taken in an integer wider than a byte before the answer becomes a Number, so `FF + 02` cannot wrap before the modulus is applied. A zero modulus diagnoses under a message naming the Function and the operand rather than reusing Division's or Modulo's.
_Avoid_: Counter, accumulator, hidden state, +=

**Interpolation Function**:
The Tick-family Function `~>`, which moves a Number toward a target by at most `rate` and never overshoots. Both operands are Numbers: `rate` is the farthest one Tick may travel and `target` is the value it is moving toward. Its Input Portal and Output Portal are the same site, one row south of the Function: the Number input reads that Portal in working Source at its Turn, including earlier same-Tick writes; empty Cells supply the initial Number `00`, and any other present Language Unit diagnoses. When below the target it steps up by `rate` or lands on the target; when above it steps down the same way; when equal it returns the target unchanged. Each distance is calculated only in the branch whose subtraction is non-negative, and the remaining step is taken in an integer wider than a byte. Rate `00` holds the current value.
_Avoid_: Lerp, tween, easing, filter, glide

**Random Function**:
The Tick-family Function `~?`, which answers a Number selected inclusively between its normalized bounds. All three operands are Numbers: `seed` is the explicit stream identity, `minimum` and `maximum` are the inclusive range, so reversed bounds describe the same range and equal bounds return that value. It derives each result from the explicit seed, the absolute Tick of the Playback run, and the Function's own Position, rather than from activation history or an OS generator: identical Source Snapshots interpreted at the same Tick therefore produce identical Tick Plans, Functions at different Positions have independent reproducible streams, and moving one intentionally changes its stream. Each result is the first `u64` of a fresh ChaCha8 stream seeded from those three facts, then mapped into the widened inclusive width so `00`–`FF` is 256 values rather than a wrapping 0. A Note at any operand diagnoses rather than converting.
_Avoid_: StdRng, entropy, activation count, dice, noise, non-deterministic random, hidden state

**Absence Marker**:
The Atom an Expression answers when it leaves no value. It displays as `_` but has no Source encoding of its own, and it is not a language value: no Function takes it as an operand, an Expression answering it plans no Cell write, and a nested Function answering it returns nothing to its parent. Equality returns it for an unequal comparison, and Delay `~*` and Euclidean `~%` return it on every Tick they do not Bang. It is not the Blank Answer: the destination keeps its characters, no root is activated, and a parent receiving it diagnoses the missing Return.
_Avoid_: Null, nil, empty value, void

**Blank Answer**:
What a value Function gives, per ADR 0062, when one of its inline operands is blank: two spaces, delivered like any other answer. An operand is blank when every Cell of it is empty, whether the Source was written that way, a Portal wrote spaces there, or it is a nested Function's blank Return. The Function does not evaluate and nothing diagnoses; it writes the two spaces through its Output Portal and, nested, returns them, so its parent gives the Blank Answer in turn. The write is what keeps a consumer fed through a Portal from reading a stale value: Timed Play whose note slot a blank answer cleared emits nothing rather than replaying its previous Note, and Increment or Interpolation, whose Output Portal is also its feedback input, restart from the initial `00`. A Jump whose aligned input is empty copies a blank and gives the Blank Answer too, and so does a Track that selects a blank Item: the rest clears its Output Portal and, nested, makes its parent answer blank and clear its own. A Terminal Output Function with a blank operand emits no Play Command. An operand only partly blank is malformed and diagnoses under the ordinary literal rules, and a Function that fails diagnoses rather than answering blank, so deliberate silence and a fault stay distinguishable. Blank operands still occupy their claim, because a claim is the arity and not the content.
_Avoid_: Empty value, null result, rest value, Absence Marker

**Atomic Function**:
A stateless value Function whose operands and result are single Atoms: the Arithmetic and Numeric Conversion Functions, Clock and Random. An operation evaluates once and returns one Atom; a type, domain, or evaluation failure diagnoses and returns nothing. Per ADR 0063 no Function accepts or answers a series of values, so none extends across one: a pattern is written into Source Cells, and a chord is several Terminal Output roots one Bang activates.
_Avoid_: Vectorised Function, Map Function, Arithmetic Function

**Portal**:
One Cell destination resolved during a Tick. A Function may read through a Portal input, write through a Portal output, or both at the same site. When a Function does not name another Position, that Portal is one row south of the Function's anchor. The site is declared by the Function and placed by its anchor, so it is known from any Source revision before a Tick runs; what is resolved during a Tick is what the Portal carries. It carries an Atom, or one destination in a Source Function's validated write bundle; it is neither a language value nor persistent state. Working Source at a Portal travels in [`FunctionInputs`] beside Playback Tick and anchor; cell operands remain on the Operand Stack.
_Avoid_: Port, address value, output coordinate, ordinary result

**Output Portal**:
The Portal through which a Function acts on the Source, as an offset from its anchor: one row south unless the Function names another, as each Jump names its own direction. Every Function has one. A Function that answers a value writes that answer there, whether it is a root or nested; Halt locks the root there instead; a Terminal Output Function writes nothing there, and a Source-writing Function's writes are its declared Source effect rather than an answer.
_Avoid_: Result Cell, output Cell, destination

**Input Portal**:
The Portal a Function declares it reads a Source input through, as an offset from its anchor. A Jump reads at the Portal opposite its Output Portal; Increment and Interpolation read at the same site as their Output Portal, which is how their feedback stays in Source, nested or not. Most Functions declare none and take every input as an operand.
_Avoid_: Input Cell, source Cell, feedback register

**Comment**:
The Language Unit the Parser establishes at the two-Cell introducer `||`, claiming every remaining Cell of its row. It records a Token and no Atom, which excludes it from evaluation: it answers no value, performs no effect, and is never scheduled. `||` opens a Comment only where a new Expression could start; inside a Function's arity-determined claim it is an operand Cell that fails to bind and diagnoses. One `|` alone is incomplete or invalid Source rather than a Comment.
_Avoid_: Comment Function, halted Expression

**Tick**:
One discrete musical-time step that interprets a Source Snapshot and atomically applies its Tick Plan. An error never stops a Tick: it costs the Expressions it reaches, and every other Expression publishes. A same-Tick dependency cycle stops the Expressions on it and those that depend on them, each diagnosed.
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
The Cells one computation's result may reach, settled while a Tick is scheduled and before any Function has evaluated. Per ADR 0063 every value Function answers one Atom, or for Track one Item's two Cells, so every value Function, root or nested, reserves one Atom's Cell pair at each of its write Portals, and a Source-writing Function reserves the Spans its declared bundle names. A Reservation orders Turns and decides nothing else: the write a Portal admits is what decides whether a Cell was written, a computation suppressed, or a root activated.
_Avoid_: Footprint, allocation, reserved Span, write window

**Effect**:
One thing a Producer contributes to the Tick Plan: a Cell write, an activation delivery, a root lock, a diagnostic, or the Play Command one terminal root performs. Effects follow execution order — ADR 0020's dependency order, with Position breaking ties — and then the order their Producer emits them, so a chord of several terminal roots one Bang activates plays its notes in that order. Cell writes validate their whole destination before any Cell of them is emitted, and resolve Cell-wise, so a later Effect wins each Cell it overlaps and leaves the rest of an earlier write standing.
_Avoid_: Action, mutation, command

**Function Replacement**:
A Cell write whose value is a Function Atom landing on the anchor Position of a computation already running in the same Tick, replacing which Function stands there for the remainder of that Tick. It is refused unless the incoming Function agrees with the running one on every fact the settled schedule was derived from: whether it answers a value, where its activation comes from, whether it can emit Bang, the Source write it declares, and whether it reads a List. Those five are held fixed because ADR 0032 fixes the schedule before any Function evaluates, so a replacement differing on any of them would leave Turns ordered from edges that no longer describe what runs, or a write landing at Cells no dependency edge names. Every value Function reserves the same one Atom's Cell pair, so the width of a result is not among them. The comparison is against the Function the computation is running rather than the one the Parser found, which tells a second replacement at one anchor from the first. A refusal names which of the five differed. No Function answers a Function Atom today — ADR 0034 defers the Source operation that would — so the rule is stated and held ahead of the language that reaches it.
_Avoid_: Overwrite, mutation, hot swap, redefinition

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

**Omitted Diagnostics**:
The per-class count of Playback diagnostics recorded since the last drain and not retained. Per ADR 0055 a Playback Engine retains a bounded number of diagnostics between drains, evicting Overruns first and clock failures last, and a drain that omitted anything ends with one summary of these counts. A count saturates rather than wrapping and is then a lower bound.
_Avoid_: Dropped diagnostics, lost events

**Live Editing**:
Changing the Source while Playback continues. An edit affects the next Tick whose Source snapshot has not yet been taken. Per ADR 0057 a Tick commits only against the revision it planned from, so an edit made while a Tick plans is never overwritten: that Tick takes its snapshot again, at the same absolute Tick, and publishes nothing from the plan it abandoned.
_Avoid_: Hot reload, live coding

**Play Command**:
One interpreted MIDI instruction emitted by an active Terminal Output Function for delivery during a Tick. It is an explicit variant per spelling — a Raw Play note, a Timed Play note, a Monophonic Play note, a Control Change, and a Pitch Bend — carrying values already validated against their MIDI domains rather than assembled wire bytes, so the output adapter alone knows the protocol encoding. Every field is a type minted for the role it plays, so two operands that share a domain — a Control Change's controller and value, a Pitch Bend's LSB and MSB — are not assignable to one another. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list; velocity `00` explicitly stops a note using MIDI's zero-velocity convention.
_Avoid_: Performance command, MIDI event

**Output Command**:
One MIDI message the Playback Engine hands an output adapter. A Play Command says what the Source asked for; an Output Command says what is delivered, and the two differ wherever the Playback Engine owns the difference: a Timed or Monophonic Play's lifetime becomes a Note On in its Tick Plan order and a Note Off at the beginning of Tick `T + length`, and a Monophonic Play's replacement of the voice its channel was sounding becomes a Note Off of the note it found there, while a command with nothing to resolve is carried through unchanged. Control Change and Pitch Bend are the commands with nothing to resolve: neither carries a lifetime and neither owns a voice, so each has a variant of its own and reaches the adapter exactly as the Source wrote it. Every Output Command is one message the adapter assembles immediately, so an adapter is never handed a lifetime it would have to schedule.
_Avoid_: Wire command, output event, delivered Play Command

**Terminal Output Function**:
The family of `!`-spelled Functions that perform an effect and answer with no language value: Raw Play `!>`, Timed Play `!~`, Monophonic Play `!%`, Control Change `!c`, Pitch Bend `!b`, and in turn Application Command `!$`. Every member performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result. Activation gates evaluation itself, so an inactive terminal root reports no evaluation-time diagnostic either: ADR 0016's operand diagnostics are outcomes of evaluation, and an Expression that never evaluates has no outcome to report. Lexical and syntax diagnostics are unaffected and still fire regardless of activation. Each MIDI member emits a Play Command carrying operands already validated against their MIDI domains, so the output adapter alone assembles the wire message. Each Expression performs exactly one Play Command, and a domain or type fault diagnoses and emits no MIDI output. An activated root with a blank operand emits nothing and diagnoses nothing, per the Blank Answer. A chord is several Terminal Output roots that one Bang activates.
_Avoid_: Effect Function, side-effecting Function, output verb, action Function

**Play Function**:
The terminal `!> channel velocity note` Function, also called Raw Play, that interprets a hexadecimal Number channel `00`–`0F`, a hexadecimal Number velocity `00`–`7F`, and a Note as one raw Play Command. It performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result.
_Avoid_: Note output, MIDI Function

**Timed Play Function**:
The terminal `!~ channel velocity note length` Function. Channel is a Number `00`–`0F`, velocity is a Number `00`–`7F`, note is a Note, and length is a Number `00`–`FF`. Velocity `00` explicitly stops the specified note and schedules no expiry. Otherwise, length `00` emits no MIDI output, while a positive length starts the note in the current Tick and schedules Note Off at the beginning of Tick `T + length`. Its fixed arity distinguishes it from Raw Play without optional operands or overloading. Several Timed Play roots one Bang activates sound a chord, each scheduling its own Note Off at its own length, because the Playback Engine keys ownership by channel and note.
_Avoid_: Play overload, optional-length Play

**Control Change Function**:
The terminal `!c channel controller value` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and controller and value `00`–`7F`, and sends them without scaling. Controller and value share that domain and are separate types regardless, one per role, so nothing that reads the command can put one where the other belongs; what remains, a transposition of the operand declaration itself, is held by a test whose three operand values all differ. Invalid operands diagnose and emit no MIDI output, naming the role the Source wrote out of range rather than the domain the two roles share.
_Avoid_: CC scaling, normalized controller value, optional value

**Pitch Bend Function**:
The terminal `!b channel lsb msb` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and each data byte `00`–`7F`, and sends the MIDI wire bytes without scaling. LSB precedes MSB on the wire, and the two halves are separate types for the reason a Control Change's controller and value are: a bend is never assembled into one fourteen-bit value anywhere in Orcvs, so nothing has to take it apart again. Invalid operands diagnose and emit no MIDI output.
_Avoid_: PB scaling, normalized bend value, combined integer bend

**Monophonic Play Function**:
The terminal `!% channel velocity note length` Function with the same operand domains as Timed Play. The Playback Engine owns one Mono voice per MIDI channel, in the schedule that holds its Timed claims and cleared by the same lifecycle actions; ADR 0016 settles that seam, because an Output Command is one message carrying no lifetime and leaves an output adapter nothing to schedule with. The two ownerships differ only in their key — Timed Play's channel and note against Monophonic Play's channel alone — so neither can name the other's voice and each expires on its own claim. That independence is the engine's, not the wire's: a Note Off carries only channel and note, so a Mono note and a Timed note at the same pitch on one channel alias at the receiver, where the first stop delivered silences whichever is sounding. This is the aliasing ADR 0016 already leaves open between Mono and Raw notes rather than a separate one. Every command stops the prior Mono-owned note on that channel first, whatever note that was: the voice is the channel, and the Source need not remember what it last put there. Velocity `00` or length `00` then starts nothing, replacing the voice with silence, which is where it parts from Timed Play's length `00` no-op — a Timed command claims the note it names, so a note that never starts claims nothing, while a Monophonic command claims its channel whether or not it sounds. Otherwise, a positive length starts the replacement and schedules its Note Off at Tick `T + length`; a later replacement retires that expiry through its generation token. Raw Play and Timed Play notes do not enter Mono ownership.
_Avoid_: Global Mono voice, Play-wide voice stealing, implicit channel sharing

**Application Command Function**:
The terminal `!$` Function that sends a command to the Orcvs host application. It can invoke only commands that the application explicitly provides; it never invokes an operating-system shell or arbitrary executable. Its command value encoding is deferred until Orcvs defines a suitable text or message value.
_Avoid_: Host Command, shell command, process execution

**Cursor**:
The one Cell the console is editing: a Position drawn with the active Cursor Effect. The Cursor holds no dimensions and does no clamping of its own — the Grid answers where a move lands.
_Avoid_: Caret, pointer, insertion point

**Region**:
A rectangle of Positions within the Grid, spanned from an anchor Cell to a live end, with the Cursor on one of its Cells. The Cursor stays one Cell, at the live end except after command `A` spans the whole Grid around it; the Region is what holds the extent.
_Avoid_: Selection, range, block, marquee

**Cursor Effect**:
The console presentation surrounding the Cursor: an eroded frame on its Cell and a faint animated Area extending through continuous space around it. It changes no Cell content or Token, and its presentation time is independent of Playback and Source revisions.
_Avoid_: Cell highlight, radial glow, focus matrix

**Sector**:
One square block of Cells the console draws the Source Grid divided into, bounded by Sector Seams: the Sector Seam spacing wide and high, and cut short at the Grid's right and bottom edges. Tab and Shift Tab step the Cursor along its row a Sector at a time. A Sector is console presentation and navigation: it carries no content and no language meaning, and no Expression, Function or Evaluator reads it.
_Avoid_: Block, tile, chunk, page

**Sector Seam**:
The graded registration mark the console draws along every sector boundary of the Source Grid, at the configured interval in both axes. It is geometry drawn over Cell edges and carries no content, occupies no Cell, and belongs to no Expression.
_Avoid_: Marker, guide, gridline, ruler dot

**Render Frame**:
One repaint of the console, in which every Position the Grid yields is drawn once. Render Frames are driven by the UI many times a second, independently of musical time: a Render Frame reads the Source and never advances Playback, so it is not a Tick.
_Avoid_: Frame, Tick, refresh

**Paint**:
The per-Cell decision of how the Positions a console draws of one Render Frame are drawn: their background, border, foreground, sector seams and the character shown. A Paint is derived from a Render Frame and the range of Positions the viewport reaches, and carries no geometry; where a Cell sits and how wide a line is drawn belong to the step that turns a Paint into what is shown.
_Avoid_: Shapes, draw list, painter

**Theme**:
The shared appearance of the Source Grid and the console around it: colours, opacity, Grid and Cell colours, borders and border widths. A Theme changes appearance while preserving Grid structure and input behaviour. Typography and layout changes are outside the current Theme scope. One Theme styles the whole console at a time: the Source Grid and the console around it are never themed separately. The console's settings choose a Theme by name and hold none of its values: one Theme for dark appearance and one for light, and whether the console follows the operating system's appearance or holds one of the two. Every Theme is either dark or light, and only a dark Theme can be chosen for dark appearance. A custom Theme is authored and changed in a file outside Orcvs, and stands beside the built-in Themes as an equal; the console selects Themes and does not edit them. A Theme a viewer makes starts from exactly one built-in Theme and states only how it differs; it never starts from another Theme a viewer made. It shares its parent's dark or light appearance. A Theme decides how presentation looks, never how much it moves: the Cursor Effect's colours are the Theme's, and how much it glitches is the viewer's setting.
_Avoid_: Palette, Visuals, chrome palette, colour settings, override

**Source View**:
The region of the Source's space the console shows, at the Source's own Cell size in points; Zoom changes how many physical pixels a point is, never that size. It shows no further than a margin of two Cells past the Grid's edges: the Grid is bounded, so its presentation is too, and the margin holds no Positions.
_Avoid_: Canvas, camera, viewport, document, scroll position

**Pan**:
Moving the Source View across the Source, as far as the margin past the Grid's edges and no further. An axis on which the whole Source already shows has nowhere to Pan.
_Avoid_: Scroll, drag

**Zoom**:
egui's whole-UI zoom: enlarging or shrinking the whole console, Source Grid included, in egui's steps.
_Avoid_: Scale, pinch, fit

**Panel**:
The console's performer-facing telemetry surface for one running Orcvs. It is static: it does not move.
_Avoid_: HUD, toolbar, dock, egui Panel

**Readout**:
One labelled value on the Panel.
_Avoid_: widget, field, metric, status item

**BPM**:
The performer's display of the Tick Grid's period, in beats per minute.
_Avoid_: speed, clock rate

**Run Clock**:
The wall-clock duration of the current Playback run, measured from Tick 0 of that run and held while Playback is stopped.
_Avoid_: frame time, elapsed, timer
