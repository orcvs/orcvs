# 05 — Verify live editing in the console

Status: ready-for-agent
Blocked by: 04

**What to build:**

Use ordinary console input and file workflows to load a tracker, edit a pair
Track selects, clear it and restore it while Playback runs.

## Acceptance criteria

- [ ] Typing or pasting a Note into a pair changes the next eligible Tick; a malformed intermediate edit recovers without spurious notes and with bounded useful diagnostics.
- [ ] Clearing a pair leaves later pairs in place; copy and paste retain empty pairs.
- [ ] The pairs east of Track paint as ordinary Source (ADR 0067).
- [ ] Open, Save, reopen and a new Playback run reproduce the written pattern and reset the Clock according to the Playback contract.
- [ ] Tests drive real console input and existing file workflows under the egui skill; no dedicated tracker editor or extra adapter is introduced.

## Comments

Rewritten for ADR 0067 and ADR 0069, which replaced the List claim, its Items, the Blank Answer and the pending state. Track's pairs are ordinary Source, and an empty pair is copied as a Copy copies empty Cells (ADR 0069).

The console presents no Source or Tick diagnostic yet (ADR 0069), so the presentation criterion needs that path first. Building it settles whether a standalone Operand Literal is diagnosed: the Language Map reports every pair east of Track as an unknown Function, two diagnostics per Cell, because a refused Function advances one Cell. Not diagnosing a Unit that spells a Number or a Note is a change to `Parser::analyze` and the Language Map's row walk, and needs an ADR answering ADR 0067's open question; its cost is that operands left behind by a deleted or replaced Function are no longer diagnosed.

### Decided 2026-10-13 by Toby

Criterion 3 is descoped to painting. The console shows no Source or Tick diagnostic messages
(`docs/adr/0069-an-unwritten-operand-is-invalid-input.md:19`), and no ticket owns presenting them,
so presenting a malformed selected pair's Tick diagnostic is not part of this ticket. Status stays
`ready-for-agent`.

### 2026-10-16

Presenting Source and Tick diagnostics is now owned by `.scratch/source-diagnostics/`, which starts with ADR 0067's open question on standalone Operand Literals. This ticket stays scoped to painting.
