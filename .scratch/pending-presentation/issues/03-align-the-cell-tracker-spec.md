# 03 — Bring the cell-tracker spec in line with ADR 0069

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

`.scratch/cell-tracker/spec.md` and tickets 05 and 06 there describe pending as ADR 0066 did and give "rest" a meaning of its own. Under ADR 0069 neither is a language concept: Track copies whatever Cells are at its Input Portal to its Output Portal, and a Function whose arguments are not valid is invalid. Start after PR #206 merges, since it rewrites the same files.

## Acceptance criteria

- [ ] No user story, decision or testing decision defines "rest" or treats empty pairs as rests (e.g. "pairs are rests" in the Solution). Empty Cells are described as what Track copies, and a Play whose note slot is empty as invalid.
- [ ] Nothing describes a pending Function or a pending state; an unwritten operand is invalid input under ADR 0069, and Implementation Decisions 9 and 10 cite it.
- [ ] The Testing Decisions pin that a Play whose note slot Track empties, directly or through nesting, emits no Play Command, and that a value Function between Track and the Play that is invalid leaves its last answer in the slot for the next Bang.
- [ ] Tickets 05 and 06 use the same vocabulary.
- [ ] The roadmap gates pass.
