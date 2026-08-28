# Orca Visual Synthesizer

Orcvs is a grid-based environment for composing and executing compact musical expressions.

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
A Function whose result may depend on Cells outside its explicit operands or may change Cells beyond the ordinary result position. Its reads observe the current Source Snapshot and its changes become part of the Tick Plan.
_Avoid_: Spatial operator, grid function

**Bang**:
A one-Tick pulse Atom encoded as `**`, distinct from every Number and Function. A Bang participates in activation routing for exactly one Tick and is then removed by that Tick's atomic commit. When its Portal lands on the root Function of an Expression, it activates that complete Expression without overwriting the Function's Cells.
_Avoid_: Boolean, trigger flag

**Always Function**:
One of the four self-scheduled directional Functions `@^`, `@v`, `@<`, and `@>`. Every Tick, an Always Function sends activation through empty Cells in its direction to the first occupied Cell; that Cell must be an Expression's root Function or it blocks the path and diagnoses. Always activates only that first Expression and produces no value of its own. Every other root Expression, including a time or Bang-producing Function, is inert without a landed Bang or directional Always activation.
_Avoid_: Uppercase Function, Always wrapper, automatic mode

**Number**:
An unsigned byte encoded as exactly two uppercase hexadecimal Cells from `00` through `FF`. General arithmetic wraps within this byte range; narrower domains such as MIDI parameters enforce their limits at their own boundaries.
_Avoid_: Base-36 value, decimal literal, single-glyph number

**Pattern**:
A flat ordered sequence of Atoms produced and consumed as one language value. Atomic Functions extend pervasively across compatible Patterns, while Pattern-specific Functions transform the sequence itself.
_Avoid_: Cell batch, write list, string

**Portal**:
One destination resolved while interpreting a Source Snapshot, through which an Atom or Pattern enters the Tick Plan as Source writes. A Portal is neither Source content nor persistent language state.
_Avoid_: Port, address value, output coordinate

**Comment**:
Source text beginning with `#` and continuing to the end of its row, excluded from Expressions and evaluation.
_Avoid_: Comment Function, halted Expression

**Tick**:
One playback step that evaluates every Expression from the same Source snapshot and commits all resulting Cell changes atomically. When results target the same Cell, the result from the later Expression in Source order wins; results outside the Source are discarded.
_Avoid_: Cycle, frame

**Tick Plan**:
The complete deterministic outcome of interpreting one Source Snapshot at a particular Tick, including activation routing, Source writes, ordered Play Commands, and diagnostics. A Bang produced onto a root Function activates it within the same Tick Plan, and each root Expression evaluates at most once even when multiple Bangs or Always rays target it.
_Avoid_: Play sequence, command batch

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
One interpreted MIDI instruction emitted by a Play Function for delivery during a Tick. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list; velocity `00` explicitly stops a note using MIDI's zero-velocity convention.
_Avoid_: Performance command, MIDI event

**Play Function**:
The terminal `>>` Function that interprets a hexadecimal MIDI channel, velocity, and note as one raw Play Command. It performs only when its root is activated, is invalid where another Function requires a value, and never writes a Cell result.
_Avoid_: Note output, MIDI Function

**Timed Play Function**:
The terminal `>?` Function that interprets a hexadecimal MIDI channel, velocity, note, and Tick length as a Timed Play Command. Its fixed arity distinguishes it from raw Play without optional operands or overloading.
_Avoid_: Play overload, optional-length Play

**Cursor**:
The one Cell the console is editing: a Position, plus the blink state that draws it. The Cursor holds no dimensions and does no clamping of its own — the Grid answers where a move lands.
_Avoid_: Caret, pointer, insertion point

**Glyph**:
The classification that decides how a Cell is painted: Function, Note, Number or Char for a Cell the Source has parsed, and Marker, Highlight or Space for a Cell it has not. A Glyph is derived from the Source, never stored as Cell content.
_Avoid_: Style, token, syntax highlight

**Marker**:
A purely visual Glyph the console draws at every marker-spacing interval of Cells in both axes, so distance across the Source can be read by eye. A Marker carries no content and belongs to no Expression; it appears only on a Cell the Source gives no Glyph of its own.
_Avoid_: Guide, gridline, ruler dot

**Render Frame**:
One repaint of the console, in which every Position the Grid yields is drawn once. Render Frames are driven by the UI many times a second, independently of musical time: a Render Frame reads the Source and never advances Playback, so it is not a Tick.
_Avoid_: Frame, Tick, refresh
