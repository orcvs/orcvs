# Cursor Line and Toasts

Status: resolved

## The decision

Failures leave the Panel and float as Toasts in the window's top-right corner, and the Panel keeps
one permanent Cursor Line beneath its row that describes what the Cursor's Cell belongs to. Toby
picked this (layout G) on 2026-10-31, from layouts captured from the console on the
[design canvas](https://claude.ai/artifact/Xzqpg9CqMJgqpDaJHYNkCT).
[ADR 0100](../../docs/adr/0100-failures-are-toasts-and-the-panel-keeps-a-line-for-the-cursor.md)
records it and supersedes ADR 0095's growing status line.

Toby named the line **Info**, set it in two lines on the Grid's Cells and made it a View toggle,
shown by default, on 2026-11-12, from the layouts on the
[Panel layout canvas](https://claude.ai/artifact/68XUFWmYGPveWQ46sDWcUf). ADR 0100's amendment
records it.

## Rules

### The Panel

- **Cell rows.** The Panel is laid on the Cell the Source presents and scales with it: a Cell row of
  room beneath its top edge, the Readouts, and a Cell row of room before the window's edge. While
  Info is shown, a Cell row of room, Info's Cells and Info's labels sit between the Readouts and the
  last row. A widget taller than a Cell, the BPM field or the Output list, overhangs its row,
  centred, so the rows stay on the Cells.
- **Gaps in Cells.** A Readout's label is half a Cell from its value, and Readout groups are two
  Cells apart; the top bar's menus share that gap.
- **One height.** Whatever Info or the Toasts show. Nothing grows or shrinks the Panel except the
  Diagnostics readouts the viewer opens and Info being shown or hidden.
- **No status line.** `show_statuses` and the status height go.

### Info

- **What it follows.** The Cursor's Cell, never the pointer.
- **Shown or hidden.** View › Info, and ⌘I on native, shows or hides it; it is shown by default.
  `[info] open` in `config.toml` sets how the console starts, and a menu change is not saved, as for
  Diagnostics and the Minimap. Hidden, its three Cell rows go back to the Source.
- **Two lines.** The first paints each entry's Cells as the Source paints them, the Cursor's outline
  on its Cell; the second names each entry beneath its Cells. Each entry is as wide as the wider of
  its two lines, and the next starts on the Cell column two Cells past it.
- **A Function.** When the Cell is any Cell of a Function's Span (its anchor or an operand), the
  entries are its spelling over its name, each Role's Cells over the Role's name (a Position's two
  Numbers are one Role), then its Output Portal's Cells over what it answers: its summary less the
  leading `answers `, or the summary as written when it says something else. A Function that
  performs an effect has no answer, so its summary is the last entry, with no Cells above it. The
  answer's own Position is not shown.

  ```text
  .+   03    04     07
  Add  left  right  left + right
  ```
- **A diagnostic.** When a Language Map diagnostic covers the Cell, Info shows that diagnostic
  instead, as `.scratch/source-diagnostics/issues/02` decided. That ticket builds it, after ticket 04
  here; this effort does not.
- **Anything else.** A Comment's opening pair reads over `Comment`. A Bang is a Bang Function, so
  Info describes it by its Function row like any Function. An empty Cell or a standalone pair leaves
  Info blank, and its rows stay: the pair is diagnosed, and its diagnostic belongs to
  `.scratch/source-diagnostics/issues/02`.
- **Where the words come from.** Each Function's name and one-line summary sit beside its row in the
  Function table in `lang/src/atom.rs`. The name is the reference heading's words
  (`.|` is `Absolute Difference`, not `AbsoluteDifference`), and a test ties the table to the
  headings in `docs/functions.md` so the two cannot drift. The console reads the Render Frame and
  the table; it parses no Markdown and re-parses no Source.
- **Past the edge.** A label past the Panel's edge is cut with an ellipsis, and hovering it shows
  it whole.
- **Look.** Labels in the Panel's monospace readout font: a name in the strong text colour, Roles
  and answers in the weak text colour.

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

Info's entries are a pure function of the Render Frame, the Cursor and the Function table, tested
on their own for a Function's anchor and operand, an effect Function, a Position, a Comment, a Bang,
a standalone pair and an empty Cell. Kittests check the Panel's Cell rows at the default Cell and a
configured one, that entries start on Cell columns two Cells apart, and that hiding Info gives its
rows back. The diagnostic case and its test belong to
`source-diagnostics/02`. The table-to-reference test reads every
heading in `docs/functions.md`. The console's kittests find a Toast's label and message beneath the
menu bar, press `×` and find it gone, advance past the configured seconds and find it gone, and
check that the Panel's height is the same with no message and with every message.

## Delivery

One pull request, one commit per ticket. Implementation uses the `egui` skill and
`mattpocock-skills:tdd`. The prototype that produced the captures is throwaway; its commits are kept
as a git bundle (`cursor-info-layouts/prototype.bundle` in the project's shared files), not on
`main`.
