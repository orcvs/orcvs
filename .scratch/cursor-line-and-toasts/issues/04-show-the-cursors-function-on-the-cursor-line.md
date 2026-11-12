# 04 — Show the Cursor's Function on the Cursor Line

Status: resolved
Type: task
Blocked by: 01, 03

**What to build:** the Panel's permanent Cursor Line beneath its row describes what the Cursor's
Cell belongs to ([spec](../spec.md), ADR 0100).

- [x] The line is always there and keeps its height when it is blank, so the Panel never changes
      height.
- [x] On any Cell of a Function's Span it reads the spelling and name, each Role's name and Cells (a
      Position's two Numbers are one Role), then the summary:
      `.+ Add   left 03  right 04   answers left + right`. A Function whose answer has a fixed
      place pins the Output Portal's two Cells and its Position to the line's right edge, as
      `→ 07 at 00 01`. An effect Function ends on its summary.
- [x] A Comment reads as its spelling and name, `|| Comment`. A Bang is a Bang Function, described by its Function row like
      any Function. An empty Cell or a standalone pair leaves the line blank: the pair is diagnosed,
      and its diagnostic belongs to `source-diagnostics/02`.
- [x] One line, never wrapped: text past the Panel's width is cut with an ellipsis, and hovering the
      line shows it whole.
- [x] Out of scope: a Cell a diagnostic covers. `source-diagnostics/02` draws the diagnostic here,
      with its own test; until then the line describes the Function as for any other Cell.
- [x] The Panel's monospace readout font: name and values in the strong text colour; summary, roles
      and Position in the weak text colour.
- [x] The text is a pure function of the Render Frame, the Cursor and the Function table, with its
      own tests for each case above, plus a kittest that moves the Cursor onto `.+0304` and reads
      the line.

## Answer

`LanguageMap::function_at` answers the Function whose Span holds a Cell (the one its Cells spell,
or the parent of the operand or Payload it lies in), its operands' Cells and its Output Portal
Reservation; the Render Frame carries it for the Cursor as `cursor_function`. `console::cursor_line`
turns that and the Function table into the line's text, and the Panel draws it in a fixed
`CURSOR_LINE_HEIGHT` beneath its row, one row of the Monospace style plus item spacing, which
`BOTTOM_PANEL_HEIGHT` now includes, so the default window keeps its 1024 by 640 Source area. A Function whose Output Portal its operands select at the
Turn (the Writes, Push, Absolute Copy) pins no answer, since no pair is reserved before the Turn.
A standalone pair such as `07` is diagnosed text, not an Operand Literal, so its Cell leaves the line
blank; only a Comment is named that way.
