# Add first-class Sequence values and Portal result writes

**Status:** ready-for-agent

## Goal

Implement ADRs 0007, 0009, and 0017: one flat ordered Sequence value, pervasive Atomic Function
behavior, structural Sequence Functions, and complete-fit result delivery through Portals.

## Required behavior

- A Sequence contains ordered Atoms, never nested Sequences.
- Compatible Atomic Functions broadcast scalar-to-sequence or pair equal-length Sequences.
- Incompatible non-scalar lengths diagnose; scalar feedback exceptions remain explicit.
- Range, Reverse, Concatenate, Select, and Replace follow ADR 0007 exactly.
- Portal output validates and plans one complete encoding without partial writes or stale-tail clears.
- Generated Cells return to ordinary Source parsing on the next Source Snapshot.

## Delivery order

1. `issues/01-add-the-sequence-language-value.md`
2. `issues/02-broadcast-atomic-functions-over-sequences.md`
3. `issues/03-add-structural-sequence-functions.md`
4. `issues/04-plan-complete-sequence-writes-through-portals.md`

## Out of scope

- Source Read and Source Write operands, deferred by ADR 0005.
- UDP, OSC, and Application Command message values.
- Hidden variables or persistent state outside Source Snapshot.
