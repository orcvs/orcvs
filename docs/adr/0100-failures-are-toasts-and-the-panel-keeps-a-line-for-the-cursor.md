# Failures are Toasts, and the Panel keeps a line for the Cursor

Status: accepted. Supersedes ADR 0095's status line. Decided by Toby on 2026-10-31.

ADR 0095 drew the Panel's status messages on a line beneath its row that is there only while a
message is, so the Panel grew by a line when the first message arrived and shrank when the last
cleared, moving the Source's bottom edge. Nothing showed what the Cursor's Cell belongs to: a
Function's name, operands and answer had to be read off the Grid or looked up in the Function
reference.

## Decision

**A failure is a Toast.** Each standing message (the MIDI selection, a MIDI output failure, a UDP or
OSC host or port the Output list refused, a failed UDP or OSC send, an engine failure) floats as its
own Toast in the window's top-right corner, beneath the menu bar and clear of the Minimap. A Toast
names the output it belongs to (`MIDI`, `UDP/OSC` or `ENGINE`), carries the message and a dismiss
button, and fades after `[toasts] seconds`. egui has no toast widget, so a Toast is an `egui::Area`
anchored to the corner; no dependency is added.

**A failure that outlives its Toast still shows.** While any message stands, the Panel's **O**
readout is drawn in the error colour, so a faded or dismissed Toast never leaves a broken output
unsignalled, and the Panel gains no line for it.

**The Panel keeps one permanent line for the Cursor.** The Cursor Line sits beneath the Panel's row
and is there whether or not it has anything to say, so the Panel's height never depends on what it
shows. The Diagnostics readouts still add their own height when the viewer opens them.

**The Cursor Line follows the Cursor, not the pointer.** Editing is keyboard- and Cursor-driven, as
the 2026-10-16 decision in `.scratch/source-diagnostics/issues/02` found; a floating tooltip was
considered and not chosen.

## Considered options

Captured from the console on the design canvas the decision was made on:

- **The growing line, kept, with the Cursor's Function after the messages, or replaced by them.**
  It mixes unrelated content and still moves the Source.
- **The Cursor's Function on the Panel's row after O.** It runs under the Minimap at the default
  window.
- **A fixed Panel split into messages and Cursor halves, with or without labels.** It spends three
  or four lines of the Panel on messages that are usually absent.
- **An inspector column beside the Source.** It takes width from the Source for one line of text.

## Consequences

- `show_statuses` and the Panel's status height go; the Panel is its row plus the Cursor Line.
- The message sources and their order are unchanged; only where they are drawn moves.
- `.scratch/source-diagnostics/issues/02` draws the diagnostic at the Cursor on the Cursor Line.
