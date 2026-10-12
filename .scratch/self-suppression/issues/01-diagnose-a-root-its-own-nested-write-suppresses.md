# 01 — Diagnose a root its own nested write suppresses

Status: ready-for-human
Blocked by: None (can start immediately)

**What to build:**

A nested Write can reach back over its own Expression. In `.<.|00.v~?000000=$35BA@<.x64FBE506`, placed in the middle row of a Grid, the nested `@<` writes west over Cells of the Functions still to come in the same Expression. ADR 0034 and ADR 0067 say a write onto a Function whose Turn is still to come suppresses it quietly. Suppression climbs to the root `.<`: it takes no Turn, reaches no Interpreter, and the Tick reports nothing.

The owner decided (2026-10-12) that this case diagnoses. A root that its own Expression's write suppresses reports why, rather than going silent. Suppression by any other writer stays quiet, as `a_write_onto_a_function_still_to_come_suppresses_or_replaces_it` asserts.

The property `a_tick_over_every_nested_expression_the_parser_accepts_settles_its_root` (`orcvs/src/source/tick.rs`) already demands this: an intrinsically active root is interpreted or diagnosed. Today it fails on about 1 run in 30 at `PROPTEST_CASES=32`, and it removed PR #214 from the merge queue.

Proposed defaults, for the owner to confirm:

- The diagnostic is reported at the root's anchor, and it names the writer's Position.
- It is raised only when the suppressing write comes from a Function in the same Expression as the root. A suppression inside the Expression that leaves the root able to take its Turn does not diagnose.
- ADR 0034's suppression rule gains the exception, through an amendment or a short new ADR.

## Acceptance criteria

- [ ] A root suppressed by a write from a Function in its own Expression produces one diagnostic, and the Source writes are otherwise unchanged.
- [ ] A Function suppressed by a write from outside its Expression is still quiet.
- [ ] The shrunk case above is pinned as a stated example beside the property.
- [ ] The nested-expression property passes at the merge tier's case count.
- [ ] The ADR text covers the exception.

## Comments
