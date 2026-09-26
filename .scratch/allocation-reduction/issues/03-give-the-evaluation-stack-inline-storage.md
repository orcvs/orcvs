# 03 — Give the evaluation stack inline storage

**What to build:** Evaluating a Turn through `Interpreter::execute_function`, the path `orcvs`
calls, uses inline storage for the Operand Stack, so a Turn takes no heap block for its stack.
Operand clones are source-audit/24's.

**Blocked by:** None — can start immediately.

Related: source-audit/09 decides whether `Interpreter::execute` stays public; source-audit/24 owns
the operand clones on the same path.

**Status:** ready-for-agent

- [ ] `Stack::new` stops taking a heap block for every Turn. The path is
      `Interpreter::execute_function` -> `Context::new(inputs, operands.len() + 1)` ->
      `Stack::new(limit)`, which is `Vec::with_capacity(limit)` over a 24-byte `Value`.
- [ ] The storage is inline, following `05e4490` and `a215af7` — the same treatment already
      applied to the Parser's pending stacks and to the inline Expression records. The inline
      capacity is the `MAX_OPERANDS + 1` bound below. Growable overflow ships only if a shipped
      caller can exceed that bound, which after source-audit/09 means only if `execute` stays
      public; otherwise no overflow branch ships that only a test could reach.
- [ ] ADR 0028's requirement still holds. On `execute_function` the depth is at most
      `MAX_OPERANDS + 1`, so an inline capacity of that size is provably sufficient; record the
      proof. If `execute` stays public after source-audit/09, exhausting its stack still answers a
      diagnostic. Inline storage must not turn an exhausted stack into a panic.
- [ ] A Turn through `execute_function` takes no heap block for its Operand Stack. An allocation
      test in `lang/tests/allocation.rs` measures `execute_function` and pins zero allocations for
      a Function whose operands and answer hold no Sequence;
      `a_tick_over_an_already_parsed_source_allocates_per_expression_and_not_per_row` measures
      `execute` and is replaced or retargeted, and its `FINDING (2026-09-09)` comment is replaced
      with a note saying what was corrected. If source-audit/24 lands first and adds this test,
      reuse it.
- [ ] If `execute` stays public, an Expression that exceeds the inline capacity still evaluates
      correctly through it, and a test covers it.
- [ ] Every existing `lang` test and property passes unchanged, including the `Stack` suite and the
      Interpreter suites.
- [ ] A benchmark over `execute_function` covers this path; add it if source-audit/24 has not.
      Note it for the comparison the benchmark workflow runs; do not run the comparison locally.

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
