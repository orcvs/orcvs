# 01: Undo and Redo Source edits

**What to build:** Command Z undoes the last user edit to the Source and
Command Shift Z redoes it, on native and the web. Tick writes are never
recorded. See `../spec.md`; its open questions are settled before this starts.

**Blocked by:** None (can start immediately once triaged)

**Status:** needs-triage

- [ ] Settle the spec's open questions and record them in an ADR.
- [ ] An edit history in `orcvs` beside the Source, recording each user edit's
      written Cells and their prior content, and never a Tick write.
- [ ] `InputEvent::Undo` and `InputEvent::Redo`, and a new edit after an Undo
      clears the Redo steps.
- [ ] The console translates Command Z and Command Shift Z (Ctrl outside
      macOS) once per press, and the Edit menu lists both.
- [ ] Tests: each kind of edit undoes and redoes; an Undo during Playback
      behaves as the ADR says; Open and New clear the history.
- [ ] `CONTEXT.md` gains Undo and Redo entries.

## Comments
