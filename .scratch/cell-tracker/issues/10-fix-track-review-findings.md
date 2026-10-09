# 10 — Fix the ticket 04 review findings

Status: wontfix
Blocked by: 08

**Where:** worktree `.worktrees/cell-tracker-track`, branch `cell-tracker/track`. First rebase it onto the current `cell-tracker/blank-results` (which carries ticket 08's fixes) and resolve conflicts. Review range: `git diff cell-tracker/blank-results...cell-tracker/track` (originally commit `657fbe50`).

**What to build:**

Resolve the Track review findings. Use TDD through the Source Tick interface for the coverage gaps.

## Acceptance criteria

Hard violations (CLAUDE.md Rust policy):

- [ ] `lang/src/functions/list.rs` `track()`: the `count == 0` branch and the `ZeroWrap` error doc (`lang/src/error.rs`) are reachable only from `track_refuses_a_zero_count`. The shipped caller (`orcvs/src/source/tick/execution.rs`) always supplies `node.list_count()` ≥ 1, the Parser refuses `00`, and `ReplacementChange::List` stops a Replacement introducing Track. Replace with an assertion of the invariant or a `NonZeroU8` count. ("Keep test-only inputs out of shipped code"; "use assertions for proven invariants.")
- [ ] `execution.rs` `deliver_item`: the `node.items.get(item)` else-arm diagnosing "selected Item … outside its List" cannot run (`item = index % list_count()`, `list_count() == items.len()`). Make it an `expect` stating the invariant.
- [ ] Wrap the doc line in `console/src/function_reference.rs` near line 343 ("operand or selects a blank Item. The Blank Answer writes two spaces there, which is what tells") within 100 columns.

Coverage gaps (ticket 04 criteria):

- [ ] Index writers as partial, competing and failed suppliers, observed in the same Tick before and after Track in Grid order (only Item writers are covered now). Criterion: "Writers to the index and selected Item are observed in the same Tick … including partial and competing writes and failed suppliers."
- [ ] Copied `**` in a selected Item: put an aligned root at Track's Output Portal and assert it is not activated. Pin the related behaviour that a Copied answer covering an aligned root or a Function anchor takes the ordinary-write path and suppresses the target (where a Jump would activate or replace it), and make sure ADR 0063 states it.

Judgement calls (apply where they simplify, or record why not):

- [ ] "The count is the last operand" is derived by position in the parser (`reads_list() && position + 1 == signature.len()`), `execution.rs operands()`, parser test `count_slot`, `tick.rs` `operand_sources`, and a `tick.rs` chain test. Declare it once, e.g. `Function::list_count_slot()` or a distinct count Token in `define_functions!`.
- [ ] Reduce `reads_list()` / `Token::Item` special cases spread across parser, `syntax_blocks`, `operands`, replacement facts, `expression.rs`, `language_map.rs`, `portal.rs`, `render_frame.rs`, `style.rs` and `app.rs` where the declared count slot makes that possible.
- [ ] Consider whether the Interpreter's `track` (only `index % count`) and `Interpretation::Item` earn their place, given `orcvs` owns the Items and the selection.
- [ ] `lang/src/expression.rs` `Token::parse`: the `Self::Item => Err(ExpectedToken)` arm cannot run (`take_list` uses `next_token`); make it `unreachable!` with the reason.
- [ ] `orcvs/src/source/tick.rs` `computations()`: merge the consecutive `if let Some(parent) = parent` checks into one match on the token.
- [ ] `execution.rs`: trim the extra `encoding.clone()` / `answer.clone()` in `deliver_item` that `Answer` losing `Copy` introduced.

## Gates

As ticket 08, from `.worktrees/cell-tracker-track`. Commit on `cell-tracker/track`; do not push.

## Comments

Resolved by `6cfa82fa` on `cell-tracker/track` (address-code-review), after rebasing Track to `6df8b9a5`. J2 partly inherent; J3 and the P2 Function-anchor question were ruled in 07 and are carried by 12.

Superseded by ADR 0067 / PR #205, which replaced the List-claim Track this review covered. `6cfa82fa` was never merged; PR #201 was closed.
