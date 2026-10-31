# 02 — Show the diagnostic at the Cursor

Status: ready-for-agent
Blocked by: 04

**What to build:**

When the Cursor's Cell is covered by a Language Map diagnostic, a status line in the bottom Panel shows that diagnostic's message. Tick diagnostics are not shown here (ticket 03).

## Acceptance criteria

- [ ] Moving the Cursor onto a diagnosed Cell shows its message in the bottom Panel's status line; moving off clears it.
- [ ] The status line reserves its space whether or not a message is showing, so the Source View does not move.
- [ ] The message comes from the Render Frame's existing accessors (`expression_at` and its `diagnostic`, `lexical_diagnostics`); nothing is re-parsed in the console.
- [ ] When a Cell is covered by both its Expression's diagnostic and a lexical diagnostic (the Cursor on `Z` of `Z `), the Expression's diagnostic is shown, because it names the spelling rather than one character.
- [ ] Pairs east of a Track show no message, per ticket 04.
- [ ] Tested by driving the shipped console under the egui skill.

## Comments

### Decided 2026-10-16 by Toby

A status line rather than a tooltip, because editing is keyboard- and Cursor-driven. Language Map diagnostics are not listed in the Diagnostics Panel, so a standing Source error cannot crowd out Tick diagnostics there.

Where an Expression diagnostic and a lexical diagnostic cover the same Cell, the Expression's is shown: it names the spelling the Parser refused, where the lexical one names a single character. Showing both would grow the status line past its reserved height.

### 2026-10-31, from `.scratch/cursor-line-and-toasts`

The status line this ticket names is now the Panel's permanent Cursor Line (ADR 0100), so the
diagnostic is drawn there, in place of the Function's description, and the line's space is reserved
by that effort rather than by this ticket. Build this after `cursor-line-and-toasts` ticket 04.
