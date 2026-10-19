# 02: Undo and Redo Source edits

**What to build:** Command Z undoes the last user edit to the Source and
Command Shift Z redoes it, on native and the web, as issue 01's ADR decides.
Tick writes are never recorded. See `../spec.md`.

**Blocked by:** 01

**Status:** needs-triage

- [ ] An edit history in `orcvs` beside the Source, recording each user
      edit's written Cells and their prior content, and never a Tick write.
- [ ] `InputEvent::Undo` and `InputEvent::Redo`, and a new edit after an Undo
      clears the Redo steps.
- [ ] The console translates Command Z and Command Shift Z (Ctrl outside
      macOS) once per press, while no text field has focus.
- [ ] Tests: each kind of edit undoes and redoes; typing groups as the ADR
      says; an Undo over a Cell a Tick write has changed behaves as the ADR
      says; Open and New clear the history; a focused text field keeps the
      chord.
- [ ] `CONTEXT.md` gains Undo and Redo entries.

## Comments
