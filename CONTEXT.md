# Orca Visual Synthesizer

Orcvs is a grid-based environment for composing and executing compact musical expressions.

## Language

**Source**:
The rectangular grid that holds the current Orca program as Cells.
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

**Expression**:
A contiguous horizontal run of occupied Cells in one Source row that is parsed and evaluated as one Orca language expression. An Expression never wraps across rows.
_Avoid_: Formula, statement

**Tick**:
One playback step that evaluates every Expression from the same Source snapshot and commits all resulting Cell changes atomically. When results target the same Cell, the result from the later Expression in Source order wins; results outside the Source are discarded.
_Avoid_: Cycle, frame

**Tick Plan**:
The complete deterministic outcome of interpreting one Source snapshot for a Tick, including Source writes, ordered Play Commands, and diagnostics.
_Avoid_: Play sequence, command batch

**Playback**:
The time-driven process that requests a new Tick Plan for each Tick and dispatches its Play Commands. Playback does not parse Expressions or own Source interpretation.
_Avoid_: Player, sequencer

**Playback Engine**:
The module that owns Playback lifecycle and musical time and dispatches each Tick's ordered Play Commands exactly as supplied. It does not parse Source, interpret musical intent, or infer note lifetime; stopping Playback or disconnecting an output adapter triggers all-notes-off as a safety action.
_Avoid_: Runtime, audio engine, MIDI engine, sequencer

**Live Editing**:
Changing the Source while Playback continues. An edit affects the next Tick whose Source snapshot has not yet been taken.
_Avoid_: Hot reload, live coding

**Play Command**:
One interpreted MIDI Note On instruction emitted by a Play Function for delivery during a Tick. Play Commands are ordered within their Tick Plan and delivered to the Playback Engine as one list; velocity `00` explicitly stops a note using MIDI's zero-velocity convention.
_Avoid_: Performance command, MIDI event

**Play Function**:
The terminal `>>` language Function that interprets a hexadecimal MIDI channel, velocity, and note as one Play Command. It is valid only at the root of an Expression, not where another Function requires a value, and it never writes a Cell result.
_Avoid_: Note output, MIDI Function

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
