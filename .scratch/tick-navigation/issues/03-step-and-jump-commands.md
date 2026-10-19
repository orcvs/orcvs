# 03: Step one Tick back or forward, and jump to a typed Tick

**What to build:** The commands issue 01's ADR chooses, on native and the
web, each sending a stop's safety action and discarding the owed Note Offs.
See `../spec.md`.

**Blocked by:** 02

**Status:** needs-triage

- [ ] `InputEvent`s for step forward, step back and jump, and the console's
      chords and Panel field for them.
- [ ] Tests: stepping back then forward returns the same Source; a jump
      silences; the Panel shows the Tick reached.
- [ ] `CONTEXT.md` gains the navigation vocabulary.

## Comments
