# 05 — Verify live editing in the console

Status: ready-for-agent
Blocked by: 04

**What to build:**

Use ordinary console input and file workflows to load a tracker, edit a pair
Track selects, clear it and restore it while Playback runs.

## Acceptance criteria

- [ ] Typing or pasting a Note into a pair changes the next eligible Tick; a malformed intermediate edit recovers without spurious notes and with bounded useful diagnostics.
- [ ] Clearing a pair leaves later pairs in place; copy and paste retain empty pairs.
- [ ] The pairs east of Track paint as ordinary Source (ADR 0067), and a malformed selected pair's Tick diagnostic is presented where Track reads it. The Language Map's diagnostics of the pairs themselves are not presented while whether a standalone Operand Literal is diagnosed stays open.
- [ ] Open, Save, reopen and a new Playback run reproduce the written pattern and reset the Clock according to the Playback contract.
- [ ] Tests drive real console input and existing file workflows under the egui skill; no dedicated tracker editor or extra adapter is introduced.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its Items, the Blank Answer and the pending state. Track's pairs are ordinary Source, and an empty pair is copied as a Jump copies empty Cells.

The console presents no Source or Tick diagnostic yet (ADR 0069), so the presentation criterion needs that path first. Building it settles whether a standalone Operand Literal is diagnosed: the Language Map reports every pair east of Track as an unknown Function, two diagnostics per Cell, because a refused Function advances one Cell. Not diagnosing a Unit that spells a Number or a Note is a change to `Parser::analyze` and the Language Map's row walk, and needs an ADR answering ADR 0067's open question; its cost is that operands left behind by a deleted or replaced Function are no longer diagnosed.
