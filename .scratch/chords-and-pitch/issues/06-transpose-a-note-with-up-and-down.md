# 06 — Transpose a Note with Up and Down

Status: ready-for-agent

Blocked by: None — can start immediately

**What to build:** The first two Pitch Functions, Up `#+ note number` and Down `#- note number`,
which answer a Note. They establish the `#` family in the Function table.

## Acceptance criteria

- [ ] Each has arity, type and domain coverage, matching what the Arithmetic Functions have.
- [ ] An answer outside `00`–`7F` diagnoses and never wraps.
- [ ] Each works as a root and nested, writing through its Output Portal as any value Function does.
- [ ] A nested Up or Down in a Play's note operand plays the transposed Note.
- [ ] Both have Function reference rows with worked examples.
- [ ] `CONTEXT.md` defines Pitch Function, Up and Down.
