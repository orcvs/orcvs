# 03 — Bring the cell-tracker spec in line with ADR 0069

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:**

`.scratch/cell-tracker/spec.md` and tickets 05 and 06 there describe pending as ADR 0066 did. Start after PR #206 merges, since it rewrites the same files.

## Acceptance criteria

- [ ] User story 17 says a rest is silent where the empty pair reaches the Play by nesting or by Track's own write, and that a Function waiting on input is presented as pending rather than as a fault.
- [ ] The Testing Decisions pin a silent rest through nesting and the replay of the last Note through a Cell-fed Play, and describe an unwritten slot as invalid input classified pending, not as a pending state.
- [ ] Ticket 06's guide shows transposition inside the Play's note slot and explains why a Cell-fed transposition replays on a rest.
- [ ] Implementation Decisions 9 and 10 cite ADR 0069.
- [ ] The roadmap gates pass.
