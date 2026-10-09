# 09 — Apply the blank-operand decisions

Status: wontfix
Blocked by: 07, 08

**Where:** worktree `.worktrees/cell-tracker-blank-results`, branch `cell-tracker/blank-results`, after ticket 08's commits.

**What to build:**

Implement the rulings recorded under `## Answer` in `07-decide-blank-operand-edge-cases.md`. Read that file first; if it has no Answer, stop.

## Acceptance criteria

- [ ] Every-operand-blank Expressions behave as ruled in 07 question 1, through the Parser, Language Map and Source Tick. If they stay incomplete, restore a Map diagnostic and no write, and keep partly blank operands malformed. If they run, add the consequences (immediate south clear while typing, reserved Output Portal) to ADR 0062's Consequences.
- [ ] A Jump copying two spaces behaves as ruled in 07 question 2. If narrowed, remove the Jump clause from ADR 0062 and CONTEXT.md and update `a_nested_jump_that_copies_a_blank_returns_blank`.
- [ ] Tests assert each ruling through observable Source Cells, Play Commands and diagnostics; tests rewritten in ticket 03 because a bare `.+` became valid are revisited.
- [ ] S1 as ruled in 07: remove `try_parse` and its strict-only error variants; move the bench, allocation test, `orcvs` properties and `interpret_source` to `analyze()`.
- [ ] ADR 0062 records that Orca's empty-reads-as-zero was considered and rejected (reason in 07), and corrects its Orca sentence per 07.
- [ ] Ticket 03's "applies only when an inline operand is blank" is reworded per 07 question 2.
- [ ] ADR 0062, ADR 0061's status line, CONTEXT.md and the Function reference (`console/src/function_reference.rs`, `console/assets/function_reference.orcvs`) agree with the rulings.

## Gates

As ticket 08. Commit on `cell-tracker/blank-results`; do not push. Then `cell-tracker/track` needs rebasing again (ticket 10 or a follow-up), and Track tests that rely on blank behaviour must be re-checked.

## Comments

Resolved by `e701944a` and `9aeefef6` on `cell-tracker/blank-operand-rulings`, PR #202 (stacked on #201).

Superseded by ADR 0066 / PR #200; see 07. `e701944a` and `9aeefef6` were never merged; PR #202 was closed.
