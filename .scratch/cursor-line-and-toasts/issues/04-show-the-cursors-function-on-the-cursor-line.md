# 04 — Show the Cursor's Function on the Cursor Line

Status: ready-for-agent
Type: task
Blocked by: 01, 03

**What to build:** the Panel's permanent Cursor Line beneath its row describes what the Cursor's
Cell belongs to ([spec](../spec.md), ADR 0100).

- [ ] The line is always there and keeps its height when it is blank, so the Panel never changes
      height.
- [ ] On any Cell of a Function's Span it reads the name, spelling and summary, each operand's role
      and two Cells, then `answer`, the Output Portal's two Cells and its Position:
      `Add  .+  answers left + right   left 03   right 04   answer 06  at 00 0C`. An effect
      Function ends after its operands.
- [ ] A Comment, a Bang and a standalone Operand Literal are named by their Language Unit; an empty
      Cell leaves the line blank.
- [ ] The Panel's monospace readout font: name and values in the strong text colour; summary, roles
      and Position in the weak text colour.
- [ ] The text is a pure function of the Render Frame, the Cursor and the Function table, with its
      own tests for each case above, plus a kittest that moves the Cursor onto `.+0304` and reads
      the line.
