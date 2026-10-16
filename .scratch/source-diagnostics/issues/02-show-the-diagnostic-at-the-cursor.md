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
