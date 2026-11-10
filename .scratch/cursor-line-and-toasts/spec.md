# Cursor Line and Toasts

Status: resolved

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
  line reads the Function's spelling and name, each Role's name and Cells (a Position's two Numbers
  are one Role), then one line on what it answers or does:
  `.+ Add   left 03  right 04   answers left + right`. A Function whose answer has a fixed place
  pins the two Cells of its Output Portal and that Portal's Position to the line's right edge, as
  `→ 07 at 00 01`. A Function that performs an effect has no answer, so the line ends on its summary.
- **A diagnostic.** When a Language Map diagnostic covers the Cell, the line shows that diagnostic
  instead, as `.scratch/source-diagnostics/issues/02` decided. That ticket builds it, after ticket 04
  here; this effort does not.
- **Anything else.** A Comment reads as its spelling and name, `|| Comment`. A Bang is a Bang Function, so the line describes
  it by its Function row like any Function. An empty Cell or a standalone pair leaves the line
  blank, and its space stays: the pair is diagnosed, and its diagnostic belongs to
  `.scratch/source-diagnostics/issues/02`.
- **Where the words come from.** Each Function's name and one-line summary sit beside its row in the
  Function table in `lang/src/atom.rs`. The name is the reference heading's words
  (`.|` is `Absolute Difference`, not `AbsoluteDifference`), and a test ties the table to the
  headings in `docs/functions.md` so the two cannot drift. The console reads the Render Frame and
  the table; it parses no Markdown and re-parses no Source.
- **One line.** Never wrapped: text past the Panel's width is cut with an ellipsis, and hovering the
  line shows it whole.
- **Look.** The Panel's monospace readout font: the name and values in the strong text colour, the
  summary, roles and Position in the weak text colour.

### Toasts

- **One per standing message**, in the order the Panel listed them: the MIDI selection, a MIDI
  output failure, a UDP or OSC host or port the Output list refused, a failed UDP or OSC send, an
  engine failure.
- **Where.** Stacked from the window's top-right corner, beneath the menu bar, 16 points in from
  both edges, over the Source and clear of the Minimap.
- **What.** A popup frame outlined in the error colour, 380 points wide, or the Source's width less
  both 16-point insets when that is narrower, holding the output's label
  (`MIDI`, `UDP/OSC` or `ENGINE`) in the Theme's diagnostic red (`diagnostic.foreground`), a `×` dismiss button, and the message in
  the error colour, wrapped to at most three lines; a longer message is cut with an ellipsis and
  shown whole on hover.
- **Size and input.** At most five Toasts, one per message source, so the stack is bounded. A Toast
  takes the pointer over its own frame only, as any egui `Area` does; a press there reaches its
  `×` or nothing, and the Source beside it is untouched. Keyboard input always reaches the Source.
  A stack drops its oldest Toasts until its bottom stays 16 points above its floor: the top of the
  Minimap's border while the Minimap is showing, the Source's bottom while it is closed or does not
  fit.
- **Identity.** A Toast is keyed by its message source (one of the five slots above) and the
  message's text. A dismissed or faded key is remembered until that source reports no message on
  a frame or reports a different text; then it is forgotten, and the next message from that source
  raises a new Toast, even one that returns to an earlier text.
- **How long.** A Toast fades out after `[toasts] seconds`, default 5, 1 to 60. `×` removes it at
  once. Escape removes every Toast at once, but only when it has nothing else to undo: a focused
  control takes Escape first, and a Region collapses before any Toast goes. A faded or dismissed Toast does not return while its key is remembered (see Identity).
- **The standing signal.** While any of the five sources reports a message, the Panel's **O**
  readout is drawn in the error colour, whether its Toast is showing, faded or dismissed, and it
  returns to its ordinary colour when every source is clear. Toby chose this on 2026-10-31 so a
  failure that outlives its Toast stays visible without a line of its own.
- **Settings.** `[toasts] seconds` is read like every other key (ADR 0091 on the web), documented in
  `console/src/config.rs`'s header, and an out-of-range or mistyped value is the usual notice with
  the default applied.

## Tests

The Cursor Line's text is a pure function of the Render Frame, the Cursor and the Function table,
tested on its own for a Function's anchor and operand, an effect Function, a Comment, a Bang, a
standalone pair and an empty Cell. The diagnostic case and its test belong to
`source-diagnostics/02`. The table-to-reference test reads every
heading in `docs/functions.md`. The console's kittests find a Toast's label and message beneath the
menu bar, press `×` and find it gone, advance past the configured seconds and find it gone, and
check that the Panel's height is the same with no message and with every message.

## Delivery

One pull request, one commit per ticket. Implementation uses the `egui` skill and
`mattpocock-skills:tdd`. The prototype that produced the captures is throwaway; its commits are kept
as a git bundle (`cursor-info-layouts/prototype.bundle` in the project's shared files), not on
`main`.
