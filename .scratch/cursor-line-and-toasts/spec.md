# Cursor Line and Toasts

Status: ready-for-agent

## The decision

Failures leave the Panel and float as Toasts in the window's top-right corner, and the Panel keeps
one permanent Cursor Line beneath its row that describes what the Cursor's Cell belongs to. Toby
picked this (layout G) on 2026-10-31, from layouts captured from the console on the
[design canvas](https://claude.ai/artifact/Xzqpg9CqMJgqpDaJHYNkCT).
[ADR 0100](../../docs/adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md)
records it and supersedes ADR 0095's growing status line.

## Rules

### The Panel

- **One height.** The Panel is its row plus the Cursor Line, whatever the Cursor Line or the Toasts
  show. Nothing grows or shrinks it except the Diagnostics readouts the viewer opens.
- **No status line.** `show_statuses` and the status height go.

### The Cursor Line

- **What it follows.** The Cursor's Cell, never the pointer.
- **A Function.** When the Cell is any Cell of a Function's Span (its anchor or an operand), the
  line reads the Function's name, its spelling, one line on what it answers, each operand's role and
  the two Cells it holds, then `answer`, the two Cells of its Output Portal and that Portal's
  Position: `Add  .+  answers left + right   left 03   right 04   answer 06  at 00 0C`. A Function
  that performs an effect has no answer, so the line ends after its operands.
- **A diagnostic.** When a Language Map diagnostic covers the Cell, the line shows that diagnostic
  instead, as `.scratch/source-diagnostics/issues/02` decided.
- **Anything else.** A Comment, a Bang or a standalone Operand Literal is named by its Language
  Unit. An empty Cell leaves the line blank, and its space stays.
- **Where the words come from.** Each Function's name and one-line summary sit beside its row in the
  Function table in `lang/src/atom.rs`. The name is the reference heading's words
  (`.|` is `Absolute Difference`, not `AbsoluteDifference`), and a test ties the table to the
  headings in `docs/functions.md` so the two cannot drift. The console reads the Render Frame and
  the table; it parses no Markdown and re-parses no Source.
- **Look.** The Panel's monospace readout font: the name and values in the strong text colour, the
  summary, roles and Position in the weak text colour.

### Toasts

- **One per standing message**, in the order the Panel listed them: the MIDI selection, a MIDI
  output failure, a UDP or OSC host or port the Output list refused, a failed UDP or OSC send, an
  engine failure.
- **Where.** Stacked from the window's top-right corner, beneath the menu bar, 16 points in from
  both edges, over the Source and clear of the Minimap.
- **What.** A popup frame outlined in the error colour holding the output's label (`MIDI`,
  `UDP/OSC` or `ENGINE`) in the weak text colour, a `×` dismiss button, and the message in the
  error colour, wrapped, never cut short.
- **How long.** A Toast fades out after `[toasts] seconds`, default 5, 1 to 60. `×` removes it at
  once. A faded or dismissed Toast does not return for the same message; a changed message, or one
  that clears and comes back, raises a new Toast.
- **Settings.** `[toasts] seconds` is read like every other key (ADR 0091 on the web), documented in
  `console/src/config.rs`'s header, and an out-of-range or mistyped value is the usual notice with
  the default applied.

## Tests

The Cursor Line's text is a pure function of the Render Frame, the Cursor and the Function table,
tested on its own for a Function's anchor and operand, an effect Function, a diagnostic, a Comment,
a Bang, a standalone Operand Literal and an empty Cell. The table-to-reference test reads every
heading in `docs/functions.md`. The console's kittests find a Toast's label and message beneath the
menu bar, press `×` and find it gone, advance past the configured seconds and find it gone, and
check that the Panel's height is the same with no message and with every message.

## Delivery

One pull request, one commit per ticket. Implementation uses the `egui` skill and
`mattpocock-skills:tdd`. The prototype that produced the captures is throwaway; its commits are kept
as a git bundle (`cursor-info-layouts/prototype.bundle` in the project's shared files), not on
`main`.
