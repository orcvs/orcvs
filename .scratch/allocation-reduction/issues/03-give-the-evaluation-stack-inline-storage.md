# 03 — Give the evaluation stack inline storage

**What to build:** Evaluating a Turn through `Interpreter::execute_function`, the path `orcvs`
calls, uses inline storage for the Operand Stack, so a Turn takes no heap block for its stack.
Operand clones are source-audit/24's.

**Blocked by:** None — can start immediately.

Related: source-audit/09 deleted `Interpreter::execute` (orcvs/orcvs#156), so `execute_function` is the only caller of the Operand Stack; source-audit/24 owns the operand clones on the same path.

**Status:** resolved

- [x] `Stack::new` stops taking a heap block for every Turn. The path is `Interpreter::execute_function` -> `Context::new(inputs, operands.len())` (`lang/src/interpreter.rs`) -> `Stack::new(limit)` (`lang/src/stack.rs`), which is `Vec::with_capacity(limit)` over a 24-byte `Value`.
- [x] The storage is inline, following `05e4490` and `a215af7` — the same treatment already applied to the Parser's pending stacks and to the inline Expression records. The inline capacity is `MAX_OPERANDS` (`lang/src/stack.rs`), the widest operand list the Function table declares; the stack holds the operands only, with no slot for the answer. `execute_function` is the only caller, so no growable overflow branch ships.
- [x] ADR 0028's requirement still holds. `execute_function` refuses an operand count other than its Function's signature length before building the stack, so the depth is at most `MAX_OPERANDS` and an inline capacity of that size is provably sufficient; record the proof. A push onto an exhausted stack still answers `OperandStackExhausted` rather than panicking.
- [x] A Turn through `execute_function` takes no heap block for its Operand Stack. `a_turn_over_atoms_allocates_nothing_but_its_operand_stack` (`lang/tests/allocation.rs`) pins zero allocations for a Function whose operands and answer hold no Sequence. `evaluating_a_parsed_source_allocates_per_call_and_not_per_row` (same file) drops its one-block-per-call ceiling to zero, and the undated `FINDING:` comment at the head of that test, which attributes the block to `Stack::new`'s `Vec::with_capacity`, is replaced with a note saying what was corrected.
- [x] Every other existing `lang` test and property passes unchanged, including the `Stack` suite, the Interpreter suites and the parser property tests. The three allocation tests the 2026-09-26 reconciliation comment names are tightened by one block each: `a_turn_over_atoms_allocates_nothing_but_its_operand_stack` to zero, and `a_sequence_operand_is_consumed_without_copying_its_members` and `a_turn_that_builds_a_sequence_allocates_only_that_answer` by dropping the Operand Stack's block from their ceilings.
- [x] The `execute_function` benchmark and the `execute_function_operands` group in `lang/benches/lang.rs` cover this path. Note them for the comparison the benchmark workflow runs; do not run the comparison locally.

## Comments

What `memory-verification/01` measured:

```
tick over 11 Expressions                   -> 11 blocks,   816 bytes
tick over 44 Expressions                   -> 44 blocks,  3264 bytes
execute [Bang]                             ->  1 block,     24 bytes
execute [Add, 1, 2]                        ->  1 block,     72 bytes
execute [Add, Add, 1, 1, Subtract, 10, 5]  ->  1 block,    168 bytes
```

Exactly one heap block per Expression, sized to its Atom count. A bare `**` Bang — one Atom, no
operands, no arithmetic — still pays for a heap allocation to be evaluated.

This is the most mechanical of the three findings, because the repository has already done it twice
to neighbouring structures. `05e4490` ("Avoid heap allocation for common parser pending stacks") and
`a215af7` ("Keep eight Expression records inline with growable overflow") are the precedent, the
`arrayvec` dependency is already in the workspace manifest with `default-features = false`, and the
shape they established is the shape to follow.

Note the interaction with `lang/src/lib.rs`'s size assertions. The test
`the_answer_seam_is_the_size_the_execute_function_benchmark_was_measured_against` pins
`size_of::<Performance>()` and `size_of::<Interpretation>()` against the `execute` benchmark's
recorded figures, and its comment explains a 46ns-to-52ns move in terms of those sizes. Inline
storage changes what `Context` and `Stack` cost, not what `Interpretation` answers, so that test
should be unaffected — but if it moves, it is notice rather than a defect, and `execute` is the
measurement to take again.

**2026-09-25 — from the source-audit review (`source-audit/24`).** `orcvs` never calls `Interpreter::execute`. Each Turn goes through `Interpreter::execute_function`, which builds `Context::new(inputs, operands.len() + 1)` and pushes a clone of every operand `Value`. Inline storage must cover that path, and the allocation test and benchmark named above should measure it: `execute` is reached only by `lang`'s tests, `lang/benches/lang.rs` and `lang/tests/allocation.rs`. `source-audit/24` defers the operand-stack allocation to this ticket.

**2026-09-25 — criteria rewritten around `execute_function`.** The criteria now target the shipped path instead of `execute`. "Allocates nothing at all" is narrowed to the Operand Stack: Sequence operand clones remain on the path until source-audit/24 removes them. The `MAX_OPERANDS + 1` bound replaces the per-Expression capacity question for the shipped path.

**2026-09-26 — `lang/narrow-api` (source-audit PR 11).** `Interpreter::execute` is deleted, not kept
public, so the "if `execute` stays public" criteria are moot and no overflow branch is needed.
`execute_function` now sizes the stack at `operands.len()` and dispatches the one Function itself.
The allocation test is retargeted as
`evaluating_a_parsed_source_allocates_per_call_and_not_per_row`, measuring `execute_function`
and publishing `lang call fixture`; its ceiling is still one block per call, so the zero pin
remains this issue's work. `lang/benches/lang.rs` benchmarks `execute_function`, with its floor
in `benches/floors.toml`.

**2026-09-26 — from review of `lang/narrow-api`.** `Stack` (`lang/src/stack.rs`) still models a
growing evaluation stack, though `execute_function` now supplies exactly one Function's operands
and returns its answer directly. When the storage goes inline, build it as a constructor over the
resolved operands, so the reversal, the capacity and the incremental pushes stay inside `lang`
rather than in `execute_function`'s loop.

**2026-09-26 — reconciled with source-audit/24 (epic PR 13).** source-audit/24 adds `execute_function` allocation tests and a benchmark group beside the ones `lang/narrow-api` retargeted; reuse them rather than adding more.

- `a_turn_over_atoms_allocates_nothing_but_its_operand_stack` measures `.+0102` at 1 block, the Operand Stack alone (`operand_stack(2)` bytes). Tighten it to zero allocations here; it is the test the fourth criterion asks for.
- `a_sequence_operand_is_consumed_without_copying_its_members` and `a_turn_that_builds_a_sequence_allocates_only_that_answer` allow the Operand Stack's block as a ceiling (`operand_stack(arity)` bytes, one block). With inline storage they pass unchanged; tighten each by one block when this lands.
- The `execute_function` benchmark and the `execute_function_operands` group in `lang/benches/lang.rs` cover this path.
- `execute_function` now takes its operands by value, so no operand clone remains on the path: after this ticket a Turn whose operands and answer hold no Sequence allocates nothing in `lang`. `orcvs` still allocates two blocks per Turn outside `lang` — the `Tokens` signature in `opens_turn` and the operand `Vec` — recorded in source-audit/24.

**2026-09-27 — criteria restated for the path on `main`.** `Interpreter::execute` is deleted, so the conditions on it staying public, the overflow-branch criterion and the exceeding-capacity test are gone. `execute_function` sizes the stack at `operands.len()` with no answer slot, so the bound is `MAX_OPERANDS`, not `MAX_OPERANDS + 1`. The allocation criterion names the two tests that now measure `execute_function`, and identifies the `FINDING:` comment by its test, since `a_tick_over_an_already_parsed_source_allocates_per_expression_and_not_per_row` and its `FINDING (2026-09-09)` comment no longer exist. The test criterion now asks for the three allocation ceilings the reconciliation comment tightens, instead of every test passing unchanged. In the first comment, read the size test as `the_answer_seam_is_the_size_the_execute_function_benchmark_was_measured_against`, pinned against the `execute_function` benchmark floor.

**2026-09-29 — implemented on `perf/inline-operand-stack`.** Every box is met.

- `Stack` (`lang/src/stack.rs`) holds `ArrayVec<Value, MAX_OPERANDS>` and a logical `limit`
  clamped to `MAX_OPERANDS` in `Stack::new`. No growable overflow branch ships. Following the
  2026-09-26 review comment, `Stack::with_operands` takes the resolved operands in signature order
  and owns the reversal, the capacity and the pushes; `execute_function` builds its `Context`
  from it. `Context::new(inputs, limit)` has only test callers now and is `#[cfg(test)]`.
- **Capacity proof (ADR 0028).** `execute_function` returns `ArgumentError::Arity` unless
  `operands.len() == function.signature().len()`, before any stack exists. `MAX_OPERANDS` is the
  maximum of `signature().len()` over `Function::ALL`, computed in a `const` from the same table
  (`no_function_declares_more_operands_than_one_broadcast_buffer_holds` pins that). The stack
  holds the operands only: the Function's answer is returned from the dispatch, never pushed. So
  the depth is `signature().len() <= MAX_OPERANDS`, and an inline capacity of `MAX_OPERANDS`
  always suffices. For any other caller, the clamp keeps `limit <= MAX_OPERANDS`, so `push`
  refuses with `OperandStackExhausted` before `ArrayVec::push` could panic. The proof is stated as
  a present-tense invariant on `Stack` and its `limit` field.
- Tests. `an_exhausted_operand_stack_diagnoses_rather_than_panicking` passes unchanged.
  `an_operand_list_wider_than_the_inline_storage_diagnoses_rather_than_panicking` pushes
  `MAX_OPERANDS + 1` operands through `with_operands` and gets `OperandStackExhausted { capacity:
  MAX_OPERANDS }`. `operands_pop_back_in_signature_order` pins the reversal.
- Allocation tests (`lang/tests/allocation.rs`). The tests were tightened first and failed before
  the storage changed:
  - `a_turn_over_atoms_allocates_nothing_but_its_operand_stack` is renamed
    `a_turn_over_atoms_allocates_nothing` and asserts zero.
  - `evaluating_a_parsed_source_allocates_per_call_and_not_per_row` is renamed
    `evaluating_a_parsed_source_allocates_nothing_per_call_or_per_row` and asserts zero for one
    pass and for four.
  - `a_sequence_operand_is_consumed_without_copying_its_members` asserts zero.
  - `a_turn_that_builds_a_sequence_allocates_only_that_answer` drops the stack's block and bytes
    from both ceilings.
  - `operand_stack` is deleted.
  The repository comment policy keeps lineage out of source, so the `FINDING:` comment is replaced
  with the present-tense invariant (the stack is inline, so a call over Atom operands allocates
  nothing), not with a note saying what was corrected. The correction itself is recorded here and
  in the commit message.
- Memory series. Four measurements are now asserted zero and are no longer published: `lang call
  fixture`, `lang call fixture written four times`, `lang turn Add`, and `lang turn Reverse over
  64 members`. The reason is the rule in `memory-verification/05`: a zero point leaves the action's
  ratio against the previous one undefined. Measured with `ORCVS_MEMORY_SERIES=1` on `main` and then
  on this branch:

  ```
  lang call fixture                    8 blocks /  408 bytes  ->  0 / 0
  lang call fixture written four times 32 blocks / 1632 bytes ->  0 / 0
  lang turn Add                        1 block  /   48 bytes  ->  0 / 0
  lang turn Reverse over 64 members    1 block  /   24 bytes  ->  0 / 0
  lang turn Concatenate (2 x 64)       2 blocks /  304 bytes  ->  1 / 256
  ```

- `lib.rs`'s `the_answer_seam_is_the_size_the_execute_function_benchmark_was_measured_against`
  passes unchanged, because `Performance` and `Interpretation` do not change size.
- Benchmarks. The comparison belongs to the benchmark workflow: the `execute_function` benchmark
  and the `execute_function_operands` group in `lang/benches/lang.rs`. It was not run locally.
