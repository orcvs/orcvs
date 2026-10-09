# 05 — Verify live editing in the console

Status: ready-for-agent
Blocked by: 04

**What to build:**

Use ordinary console input and file workflows to load a tracker, edit a pair
Track selects, clear it and restore it while Playback runs.

## Acceptance criteria

- [ ] Typing or pasting a Note into a pair changes the next eligible Tick; a malformed intermediate edit recovers without spurious notes and with bounded useful diagnostics.
- [ ] Clearing a pair leaves later pairs in place; copy and paste retain empty pairs.
- [ ] The pairs east of Track paint as ordinary Source (ADR 0067), and a malformed selected pair's diagnostic is presented where Track reads it.
- [ ] Open, Save, reopen and a new Playback run reproduce the written pattern and reset the Clock according to the Playback contract.
- [ ] Tests drive real console input and existing file workflows under the egui skill; no dedicated tracker editor or extra adapter is introduced.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its Items, the Blank Answer and the pending state. Track's pairs are ordinary Source, and an empty pair is copied as a Jump copies empty Cells.
