# 03 — Bring the cell-tracker spec in line with ADR 0069

Status: resolved
Blocked by: None — can start immediately

**What to build:**

`.scratch/cell-tracker/spec.md` and tickets 05 and 06 there describe pending as ADR 0066 did and give "rest" a meaning of its own. Under ADR 0069 neither is a language concept: Track copies whatever Cells are at its Input Portal to its Output Portal, and a Function whose arguments are not valid is invalid. Start after PR #206 merges, since it rewrites the same files.

## Acceptance criteria

- [x] No user story, decision or testing decision defines "rest" or treats empty pairs as rests (e.g. "pairs are rests" in the Solution). Empty Cells are described as what Track copies, and a Play whose note slot is empty as invalid.
- [x] Nothing describes a pending Function or a pending state; an unwritten operand is invalid input under ADR 0069, and Implementation Decisions 9 and 10 cite it.
- [x] The Testing Decisions pin that a Play whose note slot Track empties, directly or through nesting, emits no Play Command, and that a value Function between Track and the Play that is invalid leaves its last answer in the slot for the next Bang.
- [x] Tickets 05 and 06 use the same vocabulary.
- [x] The roadmap gates pass.

## Comments

Resolved on PR #206 by its commit "Describe the cell tracker under ADR 0069, without rest or pending", which rewrote `.scratch/cell-tracker/spec.md` and tickets 05 and 06 there, building on uncommitted edits another session had left in its worktree.
