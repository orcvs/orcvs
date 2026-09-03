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
One position in the Source, containing exactly one single-byte ASCII character; a space represents an empty Cell.
_Avoid_: Character slot, text position

**Language Unit**:
One semantic value or operation recognized in a Source revision, such as a Function or Atom. A Language Unit has one anchor Position and a Span of one or more character Cells. Incomplete or invalid Source text does not form a Language Unit.
_Avoid_: Logical Cell, token Cell, glyph

**Span**:
The character Cells occupied by one Language Unit, Expression, or Diagnostic in a Source revision. A row is the whole horizontal extent there is, so a Span is a contiguous run within one row, named by its first and last Cell. Spatial behavior moves, replaces, or tests the complete Span while every Position remains a Position in the character Grid.
_Avoid_: Footprint, extent, range, Cell structure, semantic Grid, bounding box

**Language Map**:
The semantic view derived from one Source revision. It identifies Expressions, roots, Language Units, their anchor Positions, and their Spans without adding stored program state or a second coordinate system. It partitions each row from left to right into non-overlapping complete Language Units: after recognizing a unit it resumes after that complete Span, and an unmatched character diagnoses without participating in an overlapping unit.
_Avoid_: Overlay Grid, parsed Source state, semantic Source

**Operand Literal**:
Two Source Cells interpreted as an Atom according to the typed operand position of the Function that consumes them. The characters have no Number or Note type outside that context, so a standalone operand literal is invalid.
_Avoid_: Typed Source Cell, intrinsically typed literal, contextual coercion

**Source Snapshot**:
The complete Source observed for one Tick. It is simultaneously an executable Orcvs program and the accumulated output of all preceding Ticks, so no persistent language state exists outside it.
_Avoid_: Program state, runtime state

**Expression**:
A contiguous horizontal run of occupied Cells in one Source row that is parsed as one Orcvs language expression. Its first Function is the root Function; activating that root evaluates every nested Function needed by the Expression. An Expression never wraps across rows.
_Avoid_: Formula, statement

**Function**:
A named Orcvs language operation evaluated within an Expression. A Function may adapt a capability found in Orca, but its syntax and behaviour follow Orcvs language rules rather than Orca compatibility.
_Avoid_: Operator, command

**Source Function**:
A Function whose result may depend on Cells outside its explicit operands or may change Cells beyond the ordinary result position. Its reads observe the current Source Snapshot. A Source-writing Function resolves one or more Portals, validates its complete effect bundle, then contributes the bundle's ordered writes to the Tick Plan.
_Avoid_: Spatial operator, grid function

**Bang**:
A one-Tick pulse Atom encoded as `**`, distinct from every Number and Function. For a Bang anchored at `(x, y)`, its aligned cardinal root anchors are north `(x, y-1)`, south `(x, y+1)`, west `(x-2, y)`, and east `(x+2, y)`; only complete Expression roots at those anchors activate when their Source-order turns occur. A Bang present in the Source Snapshot is removed by that Tick's atomic commit. A Bang generated during planning is written at the current commit, remains visible in the next Source Snapshot, and is removed by that next Tick's commit. Direct same-Tick delivery to a root is an activation event separate from the stored glyph; it does not overwrite the Function and applies only if the root's turn has not passed.
_Avoid_: Boolean, trigger flag

**Directional Bang Function**:
One of the four activation Functions `*^`, `*v`, `*<`, and `*>`. It emits a Source-resident Self-Banging Function into the two Cells immediately outside its own two-Cell Span in the selected direction. The emitted Function first receives a turn from the following Source Snapshot.
_Avoid_: Always Function, movement Function, automatic mode

**Self-Banging Function**:
One of the root-only Source Functions `^^`, `vv`, `<<`, and `>>`, emitted by the matching Directional Bang Function. At its Source-order turn it intrinsically receives Bang activation without creating a Source-resident `**`, then advances its complete two-Cell Span by one Cell in its retained direction. A successful move atomically clears the current Span and writes the Function's own spelling at the shifted destination. A blocked or out-of-Grid move instead changes its current Span to `**`. Complete aligned contact with an Expression root delivers Bang activation when that root's turn has not passed. Contact with only part of another Language Unit is an alignment diagnostic: it still blocks the move and produces `**`, but does not activate the partially contacted unit. A Self-Banging Function is not an operand, runtime value, or Sequence member.
_Avoid_: Activation Character, Self-Activating Function, Arrow Function, moving Bang, projectile

**Halt Function**:
The Activation Function `*!`. When active at its Source-order turn, it locks the Expression root directly south before that root's later turn. Orcvs does not revisit a Halt Function after its turn, and a Halt Function suppressed by another Halt Function does not lock its own target.
_Avoid_: Stop Function, control phase, retroactive suppression

**Jump Function**:
One of the directional Address Functions `&^`, `&v`, `&<`, and `&>`. It copies exactly one aligned two-Cell Language Unit from the side opposite its direction to the far side of a consecutive chain with the same spelling. The chain head is the member adjacent to the input; only the head relays, while later members produce no effect. Horizontal members have touching Spans with anchors two columns apart; vertical members share an anchor column on adjacent rows. A gap, misalignment, or different spelling ends the chain. Empty aligned input clears the two-Cell destination. Partial or invalid input diagnoses and writes nothing. An ordinary output atomically overwrites its complete destination Span. A Bang output activates an Expression root without overwriting it, writes `**` into an empty destination, and diagnoses at an occupied non-root or out-of-Grid destination. Jump reads only the Source Snapshot, but its Bang output can activate a later root in the same Tick. A Jump does not transport a Sequence or part of a Language Unit.
_Avoid_: Jumper, Jymper, Sequence transport

**Number**:
An unsigned byte interpreted from an Operand Literal as exactly two uppercase hexadecimal Cells from `00` through `FF` when a Function requires a Number. General arithmetic wraps within this byte range; narrower domains such as MIDI parameters enforce their limits at their own boundaries.
_Avoid_: Base-36 value, decimal literal, single-glyph number

**Note**:
A pitched Atom carrying one MIDI note value from `00` through `7F`. In a Note operand position, its two-Cell Operand Literal uses an uppercase pitch letter for a natural or lowercase pitch letter for a sharp followed by its octave character: `/` for the octave below `0`, then `0` through `9`, such as `C/`, `C4`, or `c4`. The same Source characters may denote a Number in a Number operand position.
_Avoid_: Number, note-shaped Number, intrinsically typed literal

**Numeric Conversion Function**:
One of the numeric-family Functions `.v` and `.^`, whose family prefix fixes the numeric domain and whose directional suffix identifies the result type. Their Source literal signatures are monomorphic: `.v` consumes a Note literal and returns its underlying Number, while `.^` consumes a Number literal from `00` through `7F` and returns the corresponding Note, diagnosing `80` through `FF`. During evaluation, either Function also accepts an already-typed value of its result type as an identity; this supports composition and atom-wise Sequence extension without making an overlapping Operand Literal ambiguous.
_Avoid_: Cast, implicit coercion, sticky Note

**Sequence**:
A flat ordered sequence of Atoms produced and consumed as one language value. Its members are Atoms of any kind other than a Self-Banging Function, which is a root-only Source effect rather than a value, and the empty result an Expression leaves when it produces no value, which has no Source encoding of its own. Per ADR 0025 membership is checked at the single point every Sequence is constructed through. Atomic Functions extend pervasively across compatible Sequences, while Sequence-specific Functions transform the sequence itself.
_Avoid_: Pattern, Cell batch, write list, string

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
One Cell destination resolved while interpreting a Source Snapshot. An ordinary result sends an Atom or intact Sequence through one Portal; a Source Function may use multiple Portals in one validated effect bundle, including clear operations that write spaces. A Portal is neither a language value, Source content, nor persistent state.
_Avoid_: Port, address value, output coordinate

**Comment**:
Source text beginning with the two-Cell introducer `##` and continuing to the end of its row, excluded from Expressions and evaluation. One `#` alone is incomplete or invalid Source rather than a Comment.
_Avoid_: Comment Function, halted Expression

**Tick**:
One discrete musical-time step that interprets a Source Snapshot and atomically applies its Tick Plan.
_Avoid_: Cycle, frame

**Tick Plan**:
The complete deterministic outcome of interpreting one Source Snapshot at a particular Tick, including activation routing, Source writes, ordered Play Commands, and diagnostics.
_Avoid_: Play sequence, command batch

**Producer**:
One Language Unit or Expression root that takes a turn when a Tick Plan is built. A Producer has one anchor Position and emits an ordered sequence of Effects. The Producers are the Expression root, the Source-resident Bang, the Self-Banging Function, the Jump chain head, and Halt; only units present in the Source Snapshot are Producers, so a planned write is never one in the Tick that plans it.
_Avoid_: Emitter, actor, source operator

**Turn**:
One Producer's opportunity to emit, taken at its anchor Position. Turns run in row-major anchor order across one Source Snapshot, and a Producer takes at most one; a Producer whose turn has passed is never revisited, which is what lets activation reach only a root still ahead of the current turn.
_Avoid_: Pass, visit, step

**Effect**:
One thing a Producer contributes to the Tick Plan: a Cell write, an activation delivery, a root lock, a diagnostic, or a terminal Play Command. Effects are ordered first by their Producer's Position and then by the order that Producer emits them. Cell writes validate their whole destination before any Cell of them is emitted, and resolve Cell-wise, so a later Effect wins each Cell it overlaps and leaves the rest of an earlier write standing.
_Avoid_: Action, mutation, command

**Playback**:
The time-driven process that requests a new Tick Plan for each Tick and dispatches its Play Commands. Playback does not parse Expressions or own Source interpretation.
_Avoid_: Player, sequencer

**Playback Engine**:
The module that owns Playback lifecycle and musical time and dispatches each Tick's ordered Play Commands exactly as supplied. It does not parse Source or interpret musical intent; when a Timed Play Command explicitly supplies a lifetime, it schedules the corresponding Note Off. Stopping Playback or disconnecting an output adapter triggers all-notes-off as a safety action.
_Avoid_: Runtime, audio engine, MIDI engine, sequencer

**Live Editing**:
Changing the Source while Playback continues. An edit affects the next Tick whose Source snapshot has not yet been taken.
_Avoid_: Hot reload, live coding

**Play Command**:
One interpreted MIDI instruction emitted by an active Terminal Output Function for delivery during a Tick. It is an explicit variant per spelling — a Raw Play note, and in turn the Timed and Monophonic Play, Control Change, and Pitch Bend outputs — carrying values already validated against their MIDI domains rather than assembled wire bytes, so the output adapter alone knows the protocol encoding. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list; velocity `00` explicitly stops a note using MIDI's zero-velocity convention.
_Avoid_: Performance command, MIDI event

**Terminal Output Function**:
The family of `!`-spelled Functions that perform an effect and answer with no language value: Raw Play `!>`, and in turn Timed Play `!~`, Monophonic Play `!%`, Control Change `!c`, Pitch Bend `!b`, and Application Command `!$`. Every member performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result. Activation gates evaluation itself, so an inactive terminal root reports no evaluation-time diagnostic either: ADR 0016's operand diagnostics are outcomes of evaluation, and an Expression that never evaluates has no outcome to report. Lexical and syntax diagnostics are unaffected and still fire regardless of activation. Each MIDI member emits a Play Command carrying operands already validated against their MIDI domains, so the output adapter alone assembles the wire message.
_Avoid_: Effect Function, side-effecting Function, output verb, action Function

**Play Function**:
The terminal `!> channel velocity note` Function, also called Raw Play, that interprets a hexadecimal Number channel `00`–`0F`, a hexadecimal Number velocity `00`–`7F`, and a Note as one raw Play Command. It performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result.
_Avoid_: Note output, MIDI Function

**Timed Play Function**:
The terminal `!~ channel velocity note length` Function. Channel is a Number `00`–`0F`, velocity is a Number `00`–`7F`, note is a Note, and length is a Number `00`–`FF`. Velocity `00` explicitly stops the specified note and schedules no expiry. Otherwise, length `00` emits no MIDI output, while a positive length starts the note in the current Tick and schedules Note Off at the beginning of Tick `T + length`. Its fixed arity distinguishes it from Raw Play without optional operands or overloading.
_Avoid_: Play overload, optional-length Play

**Control Change Function**:
The terminal `!c channel controller value` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and controller and value `00`–`7F`, and sends them without scaling. Invalid operands diagnose and emit no MIDI output.
_Avoid_: CC scaling, normalized controller value, optional value

**Pitch Bend Function**:
The terminal `!b channel lsb msb` Function. It accepts hexadecimal Number bytes, requires channel `00`–`0F` and each data byte `00`–`7F`, and sends the MIDI wire bytes without scaling. Invalid operands diagnose and emit no MIDI output.
_Avoid_: PB scaling, normalized bend value, combined integer bend

**Monophonic Play Function**:
The terminal `!% channel velocity note length` Function with the same operand domains as Timed Play. Each output adapter owns one Mono voice per MIDI channel. Every command stops the prior Mono-owned note on that channel first. Velocity `00` or length `00` then starts nothing, replacing the voice with silence. Otherwise, a positive length starts the replacement and schedules its Note Off at Tick `T + length`; a later replacement cancels that expiry. Raw Play and Timed Play notes do not enter Mono ownership.
_Avoid_: Global Mono voice, Play-wide voice stealing, implicit channel sharing

**Application Command Function**:
The terminal `!$` Function that sends a command to the Orcvs host application. It can invoke only commands that the application explicitly provides; it never invokes an operating-system shell or arbitrary executable. Its command value encoding is deferred until Orcvs defines a suitable text or message value.
_Avoid_: Host Command, shell command, process execution

**Cursor**:
The one Cell the console is editing: a Position, plus the blink state that draws it. The Cursor holds no dimensions and does no clamping of its own — the Grid answers where a move lands.
_Avoid_: Caret, pointer, insertion point

**Glyph**:
The classification that decides how a Cell is painted: Function, Note, Number or Char for a Cell the Source has parsed in its Expression context, and Marker, Highlight or Space for a Cell it has not. A Glyph is derived from the Source and typed Function operands, never stored as Cell content.
_Avoid_: Style, token, syntax highlight

**Marker**:
A purely visual Glyph the console draws at every marker-spacing interval of Cells in both axes, so distance across the Source can be read by eye. A Marker carries no content and belongs to no Expression; it appears only on a Cell the Source gives no Glyph of its own.
_Avoid_: Guide, gridline, ruler dot

**Render Frame**:
One repaint of the console, in which every Position the Grid yields is drawn once. Render Frames are driven by the UI many times a second, independently of musical time: a Render Frame reads the Source and never advances Playback, so it is not a Tick.
_Avoid_: Frame, Tick, refresh
