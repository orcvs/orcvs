# 12 — Apply the Track review rulings

Status: wontfix
Blocked by: 07, 09, 10

**Where:** a new branch `cell-tracker/track-rulings` in a new worktree `.worktrees/cell-tracker-track-rulings`, created from `cell-tracker/blank-operand-rulings` (PR #202, tip `9aeefef6`, which carries tickets 04 and 09). Open its PR with base `cell-tracker/blank-operand-rulings`.

**What to build:**

Implement the P2 and J3 rulings recorded under `## Answer` in `07-decide-blank-operand-edge-cases.md`.

## Acceptance criteria

- [ ] Track's count operand is a `NonZeroU8` in `lang`; the `expect("…never 00")` in `lang/src/functions/list.rs` is gone and no shipped path can hand Track a zero count. `Interpretation::Item` and the single Interpreter dispatch stay.
- [ ] ADR 0063 states that Track's copied characters are an ordinary Cell write: copied `**` on a root's anchor covers and suppresses it, a copied Function spelling over a running Function suppresses it, and Function Replacement does not apply because no Function answers a Function Atom. Replace "activates no root in the Tick it is copied" with "never activates a root".
- [ ] `copied_characters_activate_and_replace_nothing_where_they_land` and the `deliver_item` doc agree with the ADR wording.

## Gates

As ticket 08, from `.worktrees/cell-tracker-track-rulings`. Commit on `cell-tracker/track-rulings`.

## Comments

Resolved by `9ef4a7e4` on `cell-tracker/track-rulings`, PR #203 (stacked on #202).

Superseded by ADR 0067 / PR #205; see 07 for the reversed P2 and J3 rulings. `9ef4a7e4` was never merged; PR #203 was closed.
